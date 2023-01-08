mod models;

use models::config::Configuration;
use simplelog::{SimpleLogger, Config, LevelFilter};
use log::{debug, info, error};
use tera::{Context, Tera};
use std::str::FromStr;
use crate::models::{
    archive::ArchiveOrg,
    article::Article,
    page::Page,
    site::{Post, Layout},
    episode::Episode,
};

#[tokio::main]
async fn main(){
    let configuration = Configuration::read_configuration().await;
    let level_filter = LevelFilter::from_str(configuration.get_log_level())
        .unwrap_or(LevelFilter::Info);
    let _ = SimpleLogger::init(level_filter, Config::default());
    debug!("Configuration: {:?}", configuration);

    update(&configuration).await;

    let posts = read_episodes_and_posts().await;
    let pages = read_pages().await;
    debug!("{:?}", posts);
    info!("=== Generation ===");
    create_public(&configuration).await;
    generate_html(&configuration, &posts, &pages).await;
    generate_index(&configuration, &posts, &pages).await;
    generate_feed(&configuration, &posts).await;
    generate_stats(&configuration, &posts, &pages).await;
    let style_css = configuration.get_style_css();
    let public = if configuration.get_site().baseurl.is_empty(){
        configuration.get_public().to_owned()
    }else{
        format!("{}/{}", configuration.get_public(), configuration.get_site().baseurl)
    };
    let output = format!("{}/style.css", public);
    copy_file(style_css, &output).await;
}

