use serde::{Deserialize, Serialize};
use serde_json::Value;
use log::{debug, info, error};


#[derive(Debug, Serialize, Deserialize)]
pub struct Episode{
    // from doc
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
    pub async fn new(path: &str, filename: &str) -> Option<Self>{
        let filename = format!("{}/{}", path, filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        Self::parse(&data)
    }

    fn parse(data: &str) -> Option<Episode>{
        match serde_yaml::from_str::<Value>(&data) {
            Ok(value) => {
                debug!("Value: {:?}", value);
                Some(Self{
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
        let episode = Episode::new("episodes", "pihole.md").await;
        debug!("{:?}", episode);
        assert_eq!(episode.is_some(), true);
    }
}

