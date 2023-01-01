use std::fmt::Debug;
use regex::Regex;

#[derive(Debug, Clone)]
pub enum Token {
    Ločilo(String),
    Literal(String),
    Ime(String),
    Število(String),
    Niz(String),
}

impl Token {
    fn set(&mut self, new: &str) -> &Token {
        match self {
            Literal(val) | Ime(val) | Število(val) | Niz(val) => *val = new.to_owned(),
            Ločilo(val) => *val = if new.starts_with('\n') { "\n".to_owned() } else { new.to_owned() },
        }
        self
    }
}

pub struct Tokenizer {
    split_regex: Regex,
    regexi: [(Regex, Token); 5],
    tekst: String,
    tokens: Vec<Token>,
}

impl Debug for Tokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[\n\t{}\n]", self.tokens.iter().map(|t| format!("{:?}", t)).collect::<Vec<String>>().join(",\n\t"))
    }
}

use Token::*;

impl Tokenizer {
    pub fn new() -> Tokenizer {
        Tokenizer {
            split_regex: Regex::new(r"[^\S\r\n]+").unwrap(),
            regexi: [
                (Regex::new(r"^(resnica|laž|in|ali|če|čene|dokler|za|funkcija|vrni)").unwrap(), Literal(String::new())),
                (Regex::new(r"^[_[[:alpha:]]][\w\d]*").unwrap(), Ime(String::new())),
                (Regex::new(r"^-?([1-9][0-9]*(.[0-9]+)?|[0-9])").unwrap(), Število(String::new())),
                (Regex::new(r#"^"[[:graph:]]*""#).unwrap(), Niz(String::new())),
                (Regex::new(r#"^(;|\n+|\(|\)|\{|\}|\[|\]|,|:|"|'|=|==|<|>|<=|>=|!|\+|-|\*|/|\+=|\-=|\*=|/=|%|\^|#)"#).unwrap(), Ločilo(String::new())),
            ],
            tekst: String::new(),
            tokens: Vec::new() }
    }

    pub fn from(text: &String) -> Tokenizer {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_text(text);
        tokenizer
    }

    pub fn add_text(&mut self, text: &String) -> &Self {
        self.tekst = text.clone();
        self
    }

    pub fn tokenize(&mut self) -> &Vec<Token> {
        let besede = self.split_regex.split(&self.tekst);

        for beseda in besede {
            let mut i: usize = 0;

            while i < beseda.len() {
                let (token, dolžina) = find_token(&mut self.regexi.clone(), &beseda[i..]);
                self.tokens.push(token);
                i += dolžina;
            }
        }

        &self.tokens
    }

}

fn find_token(regexi: &mut [(Regex, Token)], beseda: &str) -> (Token, usize) {
    match regexi[0].0.find(beseda) {
        Some(mat) => (regexi[0].1.set(mat.as_str()).clone(), mat.end()),
        None => if regexi.len() > 1 {
            find_token(&mut regexi[1..], beseda)
        }
        else {
            panic!("Napaka -- neznana beseda: \"{:?}\"", beseda)
        },
    }
}
