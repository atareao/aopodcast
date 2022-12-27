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
    read_and_save(&configuration).await;
}

async fn read_and_save(configuration: &Configuration){
    create_public(&configuration).await;
    let mut items = Items::read_saved_items(configuration.get_data()).await;
    info!("{}", items.get_last().get_mtime().parse::<u64>().unwrap());
    info!("{}", items.get_last().get_date());
    let since = if items.len() == 0{
        "1971-01-01".to_string()
    }else{
        items.get_last().get_date().format("%Y-%m-%d").to_string()
    };
    info!("{}", since);
    let mut to_add = Vec::new();
    let read = false;
    if read {
        let aoc = ArchiveOrgClient::new(configuration.get_creator());
        let read_items = aoc.get_items(&since).await;
        for item in read_items{
            if !items.exists(&item){
                info!("To add {}", &item.get_identifier());
                to_add.push(item);
            }
        }
        if to_add.len() > 0 {
            items.add(&to_add);
            match items.save_items(configuration.get_data()).await{
                Ok(_) => {
                    info!("Saved");
                    generate_html(&configuration, &to_add).await;
                },
                Err(e) => error!("Some error happened, {}", e),
            }
        }
        info!("Added {} items", to_add.len());
    }
    generate_html(&configuration, items.get_items()).await;
    generate_index(&configuration, items.get_items()).await;
}


async fn generate_index(configuration: &Configuration, items: &Vec<Item>){
    let tera = match Tera::new("templates/*.html") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let public = configuration.get_public();
    let mut context = Context::new();
    context.insert("site", configuration.get_site());
    context.insert("footer_links", configuration.get_footer_links());
    let posts: Vec<Page> = items.iter().map(|item| item.get_page()).collect();
    context.insert("posts", &posts);
    match tera.render("index.html", &context){
        Ok(content) => {
            info!("{}", content);
            write_post(public, "", &content).await;
        },
        Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
    }
}

async fn generate_html(configuration: &Configuration, new_items: &Vec<Item>){
    let tera = match Tera::new("templates/*.html") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let public = configuration.get_public();
    let mut context = Context::new();
    context.insert("site", configuration.get_site());
    context.insert("footer_links", configuration.get_footer_links());
    for item in new_items.as_slice(){
        context.insert("page", &item.get_page());
        match tera.render("post.html", &context){
            Ok(content) => {
                info!("{}", &content);
                info!("Page: {:?}", &item.get_page());
                create_dir(public, &item.get_page().slug).await;
                write_post(public, &item.get_page().slug, &content).await
            },
            Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
        }
    }
}

fn clean_path(path: &str) -> &str{
    let path = if path.starts_with("/"){
        path.to_string().remove(0);
        path
    }else{
        path
    };
    if path.ends_with("/"){
        path.to_string().pop();
        path
    }else{
        path
    }
}

async fn write_post(base: &str, endpoint: &str, content: &str){
    let base = clean_path(base);
    let endpoint = clean_path(endpoint);
    let output = if endpoint.is_empty(){
        format!("{}/index.html", base)
    }else{
        format!("{}/{}/index.html", base, endpoint)
    };
    match tokio::fs::write(&output, content,).await{
        Ok(_) => info!("post {} created", &output),
        Err(e) => {
            error!("Cant create post {}, {}", &output, e);
            std::process::exit(1);
        }
    }
}

async fn create_dir(base: &str, endpoint: &str){
    let base = clean_path(base);
    let endpoint = clean_path(endpoint);
    let output = format!("{}/{}", base, endpoint);
    info!("Going to create : {}", &output);
    match tokio::fs::create_dir(&output).await{
        Ok(_) => info!("Directory {} created", output),
        Err(e) => {
            error!("Cant create directory {}, {}", &output, e);
            std::process::exit(1);
        }
    }
}

pub async fn create_public(configuration: &Configuration){
    let output = configuration.get_public();
    info!("Output dir: {}", &output);
    let exists = match tokio::fs::metadata(output).await{
        Ok(metadata) => {
            info!("Output dir {} exists", &output);
            metadata.is_dir()
        },
        Err(e) => {
            info!("Output dir {}, {}", &output, e);
            false
        },
    };
    if exists{
        match tokio::fs::remove_dir_all(output).await{
            Ok(_) => info!("Directory {} removed", output),
            Err(e) => {
                error!("Cant delete directory {}, {}", output, e);
                std::process::exit(1);
            }
        }
    }
    match tokio::fs::create_dir(output).await{
        Ok(_) => info!("Directory {} created", output),
        Err(e) => {
            error!("Cant create directory {}, {}", output, e);
            std::process::exit(1);
        }
    }
}
