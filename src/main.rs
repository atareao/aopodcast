mod models;

use models::config::Configuration;
use simplelog::{SimpleLogger, Config, LevelFilter};
use log::info;

#[tokio::main]
async fn main(){
    println!("Hello, world!");
    let configuration = Configuration::read_configuration().await;
    let level_filter = match configuration.get_log_level(){
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Off,
    };
    let _ = SimpleLogger::init(level_filter, Config::default());
    info!("Configuration: {}", configuration);
}
