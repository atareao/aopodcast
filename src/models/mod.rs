pub mod archive;
pub mod metadata;
pub mod mp3metadata;
pub mod config;
pub mod site;
pub mod article;
pub mod page;
pub mod episode;
pub mod doc;
pub mod utils;
pub mod mastodon;
pub mod telegram;

pub use archive::ArchiveOrg;
pub use article::Article;
pub use config::Configuration;
pub use doc::Doc;
pub use episode::{Episode, Metadata};
pub use mastodon::{Mastodon, get_mastodon_client};
pub use metadata::AOMetadata;
pub use mp3metadata::Mp3Metadata;
pub use page::Page;
pub use site::{Layout, Site, Post};
pub use telegram::{Telegram, get_telegram_client};

use minijinja::{Environment, path_loader};
use once_cell::sync::Lazy;
use chrono::{DateTime, FixedOffset};
use chrono_tz::Tz;
use minijinja::value::{Kwargs, Value};
use minijinja::{Error, ErrorKind, State};

pub static ENV: Lazy<Environment<'static>> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    env.add_filter("striptags", striptags);
    env.add_filter("date", date);
    env.add_filter("truncate", truncate);
    env.add_function("now", now);
    env
});

fn striptags(value: String) -> String {
    let mut data = String::new();
    let mut inside = false;
    // Step 1: loop over string chars.
    for c in value.chars() {
        // Step 2: detect markup start and end, and skip over markup chars.
        if c == '<' {
            inside = true;
            continue;
        }
        if c == '>' {
            inside = false;
            continue;
        }
        if !inside {
            // Step 3: push other characters to the result string.
            data.push(c);
        }
    }
    data
}

fn value_to_chrono_datetime(
    value: Value,
) -> Result<DateTime<FixedOffset>, Error> {
    match value.as_str(){
        Some(s) => match DateTime::parse_from_rfc3339(s){
            Ok(dt) => Ok(dt),
            Err(e) => Err(Error::new(
                ErrorKind::MissingArgument,
                e.to_string()
            )),
        },
        None => Err(Error::new(
            ErrorKind::MissingArgument,
            "Not a valid string"
        )),
    }
}

pub fn date(_state: &State, value: Value, kwargs: Kwargs) -> Result<String, Error> {
    let format = kwargs.get::<Option<&str>>("format")?;
    match kwargs.get::<Option<&str>>("timezone")?{
        Some(timezone) => {
            let tz: Tz = timezone.parse().unwrap();
            let datetime = value_to_chrono_datetime(value).unwrap().with_timezone(&tz);
            Ok(format!("{}", datetime.format(format.unwrap())))
        },
        None => {
            let datetime = value_to_chrono_datetime(value).unwrap();
            Ok(format!("{}", datetime.format(format.unwrap())))

        },
    }
}

pub fn truncate(_state: &State, value: Value, kwargs: Kwargs) -> Result<String, Error> {
    let length = kwargs.get::<Option<usize>>("length")?.unwrap();
    match value.as_str() {
        Some(s) => match value.as_str().unwrap().char_indices().nth(length) {
            None => Ok(s.to_string()),
            Some((idx, _)) => Ok(s[..idx].to_string()),
        },
        None => Err(Error::new(
            ErrorKind::MissingArgument,
            "Not a valid string"
        )),
    }
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
