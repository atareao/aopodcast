use reqwest::Client;
use std::format;
use serde::{Serialize, Deserialize};

pub struct Mastodon{
    base_uri: String,
    access_token: String,
}

#[derive(Serialize, Deserialize)]
struct Message{
    status: String,
    in_reply_to_id: Option<String>,
}

impl Mastodon{
    pub fn new(base_uri: &str, access_token: &str) -> Self{
        Mastodon {
            base_uri: base_uri.to_string(),
            access_token: access_token.to_string(),
        }
    }

    pub async fn post(&self, message: &str, in_reply_to_id: Option<String>){
        let url = format!("{}/api/v1/statuses", self.base_uri);
        println!("{}", &url);
        let client = Client::new();
        let body = Message{status: message.to_string(), in_reply_to_id};
        let response = client
            .post(&url)
            .json(&body)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await;
        println!("{:?}", response);
    }
}
