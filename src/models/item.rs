use regex::Regex;
use reqwest;
use serde::{Serialize, Deserialize};

use super::{metadata::Metadata, mp3metadata::Mp3Metadata};

#[derive(Debug, Serialize, Deserialize)]
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

impl Item {
    pub fn from_metadata(metadata: &Metadata, mp3metadata: &Mp3Metadata) -> Item{
        Self{
            identifier: metadata.identifier,
            mediatype: metadata.mediatype,
            collection: metadata.collection,
            subject: metadata.subject,
            description: metadata.description,
            filename: mp3metadata.filename,
            mtime: mp3metadata.mtime,
            size: mp3metadata.size,
            length: mp3metadata.length,
            title: mp3metadata.title,
            creator: mp3metadata.creator,
            album: mp3metadata.album,
            track: mp3metadata.track,
            artist: mp3metadata.artist,
            genre: mp3metadata.genre,
            comment: mp3metadata.comment,
            slug: "".to_string(),
            post_filename: "".to_string(),
            date: "".to_string(),
        }

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
    let since = "2022-09-01";
    let query = format!("creator:({creator}) AND date:[{since} TO 9999-12-31]", creator=creator, since=since);
    let params = [("q", query), ("fields", "identifier".to_string()), ("sorts", "date".to_string())];
    let url = reqwest::Url::parse_with_params("https://archive.org/services/search/v1/scrape", params).unwrap();
    let response = reqwest::get(url).await.unwrap();
    assert_eq!(response.status(), 200)
}


