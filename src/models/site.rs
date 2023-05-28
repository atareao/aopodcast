use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Site{
    pub author: String,
    pub title: String,
    pub description: String,
    pub podcast_feed: String,
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
    pub youtube: String,
    pub linktree: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    pub subject: Vec<String>,
    pub date: u64,
    pub pub_date: u64,
    pub identifier: String,
    pub filename: String,
    pub size: u64,
    pub length: u64,
    pub number: usize,
    pub downloads: u64,
}
