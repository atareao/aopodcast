use serde::Deserialize;
use crate::models::{metadata::Metadata, mp3metadata::Mp3Metadata};
use super::item::Item;

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
                            let metadata_result = Self::get_metadata(&item.identifier).await;
                            let mp3_metadata_result = Self::get_mp3_metadata(&item.identifier).await;
                            if metadata_result.is_some() && mp3_metadata_result.is_some(){
                                let metadata = Metadata::new(&metadata_result.unwrap());
                                let mp3_metadata = Mp3Metadata::new(&mp3_metadata_result.unwrap());
                                let item = Item::from_metadata(&metadata, &mp3_metadata);
                                println!("Item: {}", &item);
                                items.push(item);
                            }
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

    async fn get_mp3_metadata(identifier: &str) -> Option<String>{
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
    let items = aoclient.get_items("2022-10-01");
    assert!(items.await.len() > 0)
}
