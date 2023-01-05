use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::Result;

use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
use genanki_rs::{Deck, Field, Model, Note, Template};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"---\n\s*Q:(?P<question>(.|\n|\r)*?)\n---\n\s*A:(?P<answer>(.|\n|\r)*?)\n---")
            .unwrap();
}

lazy_static! {
    static ref COMRAK_OPTIONS: ComrakOptions = {
        // TODO SET OPTIONS
        ComrakOptions::default()
    };
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the markdown files
    path: PathBuf,
}

struct Card {
    front: String,
    back: String,
}

fn create_apkg_file_from_cards(cards: Vec<Card>) {
    // TODO
    let my_model = Model::new(
        1607392319,
        "Simple Model",
        vec![Field::new("Question"), Field::new("Answer")],
        vec![Template::new("Card 1")
            .qfmt("{{Question}}")
            .afmt(r#"{{FrontSide}}<hr id="answer">{{Answer}}"#)],
    );
    let mut my_deck = Deck::new(2059400110, "NAME", "SOME DESCRIPTION");
    for card in cards {
        let my_note = Note::new(my_model.clone(), vec![&card.front, &card.back]).unwrap();
        my_deck.add_note(my_note);
    }
    my_deck.write_to_file("output.apkg").unwrap();
}

fn read_file_to_string(file: &Path) -> Result<String> {
    let mut file = File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn find_all_files_by_extension(base_directory: &Path, file_extension: &str) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(base_directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            paths.append(&mut find_all_files_by_extension(&path, file_extension)?);
        } else if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.to_lowercase().ends_with(file_extension) {
                paths.push(file_name.into());
            }
        }
    }
    Ok(paths)
}

fn extract_markdown_cards(markdown: &str) -> Vec<Card> {
    let mut matches = Vec::new();
    for cap in RE.captures_iter(&markdown) {
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

///
//////////////////////////////////////////////

fn main() {
    let cli = Cli::parse();
    let path = cli.path;

    // Find all anki markdown files
    let files;
    if path.is_file() {
        files = vec![path];
    } else if path.is_dir() {
        files = find_all_files_by_extension(&path, ".anki.md").unwrap();
    } else {
        panic!("Path is neither a file nor a directory");
    }
    println!("Files: {:?}", files);


    // Read in all markdown files
    let mut markdowns = HashMap::new();
    for file in files {
        let markdown = read_file_to_string(&file).unwrap();
        markdowns.insert(file, markdown);
    }

    // Extract cards from markdown
    // Next, convert markdown to html
    let markdowns = markdowns
        .into_iter()
        .map(|(key, value)| (key, extract_markdown_cards(&value)))
        .map(|(key, value)| (key, value.into_iter().map(markdown_card_to_html_card).collect()))
        .collect::<HashMap<PathBuf, Vec<Card>>>();

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
