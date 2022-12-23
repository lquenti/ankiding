use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the markdown files
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    println!("name: {:?}", cli.path);
}
