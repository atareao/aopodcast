use log::{info, warn, error};
use serde::{Serialize, Deserialize, Deserializer};
use crate::models::{metadata::Metadata, mp3metadata::Mp3Metadata};
use super::item::Item;

const BASE_URL: &'static str = "https://archive.org";

#[derive(Debug, Deserialize)]
pub struct BaseItem{
    identifier: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveOrg{
    pub uploader: String,
    pub identifier: String,
    #[serde(deserialize_with = "deserialize_on_empty")]
    pub subject: Option<String>,
}

fn deserialize_on_empty<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where D: Deserializer<'de>{
        let o: Option<String> = Option::deserialize(deserializer)?;
        Ok(o.filter(|s| !s.is_empty()))
}

#[derive(Debug, Deserialize)]
struct Response{
    items: Vec<BaseItem>,
}

impl ArchiveOrg{
    pub fn new(uploader: &str, identifier: &str, subject: Option<String>) -> Self{
        Self{
            uploader: uploader.to_string(),
            identifier: identifier.to_string(),
            subject,
        }
    }


    pub async fn get_items(&self, since: &str) -> Vec<Item>{
        let mut items = Vec::new();
        let optional = match &self.subject{
            Some(value) => format!("AND subject:({})", value.to_string()),
            None => "".to_string(),
        };
        let query = format!(
            "creator:({creator}) AND date:[{since} TO 9999-12-31] 
                AND mediatype:(audio) {optional}",
            creator=self.uploader, since=since, optional=optional);
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
                                if let Some(mp3_metadata) = Mp3Metadata::new(&mp3_metadata_result.unwrap()){
                                    let item = Item::from_metadata(&metadata, &mp3_metadata);
                                    items.push(item);
                                }
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
    let aoclient = ArchiveOrg::new(
        "atareao@atareao.es",
        "atareao",
        Some("atareao".to_string()));
    let items = aoclient.get_items("2022-12-01").await;
    if items.len() > 0{
        println!("{}", items.get(0).unwrap());
    }
    assert!(items.len() > 0)
}
