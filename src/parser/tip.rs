use std::collections::BTreeMap;
use std::fmt::Display;
use Tip::*;

use crate::parser::napaka::OznakaNapake;
use crate::parser::tokenizer::L;

use super::napaka::{Napake, Napaka};
use super::tokenizer::Token;
use super::loci::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Tip {
    Brez,
    Bool,
    Celo,
    Real,
    Znak,
    Seznam(Box<Tip>, usize),
    Strukt(BTreeMap<String, Box<Tip>>),
    Referenca(Box<Tip>),
}

impl Tip {
    pub fn from(izraz: &[Token]) -> Result<Self, Napake> {
        use Token::{Ločilo, Operator};
        match izraz {
            [ Token::Tip("brez", ..) ] => Ok(Tip::Brez),
            [ Token::Tip("bool", ..) ] => Ok(Tip::Bool),
            [ Token::Tip("celo", ..) ] => Ok(Tip::Celo),
            [ Token::Tip("real", ..) ] => Ok(Tip::Real),
            [ Token::Tip("znak", ..) ] => Ok(Tip::Znak),
            [ Ločilo("[", ..), tip @ .., Ločilo(";", ..), len @ Token::Literal(L::Celo(..)), Ločilo("]", ..) ] => Ok(Tip::Seznam(Box::new(Tip::from(tip)?), match len.as_str().parse() {
                Ok(len) => len,
                Err(err) => panic!("Iz vrednosti ni mogoče ustvariti števila: {err} {}", len.lokacija_str())
            })),
            [ Ločilo("{", ..), vmes @ .., Ločilo("}", ..) ] => Ok(Tip::Strukt(zgradi_tip_strukta(vmes)?)),
            [ Operator("@", ..), ostanek @ .. ] => Ok(Referenca(Box::new(Tip::from(ostanek)?))),
            _ => Err(Napake::from_zaporedje(izraz, OznakaNapake::E1, "Neznan tip")),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Seznam(tip, len) => tip.len() * len,
            Strukt(polja) => polja.values().map(|p| p.len()).sum(),
            _ => panic!("Tip {self} nima dolžine.")
        }
    }

    pub fn sprememba_stacka(&self) -> isize {
        match self {
            Brez => 0,
            Bool | Celo | Real | Znak => 1,
            Seznam(tip, len) => (tip.len() * len) as isize + 1,
            Strukt(polja) => polja.values().map(|p| p.sprememba_stacka()).sum(),
            Referenca(_) => 1,
        }
    }
}

impl Display for Tip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Tip::*;
        write!(f, "{}", &match self {
            Brez => "brez".to_string(),
            Bool => "bool".to_string(),
            Celo => "celo".to_string(),
            Real => "real".to_string(),
            Znak => "znak".to_string(),
            Seznam(tip, len) => format!("[{tip}; {len}]"),
            Strukt(polja) => {
                let mut str = "{\n".to_string();
                for (ime, tip) in polja {
                    str += &format!("{ime}: {tip},\n");
                }
                str += "}";
                str
            },
            Referenca(tip) => format!("@{tip}"),
        })
    }
}

fn zgradi_tip_strukta<'a: 'b, 'b>(mut izraz: &'b [Token<'a>]) -> Result<BTreeMap<String, Box<Tip>>, Napake> {
    let mut polja = BTreeMap::new();
    let mut napake = Napake::new();

    let mut ločeno = loči_spredaj(izraz, &[","]);
    while ločeno.is_some() {
        let (polje, _, ostanek) = ločeno.unwrap()?;

        match polje {
            [ ime @ Token::Ime(..), Token::Ločilo(":", ..), tip @ .. ] => {
                match Tip::from(tip) {
                    Ok(tip) => match polja.insert(ime.to_string(), Box::new(tip)) {
                        Some(..) => _ = napake.add_napaka(Napaka::from_zaporedje(&[*ime], OznakaNapake::E1, "Polje s tem imenom že obstaja")),
                        None => (),
                    },
                    Err(n)  => _ = napake.razširi(n),
                }
            },
            _ => _ = napake.add_napaka(Napaka::from_zaporedje(polje, OznakaNapake::E1, "Neveljavno polje")),
        };

        izraz = ostanek;
        ločeno = loči_spredaj(izraz, &[";", "\n"]);
    }
    if izraz != &[] {
        match izraz {
            [ ime @ Token::Ime(..), Token::Ločilo(":", ..), tip @ .. ] => {
                match Tip::from(tip) {
                    Ok(tip) => match polja.insert(ime.to_string(), Box::new(tip)) {
                        Some(..) => _ = napake.add_napaka(Napaka::from_zaporedje(&[*ime], OznakaNapake::E1, "Polje s tem imenom že obstaja")),
                        None => (),
                    },
                    Err(n)  => _ = napake.razširi(n),
                }
            },
            _ => _ = napake.add_napaka(Napaka::from_zaporedje(izraz, OznakaNapake::E1, "Neveljavno polje")),
        };
    }

    if napake.prazno() {
        Ok(polja)
    }
    else {
        Err(napake)
    }
}

#[cfg(test)]
mod testi {
    use crate::parser::tokenizer::Tokenize;

    use super::*;

    #[test]
    fn from_string_to_string() {
        assert_eq!(Tip::from("brez".to_string().tokenize().as_slice()).unwrap().to_string(), "brez");
        assert_eq!(Tip::from("bool".to_string().tokenize().as_slice()).unwrap().to_string(), "bool");
        assert_eq!(Tip::from("celo".to_string().tokenize().as_slice()).unwrap().to_string(), "celo");
        assert_eq!(Tip::from("real".to_string().tokenize().as_slice()).unwrap().to_string(), "real");
        assert_eq!(Tip::from("znak".to_string().tokenize().as_slice()).unwrap().to_string(), "znak");

        assert_eq!(Tip::from("[celo; 6]".to_string().tokenize().as_slice()).unwrap().to_string(), "[celo; 6]");
        assert_eq!(Tip::from("[[celo; 3]; 6]".to_string().tokenize().as_slice()).unwrap().to_string(), "[[celo; 3]; 6]");

        assert_eq!(Tip::from("{ x: real, y: real }".to_string().tokenize().as_slice()).unwrap().to_string(), "{\nx: real,\ny: real,\n}");
        assert_eq!(Tip::from("{ _arr: [celo; 128], len: celo }".to_string().tokenize().as_slice()).unwrap().to_string(), "{\n_arr: [celo; 128],\nlen: celo,\n}");

        assert_eq!(Tip::from("@celo".to_string().tokenize().as_slice()).unwrap().to_string(), "@celo");
        assert_eq!(Tip::from("@[real; 32]".to_string().tokenize().as_slice()).unwrap().to_string(), "@[real; 32]");
        assert_eq!(Tip::from("@@[real; 32]".to_string().tokenize().as_slice()).unwrap().to_string(), "@@[real; 32]");
    }
}
