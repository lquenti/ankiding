use crate::parser::Card;

use std::collections::HashMap;
use std::fs::File;
use std::io::copy;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

use anyhow::Result;
use clap::Parser;
use genanki_rs::Package;
use tempfile::TempDir;
use url::Url;

mod anki;
mod io;
mod latex;
mod parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the markdown files
    path: PathBuf,
    /// The path to the output file
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Whether to use dark mode
    #[arg(long, default_value_t = false)]
    dark_mode: bool,
}

fn main() -> Result<()> {
    latex::require_executable("pdflatex");
    latex::require_executable("pdfcrop");
    latex::require_executable("dvisvgm");
    latex::render_formula("f(x) = x", &PathBuf::from("."))?;
    let mut replacements = HashMap::new();
    let cli = Cli::parse();

    let decks = io::get_all_files(cli.path)?
        .into_iter()
        // Read files
        .map(|filename| {
            let markdowns = io::read_file_to_string(&filename).unwrap();
            (filename, markdowns)
        })
        // Convert to cards
        .map(|(filename, markdowns)| (filename, Card::from_markdown(&markdowns)))
        // Convert to html
        .map(|(filename, cards)| {
            (
                filename,
                cards
                    .into_iter()
                    .map(Card::into_html)
                    .collect::<Vec<Card>>(),
            )
        })
        .map(|(filename, cards)| {
            let old_img_paths =
                Card::mass_apply_to_vec(cards.clone(), parser::extract_img_paths_from_html);
            let mut cards = cards;
            for path in &old_img_paths {
                let new_filename = format!("{}", uuid::Uuid::new_v4());
                cards = cards
                    .into_iter()
                    .map(|card| card.map(|html| html.replace(path, &new_filename)))
                    .collect::<Vec<Card>>();
                replacements.insert(path.to_string(), new_filename);
            }
            (filename, cards)
        })
        // Convert to Ankideck
        .map(|(filename, cards)| anki::from_cards(&filename, &cards, cli.dark_mode))
        .collect::<Vec<genanki_rs::Deck>>();

    // Download all files and collect them
    let temp_dir = TempDir::new()?;
    let new_files: Vec<String> = replacements
        .iter()
        .map(|(old_path, new_filename)| {
            let new_path = format!(
                "{}{}{}",
                temp_dir.path().to_str().unwrap(),
                MAIN_SEPARATOR,
                new_filename
            );
            if Path::new(old_path).is_file() {
                std::fs::copy(old_path, &new_path)?;
                Ok(new_path)
            } else if let Ok(url) = Url::parse(old_path) {
                let mut response = reqwest::blocking::get(url)?;
                let mut file = File::create(&new_path)?;
                copy(&mut response, &mut file)?;
                Ok(new_path)
            } else {
                Err(anyhow::Error::msg("Path is neither a file nor a url"))
            }
        })
        .filter_map(Result::ok)
        .collect();

    // Save file
    let xs = new_files.iter().map(|s| s.as_ref()).collect();
    let mut package = Package::new(decks, xs)?;
    match cli.output {
        // TODO duplication with file handler finding markdown files
        Some(path) => {
            if path.is_dir() {
                let output_path =
                    format!("{}{}output.apkg", path.to_str().unwrap(), MAIN_SEPARATOR);
                package.write_to_file(&output_path)?;
            } else if path.parent().unwrap().is_dir() {
                package.write_to_file(path.as_os_str().to_str().unwrap())?;
            } else {
                panic!("Path is neither a file nor a directory");
            }
        }
        None => package.write_to_file("output.apkg")?,
    }
    Ok(())
}
