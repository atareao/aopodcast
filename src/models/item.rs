use regex::Regex;
use reqwest;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Item{
    identifier: String,
    mediatype: String,
    collection: String,
    subject: String,
    date: String,
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
}

impl Item {

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


