use std::{fmt::{Debug, Display}, hash::Hash};
use regex::Regex;

#[derive(Debug, Clone, Copy)]
pub enum Token<'a> {
    Ločilo  (&'a str, usize, usize),
    Operator(&'a str, usize, usize),
    Literal (&'a str, usize, usize),
    Ime     (&'a str, usize, usize),
    Število (&'a str, usize, usize),
    Niz     (&'a str, usize, usize),
}

pub fn ločilo(from: &'static str) -> Token {
    Token::Ločilo(from, 0, 0)
}
pub fn operator(from: &'static str) -> Token {
    Token::Operator(from, 0, 0)
}
pub fn literal(from: &'static str) -> Token {
    Token::Literal(from, 0, 0)
}
pub fn ime(from: &'static str) -> Token {
    Token::Ime(from, 0, 0)
}
pub fn število(from: &'static str) -> Token {
    Token::Število(from, 0, 0)
}
pub fn niz(from: &'static str) -> Token {
    Token::Niz(from, 0, 0)
}

impl<'a> Token<'a> {
    pub fn lokacija_str(&self) -> String {
        use Token::*;
        match self {
            Operator(_, v, z) | Ločilo(_, v, z) | Literal(_, v, z) | Ime(_, v, z) | Število(_, v, z) | Niz(_, v, z) =>
                format!("({}. vrstica, {}. znak)", v, z)
        }
    }

    pub fn as_str(&self) -> &'a str {
        use Token::*;
        match self {
            Operator(val, ..) | Ločilo(val, ..) | Literal(val, ..) | Ime(val, ..) | Število(val, ..) | Niz(val, ..) => val,
        }
    }
}

impl ToString for Token<'_> {
    fn to_string(&self) -> String {
        use Token::*;
        match self {
            Operator(val, ..) | Ločilo(val, ..) | Literal(val, ..) | Ime(val, ..) | Število(val, ..) | Niz(val, ..) => val.to_string()
        }
    }
}

impl Hash for Token<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Token::*;
        match self {
            Operator(val, ..) | Ločilo(val, ..) | Literal(val, ..) | Ime(val, ..) | Število(val, ..) | Niz(val, ..) => val.hash(state),
        };
    }
}

impl PartialEq for Token<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
            && self.to_string() == other.to_string()
    }
}

impl Eq for Token<'_> {}

pub struct Tokenizer;

pub trait Tokenize<'a> {
    fn tokenize(&'a self) -> Vec<Token<'a>>;
}

impl<'a> Tokenize<'a> for String {
    fn tokenize(&'a self) -> Vec<Token<'a>> {
        Tokenizer::tokenize(self)
    }
}


impl<'a> Tokenizer {

    pub fn tokenize(tekst: &'a str) -> Vec<Token> {
        use Token::*;

        let regexi: Vec<(Regex, fn(&'a str, usize, usize) -> Token<'a>)> = vec![
            (Regex::new(r"^(resnica|laž|in|ali|čene|če|dokler|za|funkcija|vrni)").unwrap(), Literal),
            (Regex::new(r"^[_[[:alpha:]]][\w\d]*").unwrap(), Ime),
            (Regex::new(r"^-?([1-9][0-9]*(.[0-9]+)?|[0-9])").unwrap(), Število),
            (Regex::new(r#"^".*""#).unwrap(), Niz),
            (Regex::new(r#"^(==|<=|>=|\+=|\-=|\*=|/=|=|<|>|!|\+|-|\*|/|%|\^)"#).unwrap(), Operator),
            (Regex::new(r#"^(;|\n|\(|\)|\{|\}|\[|\]|,|:|#)"#).unwrap(), Ločilo),
        ];

        let mut tokeni = Vec::new();
        let mut vrstica = 1;
        let mut znak = 1;

        let mut i: usize = 0;

        while i < tekst.len() {
            match find_token(&regexi, &tekst[i..], vrstica, znak) {
                Some((token, dolžina)) => {
                    tokeni.push(token);
                    i += dolžina;
                    if let Ločilo("\n", ..) = token {
                        vrstica += 1;
                        znak = 1;
                    }
                    else {
                        znak += dolžina;
                    }
                },
                None => i += 1,
            };
        }

        tokeni
    }

}

fn find_token<'a>(regexi: &[(Regex, fn(&'a str, usize, usize) -> Token<'a>)], beseda: &'a str, vrstica: usize, znak: usize) -> Option<(Token<'a>, usize)> {
    let (regex, token) = &regexi[0];

    match regex.find(beseda) {
        Some(mat) => Some((token(mat.as_str(), vrstica, znak), mat.end())),
        None => 
            if regexi.len() > 1 {
                find_token(&regexi[1..], beseda, vrstica, znak)
            }
            else {
                None
            },
    }
}
