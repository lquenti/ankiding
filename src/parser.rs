use comrak::{markdown_to_html, ComrakOptions};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref COMRAK_OPTIONS: ComrakOptions = {
        let mut options = ComrakOptions::default();
        options.extension.table = true;
        options
    };
}

lazy_static! {
    static ref CARD_RE: Regex = Regex::new(r"(?P<card>>\s*[qQ]:.*?\n(?:>.*?\n)*>\s*[aA]:.*?\n(?:>.*?(\n|$))*)").unwrap();
    static ref IMAGE_RE: Regex = Regex::new(r"!\[(?P<alt>(?:.|\s)*?)\]\((?P<link>(?:.|\s)*?)\)").unwrap();
    static ref LATEX_RE: Regex = Regex::new(r"\$(?P<formula>.*?)\$").unwrap();
}

#[derive(Debug, Clone)]
pub struct Card {
    pub front: String,
    pub back: String,
}

impl Card {
    pub fn from_markdown(markdown: &str) -> Vec<Card> {
        let mut matches = Vec::new();

        for cap in CARD_RE.captures_iter(markdown) {
            let card = cap.name("card").unwrap().as_str().trim().to_string();
            let mut lines = card.lines();
            // Next, we trim every line and remove the ">" afterwards
            let mut unquoted = lines.into_iter()
                .map(|line| line.trim_start_matches(">").trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<&str>>();

            // Thus, we expect that the first line starts with "q:" or "Q:"
            // Lets make sure that we don't have a bug
            assert!(unquoted[0].starts_with("q:") || unquoted[0].starts_with("Q:"));
            unquoted[0] = unquoted[0].trim_start_matches("q:").trim_start_matches("Q:");

            // Split everything before the A: or a: into the front
            let mut front = String::new();
            let mut back = String::new();
            let mut is_front = true;
            for line in unquoted {
                if line.starts_with("a:") || line.starts_with("A:") {
                    is_front = false;
                    front.push_str(line.trim_start_matches("a:").trim_start_matches("A:"));
                    front.push('\n');
                    continue;
                }
                if is_front {
                    front.push_str(line);
                    front.push('\n');
                } else {
                    back.push_str(line);
                    back.push('\n');
                }
            }
            
            // Remove the last newline
            front.pop();
            back.pop();

            matches.push(Card { front, back });
        }
        matches
    }

    pub fn into_html(self) -> Self {
        let front = markdown_to_html(&self.front, &COMRAK_OPTIONS);
        let back = markdown_to_html(&self.back, &COMRAK_OPTIONS);
        Card { front, back }
    }

    pub fn map(self, f: impl Fn(String) -> String) -> Self {
        let front = f(self.front);
        let back = f(self.back);
        Card { front, back }
    }

    pub fn apply<A>(self, f: impl Fn(&str) -> A) -> (A, A) {
        (f(&self.front), f(&self.back))
    }

    // TODO Rename me
    pub fn mass_apply_to_vec<A>(cards: Vec<Self>, f: impl Fn(&str) -> Vec<A>) -> Vec<A> {
        cards
            .into_iter()
            .map(|card| card.apply(&f))
            .flat_map(|(xs, ys)| xs.into_iter().chain(ys.into_iter()).collect::<Vec<A>>())
            .collect::<Vec<A>>()
    }
}