use std::{path::PathBuf, fs};

use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
use genanki_rs::{Model, Field, Template, Deck, Note};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE: Regex = Regex::new(
        r"---\n\s*Q:(?P<question>(.|\n|\r)*?)\n---\n\s*A:(?P<answer>(.|\n|\r)*?)\n---"
    ).unwrap();
}

lazy_static! {
    static ref COMRAK_OPTIONS: ComrakOptions = {
        let mut options = ComrakOptions::default();
        // TODO SET OPTIONS
        options
    };
}


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the markdown files
    path: PathBuf,
}

struct HTMLCard {
    front: String,
    back: String,
}

fn extract_questions_and_answers(path: PathBuf) -> Vec<HTMLCard> {
    let contents = fs::read_to_string(path)
        .expect("Something went wrong reading the file");
    let mut matches = Vec::new();
    for cap in RE.captures_iter(&contents) {
        let front_markdown = cap.name("question").unwrap().as_str().trim();
        let back_markdown = cap.name("answer").unwrap().as_str().trim();

        let front = markdown_to_html(front_markdown, &COMRAK_OPTIONS);
        let back = markdown_to_html(back_markdown, &COMRAK_OPTIONS);
        matches.push(HTMLCard { front, back });
    }
    matches
}

fn create_apkg_file_from_cards(cards: Vec<HTMLCard>) {
    // TODO
    let my_model = Model::new(
        1607392319,
        "Simple Model",
        vec![Field::new("Question"), Field::new("Answer")],
        vec![Template::new("Card 1")
            .qfmt("{{Question}}")
            .afmt(r#"{{FrontSide}}<hr id="answer">{{Answer}}"#)],
    );
    let mut my_deck = Deck::new(
        2059400110,
        "NAME",
        "SOME DESCRIPTION",
    );
    for card in cards {
        let my_note = Note::new(my_model.clone(), vec![&card.front, &card.back]).unwrap();
        my_deck.add_note(my_note);
    }
    my_deck.write_to_file("output.apkg").unwrap();
}

fn main() {
    let cli = Cli::parse();
    let path = cli.path;
    let cards = extract_questions_and_answers(path);

    for card in &cards {
        println!("Front: {:?}", card.front);
        println!("Back: {:?}", card.back);
    }

    create_apkg_file_from_cards(cards);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
