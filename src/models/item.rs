use std::fmt::Display;

use chrono::{DateTime, Utc, NaiveDateTime};
use regex::Regex;
use std::fmt;
use serde::{Serialize, Deserialize};
use comrak::{markdown_to_html, ComrakOptions};

use super::{metadata::Metadata, mp3metadata::Mp3Metadata, site::Page};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item{
    identifier: String,
    mediatype: String,
    collection: Vec<String>,
    subject: Vec<String>,
    description: String,
    filename: String,
    mtime: String,
    size: String,
    length: String,
    title: String,
    creator: String,
    album: String,
    track: String,
    artist: String,
    genre: String,
    comment: String,
    slug: String,
    post_filename: String,
    date: String,
}

impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identifier: {}\n
                   Mediatype: {}\n
                   Collection: {}\n
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
                   Post_filename: {}\n
                   Date: {}\n",
            self.identifier, self.mediatype, self.collection.concat(),
            self.subject.concat(), self.description, self.filename, self.mtime,
            self.size, self.length, self.title, self.creator, self.album,
            self.track, self.artist, self.genre, self.comment, self.slug,
            self.post_filename, self.date)
    }
}

impl Item {
    pub fn get_page(&self) -> Page{
        let content = markdown_to_html(&self.description, &ComrakOptions::default());
        Page{
            excerpt: self.comment.clone(),
            title: self.title.clone(),
            content,
            date: self.mtime.parse::<u64>().unwrap(),
        }
    }
    pub fn from_metadata(metadata: &Metadata, mp3metadata: &Mp3Metadata) -> Item{
        Self{
            identifier: metadata.identifier.to_string(),
            mediatype: metadata.mediatype.to_string(),
            collection: metadata.collection.clone(),
            subject: metadata.subject.clone(),
            description: metadata.description.to_string(),
            filename: mp3metadata.filename.to_string(),
            mtime: mp3metadata.mtime.to_string(),
            size: mp3metadata.size.to_string(),
            length: mp3metadata.length.to_string(),
            title: mp3metadata.title.to_string(),
            creator: mp3metadata.creator.to_string(),
            album: mp3metadata.album.to_string(),
            track: mp3metadata.track.to_string(),
            artist: mp3metadata.artist.to_string(),
            genre: mp3metadata.genre.to_string(),
            comment: mp3metadata.comment.to_string(),
            slug: "".to_string(),
            post_filename: "".to_string(),
            date: "".to_string(),
        }

    }
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

    fn get(tag: &str, xml: &str) -> Vec<String>{
        let mut result = Vec::new();
        let pattern = format!("<{tag}>([^<]*)</{tag}>", tag=tag);
        let re = Regex::new(&pattern).unwrap();

        for cap in re.captures_iter(xml){
            for item in cap.iter(){
                result.push(item.unwrap().as_str().to_string());
            }
        }

        result
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
