mod models;

use models::config::Configuration;
use simplelog::{SimpleLogger, Config, LevelFilter};
use log::{info, error};
use tera::{Context, Tera};
use crate::models::{
    item::Item,
    items::Items,
    site::Page,
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
    info!("Configuration: {:?}", configuration);
    let new_items = read_and_save(&configuration).await;
}

async fn read_and_save(configuration: &Configuration) -> Vec<Item>{
    let mut items = Items::read_saved_items(configuration.get_data()).await;

    info!("{}", items.get_last().get_mtime().parse::<u64>().unwrap());
    info!("{}", items.get_last().get_date());
    let since = if items.len() == 0{
        "1971-01-01".to_string()
    }else{
        items.get_last().get_date().format("%Y-%m-%d").to_string()
    };
    info!("{}", since);
    let aoc = ArchiveOrgClient::new(configuration.get_creator());
    let read_items = aoc.get_items(&since).await;
    let mut to_add = Vec::new();
    for item in read_items{
        if !items.exists(&item){
            info!("To add {}", &item.get_identifier());
            to_add.push(item);
        }
    }
    generate_index(&configuration, items.get_items());
    if to_add.len() > 0 {
        items.add(&to_add);
        match items.save_items(configuration.get_data()).await{
            Ok(_) => {
                info!("Saved");
                generate_html(&configuration, &to_add);
                generate_index(&configuration, items.get_items());
            },
            Err(e) => error!("Some error happened, {}", e),
        }
    }
    info!("Added {} items", to_add.len());
    to_add
}

fn generate_index(configuration: &Configuration, items: &Vec<Item>){
    let tera = match Tera::new("templates/*.html") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let mut context = Context::new();
    context.insert("site", configuration.get_site());
    context.insert("footer_links", configuration.get_footer_links());
    let posts: Vec<Page> = items.iter().map(|item| item.get_page()).collect();
    context.insert("posts", &posts);
    match tera.render("index.html", &context){
        Ok(value) => info!("{}", value),
        Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
    }
}

fn generate_html(configuration: &Configuration, new_items: &Vec<Item>){
    let tera = match Tera::new("templates/*.html") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let mut context = Context::new();
    context.insert("site", configuration.get_site());
    context.insert("footer_links", configuration.get_footer_links());
    for item in new_items.as_slice(){
        context.insert("page", &item.get_page());
        println!("content: {}", item.get_page().content);
        match tera.render("post.html", &context){
            Ok(value) => info!("{}", value),
            Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
        }
    }
}
