use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use crate::models::{metadata::Metadata, mp3metadata::Mp3Metadata};
use super::item::Item;
use std::env;

const BASE_URL: &'static str = "https://archive.org";

#[derive(Debug, Deserialize)]
pub struct BaseItem{
    identifier: String,
}

#[derive(Debug)]
pub struct ArchiveOrgClient{
    creator: String,
}

#[derive(Debug, Deserialize)]
struct Response{
    items: Vec<BaseItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Items{
    items: Vec<Item>,
}

impl Items{
    pub fn new(items: Vec<Item>) -> Items{
        Self{items,}
    }
    pub fn len(&self) -> usize{
        self.items.len()
    }
    pub async fn read_saved_items() -> Items{
        let mut path = env::current_exe().unwrap();
        path.pop();
        path.push("data");
        path.push("podcasts.json");
        match tokio::fs::metadata(&path).await{
            Ok(metadata) =>{
                if metadata.is_file(){
                    let data = tokio::fs::read_to_string(&path).await.unwrap();
                    serde_json::from_str::<Items>(&data).unwrap()
                }else{
                    Items::new(Vec::new())
                }
            },
            Err(_) => Items::new(Vec::new())
        }
    }

    pub async fn save_items(&self) -> Result<(), std::io::Error>{
        let mut path = env::current_exe().unwrap();
        path.pop();
        path.push("data");
        path.push("podcasts.json");
        tokio::fs::write(
            path,
            serde_json::to_string_pretty(&self).unwrap(),
        ).await
    }
    pub fn exists(&self, other: &Item) -> bool{
        for item in self.items.as_slice(){
            if item.get_identifier() == other.get_identifier(){
                return true;
            }
        }
        false
    }
    pub fn add(&mut self, items: &Vec<Item>){
        for item in items{
            if !self.exists(item){
                self.items.push(item.clone())
            }
        }
    }

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
        info!("url: {}", url);
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
                        for item in response.items{
                            let metadata_result = Self::get_metadata(&item.identifier).await;
                            let mp3_metadata_result = Self::get_mp3_metadata(&item.identifier).await;
                            if metadata_result.is_some() && mp3_metadata_result.is_some(){
                                let metadata = Metadata::new(&metadata_result.unwrap());
                                let mp3_metadata = Mp3Metadata::new(&mp3_metadata_result.unwrap());
                                let item = Item::from_metadata(&metadata, &mp3_metadata);
                                items.push(item);
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error: {:?}", e);
                    },
                }
            }
            _ => {
                warn!("Nothing found?");
            }
        }
        items
    }

    async fn get_mp3_metadata(identifier: &str) -> Option<String>{
        let url = format!("{}/download/{identifier}/{identifier}_files.xml",
            BASE_URL, identifier=identifier);
        info!("url: {}", url);
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .unwrap();
        match response.status() {
            reqwest::StatusCode::OK => {
                match response.text().await{
                    Ok(value) => Some(value),
                    Err(_) => None,
                }
            }
            _ => {
                None
            }
        }
    }

    async fn get_metadata(identifier: &str) -> Option<String>{
        let url = format!("{}/download/{identifier}/{identifier}_meta.xml",
            BASE_URL, identifier=identifier);
        info!("url: {}", url);
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .unwrap();
        match response.status() {
            reqwest::StatusCode::OK => {
                match response.text().await{
                    Ok(value) => Some(value),
                    Err(_) => None,
                }
            }
            _ => {
                None
            }
        }
    }
}

#[tokio::test]
async fn test(){
    let aoclient = ArchiveOrgClient::new("Papa Friki");
    let items = aoclient.get_items("2022-12-01").await;
    if items.len() > 0{
        println!("{}", items.get(0).unwrap());
    }
    assert!(items.len() > 0)
}
