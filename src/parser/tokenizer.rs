use std::{fmt::Debug, hash::Hash, mem::size_of};
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


impl<'a> Token<'a> {
    pub fn lokacija_str(&self) -> String {
        use Token::*;
        match self {
            Operator(_, v, z) | Ločilo(_, v, z) | Literal(_, v, z) | Ime(_, v, z) | Število(_, v, z) | Niz(_, v, z) =>
                format!("({}. vrstica, {}. znak)", v, z)
        }
    }

    pub fn len(&self) -> usize {
        use Token::*;
        match self {
            Operator(val, ..) | Ločilo(val, ..) | Literal(val, ..) | Ime(val, ..) | Število(val, ..) | Niz(val, ..) => val.len(),
        }
    }

    pub fn as_str(&self) -> &'a str {
        use Token::*;
        match self {
            Operator(val, ..) | Ločilo(val, ..) | Literal(val, ..) | Ime(val, ..) | Število(val, ..) | Niz(val, ..) => val,
        }
    }

    fn vrstica(&self) -> usize {
        use Token::*;
        match self {
            Operator(_, v, _) | Ločilo(_, v, _) | Literal(_, v, _) | Ime(_, v, _) | Število(_, v, _) | Niz(_, v, _) => *v,
        }
    }

    fn znak(&self) -> usize {
        use Token::*;
        match self {
            Operator(.., z) | Ločilo(.., z) | Literal(.., z) | Ime(.., z) | Število(.., z) | Niz(.., z) => *z,
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

        const ZADNJA_MEJA: &str = r"(=|<|>|!|\+|-|\*|/|%|\^|;|\(|\)|\{|\}|\[|\]|,|:|#|\s)";

        let regexi: Vec<(Regex, fn(&'a str, usize, usize) -> Token<'a>)> = vec![
            (Regex::new(&format!(r"^(resnica|laž|in|ali|čene|če|dokler|za|funkcija|vrni){}", ZADNJA_MEJA)).unwrap(), Literal),
            (Regex::new(&format!(r"^[_[[:alpha:]]][\w\d]*{}", ZADNJA_MEJA)).unwrap(), Ime),
            (Regex::new(&format!(r"^-?([1-9][0-9]*(.[0-9]+)?|[0-9]){}", ZADNJA_MEJA)).unwrap(), Število),
            (Regex::new(&format!(r#"^".*"{}"#, ZADNJA_MEJA)).unwrap(), Niz),
            (Regex::new(r#"^(==|<=|>=|\+=|\-=|\*=|/=|=|<|>|!|\+|-|\*|/|%|\^)"#).unwrap(), Operator),
            (Regex::new(r#"^(;|\n|\(|\)|\{|\}|\[|\]|,|:|#)"#).unwrap(), Ločilo),
        ];

        let mut tokeni: Vec<Token> = Vec::new();
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
                None => {
                    i += tekst.chars().nth(i).unwrap().len_utf8();
                    znak += 1;
                }
            };
        }

        tokeni
    }

}

fn find_token<'a>(regexi: &[(Regex, fn(&'a str, usize, usize) -> Token<'a>)], beseda: &'a str, vrstica: usize, znak: usize) -> Option<(Token<'a>, usize)> {
    use Token::*;
    let (regex, token) = &regexi[0];

    match regex.find(beseda) {
        Some(mat) => match token("", 0, 0) {
            Operator(..) | Ločilo(..) => {
                Some((token(mat.as_str(), vrstica, znak), mat.end()))
            },
            _ => {
                let dolžina_zadnjega = mat.as_str().chars().last().unwrap().len_utf8();
                Some((token(&mat.as_str()[..mat.end() - dolžina_zadnjega], vrstica, znak), mat.end() - dolžina_zadnjega))
            },
        },
        None => 
            if regexi.len() > 1 {
                find_token(&regexi[1..], beseda, vrstica, znak)
            }
            else {
                None
            },
    }
}
