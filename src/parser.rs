use comrak::{markdown_to_html, ComrakOptions};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref COMRAK_OPTIONS: ComrakOptions = {
        let mut options = ComrakOptions::default();
        options.render.unsafe_ = true;
        options.extension.table = true;
        options
    };
}

lazy_static! {
    static ref MARKDOWN_CARDS_RE: Regex =
        Regex::new(r"---\n\s*Q:(?P<question>(.|\n|\r)*?)\n---\n\s*A:(?P<answer>(.|\n|\r)*?)\n---")
            .unwrap();
    //static ref IMG_RE: Regex = Regex::new(r#"<img[^>]*?src=["'](?P<src>.*?)["'].*?/?>"#).unwrap();
    //static ref IMG_RE: Regex = Regex::new(r#"<(img|source|video)[^>]*?src=["'](?P<src>.*?)["'][^>]*?/?>"#).unwrap();
    static ref IMG_RE: Regex = Regex::new(r#"<video[^>]*?src=["'](?P<src>.*?)["'][^>]*?/?>"#).unwrap();
}

#[derive(Debug, Clone)]
pub struct Card {
    pub front: String,
    pub back: String,
}

impl Card {
    pub fn from_markdown(markdown: &str) -> Vec<Card> {
        let mut matches = Vec::new();
        for cap in MARKDOWN_CARDS_RE.captures_iter(markdown) {
            let front = cap.name("question").unwrap().as_str().trim().to_string();
            let back = cap.name("answer").unwrap().as_str().trim().to_string();

            matches.push(Card { front, back });
        }
        matches
    }

    pub fn into_html(self) -> Self {
        let front = markdown_to_html(&self.front, &COMRAK_OPTIONS);
        let back = markdown_to_html(&self.back, &COMRAK_OPTIONS);
        Card { front, back }
    }
}
