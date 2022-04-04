#![deny(warnings)]
extern crate serde_json;
extern crate valico;
use argh::FromArgs;
use log::{debug, info, LevelFilter};
#[cfg(unix)]
use nix::sys::stat;
use std::fs;
use std::path::Path;
use valico::json_schema;
use zeroize::Zeroize;

#[derive(FromArgs, Debug)]
#[argh(
    description = "Bitwarden backup",
    example = "Set log level to debug.\n$ {command_name} -v -v"
)]
struct BitwardenBackup {
    /// control the verbosity of logging. One = info, two = debug
    #[argh(switch, short = 'v')]
    verbose: i32,

    /// file or directory where you save the unencrypted Bitwarden backup [REQUIRED]
    #[argh(option, short = 'p')]
    path: Option<String>,

    /// whether or not to use file system watching on path
    #[argh(switch)]
    fswatch: bool,

    /// check version
    #[argh(switch)]
    version: bool,
}

// FIXME: Return a Result instead and use map_err to add to the errors
fn validate_backup(backup_json: &str) -> bool {
    let schema = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/resources/bitwarden_export_schema.json"
    );
    let json_schema_file = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/resources/bitwarden_export_schema.json"
    ));

    let json_schema = serde_json::from_str(json_schema_file)
        .unwrap_or_else(|e| panic!("Couldn't parse schema file: {}: {:?}", &schema, e));

    let mut scope = json_schema::Scope::new();
    let json_schema = scope
        .compile_and_return(json_schema, true)
        .expect("Couldn't compile schema");
    let backup_json_parsed =
        serde_json::from_str(backup_json).expect("Bitwarden backup is not valid JSON");
    let valid = json_schema.validate(&backup_json_parsed).is_valid();
    debug!("Is valid: {:?}", valid);

    valid
}

#[test]
fn test_bare_minimum() {
    let json = String::from(
        r#"
{
  "folders": [
   {}
  ],
  "items": [
   {}
  ]
}
"#,
    );
    let valid = validate_backup(&json);
    assert!(valid);
}

#[test]
fn test_bitwarden_example() {
    let json = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/bitwarden_export.json"
    ))
    .unwrap();

    let valid = validate_backup(&json);
    assert!(valid);
}

fn main() {
    let args: BitwardenBackup = argh::from_env();

    if args.version {
        println!(env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut loglevel: LevelFilter = LevelFilter::Error;
    if args.verbose == 1 {
        loglevel = LevelFilter::Info;
    } else if args.verbose == 2 {
        loglevel = LevelFilter::Debug;
    }
    env_logger::Builder::from_default_env()
        .format_level(true)
        .format_module_path(false)
        .format_timestamp(None)
        .filter(None, loglevel)
        .init();

    let path = args.path.expect("Required options not provided: --path");
    info!("Path: {:?}", path);
    info!("fswatch: {:?}", args.fswatch);

    let mut bitwarden_backup;
    if cfg!(unix) {
        // Ignore if it doesn't exist
        let _ = fs::remove_file(&path);
        nix::unistd::mkfifo(Path::new(&path), stat::Mode::S_IRWXU).unwrap();
        bitwarden_backup = fs::read_to_string(&path).unwrap();
    } else if cfg!(windows) {
        bitwarden_backup = String::from(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/bitwarden_export.json"
        )));
    } else {
        panic!("Unknown platform");
    };

    if validate_backup(&bitwarden_backup) {
        info!("Backup is valid!");
        print!("{}", &bitwarden_backup);
        bitwarden_backup.zeroize();
        fs::remove_file(&path).unwrap();
    } else {
        fs::remove_file(&path).unwrap();
        panic!("Could not validate backup");
    }
}
