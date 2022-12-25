mod models;

use models::config::Configuration;
use simplelog::{SimpleLogger, Config, LevelFilter};
use log::info;
use crate::models::{
    items::Items,
    archive::ArchiveOrgClient,
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
    let mut items = Items::read_saved_items(configuration.get_data()).await;

    info!("{}", items.get_last());
    let since = if items.len() == 0{
        //"1971-01-01"
        "2022-12-22"
    }else{
        "2022-12-22"
    };
    let aoc = ArchiveOrgClient::new(configuration.get_creator());
    let new_items = aoc.get_items(&since).await;
    info!("{:?}", &new_items);
    items.add(&new_items);
    info!("{:?}", &items);
    items.save_items(configuration.get_data()).await;
}
