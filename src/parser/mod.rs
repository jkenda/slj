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
    št_klicev: HashMap<String, usize>,
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
        Parser { 
            spremenljivke_stack: vec![],
            funkcije_stack: vec![],
            reference_stack: vec![],
            spremenljivke: HashMap::new(),
            funkcije: HashMap::new(),
            št_klicev: HashMap::new(),
            reference: HashMap::new(),
            znotraj_funkcije: false,
        }
    }

    fn parse(&mut self, izraz: &[Token<'a>]) -> Result<Drevo, Napake> {
        let izraz = [
            Self::standard().as_slice(),
            &[Ločilo("\n", 0, 0)],
            &Parser::predprocesiraj(izraz),
        ].concat();
        let okvir = self.okvir(izraz.as_slice())?;
        Ok(Drevo::new(okvir, self.št_klicev.clone()))
    }

    fn standard() -> Vec<Token<'static>> {
        const MATEMATIKA: &str = include_str!("../../jedro/matematika.slj");
        const NATISNI: &str = include_str!("../../jedro/natisni.slj");
        const PREBERI: &str = include_str!("../../jedro/preberi.slj");
        [
            Parser::predprocesiraj(&MATEMATIKA.tokenize()).as_slice(),
            &[Ločilo("\n", 0, 0)],
            Parser::predprocesiraj(&NATISNI.tokenize()).as_slice(),
            &[Ločilo("\n", 0, 0)],
            &Parser::predprocesiraj(&PREBERI.tokenize()),
            &[Ločilo("\n", 0, 0)],
        ].concat()
    }

    fn dodaj_spremenljivko(&mut self, ime: String, tip: Tip, spremenljiva: bool) -> Rc<Vozlišče> {
        let naslov = match self.znotraj_funkcije {
            true  => self.spremenljivke_stack.last().unwrap().values().map(|s| s.sprememba_stacka()).sum(),
            false => self.spremenljivke.values().map(|s| s.sprememba_stacka()).sum(),
        };
        let z_odmikom = self.znotraj_funkcije;
        let spr = Spremenljivka { tip, ime: ime.clone(), naslov, z_odmikom, spremenljiva }.rc();

        self.spremenljivke_stack.last_mut().unwrap().insert(ime.clone(), spr.clone());
        self.spremenljivke.insert(ime, spr.clone());
        spr
    }

    fn poišči_spr(&self, ime: &Token) -> Result<Rc<Vozlišče>, Napake> {
        Ok(self.spremenljivke.get(ime.as_str())
            .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?.clone())
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
