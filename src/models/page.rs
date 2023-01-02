use serde::{Deserialize, Serialize};
use serde_json::Value;
use log::{debug, info, error};
use comrak::{markdown_to_html, ComrakOptions};

use super::{
    site::{Post, Layout},
    utils::{get_slug, get_unix_time},
};


#[derive(Debug, Serialize, Deserialize)]
pub struct Page{
    pub slug: String,
    pub title: String,
    pub date: String,
    pub excerpt: String,
    pub content: String,
    pub filename: String,
}

impl Page{
    pub fn get_post(&self) -> Post{
        let slug = get_slug(&self.title);
        let identifier = get_slug(&self.title);
        let date = get_unix_time(&self.date);
        let content = markdown_to_html(&self.content, &ComrakOptions::default());
        Post{
            layout: Layout::PAGE,
            slug,
            excerpt: self.excerpt.clone(),
            title: self.title.clone(),
            content,
            date,
            identifier,
            filename: "".to_string(),
            length: 0,
        }
    }
    pub async fn new(filename: &str) -> Option<Self>{
        let filename = format!("pages/{}", filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        Self::parse(&data, &filename)
    }

    fn parse(data: &str, filename: &str) -> Option<Page>{
        match serde_yaml::from_str::<Value>(&data) {
            Ok(value) => {
                info!("Filename: {}", filename);
                debug!("Value: {:?}", value);
                let slug = filename.replace(".yml", "");
                Some(Self{
                    slug,
                    title: value["title"].as_str().unwrap().to_string(),
                    date: value["date"].as_str().unwrap().to_string(),
                    excerpt: value["excerpt"].as_str().unwrap().to_string(),
                    content: value["content"].as_str().unwrap().to_string(),
                    filename: filename.to_string(),
                })
            },
            Err(e) => {
                error!("Cant parse page. {}", e);
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use simplelog::{LevelFilter, SimpleLogger, Config};
    use crate::models::page::Page;
    use log::debug;

    #[tokio::test]
    async fn test_page(){
        let level_filter = LevelFilter::Trace;
        let _ = SimpleLogger::init(level_filter, Config::default());
        let page = Page::new("about.md").await;
        debug!("{:?}", page);
        assert_eq!(page.is_some(), true);
    }
}

