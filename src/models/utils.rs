use regex::Regex;
use chrono::{DateTime, Utc, NaiveDate, NaiveTime, NaiveDateTime};
use log::{info, debug};

pub fn get_slug(title: &str) -> String{
    info!("Slug from: '{}'", title);
    let title: String = title
        .to_lowercase().
        chars()
        .map(|c| match c {
            'a'..='z'|'0'..='9' => c,
            'á'|'ä'|'à'|'â'     => 'a',
            'é'|'ë'|'è'|'ê'     => 'e',
            'í'|'ï'|'ì'|'î'     => 'i',
            'ó'|'ö'|'ò'|'ô'     => 'o',
            'ú'|'ü'|'ù'|'û'     => 'u',
            'ñ'                 => 'n',
            _                   => '-'
        })
        .collect();
    debug!("Slug step 1: '{}'", title);
    let re = Regex::new(r"\-{2,}").unwrap();
    let mut title = re.replace_all(&title, "-").to_string();
    debug!("Slug step 2: '{}'", title);
    let mut title = if title.starts_with("-"){
        title.remove(0).to_string();
        title
    }else{
        title
    };
    debug!("Slug step 3: '{}'", title);
    if title.ends_with("-"){
        title.pop();
        title
    }else{
        title.to_string()
    }
}

pub fn get_date(mtime: &str) -> String{
    let timestamp = mtime.parse::<i64>().unwrap();
    let naive_date_time = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let date = DateTime::<Utc>::from_utc(naive_date_time, Utc);
    date.format("%Y-%m-%d").to_string()
}

pub fn get_unix_time(ymd: &str) -> u64{
    let pattern = Regex::new("[^0-9-]").unwrap();
    let clean_ymd = pattern.replace_all(ymd, "");
    let nd = clean_ymd.trim().parse::<NaiveDate>().unwrap();
    let nt = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let ndt = nd.and_time(nt);
    ndt.timestamp().try_into().unwrap()
}

#[test]
fn test_get_unix_time(){
    let date = "\"2022-12-10\"";
    println!("{}", date);
    let ut = get_unix_time(date);
    println!("ut: {}", ut);
    assert_ne!(ut, 1);
}
