use std::{path::PathBuf, fs};

use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
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

struct MarkdownCard {
    front: String,
    back: String,
}

fn extract_questions_and_answers(path: PathBuf) -> Vec<MarkdownCard> {
    let contents = fs::read_to_string(path)
        .expect("Something went wrong reading the file");
    let mut matches = Vec::new();
    for cap in RE.captures_iter(&contents) {
        let front_markdown = cap.name("question").unwrap().as_str().trim();
        let back_markdown = cap.name("answer").unwrap().as_str().trim();

        let front = markdown_to_html(front_markdown, &COMRAK_OPTIONS);
        let back = markdown_to_html(back_markdown, &COMRAK_OPTIONS);
        matches.push(MarkdownCard { front, back });
    }
    matches
}


fn main() {
    let cli = Cli::parse();
    let path = cli.path;
    let cards = extract_questions_and_answers(path);

    let options = ComrakOptions::default();
    for card in cards {

        println!("Front: {:?}", card.front);
        println!("Back: {:?}", card.back);
    }
}
