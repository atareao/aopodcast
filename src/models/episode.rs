use serde::{Deserialize, Serialize};
use serde_json::Value;
use log::{debug, info, error};


#[derive(Debug, Serialize, Deserialize)]
pub struct Episode{
    pub identifier: String,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content: String,
    pub filename: String,
    pub audio: String,
    pub mtime: String,
    pub date: String,
    pub size: String,
    pub length: String,
    pub tags: Vec<String>,
}

impl Episode{
    pub async fn new(path: &str, filename: &str) -> Option<Self>{
        let filename = format!("{}/{}", path, filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        Self::parse(&data, &filename)
    }

    fn parse(data: &str, filename: &str) -> Option<Episode>{
        match serde_yaml::from_str::<Value>(&data) {
            Ok(value) => {
                info!("Filename: {}", filename);
                debug!("Value: {:?}", value);
                Some(Self{
                    identifier: value["identifier"].to_string(),
                    title: value["title"].to_string(),
                    slug: value["slug"].to_string(),
                    excerpt: value["excerpt"].to_string(),
                    content: value["content"].to_string(),
                    filename: value["filename"].to_string(),
                    audio: value["audio"].to_string(),
                    mtime: value["mtime"].to_string(),
                    date: value["date"].to_string(),
                    size: value["size"].to_string(),
                    length: value["length"].to_string(),
                    tags: value["tags"].as_array()
                            .unwrap()
                            .into_iter()
                            .map(|item| item.as_str().unwrap().to_string())
                            .collect(),
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

