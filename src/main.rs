mod models;

use models::config::Configuration;
use simplelog::{SimpleLogger, Config, LevelFilter};
use log::info;
use crate::models::archive::{
    Items,
    ArchiveOrgClient,
};

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
    let mut items = Items::read_saved_items().await;
    let since = if items.len() == 0{
        "1971-01-01"
    }else{
        "2022-12-12"
    };
    let aoc = ArchiveOrgClient::new(configuration.get_creator());
    let new_items = aoc.get_items(&since).await;
    items.add(&new_items);
    items.save_items().await;
}
