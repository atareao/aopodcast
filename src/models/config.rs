use serde::{Serialize, Deserialize};
use tokio::fs::read_to_string;
use std::{process, fmt::{self, Display}};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration{
    log_level: String,
    creator: String,
}

impl Display for Configuration{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "log_level: {},\ncreator: {}",
          self.get_log_level(),
          self.get_creator())
    }
}

impl Configuration {
    pub fn get_log_level(&self) -> &str{
        &self.log_level
    }
    pub fn get_creator(&self) -> &str{
        &self.creator
    }

    pub async fn read_configuration() -> Configuration{
        let content = match read_to_string("config.yml")
            .await {
                Ok(value) => value,
                Err(e) => {
                    println!("Error with config file `config.yml`: {}",
                        e.to_string());
                    process::exit(0);
                }
            };
        match serde_yaml::from_str(&content){
            Ok(configuration) => configuration,
            Err(e) => {
                println!("Error with config file `config.yml`: {}",
                    e.to_string());
                process::exit(0);
            }
        }
    }
}

