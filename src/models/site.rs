use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Site{
    pub author: String,
    pub title: String,
    pub description: String,
    pub baseurl: String,
    pub url: String,
    pub avatar: String,
    pub category: String,
    pub subcategory: String,
    pub explicit: bool,
    pub email: String,
    pub gitlab: String,
    pub rss: String,
    pub twitter: String,
    pub google_analytics: String,
    pub disqus: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Layout{
    POST,
    PODCAST,
    PAGE,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post{
    pub layout: Layout,
    pub slug: String,
    pub excerpt: String,
    pub title: String,
    pub content: String,
    pub date: u64,
    pub identifier: String,
    pub filename: String,
    pub length: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Podcast{
    pub slug: String,
    pub excerpt: String,
    pub title: String,
    pub content: String,
    pub date: u64,
    pub identifier: String,
    pub filename: String,
    pub length: String,
}
