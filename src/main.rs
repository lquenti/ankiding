use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

use anyhow::Result;

use chrono::prelude::*;
use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
use genanki_rs::{Deck, Field, Model, Note, Package, Template};
use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;
use tempfile::TempDir;
use url::Url;

lazy_static! {
    // TODO rename me
    static ref RE: Regex =
        Regex::new(r"---\n\s*Q:(?P<question>(.|\n|\r)*?)\n---\n\s*A:(?P<answer>(.|\n|\r)*?)\n---")
            .unwrap();
}

lazy_static! {
    static ref IMG_RE: Regex = Regex::new(r#"<img src=["'](?P<src>.*?)["'].*?/>"#).unwrap();
}

lazy_static! {
    static ref COMRAK_OPTIONS: ComrakOptions = {
        // TODO SET OPTIONS
        ComrakOptions::default()
    };
}

const DEFAULT_ANKI_CSS: &str = include_str!("../assets/templates/base.css");

const FRONT_SIDE_TEMPLATE: &str = include_str!("../assets/templates/front.html");
const BACK_SIDE_TEMPLATE: &str = include_str!("../assets/templates/back.html");


lazy_static! {
    static ref ANKI_MODEL: Model = Model::new(
        0x1337420,
        "Ankiding Model",
        vec![Field::new("Question"), Field::new("Answer"),],
        vec![Template::new("Card 1")
            .qfmt(FRONT_SIDE_TEMPLATE)
            .afmt(BACK_SIDE_TEMPLATE)],
    )
    .css(DEFAULT_ANKI_CSS);
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the markdown files
    path: PathBuf,
    /// The path to the output file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug)]
struct Card {
    front: String,
    back: String,
}

fn read_file_to_string(file: &Path) -> Result<String> {
    let mut file = File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn find_all_files_by_extension(
    base_directory: &Path,
    file_extension: &str,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(base_directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            paths.append(&mut find_all_files_by_extension(&path, file_extension)?);
        } else if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.to_lowercase().ends_with(file_extension) {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

fn extract_markdown_cards(markdown: &str) -> Vec<Card> {
    let mut matches = Vec::new();
    for cap in RE.captures_iter(markdown) {
        let front = cap.name("question").unwrap().as_str().trim().to_string();
        let back = cap.name("answer").unwrap().as_str().trim().to_string();

        matches.push(Card { front, back });
    }
    matches
}

fn markdown_card_to_html_card(card: Card) -> Card {
    let front = markdown_to_html(&card.front, &COMRAK_OPTIONS);
    let back = markdown_to_html(&card.back, &COMRAK_OPTIONS);
    Card { front, back }
}

fn create_anki_deck(file: &Path, cards: &[Card]) -> Deck {
    let deck_name = file
        .to_str()
        .unwrap()
        .replace(MAIN_SEPARATOR, "::")
        .replace(".anki.md", "");

    let deck_id: i64 = rand::thread_rng().gen();

    let desc = format!(
        "Generated by ankiding at {}",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    let mut deck = Deck::new(deck_id, &deck_name, &desc);
    for card in cards {
        let note = Note::new(ANKI_MODEL.clone(), vec![&card.front, &card.back]).unwrap();
        deck.add_note(note);
    }
    deck
}

fn extact_img_paths_from_html(html: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for cap in IMG_RE.captures_iter(html) {
        let path = cap.name("src").unwrap().as_str().to_string();
        paths.push(path);
    }
    paths
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = cli.path;

    // Find all anki markdown files
    let files = {
        if path.is_file() {
            vec![path]
        } else if path.is_dir() {
            find_all_files_by_extension(&path, ".anki.md").unwrap()
        } else {
            panic!("Path is neither a file nor a directory");
        }
    };

    // Read in all markdown files
    let mut markdowns = HashMap::new();
    for file in files {
        let markdown = read_file_to_string(&file).unwrap();
        markdowns.insert(file, markdown);
    }

    // Extract cards from markdown
    // Next, convert markdown to html
    let mut htmls = markdowns
        .into_iter()
        .map(|(filename, cards)| (filename, extract_markdown_cards(&cards)))
        .map(|(filename, cards)| {
            (
                filename,
                cards.into_iter().map(markdown_card_to_html_card).collect(),
            )
        })
        .collect::<HashMap<PathBuf, Vec<Card>>>();

    let temp_dir = TempDir::new()?;
    let mut all_new_files = Vec::new();
    for (_, cards) in htmls.iter_mut() {
        for card in cards {
            for img_path in extact_img_paths_from_html(&card.front) {
                let path = Path::new(&img_path);
                if path.is_file() {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let new_path = format!(
                        "{}{}{}",
                        temp_dir.path().to_str().unwrap(),
                        MAIN_SEPARATOR,
                        file_name
                    );
                    println!("Processing {}", img_path);
                    fs::copy(path, &new_path)?;
                    card.front = card.front.replace(&img_path, &file_name);
                    all_new_files.push(new_path);
                } else if Url::parse(&img_path).is_ok() {
                    let file_name = img_path.split('/').last().unwrap();
                    let new_path = format!(
                        "{}{}{}",
                        temp_dir.path().to_str().unwrap(),
                        MAIN_SEPARATOR,
                        file_name
                    );
                    println!("Downloading {}", img_path);
                    let mut response = reqwest::blocking::get(&img_path)?;
                    let mut file = File::create(&new_path)?;
                    io::copy(&mut response, &mut file)?;
                    card.front = card.front.replace(&img_path, &file_name);
                    println!("Temp path:  {}", new_path);
                    all_new_files.push(new_path);
                } else {
                    panic!("Image path {} is neither a local file nor a url", img_path);
                }
            }
            for img_path in extact_img_paths_from_html(&card.back) {
                let path = Path::new(&img_path);
                if path.is_file() {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let new_path = format!(
                        "{}{}{}",
                        temp_dir.path().to_str().unwrap(),
                        MAIN_SEPARATOR,
                        file_name
                    );
                    println!("Processing {}", img_path);
                    fs::copy(path, &new_path)?;
                    card.back = card.back.replace(&img_path, &file_name);
                    all_new_files.push(new_path);
                } else if Url::parse(&img_path).is_ok() {
                    let file_name = img_path.split('/').last().unwrap();
                    let new_path = format!(
                        "{}{}{}",
                        temp_dir.path().to_str().unwrap(),
                        MAIN_SEPARATOR,
                        file_name
                    );
                    println!("Downloading {}", img_path);
                    let mut response = reqwest::blocking::get(&img_path)?;
                    let mut file = File::create(&new_path)?;
                    io::copy(&mut response, &mut file)?;
                    card.back = card.back.replace(&img_path, &file_name);
                    println!("Temp path:  {}", new_path);
                    all_new_files.push(new_path);
                } else {
                    panic!("Image path {} is neither a local file nor a url", img_path);
                }
            }
        }
    }

    // Create and save the apkg
    let decks = htmls
        .into_iter()
        .map(|(key, value)| create_anki_deck(&key, &value))
        .collect::<Vec<Deck>>();
    let all_new_files = all_new_files.iter().map(|s| s.as_ref()).collect::<Vec<&str>>();
    println!("All new files: {:?}", all_new_files);
    let mut package = Package::new(decks, all_new_files)?;
    match cli.output {
        // TODO duplication with file handler finding markdown files
        Some(path) => {
            if path.is_dir() {
                let output_path =
                    format!("{}{}output.apkg", path.to_str().unwrap(), MAIN_SEPARATOR);
                package.write_to_file(&output_path)?;
            } else if path.is_file() {
                package.write_to_file(&path.to_str().unwrap())?;
            } else {
                panic!("Path is neither a file nor a directory");
            }
        }
        None => package.write_to_file("output.apkg")?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}