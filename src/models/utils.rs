use regex::Regex;
use chrono::{DateTime, Utc, NaiveDate, NaiveTime, NaiveDateTime};
use log::debug;

const EXCERPT_LENGTH: usize = 55;

pub fn get_excerpt<'a>(content: &'a str) -> &'a str{
    get_first_words(content, EXCERPT_LENGTH)
}

pub fn get_slug(title: &str) -> String{
    debug!("Slug from: '{}'", title);
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

pub fn get_first_words<'a>(content: &'a str, number: usize) -> &'a str{
    debug!("get_first_words");
    debug!("Content: {}", &content);
    let re1 = Regex::new(r"[ ]{2,}").unwrap();
    let re2 = Regex::new(r"[\n]{2,}").unwrap();
    let re3 = Regex::new(r"[\t]{2,}").unwrap();
    let clean_content = re1.replace_all(&content, " ").to_string();
    let clean_content = re2.replace_all(&clean_content, "\n").to_string();
    let clean_content = re3.replace_all(&clean_content, "\t").to_string();
    let positions = clean_content.chars()
        .enumerate()
        .filter(|(_, c)| *c == ' ' || *c == '\n' || *c == '\t')
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if positions.len() > number{
        let position = positions[number];
        &content[..position].trim()
    }else{
        content.trim()
    }
}

#[allow(dead_code)]
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

#[test]
fn test_get_first_words(){
    let content = "En un lugar de la Mancha, de cuyo nombre no quiero acordarme, no ha mucho tiempo que vivía un hidalgo de los de lanza en astillero, adarga antigua, rocín flaco y galgo corredor. Una olla de algo más vaca que carnero, salpicón las más noches, duelos y quebrantos los sábados, lantejas los viernes, algún palomino de añadidura los domingos, consumían las tres partes de su hacienda. El resto della concluían sayo de velarte, calzas de velludo para las fiestas, con sus pantuflos de lo mesmo, y los días de entresemana se honraba con su vellorí de lo más fino. Tenía en su casa una ama que pasaba de los cuarenta, y una sobrina que no llegaba a los veinte, y un mozo de campo y plaza, que así ensillaba el rocín como tomaba la podadera. Frisaba la edad de nuestro hidalgo con los cincuenta años; era de complexión recia, seco de carnes, enjuto de rostro, gran madrugador y amigo de la caza. Quieren decir que tenía el sobrenombre de Quijada, o Quesada, que en esto hay alguna diferencia en los autores que deste caso escriben; aunque por conjeturas verosímiles se deja entender que se llamaba Quijana. Pero esto importa poco a nuestro cuento: basta que en la narración dél no se salga un punto de la verdad.";
    let fw = get_first_words(content, 55);
    println!("FW: {}", fw);
    assert_ne!(fw.len(), 55);
}
