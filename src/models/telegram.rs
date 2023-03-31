use reqwest::Client;
use serde_json::json;
use log::{info, error};

pub struct Telegram{
    access_token: String,
    chat_id: String,
}

pub fn get_telegram_client() -> Option<Telegram>{
    match std::env::var("TELEGRAM_TOKEN"){
        Ok(token) => {
            match std::env::var("TELEGRAM_CHAT_ID"){
                Ok(chat_id) => Some(Telegram::new(&token, &chat_id)),
                Err(_) => None,
            }
        },
        Err(_) => None,
    }
}

impl Telegram{
    pub fn new(access_token: &str, chat_id: &str) -> Self{
        Self{
            access_token: access_token.to_string(),
            chat_id: chat_id.to_string(),
        }
    }
    pub async fn post(&self, message: &str){
        let url = format!("https://api.telegram.org/bot{}/sendMessage",
            self.access_token);
        let message = json!({
            "chat_id": self.chat_id,
            "text": message,
        });
        match Client::new()
            .post(url)
            .json(&message)
            .send()
            .await{
                Ok(response) => {
                    info!("Mensaje envíado a Telegram: {}",
                        response.status().to_string());
                },
                Err(error) => {
                    error!("No he podido enviar el mensaje a Telegram: {}",
                        error.to_string());
                },
            }
        }
}

