use argh::FromArgs;
use log::{info, LevelFilter};
use nix::sys::stat;
use std::fs;
use std::path::Path;

#[deny(warnings)]
#[derive(FromArgs, Debug)]
#[argh(
    description = "Bitwarden backup",
    example = "Set log level to debug.\n$ {command_name} -vv"
)]
struct BitwardenBackup {
    /// control the verbosity of logging. One = info, two = debug
    #[argh(switch, short = 'v')]
    verbose: i32,

    /// file or directory where you save the unencrypted Bitwarden backup
    #[argh(option, short = 'p')]
    path: String,

    /// whether or not to use file system watching on path
    #[argh(switch)]
    fswatch: bool,
}

fn main() {
    let args: BitwardenBackup = argh::from_env();
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

    info!("Path: {:?}", args.path);
    info!("fswatch: {:?}", args.fswatch);

    nix::unistd::mkfifo(Path::new(&args.path), stat::Mode::S_IRWXU).unwrap();
    let bitwarden_backup = fs::read_to_string(&args.path).unwrap();
    print!("{}", bitwarden_backup);
    fs::remove_file(args.path).unwrap();
}
