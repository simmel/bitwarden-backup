use argh::FromArgs;
use log::{info, LevelFilter};

#[derive(FromArgs, Debug)]
#[argh(
    description = "Bitwarden backup",
    example = "Set log level to debug.\n$ {command_name} -vv"
)]
struct BitwardenBackup {
    /// control the verbosity of logging. One = info, two = debug
    #[argh(switch, short = 'v')]
    verbose: i32,
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

    info!("Hello, world!");
}
