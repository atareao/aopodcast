use log::{info, warn, error};
use serde::{Serialize, Deserialize, Deserializer};
use serde_json::Value;
use crate::models::{
    metadata::AOMetadata,
    mp3metadata::Mp3Metadata,
    doc::Doc,
};
use async_recursion::async_recursion;
use log::debug;

const BASE_URL: &'static str = "https://archive.org";
const PAGESIZE: usize = 200;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveOrg{
    pub creator: String,
    pub link: String,
    #[serde(deserialize_with = "deserialize_on_empty")]
    pub subject: Option<String>,
}

fn deserialize_on_empty<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where D: Deserializer<'de>{
        let o: Option<String> = Option::deserialize(deserializer)?;
        Ok(o.filter(|s| !s.is_empty()))
}

impl ArchiveOrg{
    pub fn new(creator: &str, link: &str, subject: Option<String>) -> Self{
        Self{
            creator: creator.to_string(),
            link: link.to_string(),
            subject,
        }
    }

    pub async fn get_all_docs(&self) -> Vec<Doc>{
        let since = "1970-01-01";
        let page = 1;
        self.get_docs(since, page).await
    }

    #[async_recursion]
    async fn get_docs(&self, since: &str, page: usize) -> Vec<Doc>{
        let mut items = Vec::new();
        let optional = match &self.subject{
            Some(value) => format!("AND subject:({})", value.to_string()),
            None => "".to_string(),
        };
        let fields: String = vec!["description", "downloads", "identifier",
            "item_size", "name", "publicdate",
            "publisher", "subject", "title"]
            .into_iter()
            .map(|field| format!("fl[]={}", field))
            .collect::<Vec<String>>()
        .join("&");
        
        let sort = "publicdate asc";
        let output = "json";
        let url = format!("{base}/advancedsearch.php?q=creator:({creator}) \
            AND date:[{since} TO 9999-12-31] \
            AND mediatype:(audio) \
            AND format:(VBR MP3) \
            {optional} \
            &{fields}\
            &sort[]={sort}\
            &output={output}\
            &rows={rows}\
            &page={page}",
            base=BASE_URL, creator=self.creator, since=since,
            optional=optional, fields=fields,sort=sort, output=output,
            rows=PAGESIZE, page=page);
        let client = reqwest::Client::new();
        info!("url: {}", url);
        let response = client
            .get(url)
            .send()
            .await
            .unwrap();
        match response.status() {
            reqwest::StatusCode::OK => {
                match response.json::<Value>().await {
                    Ok(value) => {
                        let response = &value["response"];
                        let num_found = response["numFound"].as_u64().unwrap();
                        let start = response["start"].as_u64().unwrap();
                        debug!("Page: {}", page);
                        debug!("Start: {}", start);
                        debug!("Found: {}", num_found);
                        let pagesize: u64 = PAGESIZE.try_into().unwrap();
                        if num_found > start + pagesize {
                            debug!("Recursion");
                            let new_page = page + 1;
                            debug!("Page: {}", new_page);
                            let mut more_items = self.get_docs(since, new_page).await;
                            items.append(&mut more_items)
                        }
                        for (i, doc) in response["docs"].as_array().unwrap().iter().enumerate(){
                            //debug!("Doc: {:?}", doc);
                            let mut doc: Doc = serde_json::from_value(doc.clone()).unwrap();
                            let number = i + 1 + (page - 1) * PAGESIZE;
                            debug!("Doc {}. Number: {} => {}", doc.get_identifier(), i, number);
                            doc.set_number(number);
                            items.push(doc);
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

    pub async fn get_mp3_metadata(identifier: &str) -> Option<Mp3Metadata>{
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
                    Ok(content) => Mp3Metadata::new(&content),
                    Err(_) => None,
                }
            }
            _ => {
                None
            }
        }
    }

    pub async fn get_metadata(identifier: &str) -> Option<AOMetadata>{
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
                    Ok(content) => Some(AOMetadata::new(&content)),
                    Err(_) => None,
                }
            }
            _ => {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use simplelog::{LevelFilter, SimpleLogger, Config};
    use crate::models::archive::ArchiveOrg;
    use log::debug;

    #[tokio::test]
    async fn test_get_docs(){
        let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());

        let aoclient = ArchiveOrg::new(
            "Papá Friki",
            "papafiki",
            None);
        let docs = aoclient.get_docs("1970-01-01", 1).await;
        if docs.len() > 0{
            debug!("{:?}", docs.get(0).unwrap());
        }
        assert!(docs.len() > 0)
    }
}
