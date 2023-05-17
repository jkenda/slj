pub mod drevo;
pub mod tokenizer;
pub mod tip;
pub mod napaka;

mod loci;
mod operatorji;
mod predprocesiraj;
mod okvir;
mod stavek;
mod funkcija;
mod izraz;
mod argumenti;

use std::{collections::HashMap, rc::Rc, iter};
use rand::{distributions::Alphanumeric, Rng};

use drevo::{Drevo, Vozlišče::{*, self}, VozliščeOption::*};
use tip::Tip;
use tokenizer::{Token::{*, self}, L, Tokenize};
use loci::*;

use self::napaka::{Napake, OznakaNapake::*, Napaka};
use self::operatorji::*;

#[derive(Debug)]
struct Parser<'a> {
    spremenljivke_stack: Vec<HashMap<String, Rc<Vozlišče>>>,
    spremenljivke: HashMap<String, Rc<Vozlišče>>,
    funkcije_stack: Vec<HashMap<String, Rc<Vozlišče>>>,
    funkcije: HashMap<String, Rc<Vozlišče>>,
    reference_stack: Vec<HashMap<&'a str, Rc<Vozlišče>>>,
    reference: HashMap<&'a str, Rc<Vozlišče>>,
    znotraj_funkcije: bool,
}

pub trait Parse {
    fn parse(&self) -> Result<Drevo, Napake>;
}

impl Parse for Vec<Token<'_>> {
    fn parse(&self) -> Result<Drevo, Napake> {
        Parser::new().parse(self)
    }
}


impl<'a> Parser<'a> {
    fn new() -> Parser<'a> {
        Parser{ 
            spremenljivke_stack: vec![],
            funkcije_stack: vec![],
            reference_stack: vec![],
            spremenljivke: HashMap::new(),
            funkcije: HashMap::new(),
            reference: HashMap::new(),
            znotraj_funkcije: false,
        }
    }

    fn parse(&mut self, izraz: &[Token<'a>]) -> Result<Drevo, Napake> {
        let izraz = [
            Self::standard().as_slice(),
            izraz,
        ].concat();
        let okvir = self.okvir(&Parser::predprocesiraj(izraz.as_slice()))?;
        Ok(Drevo::new(okvir))
    }

    fn standard() -> Vec<Token<'static>> {
        const STANDARD: &str = include_str!("../../standard/natisni.slj");
        let drevo = STANDARD.tokenize();
        Parser::predprocesiraj(&drevo)
    }

    fn dodaj_spremenljivko(&mut self, ime: String, tip: Tip) -> Rc<Vozlišče> {
        let naslov = match self.znotraj_funkcije {
            true  => self.spremenljivke_stack.last().unwrap().values().map(|s| s.sprememba_stacka() as u32).sum::<u32>(),
            false => self.spremenljivke.values().map(|s| s.sprememba_stacka() as u32).sum::<u32>(),
        };
        let spr = Spremenljivka { tip, ime: ime.clone(), naslov, z_odmikom: self.znotraj_funkcije }.rc();
        self.spremenljivke_stack.last_mut().unwrap().insert(ime.clone(), spr.clone());
        self.spremenljivke.insert(ime, spr.clone());
        spr
    }

    fn naključno_ime(&self, dolžina: usize) -> String {
        let mut ime = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(dolžina)
            .map(char::from)
            .collect();

        while self.spremenljivke.contains_key(&ime) {
            ime = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(dolžina)
                .map(char::from)
                .collect();
        }

        ime
    }
}