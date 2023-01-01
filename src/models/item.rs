use std::fmt::Display;
use core::cmp::Ordering;

use chrono::{DateTime, Utc, NaiveDateTime};
use regex::Regex;
use std::fmt;
use serde::{Serialize, Deserialize};
use comrak::{markdown_to_html, ComrakOptions};
use log::{debug, info};

use super::{
    metadata::Metadata,
    mp3metadata::Mp3Metadata,
    site::{
        Post,
        Podcast,
    },
    article::Article
};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord)]
pub struct Item{
    identifier: String,
    title: String,
    subject: Vec<String>,
    description: String,
    downloads: usize,
    item_size: u64,
    publicdate: String,
    filename: String,
    mtime: String,
    size: String,
    length: String,
    creator: String,
    album: String,
    track: String,
    artist: String,
    genre: String,
    comment: String,
    slug: String,
    date: String,
}

impl From<Article> for Item{
    fn from(article: Article) -> Self{
        Self{
            identifier: article.filename.to_string(),
            title: article.title.to_string(),
            subject: Vec::new(),
            description: article.content,
            downloads: 0,
            item_size: 0,
            publicdate: "".to_string(),
            filename: article.filename.to_string(),
            mtime: "".to_string(),
            size: "".to_string(),
            length: "".to_string(),
            creator: "".to_string(),
            album: "".to_string(),
            track: "".to_string(),
            artist: "".to_string(),
            genre: "".to_string(),
            comment: article.excerpt.to_string(),
            slug: article.filename.to_string(),
            date: "".to_string(),
        }
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.mtime == other.mtime {
            return Some(other.mtime.cmp(&self.mtime));
        }
        Some(other.mtime.cmp(&self.mtime))
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identifier: {}\n
                   Subject: {}\n
                   Description: {}\n
                   Filename: {}\n
                   Mtime: {}\n
                   Size: {}\n
                   Length: {}\n
                   Title: {}\n
                   Creator: {}\n
                   Album: {}\n
                   Track: {}\n
                   Artist: {}\n
                   Genre: {}\n
                   Comment: {}\n
                   Slug: {}\n
                   Date: {}\n",
            self.identifier,
            self.subject.concat(), self.description, self.filename, self.mtime,
            self.size, self.length, self.title, self.creator, self.album,
            self.track, self.artist, self.genre, self.comment, self.slug,
            self.date)
    }
}

impl Item {
    pub fn get_post(&self) -> Post{
        let content = markdown_to_html(&self.description, &ComrakOptions::default());
        let date = self.get_mtime().parse::<u64>().unwrap();
        let slug = if self.slug.is_empty(){
            get_slug(&self.title)
        }else{
            self.slug.clone()
        };
        Post{
            slug,
            excerpt: self.comment.clone(),
            title: self.title.clone(),
            content,
            date,
        }
    }
    pub fn get_pocast(&self) -> Podcast{
        let content = markdown_to_html(&self.description, &ComrakOptions::default());
        let date = self.get_mtime().parse::<u64>().unwrap();
        let slug = if self.slug.is_empty(){
            get_slug(&self.title)
        }else{
            self.slug.clone()
        };
        Podcast{
            identfier: self.identifier.clone(),
            slug,
            excerpt: self.comment.clone(),
            title: self.title.clone(),
            content,
            date,
            filename: self.filename.clone(),
            length: self.length.clone(),
        }

    }
    //pub fn from_metadata(metadata: &Metadata, mp3metadata: &Mp3Metadata) -> Item{
    //    Self{
    //        identifier: metadata.identifier.to_string(),
    //        subject: metadata.subject.clone(),
    //        description: metadata.description.to_string(),
    //        filename: mp3metadata.filename.to_string(),
    //        mtime: mp3metadata.mtime.to_string(),
    //        size: mp3metadata.size.to_string(),
    //        length: mp3metadata.length.to_string(),
    //        title: mp3metadata.title.to_string(),
    //        creator: mp3metadata.creator.to_string(),
    //        album: mp3metadata.album.to_string(),
    //        track: mp3metadata.track.to_string(),
    //        artist: mp3metadata.artist.to_string(),
    //        genre: mp3metadata.genre.to_string(),
    //        comment: mp3metadata.comment.to_string(),
    //        slug: get_slug(&mp3metadata.title),
    //        date: get_date(&mp3metadata.mtime),
    //    }

    //}
    pub fn get_mtime(&self) -> &str{
        &self.mtime
    }

    pub fn get_identifier(&self) -> &str{
        &self.identifier
    }

    pub fn get_date(&self) -> DateTime<Utc>{
        let timestamp = self.get_mtime().parse::<i64>().unwrap();
        let naive_date_time = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
        DateTime::<Utc>::from_utc(naive_date_time, Utc)
    }
}

fn get_date(mtime: &str) -> String{
    let timestamp = mtime.parse::<i64>().unwrap();
    let naive_date_time = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let date = DateTime::<Utc>::from_utc(naive_date_time, Utc);
    date.format("%Y-%m-%d").to_string()
}

fn get_slug(title: &str) -> String{
    info!("Slug from: '{}'", title);
    let title: String = title
        .to_lowercase().
        chars()
        .map(|c| match c {
            'a'..='z'|'0'..='9' => c,
            'á'|'ä'|'à'|'â'     => 'a',
            'é'|'ë'|'è'|'ê'     => 'e',
            'í'|'ï'|'ì'|'î'     => 'i',
            'ó'|'ö'|'ò'|'ô'     => 'o',
            'ú'|'ü'|'ù'|'û'     => 'u',
            'ñ'                 => 'n',
            _                   => '-'
        })
        .collect();
    debug!("Slug step 1: '{}'", title);
    let re = Regex::new(r"\-{2,}").unwrap();
    let mut title = re.replace_all(&title, "-").to_string();
    debug!("Slug step 2: '{}'", title);
    let mut title = if title.starts_with("-"){
        title.remove(0).to_string();
        title
    }else{
        title
    };
    debug!("Slug step 3: '{}'", title);
    if title.ends_with("-"){
        title.pop();
        title
    }else{
        title.to_string()
    }
}

#[tokio::test]
async fn test1(){
    let creator = "Papa Friki";
    let since = "2022-12-01";
    let query = format!("creator:({creator}) AND date:[{since} TO 9999-12-31]", creator=creator, since=since);
    let params = [("q", query), ("fields", "identifier".to_string()), ("sorts", "date".to_string())];
    let url = reqwest::Url::parse_with_params("https://archive.org/services/search/v1/scrape", params).unwrap();
    let response = reqwest::get(url).await.unwrap();
    assert_eq!(response.status(), 200)
}
