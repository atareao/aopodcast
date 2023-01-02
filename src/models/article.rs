use serde::{Deserialize, Serialize};
use serde_json::Value;
use log::{debug, info, error};

use super::{
    site::{Post, Layout},
    utils::{get_slug, get_unix_time},
};


#[derive(Debug, Serialize, Deserialize)]
pub struct Article{
    pub slug: String,
    pub title: String,
    pub date: String,
    pub excerpt: String,
    pub content: String,
    pub filename: String,
}

impl Article{
    pub fn get_post(&self) -> Post{
        let slug = get_slug(&self.title);
        let identifier = get_slug(&self.title);
        let date = get_unix_time(&self.date);
        Post{
            layout: Layout::POST,
            slug,
            excerpt: self.excerpt.clone(),
            title: self.title.clone(),
            content: self.content.clone(),
            date,
            identifier,
            filename: "".to_string(),
            length: 0,
        }
    }
    pub async fn new(filename: &str) -> Option<Self>{
        let filename = format!("posts/{}", filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        Self::parse(&data, &filename)
    }

    fn parse(data: &str, filename: &str) -> Option<Article>{
        match serde_yaml::from_str::<Value>(&data) {
            Ok(value) => {
                info!("Filename: {}", filename);
                debug!("Value: {:?}", value);
                let slug = filename.replace(".yml", "");
                Some(Self{
                    slug,
                    title: value["title"].to_string(),
                    date: value["date"].to_string(),
                    excerpt: value["excerpt"].to_string(),
                    content: value["content"].to_string(),
                    filename: filename.to_string(),
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
    use crate::models::article::Article;
    use log::debug;

    #[tokio::test]
    async fn test1(){
        let level_filter = LevelFilter::Trace;
        let _ = SimpleLogger::init(level_filter, Config::default());
        let article = Article::new("pihole.md").await;
        debug!("{:?}", article);
        assert_eq!(article.is_some(), true);
    }
}
