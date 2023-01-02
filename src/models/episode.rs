use serde::{Deserialize, Serialize};
use serde_json::Value;
use regex::Regex;
use log::{debug, info, error};

use super::{
    doc::Doc,
    metadata::Metadata,
    mp3metadata::Mp3Metadata
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

fn get_slug(title: &str) -> String{
    info!("Slug from: '{}'", title);
    let title: String = title
        .to_lowercase().
        chars()
        .map(|c| match c {
            'a'..='z'|'0'..='9' => c,
            'á'|'ä'|'à'|'â'     => 'a',
            'é'|'ë'|'è'|'ê'     => 'e',
            'í'|'ï'|'ì'|'î'     => 'i',
            'ó'|'ö'|'ò'|'ô'     => 'o',
            'ú'|'ü'|'ù'|'û'     => 'u',
            'ñ'                 => 'n',
            _                   => '-'
        })
        .collect();
    debug!("Slug step 1: '{}'", title);
    let re = Regex::new(r"\-{2,}").unwrap();
    let mut title = re.replace_all(&title, "-").to_string();
    debug!("Slug step 2: '{}'", title);
    let mut title = if title.starts_with("-"){
        title.remove(0).to_string();
        title
    }else{
        title
    };
    debug!("Slug step 3: '{}'", title);
    if title.ends_with("-"){
        title.pop();
        title
    }else{
        title.to_string()
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

