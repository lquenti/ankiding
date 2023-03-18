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
    /// Do not render formulas
    #[arg(long, default_value_t = false)]
    no_latex: bool,
}

fn require_executables() {
    println!("Checking for required executables...");
    println!("Checking for pdflatex...");
    latex::require_executable("pdflatex");
    println!("Checking for pdfcrop...");
    latex::require_executable("pdfcrop");
    println!("Checking for dvisvgm...");
    latex::require_executable("dvisvgm");
}

/* TODO: Move everything out of here */

fn get_cards_from_path(path: &Path) -> Result<HashMap<PathBuf, Vec<Card>>> {
    let filenames = io::get_all_files(path)?;
    let mut cards = HashMap::new();
    for filename in filenames {
        let markdowns = io::read_file_to_string(&filename)?;
        cards.insert(filename, Card::from_markdown(&markdowns));
    }
    Ok(cards)
}

fn render_formula(cards: &mut HashMap<PathBuf, Vec<Card>>, path: &Path) -> Result<()> {
    for cards in cards.values_mut() {
        for card in cards {
            let formulas = card.get_all_formulas();
            if formulas.is_empty() {
                continue;
            }
            for formula in formulas {
                let output_file = latex::render_formula(&formula, path)?;
                let new_formula = format!("![latex-render]({})", output_file.to_str().unwrap());
                *card = card.replace_formula(&formula, &new_formula);
            }
        }
    }
    Ok(())
}

fn download_images(cards: &mut HashMap<PathBuf, Vec<Card>>, path: &Path) -> Result<()> {
    for (markdownpath, cards) in cards {
        for card in cards {
            let images = card.get_all_images();
            if images.is_empty() {
                continue;
            }
            for image in images {
                let new_filepath = format!(
                    "{}{}{}",
                    path.to_str().unwrap(),
                    MAIN_SEPARATOR,
                    uuid::Uuid::new_v4()
                );
                let new_filename = Path::new(&new_filepath)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap();

                // Maybe its an URL
                if let Ok(url) = Url::parse(&image) {
                    let mut response = reqwest::blocking::get(url)?;
                    let mut file = File::create(&new_filepath)?;
                    copy(&mut response, &mut file)?;
                    continue;
                }

                // Maybe its an path
                // Either its an absolute path, then it already works
                // If its a relative path, we paste it on the directory the markdown file is in
                let image_path = PathBuf::from(&image);
                if image_path.is_absolute() {
                    let absolute_path = image_path.to_str().unwrap().to_string();
                    std::fs::copy(&absolute_path, &new_filepath)?;
                } else {
                    let mut absolute_path = markdownpath.to_path_buf();
                    absolute_path.pop();
                    absolute_path.push(&image);
                    let absolute_path = absolute_path.to_str().unwrap().to_string();
                    std::fs::copy(&absolute_path, &new_filepath)?;
                }
                *card = card.replace_image_link(&image, new_filename);
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if !cli.no_latex {
        require_executables();
    }
    let tempdir = TempDir::new()?;
    let path = tempdir.path();

    println!("Loading markdown files from {:?}...", &cli.path);
    let mut cards = get_cards_from_path(&cli.path)?;

    if !cli.no_latex {
        println!("Rendering formulas...");
        render_formula(&mut cards, path)?;
    }

    println!("Downloading images...");
    download_images(&mut cards, path)?;

    let decks = cards
        .into_iter()
        .map(|(filename, cards)| {
            (
                filename,
                cards
                    .into_iter()
                    .map(Card::into_html)
                    .collect::<Vec<Card>>(),
            )
        })
        .map(|(filename, cards)| anki::from_cards(&filename, &cards, cli.dark_mode))
        .collect::<Vec<genanki_rs::Deck>>();

    let new_files = std::fs::read_dir(path).unwrap();
    let xs_owned = new_files
        .map(|x| x.unwrap().path())
        .filter(|x| x.is_file())
        .collect::<Vec<PathBuf>>();
    let xs = xs_owned
        .iter()
        .map(|x| x.to_str().unwrap())
        .collect::<Vec<&str>>();

    println!("Creating package...");
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
                panic!("\"{:?}\" is neither a file nor a directory", path);
            }
        }
        None => package.write_to_file("output.apkg")?,
    }

    /*
        // Convert to html
    let decks = cards.into_iter().map(|(filename, cards)| {
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
    */
    Ok(())
}
