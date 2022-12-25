use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Site{
    pub author: String,
    pub name: String,
    pub description: String,
    pub baseurl: String,
    pub avatar: String,
    pub google_analytics: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FooterLinks{
    pub dribble: String,
    pub email: String,
    pub facebook: String,
    pub flickr: String,
    pub gitlab: String,
    pub instagram: String,
    pub linkedin: String,
    pub pinterest: String,
    pub rss: String,
    pub twitter: String,
    pub stackoverflow: String,
    pub youtube: String,
    pub googleplus: String,
    pub telegram: String,
    pub archiveorg: String,
    pub mastodon: String,
    pub android: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page{
    pub excerpt: String,
    pub title: String,
}

