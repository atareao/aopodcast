use serde::{Deserialize, Serialize};
use super::item::Item;
use log::info;



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
    pub async fn read_saved_items(path: &str) -> Items{
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

    pub async fn save_items(&self, path: &str) -> Result<(), std::io::Error>{
        info!("{:?}", path);
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
                self.items.push(item.clone());
            }else{
                info!("Exists {}", item.get_identifier());
            }
        }
    }

    pub fn get_last(&self) -> &Item{
        let mut last = self.items.get(0).unwrap();
        for item in self.items.as_slice(){
            if item.get_mtime() > last.get_mtime(){
                last = item;
            }
        }
        last
    }

    pub fn count(&self) -> usize{
        self.items.len()
    }
}
