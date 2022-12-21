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
    pub fn new(content: &str) -> Mp3Metadata{
        println!("{}", content);
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
        println!("{:?}", mp3_metadata);
        let text = mp3_metadata.concat();
        let mtime = Self::get("mtime", &text).get(0).unwrap().to_string();
        let size = Self::get("size", &text).get(0).unwrap().to_string();
        let length = Self::get("length", &text).get(0).unwrap().to_string();
        let title = Self::get("title", &text).get(0).unwrap().to_string();
        let creator = Self::get("creator", &text).get(0).unwrap().to_string();
        let album = Self::get("album", &text).get(0).unwrap().to_string();
        let track = Self::get("track", &text).get(0).unwrap().to_string();
        let artist = Self::get("artist", &text).get(0).unwrap().to_string();
        let genre = Self::get("genre", &text).get(0).unwrap().to_string();
        let comment = Self::get("comment", &text).get(0).unwrap().to_string();
        let pattern = r#"<file name="([^"]*)" source="original">"#;
        let re = Regex::new(pattern).unwrap();
        let caps = re.captures(&text).unwrap();
        let filename = caps.get(1).unwrap().as_str().to_string();
        Mp3Metadata{
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
        }
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
}
