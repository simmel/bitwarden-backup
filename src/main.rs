#![deny(warnings)]
#[cfg(windows)]
extern crate notify;
extern crate serde_json;
extern crate valico;
use anyhow::{anyhow, Result};
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
fn validate_backup(backup_json: &str) -> Result<()> {
    let schema = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/resources/bitwarden_export_schema.json"
    );
    let json_schema_file = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/resources/bitwarden_export_schema.json"
    ));

    let json_schema = serde_json::from_str(json_schema_file)
        .map_err(|e| anyhow!("Couldn't parse schema file: {}: {:?}", &schema, e))?;

    let mut scope = json_schema::Scope::new();
    let json_schema = scope
        .compile_and_return(json_schema, true)
        .map_err(|e| anyhow!("Couldn't compile schema: {:?}", e))?;
    let backup_json_parsed = serde_json::from_str(backup_json)
        .map_err(|e| anyhow!("Bitwarden backup is not valid JSON: {:?}", e))?;
    let schema_validation = json_schema.validate(&backup_json_parsed);

    debug!("Schema validation: {:?}", schema_validation);
    debug!("Is valid: {:?}", schema_validation.is_valid());

    #[allow(clippy::unnecessary_lazy_evaluations)]
    #[allow(clippy::or_fun_call)]
    schema_validation
        .is_valid()
        .then(|| ())
        .ok_or(anyhow!("Could not validate backup"))
}

#[cfg(unix)]
fn get_backup(path: &Path) -> Result<(String, PathBuf)> {
    // Ignore if it doesn't exist
    let _ = fs::remove_file(path);
    nix::unistd::mkfifo(path, stat::Mode::S_IRWXU)?;
    Ok((fs::read_to_string(path).unwrap(), path.to_path_buf()))
}

#[cfg(windows)]
fn get_backup(path: &Path) -> Result<(String, PathBuf)> {
    // If path exists and isn't a dir
    #[allow(clippy::or_fun_call)]
    if path.exists() {
        (path.is_dir())
            .then(|| path)
            .ok_or(anyhow!("{:?} is not a directory", path))?;
    } else {
        fs::create_dir_all(path).unwrap();
        // FIXME: Fix permissions here windows_permissions looks like it's it but I can't
        //        understand how to use it
    }
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(500)).unwrap();
    watcher.watch(path, RecursiveMode::Recursive).unwrap();
    let bitwarden_backup;
    let path: PathBuf;
    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Create(backup)) => {
                path = backup;
                bitwarden_backup = fs::read_to_string(&path).unwrap();
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
    Ok((bitwarden_backup, path))
}

#[test]
fn test_bare_minimum() -> Result<()> {
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
    validate_backup(&json)
}

#[test]
fn test_bitwarden_example() -> Result<()> {
    let json = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/bitwarden_export.json"
    ))
    .unwrap();

    validate_backup(&json)
}

#[test]
fn test_not_valid_json_schema() {
    let json = String::from(
        r#"
        {"my-secret-key": "my-secret-key"}
"#,
    );
    let valid = validate_backup(&json);
    assert!(valid.is_err());
}

fn main() -> Result<()> {
    let args: BitwardenBackup = argh::from_env();

    if args.version {
        println!(env!("CARGO_PKG_VERSION"));
        return Ok(());
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

    #[allow(clippy::or_fun_call)]
    let path = args
        .path
        .ok_or(anyhow!("Required options not provided: --path"))?;
    info!("Save Bitwarden backup to this file: {:?}", path);

    let (mut bitwarden_backup, path) = get_backup(&path)?;

    validate_backup(&bitwarden_backup).map_err(|err| {
        bitwarden_backup.zeroize();
        err
    })?;

    info!("Backup is valid!");
    print!("{}", &bitwarden_backup);

    bitwarden_backup.zeroize();
    fs::remove_file(path)?;

    Ok(())
}
