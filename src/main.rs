#![deny(warnings)]
#[cfg(windows)]
extern crate notify;
extern crate serde_json;
extern crate valico;
use argh::FromArgs;
use log::{debug, info, LevelFilter};
#[allow(unused_imports)]
#[cfg(unix)]
use nix::sys::stat;
#[cfg(windows)]
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
#[cfg(windows)]
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
#[cfg(windows)]
use std::sync::mpsc::channel;
#[cfg(windows)]
use std::time::Duration;
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
    path: Option<PathBuf>,

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

#[cfg(unix)]
fn get_backup(path: &Path) -> String {
    // Ignore if it doesn't exist
    let _ = fs::remove_file(path);
    nix::unistd::mkfifo(path, stat::Mode::S_IRWXU).unwrap();
    fs::read_to_string(path).unwrap()
}

#[cfg(windows)]
fn get_backup(path: &Path) -> String {
    // If path exists and is a dir
    if path.exists() {
        (!path.is_dir())
            .then(|| path)
            .unwrap_or_else(|| panic!("{:?} is a directory, not a file", path));
    } else {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        // FIXME: Fix permissions here windows_permissions looks like it's it but I can't
        //        understand how to use it
        let _file = fs::File::create(path).unwrap();
    }
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(20)).unwrap();
    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();
    let bitwarden_backup;
    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Write(path)) => {
                bitwarden_backup = fs::read_to_string(path).unwrap();
                break;
            }
            Ok(event) => debug!("{:?}", event),
            Err(e) => panic!("watch error: {:?}", e),
        };
    }
    let zeroes = vec![0; fs::metadata(&path).unwrap().len().try_into().unwrap()];
    let mut buffer = fs::File::create(&path).unwrap();
    buffer.write_all(&zeroes).unwrap();
    buffer.flush().unwrap();
    bitwarden_backup
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
    info!("Save Bitwarden backup to this file: {:?}", path);

    let mut bitwarden_backup = get_backup(&path);

    let backup_valid = validate_backup(&bitwarden_backup);

    if backup_valid {
        info!("Backup is valid!");
        print!("{}", &bitwarden_backup);
    }

    bitwarden_backup.zeroize();
    fs::remove_file(&path).unwrap();

    if !backup_valid {
        panic!("Could not validate backup");
    }
}
