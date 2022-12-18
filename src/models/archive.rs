use serde::Deserialize;
use regex::Regex;
use super::item::Item;
use html_escape::decode_html_entities;
use html2md;

const BASE_URL: &'static str = "https://archive.org";

#[derive(Debug, Deserialize)]
pub struct BaseItem{
    identifier: String,
}

#[derive(Debug)]
struct ArchiveOrgClient{
    creator: String,
}

#[derive(Debug, Deserialize)]
struct Response{
    items: Vec<BaseItem>,
}

#[derive(Debug)]
struct Metadata{
    identifier: String,
    mediatype: String,
    collection: Vec<String>,
    subject: Vec<String>,
    description: String,
}

#[derive(Debug)]
struct Mp3Metadata{
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
}

impl ArchiveOrgClient{
    pub fn new(creator: &str) -> Self{
        Self{
            creator: creator.to_string(),
        }
    }
    pub async fn get_items(&self, since: &str) -> Vec<Item>{
        let mut items = Vec::new();
        let query = format!(
            "creator:({creator}) AND date:[{since} TO 9999-12-31]",
            creator=self.creator, since=since);
        let params = [
            ("q", query),
            ("fields", "identifier".to_string()),
            ("sorts", "date".to_string())
        ];
        let call_url = format!("{}/services/search/v1/scrape", BASE_URL);
        let url = reqwest::Url::parse_with_params(&call_url, params).unwrap();
        println!("url: {}", url);
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .unwrap();
        match response.status() {
            reqwest::StatusCode::OK => {
                match response.json::<Response>().await {
                    Ok(response) => {
                        println!("Response");
                        println!("{:?}", response.items);
                        for item in response.items{
                            let metadata = Self::get_metadata(&item.identifier).await;
                            let mp3_metadata = Self::get_mp3_metadata(&item.identifier).await;
                            println!("{:?}", metadata);

                        }
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                    },
                }
            }
            _ => {
                println!("Callinggg caca");
            }
        }
        return items
    }
    async fn get_mp3_metadata(identifier: &str) -> Option<Mp3Metadata>{
        let url = format!("{}/download/{identifier}/{identifier}_files.xml",
            BASE_URL, identifier=identifier);
        println!("url: {}", url);
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .unwrap();
        match response.status() {
            reqwest::StatusCode::OK => {
                match response.text().await{
                    Ok(value) => {
                        println!("{}", value);
                        let pattern_init = Regex::new(r#"^\s+<file name=".*\.mp3" source="original">"#).unwrap();
                        let pattern_end = Regex::new(r#"^\s+</file>"#).unwrap();
                        let mut mp3 = false;
                        let mut mp3_metadata: Vec<String> = Vec::new();
                        for line in value.lines(){
                            if !mp3 && pattern_init.is_match(line){
                                mp3 = true;
                            }
                            if mp3{
                                mp3_metadata.push(line.to_string());
                            }
                            if mp3 && pattern_end.is_match(line){
                                break;
                            }
                        }
                        println!("{:?}", mp3_metadata);
                        Some(Self::extract_mp3_metadata(mp3_metadata))
                    },
                    Err(_) => None,
                }
            }
            _ => {
                None
            }
        }
    }
    async fn get_metadata(identifier: &str) -> Option<Metadata>{
        let url = format!("{}/download/{identifier}/{identifier}_meta.xml",
            BASE_URL, identifier=identifier);
        println!("url: {}", url);
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .unwrap();
        match response.status() {
            reqwest::StatusCode::OK => {
                match response.text().await{
                    Ok(value) => {
                        let mediatype = Self::get("mediatype", &value).get(0).unwrap().to_string();
                        let collection = Self::get("collection", &value);
                        let subject = Self::get("subject", &value);
                        let description = html2md::parse_html(
                            &decode_html_entities(
                                &Self::get("description", &value)
                                .get(0)
                                .unwrap()
                                .to_string()).to_string());
                        Some(Metadata{
                            identifier: identifier.to_string(),
                            mediatype,
                            collection,
                            subject,
                            description,
                        })
                    },
                    Err(_) => None,
                }
            }
            _ => {
                None
            }
        }
    }
    fn get(tag: &str, xml: &str) -> Vec<String>{
        let mut result = Vec::new();
        let pattern = format!("<{tag}>([^<]*)</{tag}>", tag=tag);
        let re = Regex::new(&pattern).unwrap();

        for cap in re.captures_iter(xml){
            result.push(cap.get(1).unwrap().as_str().to_string());
        }
        result
    }
    fn extract_mp3_metadata(data: Vec<String>) -> Mp3Metadata{
        let text = data.concat();
        let mtime = Self::get("mtime", &text).get(0).unwrap().to_string();
        let size = Self::get("size", &text).get(0).unwrap().to_string();
        let length = Self::get("length", &text).get(0).unwrap().to_string();
        let title = Self::get("title", &text).get(0).unwrap().to_string();
        let creator = Self::get("creator", &text).get(0).unwrap().to_string();
        let album = Self::get("album", &text).get(0).unwrap().to_string();
        let track = Self::get("track", &text).get(0).unwrap().to_string();
        let artist = Self::get("artist", &text).get(0).unwrap().to_string();
        let genre = Self::get("genre", &text).get(0).unwrap().to_string();
        let comment = Self::get("comment", &text).get(0).unwrap().to_string();
        let pattern = r#"<file name="([^"]*)" source="original">"#;
        let re = Regex::new(pattern).unwrap();
        let caps = re.captures(&text).unwrap();
        let filename = caps.get(1).unwrap().as_str().to_string();
        Mp3Metadata{
            filename,
            mtime,
            size,
            length,
            title,
            creator,
            album,
            track,
            artist,
            genre,
            comment,
        }
    }
}

#[tokio::test]
async fn test(){
    let aoclient = ArchiveOrgClient::new("Papa Friki");
    let items = aoclient.get_items("2022-10-01");
    assert!(items.await.len() > 0)
}
