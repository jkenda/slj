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

pub struct Tokenizer {
    ločilo_regex: Regex,
    literal_regex: Regex,
    ime_regex: Regex,
    število_regex: Regex,
    niz_regex: Regex,
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
            ločilo_regex:  Regex::new(r#"^(\(|\)|\{|\}|\[|\]|,|;|:|"|'|=|==|<|>|<=|>=|!|\+|-|\*|/|\+=|\-=|\*=|/=|%|\^|#)"#).unwrap(),
            literal_regex: Regex::new(r"^(resnica|laž|in|ali|če|čene|dokler|za|funkcija|vrni)").unwrap(),
            ime_regex:     Regex::new(r"^[_[[:alpha:]]][\w\d]*").unwrap(),
            število_regex: Regex::new(r"^-?([1-9][0-9]*(.[0-9]+)?|[0-9])").unwrap(),
            niz_regex: Regex::new(r#"^"[[:graph:]]*""#).unwrap(),
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
        let besede = self.tekst.split_whitespace();

        for beseda in besede {
            let mut i: usize = 0;

            while i < beseda.len() {
                let (token, dolžina) = match self.literal_regex.find(&beseda[i..]) {
                    Some(mat) => (Literal(mat.as_str().to_owned()), mat.end()),
                    None => match self.ime_regex.find(&beseda[i..]) {
                        Some(mat) => (Ime(mat.as_str().to_owned()), mat.end()),
                        None => match self.število_regex.find(&beseda[i..]) {
                            Some(mat) => (Število(mat.as_str().to_owned()), mat.end()),
                            None => match self.niz_regex.find(&beseda[i..]) {
                                Some(mat) => (Niz(mat.as_str().to_owned()), mat.end()),
                                None => match self.ločilo_regex.find(&beseda[i..]) {
                                    Some(mat) => (Ločilo(mat.as_str().to_owned()), mat.end()),
                                    None => panic!("Napaka -- neznana beseda: {}", &beseda[i..]),
                                }
                            }
                        }
                    }
                };

                self.tokens.push(token);
                i += dolžina;
            }
        }

        &self.tokens
    }

}
