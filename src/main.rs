use crate::parser::Card;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use genanki_rs::Package;

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
                cards.into_iter().map(Card::into_html).collect::<Vec<Card>>(),
            )
        })
        // Convert to Ankideck
        .map(|(filename, cards)| {
            anki::from_cards(&filename, &cards)
        })
        .collect::<Vec<genanki_rs::Deck>>();

    let mut package = Package::new(decks, vec!["/home/lquenti/debug/590d.mp4"])?;
    package.write_to_file("output.apkg")?;

    Ok(())
}
