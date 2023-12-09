use reqwest::Client;
use serde_json::json;
use regex::Regex;
use tracing::{info, error};

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

    #[allow(dead_code)]
    pub async fn post(&self, message: &str){
        let url = format!("https://api.telegram.org/bot{}/sendMessage",
            self.access_token);
        let message = json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "HTML",
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

    pub async fn send_audio(&self, audio: &str, caption: &str){
        let url = format!("https://api.telegram.org/bot{}/sendAudio",
            self.access_token);
        let message = json!({
            "chat_id": self.chat_id,
            "audio": audio,
            "caption": Self::prepare(caption),
            "parse_mode": "HTML",
        });
        match Client::new()
            .post(url)
            .json(&message)
            .send()
            .await {
                Ok(response) => {
                    match response.text().await{
                        Ok(content) => info!(
                            "Mensaje envíado a Telegram. Response: {}",
                            content),
                        Err(error) => error!(
                            "No he podido enviar el mensaje a Telegram: {}",
                            error.to_string())
                    }
                },
                Err(error) => {
                    error!("No he podido enviar el mensaje a Telegram: {}",
                        error.to_string());
                },
            }
    }

    fn prepare(text: &str) -> String{
        let re = Regex::new(r#""([^"]*)""#).unwrap();
        re.replace_all(text, "<i>$1</i>").to_string()
    }
}

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use std::env;
    use crate::models::telegram::Telegram;
    use tokio;

    #[tokio::test]
    async fn send_audio_test(){
        dotenv().ok();
        let token = env::var("TOKEN").unwrap();
        let chat_id = env::var("CHAT_ID").unwrap();
        let audio = env::var("AUDIO").unwrap();
        let caption = r#"Este es un "título" de prueba"#;
        println!("{}, {}, {}, {}", token, chat_id, audio, caption);
        
        let telegram = Telegram::new(&token, &chat_id);
        let answer = telegram.send_audio(&audio, caption).await;
        println!("{:?}", answer);
    }
}

