use log::info;
use regex::Regex;

#[derive(Debug)]
pub struct Mp3Metadata{
    pub filename: String,
    pub mtime: String,
    pub size: String,
    pub length: String,
    pub title: String,
    pub creator: String,
    pub album: String,
    pub track: String,
    pub artist: String,
    pub genre: String,
    pub comment: String,
}

impl Mp3Metadata {
    pub fn new(content: &str) -> Option<Mp3Metadata>{
        let pattern_init = Regex::new(r#"^\s+<file name=".*\.mp3" source="original">"#).unwrap();
        let pattern_end = Regex::new(r#"^\s+</file>"#).unwrap();
        let mut mp3 = false;
        let mut mp3_metadata: Vec<String> = Vec::new();
        for line in content.lines(){
            if !mp3 && pattern_init.is_match(line){
                mp3 = true;
            }
            if mp3{
                mp3_metadata.push(line.to_string());
            }
            if mp3 && pattern_end.is_match(line){
                break;
            }
        }
        let text = mp3_metadata.concat();
        if text.is_empty(){
            return None;
        }
        info!("Text: {}", &text);
        let mtime = Self::get_value("mtime", &text);
        let size = Self::get_value("size", &text);
        let length = Self::get_value("length", &text);
        let title = Self::get_value("title", &text);
        let creator = Self::get_value("creator", &text);
        let album = Self::get_value("album", &text);
        let track = Self::get_value("track", &text);
        let artist = Self::get_value("artist", &text);
        let genre = Self::get_value("genre", &text);
        let comment = Self::get_value("comment", &text);
        let pattern = r#"<file name="([^"]*)" source="original">"#;
        let re = Regex::new(pattern).unwrap();
        let caps = re.captures(&text).unwrap();
        let filename = caps.get(1).unwrap().as_str().to_string();
        Some(Mp3Metadata{
            filename,
            mtime,
            size,
            length,
            title,
            creator,
            album,
            track,
            artist,
            genre,
            comment,
        })
    }

    fn get(tag: &str, xml: &str) -> Vec<String>{
        let mut result = Vec::new();
        let pattern = format!("<{tag}>([^<]*)</{tag}>", tag=tag);
        let re = Regex::new(&pattern).unwrap();

        for cap in re.captures_iter(xml){
            result.push(cap.get(1).unwrap().as_str().to_string());
        }
        result
    }
    fn get_value(tag: &str, xml: &str) -> String{
        let value = Self::get(tag, xml);
        if value.len() > 0{
            match value.get(0) {
                Some(value) => value.to_string(),
                None => "".to_string()
            }
        }else{
            "".to_string()
        }

    }
}
