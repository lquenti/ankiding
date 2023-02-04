use crate::parser::Card;

use std::path::{Path, MAIN_SEPARATOR};

use chrono::prelude::*;
use genanki_rs::{Deck, Field, Model, Note, Template};
use lazy_static::lazy_static;
use rand::Rng;

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

pub fn from_cards(filename: &Path, cards: &[Card]) -> Deck {
    let deck_name = filename
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