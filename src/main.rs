mod models;

use models::config::Configuration;
use simplelog::{SimpleLogger, Config, LevelFilter};
use log::{debug, info, error};
use tera::{Context, Tera};
use std::str::FromStr;
use crate::models::{
    archive::ArchiveOrg,
    article::Article,
    item::Item,
    items::Items,
    site::Post, episode::Episode,
};

#[tokio::main]
async fn main(){
    let configuration = Configuration::read_configuration().await;
    let level_filter = LevelFilter::from_str(configuration.get_log_level())
        .unwrap_or(LevelFilter::Info);
    let _ = SimpleLogger::init(level_filter, Config::default());
    debug!("Configuration: {:?}", configuration);

    let mut new_docs = Vec::new();
    let aoclient = configuration.get_archiveorg();
    let docs = aoclient.get_all_docs().await;
    for doc in docs{
        if doc.exists().await{
            info!("Doc {} exists", doc.get_identifier());
            let filename = doc.get_filename();
            match Episode::new(&filename).await{
                Some(ref mut episode) => {
                    if episode.downloads != doc.get_downloads(){
                        episode.downloads = doc.get_downloads();
                        episode.save().await;
                    }
                },
                None => new_docs.push(doc),
            }
        }else{
            new_docs.push(doc);
        } 
    }
    for doc in new_docs{
        match ArchiveOrg::get_metadata(doc.get_identifier()).await{
            Some(metadata) => {
                match ArchiveOrg::get_mp3_metadata(doc.get_identifier()).await{
                    Some(mp3) => {
                        let episode = Episode::combine(&doc, &metadata, &mp3);
                        episode.save().await;
                    },
                    None => error!("Cant download from {}", doc.get_identifier()),
                }
            },
            None => error!("Cant download from {}", doc.get_identifier()),
        }

    }
    let posts = read_episodes_and_posts().await;
    debug!("{:?}", posts);
    //read_and_save(&configuration).await;
}

async fn read_and_save(configuration: &Configuration){

    let mut items = Items::read_saved_items(configuration.get_data()).await;
    debug!("{}", items.get_last().get_mtime().parse::<u64>().unwrap());
    debug!("{}", items.get_last().get_date());
    let since = if items.len() == 0{
        "1971-01-01".to_string()
    }else{
        items.get_last().get_date().format("%Y-%m-%d").to_string()
    };
    info!("{}", since);
    let mut to_add = Vec::new();
    let archiveorg = configuration.get_archiveorg();
    let read_items = archiveorg.get_items(&since).await;
    for item in read_items{
        if !items.exists(&item){
            debug!("To add {}", &item.get_identifier());
            to_add.push(item);
        }
    }
    if to_add.len() > 0 {
        items.add(&to_add);
        info!("Added {} items", to_add.len());
        match items.save_items(configuration.get_data()).await{
            Ok(_) => {
                info!("Saved");
                create_public(&configuration).await;
                //generate_html(&configuration, items.get_items()).await;
                //generate_index(&configuration, items.get_items()).await;
                //generate_feed(&configuration, items.get_items()).await;
                let style_css = configuration.get_style_css();
                let public = configuration.get_public();
                let output = format!("{}/style.css", public);
                copy_file(style_css, &output).await;
            },
            Err(e) => error!("Some error happened, {}", e),
        }
    }
}

async fn read_episodes_and_posts() -> Vec<Post>{
    let mut posts = Vec::new();
    let mut episods_dir = tokio::fs::read_dir("episodes").await.unwrap();
    while let Some(file) = episods_dir.next_entry().await.unwrap(){
        if file.metadata().await.unwrap().is_file(){
            let filename = file.file_name().to_str().unwrap().to_string();
            if let Some(episode) = Episode::new(&filename).await{
                posts.push(episode.get_post());
            }
        }
    }
    let mut posts_dir = tokio::fs::read_dir("posts").await.unwrap();
    while let Some(file) = posts_dir.next_entry().await.unwrap(){
        if file.metadata().await.unwrap().is_file(){
            let filename = file.file_name().to_str().unwrap().to_string();
            if let Some(article) = Article::new(&filename).await{
                posts.push(article.get_post());
            }
        }
    }
    posts.sort_by(|a, b| a.date.cmp(&b.date));
    posts
}

async fn generate_feed(configuration: &Configuration, items: &Vec<Item>){
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
    //let posts: Vec<Post> = items.iter().map(|item| item.get_post()).collect();
    //context.insert("posts", &posts);
    match tera.render("index.html", &context){
        Ok(content) => {
            debug!("{}", content);
            write_post(public, "", &content).await;
        },
        Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
    }

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
    //let posts: Vec<Post> = items.iter().map(|item| item.get_post()).collect();
    //context.insert("posts", &posts);
    match tera.render("index.html", &context){
        Ok(content) => {
            debug!("{}", content);
            write_post(public, "", &content).await;
        },
        Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
    }
}

async fn generate_html(configuration: &Configuration, posts: &Vec<Post>){
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
    for post in posts.as_slice(){
        context.insert("post", post);
        match tera.render("post.html", &context){
            Ok(content) => {
                debug!("{}", &content);
                debug!("Post: {:?}", &post);
                create_dir(public, &post.slug).await;
                write_post(public, &post.slug, &content).await
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
        Ok(_) => debug!("post {} created", &output),
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
    debug!("Going to create : {}", &output);
    let exists = match tokio::fs::metadata(&output).await{
        Ok(metadata) => {
            debug!("Output dir {} exists", &output);
            metadata.is_dir()
        },
        Err(e) => {
            debug!("Output dir {}, {}", &output, e);
            false
        },
    };
    if exists{
        match tokio::fs::remove_dir_all(&output).await{
            Ok(_) => info!("Directory {} removed", output),
            Err(e) => {
                error!("Cant delete directory {}, {}", &output, e);
                std::process::exit(1);
            }
        }
    }
    match tokio::fs::create_dir(&output).await{
        Ok(_) => info!("Directory {} created", output),
        Err(e) => {
            error!("Cant create directory {}, {}", &output, e);
            std::process::exit(1);
        }
    }
}

pub async fn copy_file(from: &str, to: &str){
    match tokio::fs::copy(from, to).await{
        Ok(_) => info!("Copied from {} to {}", from, to),
        Err(e) => {
            error!("Cant copy from {} to {}. {}", from, to, e);
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
            error!("Output dir {}, {}", &output, e);
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
