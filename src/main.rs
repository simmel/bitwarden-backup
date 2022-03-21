use log::{info, LevelFilter};

fn main() {
    let loglevel: LevelFilter = LevelFilter::Info;
    env_logger::Builder::from_default_env()
        .format_level(true)
        .format_module_path(false)
        .format_timestamp(None)
        .filter(None, loglevel)
        .init();

    info!("Hello, world!");
}
