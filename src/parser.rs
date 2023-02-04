use std::collections::HashMap;
use std::path::PathBuf;

use comrak::{markdown_to_html, ComrakOptions};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref COMRAK_OPTIONS: ComrakOptions = {
        // TODO SET OPTIONS
        ComrakOptions::default()
    };
}

lazy_static! {
    static ref MARKDOWN_CARDS_RE: Regex =
        Regex::new(r"---\n\s*Q:(?P<question>(.|\n|\r)*?)\n---\n\s*A:(?P<answer>(.|\n|\r)*?)\n---")
            .unwrap();
    static ref IMG_RE: Regex = Regex::new(r#"<img src=["'](?P<src>.*?)["'].*?/>"#).unwrap();
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

    pub fn to_html(self) -> Self {
        let front = markdown_to_html(&self.front, &COMRAK_OPTIONS);
        let back = markdown_to_html(&self.back, &COMRAK_OPTIONS);
        Card { front, back }
    }

    pub fn map(self, f: impl Fn(String) -> String) -> Self {
        let front = f(self.front);
        let back = f(self.back);
        Card { front, back }
    }

    pub fn apply<A>(self, f: impl Fn(&str) -> A) -> (A, A) {
        (f(&self.front), f(&self.back))
    }

    // TODO Rename me
    pub fn mass_apply_to_vec<A>(cards: Vec<Self>, f: impl Fn(&str) -> Vec<A>) -> Vec<A> {
        cards
            .into_iter()
            .map(|card| card.apply(&f))
            .flat_map(|(xs, ys)| xs.into_iter().chain(ys.into_iter()).collect::<Vec<A>>())
            .collect::<Vec<A>>()
    }

    pub fn mass_apply_to_hashmap<A, B>(
        cards: Vec<Self>,
        f: impl Fn(&str) -> HashMap<A, B>,
    ) -> HashMap<A, B>
    where
        A: std::cmp::Eq + std::hash::Hash + std::clone::Clone,
        B: std::clone::Clone,
    {
        cards
            .into_iter()
            .map(|card| card.apply(&f))
            .flat_map(|(xs, ys)| {
                xs.into_iter()
                    .chain(ys.into_iter())
                    .collect::<HashMap<A, B>>()
            })
            .collect::<HashMap<A, B>>()
    }
}

// TODO: MOVE ME
pub fn extract_img_paths_from_html(html: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for cap in IMG_RE.captures_iter(html) {
        let path = cap.name("src").unwrap().as_str().trim().to_string();
        paths.push(path);
    }
    paths
}

pub fn create_img_paths_mapping_from_html(
    base_path: &PathBuf,
    html: &str,
) -> HashMap<String, PathBuf> {
    let mut paths = HashMap::new();
    for cap in IMG_RE.captures_iter(html) {
        let path = cap.name("src").unwrap().as_str().trim().to_string();
        let filename = PathBuf::from(&path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        paths.insert(path, base_path.join(filename));
    }
    paths
}

// TODO join two functions above