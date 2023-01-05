use serde::{Deserialize, Serialize};
use log::{debug, info, error};
use gray_matter::{Matter, engine::YAML};
use comrak::{markdown_to_html, ComrakOptions};

use crate::models::utils::{self, get_excerpt};

use super::{
    site::{Post, Layout},
    utils::{get_slug, get_unix_time},
};


#[derive(Debug, Serialize, Deserialize)]
struct Metadata{
    pub title: String,
    pub date: String,
    pub slug: String,
    pub update: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Article{
    pub metadata: Metadata,
    pub excerpt: String,
    pub content: String,
}

impl Metadata{
    pub fn get_filename(&self) -> String {
        format!("posts/{}.md", self.slug)
    }
}

impl Article{
    pub fn get_post(&self) -> Post{
        let identifier = get_slug(&self.metadata.title);
        let date = get_unix_time(&self.metadata.date);
        let content = markdown_to_html(&self.content, &ComrakOptions::default());
        Post{
            title: self.metadata.title.clone(),
            date,
            excerpt: self.excerpt.clone(),
            layout: Layout::POST,
            slug: self.metadata.slug.clone(),
            content,
            identifier: self.metadata.slug.clone(),
            filename: "".to_string(),
            length: 0,
        }
    }

    pub async fn new(filename: &str) -> Result<Self, serde_json::Error>{
        let filename = format!("posts/{}", filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&data);
        let mut metadata: Metadata = result.data.unwrap().deserialize()?;
        if metadata.slug.is_empty(){
            metadata.slug = utils::get_slug(&metadata.title);
            let ok_filename = metadata.get_filename();
            if filename != ok_filename{
                tokio::fs::rename(filename, ok_filename).await;
            }
        }
        let excerpt = match result.excerpt {
            Some(excerpt) => excerpt,
            None => get_excerpt(&result.content),
        };
        Ok(Self{
            metadata,
            excerpt,
            content: result.content,
        })
    }

    pub fn get_filename(&self) -> String{
        self.metadata.get_filename()
    }

    pub async fn save(&self){

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
