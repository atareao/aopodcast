use serde::{Deserialize, Serialize};
use log::{debug, info, error};
use gray_matter::{Matter, engine::YAML};
use comrak::{markdown_to_html, ComrakOptions};

use super::{
    site::{Post, Layout},
    utils::{get_slug,
            get_unix_time,
            get_excerpt,
    },
};


#[derive(Debug, Serialize, Deserialize)]
struct Metadata{
    pub title: String,
    pub date: String,
    pub slug: String,
}

impl Metadata{
    pub fn get_filename(&self) -> String {
        format!("pages/{}.md", self.slug)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page{
    metadata: Metadata,
    pub excerpt: String,
    pub content: String,
}

impl Page{
    pub fn get_post(&self) -> Post{
        let date = get_unix_time(&self.metadata.date);
        let content = markdown_to_html(&self.content, &ComrakOptions::default());
        Post{
            title: self.metadata.title.clone(),
            date,
            excerpt: self.excerpt.clone(),
            layout: Layout::PAGE,
            slug: self.metadata.slug.clone(),
            content,
            identifier: self.metadata.slug.clone(),
            filename: "".to_string(),
            length: 0,
        }
    }

    pub async fn new(filename: &str) -> Result<Self, serde_json::Error>{
        let mut save = false;
        let filename = format!("pages/{}", filename);
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
        let excerpt = match result.excerpt {
            Some(excerpt) => {
                save = true;
                excerpt
            },
            None => get_excerpt(&result.content),
        };
        debug!("Metadata: {:?}", &metadata);
        let page = Page{
            metadata,
            excerpt,
            content: result.content,
        };
        if save{
            match page.save().await{
                Ok(_) => {
                    info!("Saved page {}", page.get_filename());
                    if filename != page.get_filename(){
                        match tokio::fs::remove_file(&filename).await{
                            Ok(_) => info!("Removed {}", &filename),
                            Err(e) => error!("Cant remove {}. {}", &filename, e),
                        }
                    }
                },
                Err(_) => error!("Cant save page {}", page.get_filename()),
            }
        }
        Ok(page)
    }

    pub fn get_filename(&self) -> String{
        self.metadata.get_filename()
    }

    pub fn get_title(&self) -> String{
        self.metadata.title.to_string()
    }

    pub fn get_date(&self) -> String{
        self.metadata.date.to_string()
    }

    pub fn get_slug(&self) -> String{
        self.metadata.slug.to_string()
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
    use crate::models::page::Page;
    use log::debug;

    #[tokio::test]
    async fn test_page(){
        let level_filter = LevelFilter::Trace;
        let _ = SimpleLogger::init(level_filter, Config::default());
        let page = Page::new("about.md").await.unwrap();
        debug!("Title: {}", page.get_title());
        debug!("=========================");
        debug!("{:?}", page);
        debug!("=========================");
        assert_eq!(page.get_title().is_empty(), false);
    }
}

