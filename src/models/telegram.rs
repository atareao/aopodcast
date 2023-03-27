use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Message{
    chat_id: i64,
    text: String,

}

impl Message {
    fn new(chat_id: i64, text: &str) -> Self{
        Message{
            chat_id,
            text: text.to_string(),
        }
    }
}

pub async fn send_message(chat_id: i64, text: &str) -> Option<String>{
    let token = env::var("TG_TOKEN").expect("TG_TOKEN not set");
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let message = Message::new(chat_id, text);
    println!("{}", serde_json::to_string(&message).unwrap());
    match Client::new()
        .post(url)
        .json(&message)
        .send()
        .await{
            Ok(response) => {
                println!("Mensaje envíado: {}", response.status().to_string());
                Some(response.status().to_string())
            },
            Err(error) => {
                println!("No he podido enviar el mensaje: {}",error.to_string());
                None
            },
        }
}

