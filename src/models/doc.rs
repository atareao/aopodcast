use serde::{Serialize, Deserialize};
use log::debug;
use tokio::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Doc{
    identifier: String,
    title: String,
    subject: Vec<String>,
    description: String,
    downloads: u64,
}

impl Doc{
    pub async fn exists(&self) -> bool{
        let file = format!("episodes/{}.md", self.identifier);
        match fs::metadata(&file).await{
            Ok(metadata) => {
                debug!("Output file {} exists", &file);
                metadata.is_file()
            },
            Err(e) => {
                debug!("Output dir {}, {}", &file, e);
                false
            },
        }
    }
}