async fn read_pages() -> Vec<Post>{
    let mut posts = Vec::new();
    let mut posts_dir = tokio::fs::read_dir("pages").await.unwrap();
    while let Some(file) = posts_dir.next_entry().await.unwrap(){
        if file.metadata().await.unwrap().is_file(){
            let filename = file.file_name().to_str().unwrap().to_string();
            if filename.ends_with(".md"){
                match Page::new(&filename).await{
                    Ok(page) => posts.push(page.get_post()),
                    Err(e) => error!("Cant write {}. {}", filename, e),
                }
            }
        }
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

async fn read_episodes_and_posts() -> Vec<Post>{
    let mut posts = Vec::new();
    let mut episodes_dir = tokio::fs::read_dir("episodes").await.unwrap();
    while let Some(file) = episodes_dir.next_entry().await.unwrap(){
        if file.metadata().await.unwrap().is_file(){
            let filename = file.file_name().to_str().unwrap().to_string();
            if filename.ends_with(".md"){
                debug!("Read episode: {}", filename);
                match Episode::new(&filename).await{
                    Ok(episode) => posts.push(episode.get_post()),
                    Err(e) => error!("Cant write {}. {}", filename, e),
                }
            }
        }
    }

    let mut posts_dir = tokio::fs::read_dir("posts").await.unwrap();
    while let Some(file) = posts_dir.next_entry().await.unwrap(){
        if file.metadata().await.unwrap().is_file(){
            let filename = file.file_name().to_str().unwrap().to_string();
            if filename.ends_with(".md"){
                match Article::new(&filename).await{
                    Ok(article) => posts.push(article.get_post()),
                    Err(e) => error!("Cant write {}. {}", filename, e),
                }
            }
        }
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

async fn generate_feed(configuration: &Configuration, posts: &Vec<Post>){
    let tera = match Tera::new("templates/*.xml") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let public = if configuration.get_site().baseurl.is_empty(){
        configuration.get_public().to_owned()
    }else{
        format!("{}/{}", configuration.get_public(), configuration.get_site().baseurl)
    };
    let mut context = Context::new();
    let url = if configuration.get_site().baseurl.is_empty(){
        "".to_string()
    }else{
        if configuration.get_site().baseurl.starts_with("/"){
            configuration.get_site().baseurl.to_owned()
        }else{
            format!("/{}", configuration.get_site().baseurl)
        }
    };
    context.insert("url", &url);
    context.insert("site", configuration.get_site());
    let filter_posts: Vec<&Post> = posts
        .iter()
        .filter(|post| post.layout == Layout::PODCAST).collect();
    context.insert("posts", &filter_posts);
    match tera.render("feed.xml", &context){
        Ok(content) => {
            debug!("{}", content);
            write_post(&public, "", Some("feed.xml"), &content).await;
        },
        Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
    }
}

async fn generate_stats(configuration: &Configuration, posts: &Vec<Post>, pages: &Vec<Post>){
    let tera = match Tera::new("templates/*.html") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let public = if configuration.get_site().baseurl.is_empty(){
        configuration.get_public().to_owned()
    }else{
        format!("{}/{}", configuration.get_public(), configuration.get_site().baseurl)
    };
    let mut context = Context::new();
    let url = if configuration.get_site().baseurl.is_empty(){
        "".to_string()
    }else{
        if configuration.get_site().baseurl.starts_with("/"){
            configuration.get_site().baseurl.to_owned()
        }else{
            format!("/{}", configuration.get_site().baseurl)
        }
    };
    context.insert("url", &url);
    context.insert("site", configuration.get_site());
    context.insert("pages", pages);
    context.insert("posts", posts);
    match tera.render("statistics.html", &context){
        Ok(content) => {
            debug!("{}", content);
            create_dir(&format!("{}/{}", public, "statistics")).await;
            write_post(&public, "statistics", None, &content).await;
        },
        Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
    }
}

async fn generate_index(configuration: &Configuration, posts: &Vec<Post>, pages: &Vec<Post>){
    let tera = match Tera::new("templates/*.html") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let public = if configuration.get_site().baseurl.is_empty(){
        configuration.get_public().to_owned()
    }else{
        format!("{}/{}", configuration.get_public(), configuration.get_site().baseurl)
    };
    let mut context = Context::new();
    let url = if configuration.get_site().baseurl.is_empty(){
        "".to_string()
    }else{
        if configuration.get_site().baseurl.starts_with("/"){
            configuration.get_site().baseurl.to_owned()
        }else{
            format!("/{}", configuration.get_site().baseurl)
        }
    };
    context.insert("url", &url);
    context.insert("site", configuration.get_site());
    context.insert("pages", pages);
    context.insert("posts", &posts);
    match tera.render("index.html", &context){
        Ok(content) => {
            debug!("{}", content);
            write_post(&public, "", None, &content).await;
        },
        Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
    }
}

async fn generate_html(configuration: &Configuration, posts: &Vec<Post>, pages: &Vec<Post>){
    let tera = match Tera::new("templates/*.html") {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let public = if configuration.get_site().baseurl.is_empty(){
        configuration.get_public().to_owned()
    }else{
        format!("{}/{}", configuration.get_public(), configuration.get_site().baseurl)
    };
    let mut context = Context::new();
    let url = if configuration.get_site().baseurl.is_empty(){
        "".to_string()
    }else{
        if configuration.get_site().baseurl.starts_with("/"){
            configuration.get_site().baseurl.to_owned()
        }else{
            format!("/{}", configuration.get_site().baseurl)
        }
    };
    context.insert("url", &url);
    context.insert("site", configuration.get_site());
    context.insert("pages", pages);
    let mut post_and_pages = Vec::new();
    post_and_pages.extend_from_slice(posts);
    post_and_pages.extend_from_slice(pages);
    for post in post_and_pages.as_slice(){
        context.insert("post", post);
        match tera.render("post.html", &context){
            Ok(content) => {
                debug!("{}", &content);
                debug!("Post: {:?}", &post);
                create_dir(&format!("{}/{}",public, &post.slug)).await;
                write_post(&public, &post.slug, None, &content).await
            },
            Err(e) => error!("Algo no ha funcionado correctamente, {}", e),
        }
    }
}

async fn update(configuration: &Configuration){
    let mut new_docs = Vec::new();
    let aoclient = configuration.get_archiveorg();
    let docs = aoclient.get_all_docs().await;
    for doc in docs{
        if doc.exists().await{
            info!("Doc {} exists", doc.get_identifier());
            let filename = doc.get_filename();
            match Episode::new(&filename).await{
                Ok(ref mut episode) => {
                    if episode.get_downloads() != doc.get_downloads(){
                        episode.set_downloads(doc.get_downloads());
                        match episode.save().await{
                            Ok(_) => info!("Episode {} saved", episode.get_slug()),
                            Err(e) => error!("Cant save episode. {}", e),
                        }
                    }
                },
                Err(e) => {
                    error!("Cant create episode. {}", e);
                    new_docs.push(doc);
                }
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
                        match episode.save().await{
                            Ok(_) => info!("Episode {} saved", episode.get_slug()),
                            Err(e) => error!("Cant save episode. {}", e),
                        }
                    },
                    None => error!("Cant download from {}", doc.get_identifier()),
                }
            },
            None => error!("Cant download from {}", doc.get_identifier()),
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

async fn write_post(base: &str, endpoint: &str, filename: Option<&str>, content: &str){
    let base = clean_path(base);
    let endpoint = clean_path(endpoint);
    let filename =filename.unwrap_or("index.html");
    let output = if endpoint.is_empty(){
        format!("{}/{}", base, filename)
    }else{
        format!("{}/{}/{}", base, endpoint, filename)
    };
    match tokio::fs::write(&output, content,).await{
        Ok(_) => debug!("post {} created", &output),
        Err(e) => {
            error!("Cant create post {}, {}", &output, e);
            std::process::exit(1);
        }
    }
}

async fn create_dir(output: &str){
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
    match tokio::fs::create_dir_all(&output).await{
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
    match tokio::fs::create_dir_all(output).await{
        Ok(_) => info!("Directory {} created", output),
        Err(e) => {
            error!("Cant create directory {}, {}", output, e);
            std::process::exit(1);
        }
    }
}
