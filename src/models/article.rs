use serde::{Deserialize, Serialize};
use log::{debug, info, error};
use gray_matter::{Matter, engine::YAML};
use comrak::{markdown_to_html, ComrakOptions};

use super::{
    site::{Post, Layout},
    utils::{
        get_slug,
        get_unix_time,
        get_excerpt,
    },
};


#[derive(Debug, Serialize, Deserialize)]
struct Metadata{
    pub title: String,
    pub date: String,
    pub excerpt: String,
    pub slug: String,
}

impl Metadata{
    pub fn get_filename(&self) -> String {
        format!("posts/{}.md", self.slug)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Article{
    metadata: Metadata,
    pub content: String,
}


impl Article{
    pub fn get_post(&self) -> Post{
        let date = get_unix_time(&self.metadata.date);
        let content = markdown_to_html(&self.content, &ComrakOptions::default());
        Post{
            title: self.metadata.title.clone(),
            date,
            excerpt: self.metadata.excerpt.clone(),
            layout: Layout::POST,
            slug: self.metadata.slug.clone(),
            subject: Vec::new(),
            content,
            identifier: self.metadata.slug.clone(),
            filename: "".to_string(),
            length: 0,
            size: 0,
            number: 0,
            downloads: 0,
        }
    }

    pub async fn new(filename: &str) -> Result<Self, serde_json::Error>{
        let mut save = false;
        let filename = format!("posts/{}", filename);
        debug!("Filename: {}", filename);
        let data = tokio::fs::read_to_string(&filename)
            .await
            .unwrap();
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&data);
        let mut metadata: Metadata = result.data.unwrap().deserialize()?;
        debug!("Metadata: {:?}", &metadata);
        if metadata.slug.is_empty(){
            debug!("Is empty");
            metadata.slug = get_slug(&metadata.title);
            save = true;
        }
        if metadata.excerpt.is_empty(){
            metadata.excerpt = match result.excerpt {
                Some(excerpt) => {
                    save = true;
                    excerpt
                },
                None => get_excerpt(&result.content).to_string(),
            };
        }
        debug!("Metadata: {:?}", &metadata);
        let article = Article{
            metadata,
            content: result.content,
        };
        if save{
            match article.save().await{
                Ok(_) => {
                    info!("Saved article {}", article.get_filename());
                    if filename != article.get_filename(){
                        match tokio::fs::remove_file(&filename).await{
                            Ok(_) => info!("Removed {}", &filename),
                            Err(e) => error!("Cant remove {}. {}", &filename, e),
                        }
                    }
                },
                Err(_) => error!("Cant save article {}", article.get_filename()),
            }
        }
        Ok(article)
    }

    pub fn get_filename(&self) -> String{
        self.metadata.get_filename()
    }

    pub async fn save(&self)-> tokio::io::Result<()>{
        let mut content = String::new();
        content.push_str("---\n");
        content.push_str(&serde_yaml::to_string(&self.metadata).unwrap());
        content.push_str("---\n");
        content.push_str(&self.content);
        debug!("Content: {}", content);
        tokio::fs::write(self.get_filename(), content).await
    }
}

#[cfg(test)]
mod tests {
    use simplelog::{LevelFilter, SimpleLogger, Config};
    use crate::models::article::Article;
    use log::debug;

    #[tokio::test]
    async fn test_article(){
        let level_filter = LevelFilter::Debug;
        let _ = SimpleLogger::init(level_filter, Config::default());
        let article = Article::new("pihole.md").await.unwrap();
        debug!("=========================");
        debug!("{:?}", article);
        debug!("=========================");
        assert_eq!(article.metadata.title.is_empty(), false);
    }
}
