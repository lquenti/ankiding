use crate::parser::Card;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tempfile::TempDir;

mod anki;
mod io;
mod parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the markdown files
    path: PathBuf,
    /// The path to the output file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let mut replacements = HashMap::new();
    let temp_dir = TempDir::new()?;

    let _markdowns = io::get_all_files(Cli::parse().path)?
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
                cards.into_iter().map(Card::to_html).collect::<Vec<Card>>(),
            )
        })
        // Extract and replace img tags
        .map(|(filename, cards)| {
            let img_map = Card::mass_apply_to_hashmap(cards.clone(), |html| {
                parser::create_img_paths_mapping_from_html(&temp_dir.path().to_path_buf(), html)
            });
            let cards = cards
                .into_iter()
                .map(|card| {
                    card.map(|html| {
                        let mut html = html;
                        for (key, value) in img_map.iter() {
                            html = html.replace(key, value.to_str().unwrap());
                        }
                        html
                    })
                })
                .collect::<Vec<Card>>();
            replacements.extend(img_map);
            (filename, cards)
        })
        // Convert to Ankideck
        .map(|(filename, cards)| {
            let deck = anki::from_cards(&filename, &cards);
            (filename, deck)
        })
        .collect::<Vec<(PathBuf, genanki_rs::Deck)>>();

    Ok(())
}
