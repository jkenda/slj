pub mod drevo;
pub mod lekser;
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

use drevo::{Drevo, Vozlišče::{*, self}, VozliščeOption::*};
use tip::Tip;
use lekser::{Žeton::{*, self}, L, Razčleni};
use loci::*;

use self::napaka::{Napake, OznakaNapake::*, Napaka};
use self::operatorji::*;

#[derive(Debug, Clone)]
struct Parser<'a> {
    spremenljivke_stack: Vec<HashMap<&'a str, Rc<Vozlišče>>>,
    spremenljivke: HashMap<&'a str, Rc<Vozlišče>>,
    konstante_stack: Vec<HashMap<String, Rc<Vozlišče>>>,
    konstante: HashMap<String, Rc<Vozlišče>>,
    funkcije_stack: Vec<HashMap<String, Rc<Vozlišče>>>,
    funkcije: HashMap<String, Rc<Vozlišče>>,
    št_klicev: HashMap<String, usize>,
    znotraj_funkcije: bool,
}

pub trait Parse {
    fn analiziraj(&self) -> Result<Drevo, Napake>;
}

impl Parse for Vec<Žeton<'_>> {
    fn analiziraj(&self) -> Result<Drevo, Napake> {
        Parser::new().parse(self)
    }
}


impl<'a> Parser<'a> {
    fn new() -> Parser<'a> {
        Parser { 
            spremenljivke_stack: vec![],
            funkcije_stack: vec![],
            konstante_stack: vec![],
            spremenljivke: HashMap::new(),
            konstante: HashMap::new(),
            funkcije: HashMap::new(),
            št_klicev: HashMap::new(),
            znotraj_funkcije: false,
        }
    }

    fn parse(&mut self, izraz: &[Žeton<'a>]) -> Result<Drevo, Napake> {
        let izraz = [
            Self::standard().as_slice(),
            &[Ločilo("\n", 0, 0)],
            &Parser::predprocesiraj(izraz),
        ].concat();
        let okvir = self.okvir(izraz.as_slice())?;
        Ok(Drevo::new(okvir, self.št_klicev.clone()))
    }

    fn standard() -> Vec<Žeton<'static>> {
        const MATEMATIKA: &str = include_str!("../../jedro/matematika.slj");
        const NATISNI: &str = include_str!("../../jedro/natisni.slj");
        const PREBERI: &str = include_str!("../../jedro/preberi.slj");
        [
            Parser::predprocesiraj(&MATEMATIKA.razčleni()).as_slice(),
            &[Ločilo("\n", 0, 0)],
            Parser::predprocesiraj(&NATISNI.razčleni()).as_slice(),
            &[Ločilo("\n", 0, 0)],
            &Parser::predprocesiraj(&PREBERI.razčleni()),
            &[Ločilo("\n", 0, 0)],
        ].concat()
    }

    fn dodaj_spremenljivko(&mut self, ime: &'a str, tip: Tip, spremenljiva: bool) -> Rc<Vozlišče> {
        let naslov = match self.znotraj_funkcije {
            true  => self.spremenljivke_stack.last().unwrap().values().map(|s| s.sprememba_stacka()).sum(),
            false => self.spremenljivke.values().map(|s| s.sprememba_stacka()).sum(),
        };
        let z_odmikom = self.znotraj_funkcije;
        let spr = Spremenljivka { tip, ime: ime.to_string(), naslov, z_odmikom, spremenljiva }.rc();

        self.spremenljivke_stack.last_mut().unwrap().insert(&ime, spr.clone());
        self.spremenljivke.insert(&ime, spr.clone());
        spr
    }

    fn dodaj_konstanto(&mut self, ime: String, vrednost: Rc<Vozlišče>) -> Rc<Vozlišče> {
        self.konstante_stack.last_mut().unwrap().insert(ime.clone(), vrednost.clone());
        self.konstante.insert(ime, vrednost.clone());
        vrednost
    }

    fn poišči_spr(&self, ime: &Žeton) -> Result<Rc<Vozlišče>, Napake> {
        match self.konstante.get(ime.as_str()) {
            Some(spr) => Ok(spr.clone()),
            None => match self.spremenljivke.get(ime.as_str()) {
                Some(spr) => Ok(spr.clone()),
                None => Err(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka")),
            }
        }
    }
}
