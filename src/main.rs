mod models;

use crate::models::{
    archive::ArchiveOrg,
    article::Article,
    episode::Episode,
    page::Page,
    site::{Layout, Post},
};
use minijinja::context;
use models::{
    config::Configuration,
    mastodon::{get_mastodon_client, Mastodon},
    telegram::{get_telegram_client, Telegram},
    ENV,
};
use std::str::FromStr;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
const VERSION: usize = 1;

#[tokio::main]
async fn main() {
    let configuration = Configuration::read_configuration().await;
    let log_level = configuration.get_log_level();

    tracing_subscriber::registry()
        .with(EnvFilter::from_str(log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();

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
    generate_stats(&configuration, &posts).await;
    let public = if configuration.get_site().baseurl.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_site().baseurl
        )
    };
    //TODO: Copy directory assets a /public/{podcast}/assets
    //let output = format!("{}/style.css", public);
    let assets_dir = format!("{}/assets", public);
    create_dir(&assets_dir).await;
    copy_all_files("assets", &assets_dir).await;
}

async fn read_pages() -> Vec<Post> {
    let mut posts = Vec::new();
    let mut posts_dir = tokio::fs::read_dir("pages").await.unwrap();
    while let Some(file) = posts_dir.next_entry().await.unwrap() {
        if file.metadata().await.unwrap().is_file() {
            let filename = file.file_name().to_str().unwrap().to_string();
            if filename.ends_with(".md") {
                match Page::new(&filename).await {
                    Ok(page) => posts.push(page.get_post()),
                    Err(err) => {
                        error!("Can not write {}. {:#}", filename, err);
                        // render causes as well
                        let mut err = &err as &dyn std::error::Error;
                        while let Some(next_err) = err.source() {
                            error!("caused by: {:#}", next_err);
                            err = next_err;
                        }
                    }
                }
            }
        }
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

async fn read_episodes_and_posts() -> Vec<Post> {
    let mut posts = Vec::new();
    let mut episodes_dir = tokio::fs::read_dir("episodes").await.unwrap();
    while let Some(file) = episodes_dir.next_entry().await.unwrap() {
        if file.metadata().await.unwrap().is_file() {
            let filename = file.file_name().to_str().unwrap().to_string();
            if filename.ends_with(".md") {
                info!("Read episode: {}", filename);
                match Episode::new(&filename).await {
                    Ok(episode) => posts.push(episode.get_post()),
                    Err(err) => {
                        error!("Can not write {}. {:#}", filename, err);
                        // render causes as well
                        let mut err = &err as &dyn std::error::Error;
                        while let Some(next_err) = err.source() {
                            error!("caused by: {:#}", next_err);
                            err = next_err;
                        }
                    }
                }
            }
        }
    }

    let mut posts_dir = tokio::fs::read_dir("posts").await.unwrap();
    while let Some(file) = posts_dir.next_entry().await.unwrap() {
        if file.metadata().await.unwrap().is_file() {
            let filename = file.file_name().to_str().unwrap().to_string();
            if filename.ends_with(".md") {
                match Article::new(&filename).await {
                    Ok(article) => posts.push(article.get_post()),
                    Err(err) => {
                        error!("Can not write {}. {:#}", filename, err);
                        // render causes as well
                        let mut err = &err as &dyn std::error::Error;
                        while let Some(next_err) = err.source() {
                            error!("caused by: {:#}", next_err);
                            err = next_err;
                        }
                    }
                }
            }
        }
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

async fn post_with_mastodon(configuration: &Configuration, episode: &Episode, mastodon: &Mastodon) {
    let url = if configuration.get_site().baseurl.is_empty() {
        "".to_string()
    } else if configuration.get_site().baseurl.starts_with('/') {
        configuration.get_site().baseurl.to_owned()
    } else {
        format!("/{}", configuration.get_site().baseurl)
    };
    let ctx = context! {
        url => url,
        site => configuration.get_site(),
        post => episode.get_post(),
    };
    let template = ENV.get_template("mastodon.html").unwrap();
    match template.render(ctx) {
        Ok(content) => {
            debug!("{}", content);
            mastodon.post(&content).await;
        }
        Err(err) => {
            error!("Algo no ha funcionado correctamente. {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn post_with_telegram(configuration: &Configuration, episode: &Episode, telegram: &Telegram) {
    let url = if configuration.get_site().baseurl.is_empty() {
        "".to_string()
    } else if configuration.get_site().baseurl.starts_with('/') {
        configuration.get_site().baseurl.to_owned()
    } else {
        format!("/{}", configuration.get_site().baseurl)
    };
    let post = episode.get_post();
    let audio = format!(
        "https://archive.org/download/{}/{}",
        &post.identifier, &post.filename
    );
    let ctx = context! {
        url => url,
        site => configuration.get_site(),
        audio => audio,
        post => episode.get_post(),
    };
    let template = ENV.get_template("telegram.html").unwrap();
    match template.render(ctx) {
        Ok(caption) => {
            debug!("{}", caption);
            telegram.send_audio(&audio, &caption).await;
        }
        Err(err) => {
            error!("Algo no ha funcionado correctamente. {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn generate_feed(configuration: &Configuration, posts: &[Post]) {
    info!("generate_feed");
    let public = if configuration.get_site().baseurl.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_site().baseurl
        )
    };
    let url = if configuration.get_site().baseurl.is_empty() {
        "".to_string()
    } else if configuration.get_site().baseurl.starts_with('/') {
        configuration.get_site().baseurl.to_owned()
    } else {
        format!("/{}", configuration.get_site().baseurl)
    };
    let filter_posts: Vec<&Post> = posts
        .iter()
        .filter(|post| post.layout == Layout::PODCAST)
        .collect();
    let ctx = context! {
        url => url,
        site => configuration.get_site(),
        posts => filter_posts,
    };
    let template = ENV.get_template("feed.xml").unwrap();
    match template.render(ctx) {
        Ok(content) => {
            write_post(
                &public,
                "",
                Some(&configuration.get_site().podcast_feed),
                &content,
            )
            .await;
            debug!("write feed");
        }
        Err(err) => {
            error!("Could not render template: {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn generate_stats(configuration: &Configuration, posts: &Vec<Post>) {
    info!("generate_stats");
    let public = if configuration.get_site().baseurl.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_site().baseurl
        )
    };
    let url = if configuration.get_site().baseurl.is_empty() {
        "".to_string()
    } else if configuration.get_site().baseurl.starts_with('/') {
        configuration.get_site().baseurl.to_owned()
    } else {
        format!("/{}", configuration.get_site().baseurl)
    };
    let ctx = context! {
        url => url,
        site => configuration.get_site(),
        posts => posts,
    };
    let template = ENV.get_template("statistics.html").unwrap();
    match template.render(ctx) {
        Ok(content) => {
            debug!("{}", content);
            create_dir(&format!("{}/{}", public, "statistics")).await;
            write_post(&public, "statistics", None, &content).await;
        }
        Err(err) => {
            error!("Could not render template: {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn generate_index(configuration: &Configuration, posts: &Vec<Post>, pages: &Vec<Post>) {
    info!("generate_index");
    let public = if configuration.get_site().baseurl.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_site().baseurl
        )
    };
    let url = if configuration.get_site().baseurl.is_empty() {
        "".to_string()
    } else if configuration.get_site().baseurl.starts_with('/') {
        configuration.get_site().baseurl.to_owned()
    } else {
        format!("/{}", configuration.get_site().baseurl)
    };
    let ctx = context! {
        url => url,
        site => configuration.get_site(),
        pages => pages,
        posts => posts,
    };
    let template = ENV.get_template("index.html").unwrap();
    match template.render(ctx) {
        Ok(content) => {
            debug!("{}", content);
            write_post(&public, "", None, &content).await;
        }
        Err(err) => {
            error!("Could not render template: {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn generate_html(configuration: &Configuration, posts: &[Post], pages: &Vec<Post>) {
    info!("generate_html");
    let public = if configuration.get_site().baseurl.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_site().baseurl
        )
    };
    let url = if configuration.get_site().baseurl.is_empty() {
        "".to_string()
    } else if configuration.get_site().baseurl.starts_with('/') {
        configuration.get_site().baseurl.to_owned()
    } else {
        format!("/{}", configuration.get_site().baseurl)
    };
    let mut post_and_pages = Vec::new();
    post_and_pages.extend_from_slice(posts);
    post_and_pages.extend_from_slice(pages);
    for post in post_and_pages.as_slice() {
        let ctx = context!(
            url => url,
            site => configuration.get_site(),
            pages => pages,
            post => post,
        );
        let template = ENV.get_template("post.html").unwrap();
        match template.render(ctx) {
            Ok(content) => {
                debug!("{}", &content);
                debug!("Post: {:?}", &post);
                create_dir(&format!("{}/{}", public, &post.slug)).await;
                write_post(&public, &post.slug, None, &content).await
            }
            Err(err) => {
                error!("Could not render template: {:#}", err);
                // render causes as well
                let mut err = &err as &dyn std::error::Error;
                while let Some(next_err) = err.source() {
                    error!("caused by: {:#}", next_err);
                    err = next_err;
                }
            }
        }
    }
}

async fn update(configuration: &Configuration) {
    info!("update");
    let mastodon_client = get_mastodon_client();
    let telegram_client = get_telegram_client();
    let mut new_docs = Vec::new();
    let aoclient = configuration.get_archiveorg();
    let docs = aoclient.get_all_docs().await;
    for doc in docs {
        if doc.exists().await {
            info!("Doc {} exists", doc.get_identifier());
            debug!("Doc: {:?}", &doc);
            let filename = doc.get_filename();
            //BUG: Esto hay que revisar
            match Episode::new(&filename).await {
                Ok(ref mut episode) => {
                    if episode.get_downloads() != doc.get_downloads()
                        || episode.get_version() == 0
                        || episode.get_version() != VERSION
                    {
                        if episode.get_version() == 0 {
                            episode.set_datetime(doc.get_datetime());
                        }
                        episode.set_version(VERSION);
                        episode.set_downloads(doc.get_downloads());
                        match episode.save().await {
                            Ok(_) => info!("Episode {} saved", episode.get_slug()),
                            Err(err) => {
                                error!("1 Can not save episode {}. {:#}", episode.get_slug(), err);
                                // render causes as well
                                let mut err = &err as &dyn std::error::Error;
                                while let Some(next_err) = err.source() {
                                    error!("caused by: {:#}", next_err);
                                    err = next_err;
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("Can not create episode. {:#}", err);
                    // render causes as well
                    let mut err = &err as &dyn std::error::Error;
                    while let Some(next_err) = err.source() {
                        error!("caused by: {:#}", next_err);
                        err = next_err;
                    }
                }
            }
        } else {
            new_docs.push(doc);
        }
    }
    for doc in new_docs {
        match ArchiveOrg::get_metadata(doc.get_identifier()).await {
            Some(metadata) => {
                match ArchiveOrg::get_mp3_metadata(doc.get_identifier()).await {
                    Some(mp3) => {
                        let episode = Episode::combine(&doc, &metadata, &mp3);
                        match episode.save().await {
                            Ok(_) => {
                                match &telegram_client {
                                    Some(client) => {
                                        post_with_telegram(configuration, &episode, client).await;
                                    }
                                    None => {}
                                };
                                match &mastodon_client {
                                    Some(client) => {
                                        post_with_mastodon(configuration, &episode, client).await;
                                    }
                                    None => {}
                                }
                                info!("Episode {} saved", episode.get_slug());
                            }
                            Err(err) => {
                                error!("2 Can not save episode {}. {:#}", episode.get_slug(), err);
                                // render causes as well
                                let mut err = &err as &dyn std::error::Error;
                                while let Some(next_err) = err.source() {
                                    error!("caused by: {:#}", next_err);
                                    err = next_err;
                                }
                            }
                        }
                    }
                    None => error!("Cant download from {}", doc.get_identifier()),
                }
            }
            None => error!("Cant download from {}", doc.get_identifier()),
        }
    }
}

fn clean_path(path: &str) -> &str {
    let path = if path.starts_with('/') {
        path.to_string().remove(0);
        path
    } else {
        path
    };
    if path.ends_with('/') {
        path.to_string().pop();
        path
    } else {
        path
    }
}

async fn write_post(base: &str, endpoint: &str, filename: Option<&str>, content: &str) {
    debug!(
        "write_post. Base: {base}. Endpoint {endpoint}. Filename: {:?}",
        filename
    );
    let base = clean_path(base);
    let endpoint = clean_path(endpoint);
    let filename = filename.unwrap_or("index.html");
    let output = if endpoint.is_empty() {
        format!("{}/{}", base, filename)
    } else {
        format!("{}/{}/{}", base, endpoint, filename)
    };
    match tokio::fs::write(&output, content).await {
        Ok(_) => debug!("post {} created", &output),
        Err(err) => {
            error!("Can not create post {}. {:#}", &output, err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            std::process::exit(1);
        }
    }
}

async fn copy_all_files(from_dir: &str, to_dir: &str) {
    debug!("Going to copy from {} to {}", from_dir, to_dir);
    let mut episodes_dir = tokio::fs::read_dir(from_dir).await.unwrap();
    while let Some(file) = episodes_dir.next_entry().await.unwrap() {
        if file.metadata().await.unwrap().is_file() {
            let filename = file.file_name().to_str().unwrap().to_string();
            let input_file = format!("{}/{}", from_dir, filename);
            let output_file = format!("{}/{}", to_dir, filename);
            copy_file(&input_file, &output_file).await;
        }
    }
}

async fn create_dir(output: &str) {
    info!("Going to create : {}", &output);
    let exists = match tokio::fs::metadata(&output).await {
        Ok(metadata) => {
            debug!("Output dir {} exists", &output);
            metadata.is_dir()
        }
        Err(err) => {
            debug!("Can not get metadata for dir {}, {:#}", &output, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                debug!("caused by: {:#}", next_err);
                err = next_err;
            }
            false
        }
    };
    if exists {
        match tokio::fs::remove_dir_all(&output).await {
            Ok(_) => info!("Directory {} removed", output),
            Err(err) => {
                error!("Cant delete directory {}, {:#}", &output, err);
                let mut err = &err as &dyn std::error::Error;
                while let Some(next_err) = err.source() {
                    error!("caused by: {:#}", next_err);
                    err = next_err;
                }
                std::process::exit(1);
            }
        }
    }
    match tokio::fs::create_dir_all(&output).await {
        Ok(_) => info!("Directory {} created", output),
        Err(err) => {
            error!("Cant create directory {}, {:#}", &output, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            std::process::exit(1);
        }
    }
}

pub async fn copy_file(from: &str, to: &str) {
    match tokio::fs::copy(from, to).await {
        Ok(_) => info!("Copied from {} to {}", from, to),
        Err(err) => {
            error!("Cant copy from {} to {}. {:#}", from, to, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            std::process::exit(1);
        }
    }
}

pub async fn create_public(configuration: &Configuration) {
    info!("create_public");
    let output = configuration.get_public();
    info!("Output dir: {}", &output);
    let exists = match tokio::fs::metadata(output).await {
        Ok(metadata) => {
            debug!("Output dir {} exists", &output);
            metadata.is_dir()
        }
        Err(err) => {
            debug!("Output dir {}, {:#}", &output, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            false
        }
    };
    if exists {
        match tokio::fs::remove_dir_all(output).await {
            Ok(_) => info!("Directory {} removed", output),
            Err(err) => {
                error!("Cant delete directory {}, {}", output, err);
                let mut err = &err as &dyn std::error::Error;
                while let Some(next_err) = err.source() {
                    error!("caused by: {:#}", next_err);
                    err = next_err;
                }
                std::process::exit(1);
            }
        }
    }
    match tokio::fs::create_dir_all(output).await {
        Ok(_) => info!("Directory {} created", output),
        Err(err) => {
            error!("Cant create directory {}, {}", output, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            std::process::exit(1);
        }
    }
}
