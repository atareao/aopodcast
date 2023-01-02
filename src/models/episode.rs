use serde::{Deserialize, Serialize};
use serde_json::Value;
use log::{debug, info, error};
use comrak::{markdown_to_html, ComrakOptions};

use super::{
    doc::Doc,
    site::{Post, Layout},
    metadata::Metadata,
    mp3metadata::Mp3Metadata,
    utils::get_slug,
};

const EXCERPT_LENGTH: usize = 150;

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode{
    // from doc
    pub number: usize,
    pub identifier: String,
    pub title: String,
    pub subject: Vec<String>,
    pub description: String,
    pub downloads: u64,
    // from mp3 metadata
    pub filename: String,
    pub mtime: u64,
    pub size: u64,
    pub length: u64,
    pub comment: String,
    // more
    pub slug: String,
}

impl Episode{
    pub async fn save(&self) {
        match serde_yaml::to_string(self){
            Ok(contents) => {
                let path = format!("episodes/{}.yml", self.identifier);
                match tokio::fs::write(path, contents).await{
                    Ok(_) => info!("Episode {}.yml saved", self.identifier),
                    Err(e) => error!("Cant save episode {}. {}", self.identifier, e),
                }
            },
            Err(e) => error!("Cant save episode {}. {}", self.identifier, e),
        }
    }

    pub async fn new(filename: &str) -> Option<Self>{
        let filename = format!("episodes/{}", filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        Self::parse(&data)
    }

    pub fn get_post(&self) -> Post{
        let content = markdown_to_html(&self.description, &ComrakOptions::default());
        Post{
            layout: Layout::PODCAST,
            slug: self.slug.clone(),
            excerpt: self.comment.clone(),
            title: self.title.clone(),
            content,
            date: self.mtime,
            identifier: self.identifier.clone(),
            filename: self.filename.clone(),
            length: self.length,
        }
    }

    pub fn combine(doc: &Doc, metadata: &Metadata, mp3: &Mp3Metadata) -> Episode{
        let title = if mp3.title.is_empty(){
            doc.get_identifier()
        }else{
            &mp3.title
        };
        let comment = if mp3.comment.is_empty(){
            if metadata.description.len() > EXCERPT_LENGTH{
                debug!("Description ({}): {}", metadata.description.len(),
                    metadata.description);
                let item = metadata.description
                    .split("\n")
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    .to_string();
                debug!("Sort description: item");
                if item.len() > EXCERPT_LENGTH{
                   item.as_str()
                        .chars()
                        .into_iter()
                        .take(EXCERPT_LENGTH)
                        .collect::<String>()
                        .to_string()
                }else{
                    item
                }
            }else{
                metadata.description.to_string()
            }
        }else{
            mp3.comment.to_string()
        };
        Self{
            number: doc.get_number(),
            identifier: doc.get_identifier().to_string(),
            subject: doc.get_subject(),
            description: metadata.description.to_string(),
            downloads: doc.get_downloads(),
            title: title.to_string(),
            filename: mp3.filename.to_string(),
            mtime: mp3.mtime,
            size: mp3.size,
            length: mp3.length,
            comment: comment.to_string(),
            slug: get_slug(title),
        }
    }

    fn parse(data: &str) -> Option<Episode>{
        match serde_yaml::from_str::<Value>(&data) {
            Ok(value) => {
                debug!("Value: {:?}", value);
                Some(Self{
                    number: value["number"].as_u64().unwrap().try_into().unwrap(),
                    identifier: value["identifier"].as_str().unwrap().to_string(),
                    title: value["title"].as_str().unwrap().to_string(),
                    subject: value["subject"]
                        .as_array()
                        .unwrap()
                        .into_iter()
                        .map(|subject| subject.as_str().unwrap().to_string())
                        .collect(),
                    description: value["description"].as_str().unwrap().to_string(),
                    downloads: value["downloads"].as_u64().unwrap(),
                    filename: value["filename"].as_str().unwrap().to_string(),
                    mtime: value["mtime"].as_u64().unwrap(),
                    size: value["size"].as_u64().unwrap(),
                    length: value["length"].as_u64().unwrap(),
                    comment: value["comment"].as_str().unwrap().to_string(),
                    slug: value["slug"].as_str().unwrap().to_string(),
                })
            },
            Err(e) => {
                error!("Cant parse post. {}", e);
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use simplelog::{LevelFilter, SimpleLogger, Config};
    use crate::models::episode::Episode;
    use log::debug;

    #[tokio::test]
    async fn test2(){
        let level_filter = LevelFilter::Trace;
        let _ = SimpleLogger::init(level_filter, Config::default());
        let episode = Episode::new("pihole.md").await;
        debug!("{:?}", episode);
        assert_eq!(episode.is_some(), true);
    }
}

