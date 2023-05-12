pub mod drevo;
pub mod tokenizer;
pub mod tip;
pub mod napaka;
mod loci;

use std::{collections::HashMap, rc::Rc, iter};
use rand::{distributions::Alphanumeric, Rng};

use drevo::{Drevo, Vozlišče::{*, self}, VozliščeOption::{*, self}};
use tip::Tip;
use tokenizer::{Token::{*, self}, L};
use loci::*;

use self::napaka::{Napake, OznakaNapake::*, Napaka};

struct Argumenti {
    tipi: Vec<Tip>,
    spremenljivke: Vec<Rc<Vozlišče>>,
    argumenti: Vec<Rc<Vozlišče>>,
}

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

fn prireditveni_op(op: &str) -> VozliščeOption {
    match op {
        "+="  => Aritmetični(Seštevanje),
        "-="  => Aritmetični(Odštevanje),
        "*="  => Aritmetični(Množenje),
        "/="  => Aritmetični(Deljenje),
        "%="  => Aritmetični(Modulo),
        "**=" => Aritmetični(Potenca),
        "||=" => Logični(Disjunkcija),
        "&&=" => Logični(Konjunkcija),
        "<<=" => Bitni(BitniPremikLevo),
        ">>=" => Bitni(BitniPremikDesno),
        "|="  => Bitni(BitniAli),
        "^="  => Bitni(BitniXor),
        "&="  => Bitni(BitniIn),
        _     => Brez,
    }
}

const PRIMERJALNI_OP: [&str; 6] = ["==", "!=", ">", ">=", "<", "<="];
fn primerjalni_op(op: &str) -> Option<fn(Tip, Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče> {
    match op {
        "==" => Some(Enako),
        "!=" => Some(NiEnako),
        ">"  => Some(Večje),
        ">=" => Some(VečjeEnako),
        "<"  => Some(Manjše),
        "<=" => Some(ManjšeEnako),
        _    => None,
    }
}

fn aritmetični_op(op: &str) -> fn(Tip, Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
    match op {
        "+"  => Seštevanje,
        "-"  => Odštevanje,
        "*"  => Množenje,
        "/"  => Deljenje,
        "%"  => Modulo,
        "**" => Potenca,
        _    => unreachable!()
    }
}

fn bitni_op(op: &str) -> fn(Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
    match op {
        "|"  => BitniAli,
        "^"  => BitniXor,
        "&"  => BitniIn,
        "<<"  => BitniPremikLevo,
        ">>"  => BitniPremikDesno,
        _    => unreachable!()
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

    fn predprocesiraj(izraz: &[Token<'a>]) -> Vec<Token<'a>> {
        let mut predproc = Vec::new();
        let mut znotraj_komentarja = false;

        // predproc = izraz brez komentarjev
        for i in 0..izraz.len() {
            match izraz[i] {
                Ločilo("#", ..)  => znotraj_komentarja = true,
                Ločilo("\n", ..) => { if znotraj_komentarja { znotraj_komentarja = false }; predproc.push(izraz[i]) },
                _ => if !znotraj_komentarja { predproc.push(izraz[i]) },
            }
        }

        let mut i = 0;

        // odstrani razna zaporedja oklepajev, ločil in "\n",
        // da se pravilno prevede
        while i < predproc.len() - 1 {
            i += match predproc[i..] {
                [ Ločilo("\n", ..), Ločilo("{", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("\n", ..), Ločilo("}", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("{", ..),  Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 0 },

                [ Ločilo("\n", ..), Ločilo("(", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("\n", ..), Ločilo(")", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("(", ..),  Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 0 },

                [ Ločilo("\n", ..), Ločilo("=", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("=", ..),  Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 0 },

                [ Ločilo("\n", ..), Rezerviranka("čene", ..) , .. ] => { predproc.remove(i+1); 0 },
                [ Rezerviranka(..), Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 0 },

                [ Ločilo("\n", ..), Ločilo("\n", ..), .. ] => { predproc.remove(i+0); 0 }
                _ => 1,
            };
        }
        predproc
    }

    fn parse(&mut self, izraz: &[Token<'a>]) -> Result<Drevo, Napake> {
        let okvir = self.okvir(&Parser::predprocesiraj(izraz))?;
        Ok(Drevo::new(okvir))
    }

    fn okvir(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        if !self.znotraj_funkcije {
            self.spremenljivke_stack.push(HashMap::new());
            self.funkcije_stack.push(HashMap::new());
            self.reference_stack.push(HashMap::new());
        }

        let zaporedje = self.zaporedje(izraz)?;

        let št_spr = if !self.znotraj_funkcije {
            self.spremenljivke_stack.last().unwrap()
                .values()
                .map(|s| s.sprememba_stacka() as usize)
                .sum()
        }
        else {
            0
        };

        if !self.znotraj_funkcije {
            for (ime, _) in self.spremenljivke_stack.pop().unwrap() {
                self.spremenljivke.remove(&ime);
            }
            for (ime, _) in self.funkcije_stack.pop().unwrap() {
                self.funkcije.remove(&ime);
            }
            for (ime, _) in self.reference_stack.pop().unwrap() {
                self.reference.remove(&ime);
            }
        }

        Ok(Okvir { zaporedje, št_spr }.rc())
    }

    // zaporedje izrazov, ločeno z ";" in "\n"
    fn zaporedje(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let zaporedje = razdeli(izraz, &[";", "\n"])?;
        let mut izrazi: Vec<Rc<Vozlišče>> = Vec::new();
        let mut napake = Napake::new();

        for stavek in zaporedje {
            match self.stavek(stavek) {
                Ok(stavek) => izrazi.push(stavek),
                Err(n) => napake.razširi(n),
            }
        }

        if napake.prazno() {
            Ok(Zaporedje(izrazi).rc())
        }
        else {
            Err(napake)
        }
    }

    fn stavek<'b>(&mut self, izraz: &'b[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> where 'a: 'b {
        match izraz {
            // multifunkcijski klic
            [ ime @ Ime(..), Operator("!", ..), Ločilo("(", ..), argumenti @ .., Ločilo(")", ..) ] => self.multi_klic(ime, argumenti),
            // inicializacija
            [ Rezerviranka("naj", ..), ime @ Ime(..), Operator("=", ..), ostanek @ .. ] => self.inicializacija(ime, ostanek),
            // prirejanje referenci
            [ ime @ Ime(..), Operator("@", ..), Operator("=", ..), ostanek @ .. ] => self.prirejanje_ref(ime, ostanek),
            // prirejanje
            [ ime @ Ime(..), Operator("=", ..), ostanek @ .. ] => self.prirejanje(ime, ostanek),
            // kombinirano prirejanje referenci (+=, -=, *= ...)
            [ ime @ Ime(..), Operator("@", ..), operator @ Operator(op, ..), ostanek @ .. ] => {
                match prireditveni_op(op) {
                    Brez => Err(Napake::from_zaporedje(izraz, E1, "Neznan izraz")),
                    _ => self.kombinirano_prirejanje_ref(ime, operator, ostanek),
                }
            },
            // kombinirano prirejanje (+=, -=, *= ...)
            [ ime @ Ime(..), operator @ Operator(op, ..), ostanek @ .. ] => {
                match prireditveni_op(op) {
                    Brez => Err(Napake::from_zaporedje(izraz, E1, "Neznan izraz")),
                    _ => self.kombinirano_prirejanje(ime, operator, ostanek),
                }
            },
            // okvir
            [ Ločilo("{", ..), vmes @ .., Ločilo("}", ..) ] => self.okvir(vmes),
            // funkcija natisni (zaenkrat še posebna funkcija)
            [ Ime("natisni", ..), Ločilo("(", ..), vmes @ .., Ločilo(")", ..) ] => Ok(Natisni(self.argumenti(vmes)?.argumenti).rc()),
            // funkcijski klic
            [ ime @ Ime(..), Ločilo("(", ..), argumenti @ .., Ločilo(")", ..) ] => self.funkcijski_klic_zavrzi_izhod(ime, argumenti),
            // pogojni stavek
            [ Rezerviranka("če", ..), ostanek @ .. ] => self.pogojni_stavek(ostanek),
            // zanka dokler (while loop)
            [ Rezerviranka("dokler", ..), ostanek @ .. ] => self.zanka_dokler(ostanek),
            // deklaracija funkcije
            [ Rezerviranka("funkcija", ..), ime @ Ime(..), ostanek @ .. ] => self.funkcija(ime, ostanek),
            // vrni (return)
            [ vrni @ Rezerviranka("vrni", ..), ostanek @ .. ] => self.vrni(vrni, ostanek),
            // prazen stavek
            [  ] => Ok(Prazno.rc()),
            // neznan stavek (noben od zgornjih)
            _ => Err(Napake::from_zaporedje(izraz, E1, "Neznan izraz")),
        }
    }

    fn inicializacija(&mut self, ime: &Token<'a>, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let izraz = self.drevo(izraz)?;
        let spremenljivka = match self.spremenljivke.get(ime.as_str()) {
            Some(_) => Err(Napake::from_zaporedje(&[*ime], E2, "Spremenljivka že obstaja")),
            None => {
                Ok(self.dodaj_spremenljivko(ime.to_string(), izraz.tip()))
            }
        }?;

        Ok(Prirejanje { spremenljivka, izraz }.rc())
    }

    fn prirejanje(&mut self, ime: &Token<'a>, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let izraz = self.drevo(izraz)?;
        let spremenljivka = self.spremenljivke.get(ime.as_str())
            .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?
            .clone();

        Ok(Prirejanje { spremenljivka, izraz }.rc())
    }

    fn prirejanje_ref(&mut self, ime: &Token<'a>, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let izraz = self.drevo(izraz)?;
        let referenca = self.spremenljivke.get(ime.as_str())
            .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?
            .clone();

        Ok(PrirejanjeRef { referenca, izraz }.rc())
    }

    fn kombinirano_prirejanje(&mut self, ime: &Token, operator: &Token, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let spremenljivka = self.spremenljivke.get(ime.as_str())
            .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?.clone();
        let drevo = self.drevo(izraz)?;

        let izraz = match prireditveni_op(operator.as_str()) {
            Aritmetični(op) => match (spremenljivka.tip(), drevo.tip()) {
               (Tip::Celo, Tip::Celo) => Ok(op(Tip::Celo, spremenljivka.clone(), drevo)),
               (Tip::Real, Tip::Real) => Ok(op(Tip::Real, spremenljivka.clone(), drevo)),
               _ => Err(Napake::from_zaporedje(&[*operator], E3,
                                               &format!("Nekompatibilna tipa: {} {} {}", spremenljivka.tip(), operator.as_str(), drevo.tip()))),
            },
            Logični(op) => match (spremenljivka.tip(), drevo.tip()) {
                (Tip::Bool, Tip::Bool) => Ok(op(spremenljivka.clone(), drevo)),
               _ => Err(Napake::from_zaporedje(&[*operator], E3,
                                               &format!("Nekompatibilna tipa: {} {} {}", spremenljivka.tip(), operator.as_str(), drevo.tip()))),
            }
            Bitni(op) => match (spremenljivka.tip(), drevo.tip()) {
                (Tip::Celo, Tip::Celo) => Ok(op(spremenljivka.clone(), drevo)),
               _ => Err(Napake::from_zaporedje(&[*operator], E3,
                                               &format!("Nekompatibilna tipa: {} {} {}", spremenljivka.tip(), operator.as_str(), drevo.tip()))),
            }
            Brez => Err(Napake::from_zaporedje(&[*operator], E4, "Neznan operator"))
        }?.rc();

        Ok(Prirejanje { spremenljivka, izraz }.rc())
    }

    fn kombinirano_prirejanje_ref(&mut self, ime: &Token, operator: &Token, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let referenca = self.spremenljivke.get(ime.as_str())
            .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?.clone();
        let drevo = self.drevo(izraz)?;

        let izraz = match prireditveni_op(operator.as_str()) {
            Aritmetični(op) => match (referenca.tip(), drevo.tip()) {
               (Tip::Celo, Tip::Celo) => Ok(op(Tip::Celo, referenca.clone(), drevo)),
               (Tip::Real, Tip::Real) => Ok(op(Tip::Real, referenca.clone(), drevo)),
               _ => Err(Napake::from_zaporedje(&[*operator], E3,
                                               &format!("Nekompatibilna tipa: {} {} {}", referenca.tip(), operator.as_str(), drevo.tip()))),
            },
            Logični(op) => match (referenca.tip(), drevo.tip()) {
                (Tip::Bool, Tip::Bool) => Ok(op(referenca.clone(), drevo)),
               _ => Err(Napake::from_zaporedje(&[*operator], E3,
                                               &format!("Nekompatibilna tipa: {} {} {}", referenca.tip(), operator.as_str(), drevo.tip()))),
            }
            Bitni(op) => match (referenca.tip(), drevo.tip()) {
                (Tip::Celo, Tip::Celo) => Ok(op(referenca.clone(), drevo)),
               _ => Err(Napake::from_zaporedje(&[*operator], E3,
                                               &format!("Nekompatibilna tipa: {} {} {}", referenca.tip(), operator.as_str(), drevo.tip()))),
            }
            Brez => Err(Napake::from_zaporedje(&[*operator], E4, "Neznan operator"))
        }?.rc();

        Ok(PrirejanjeRef { referenca, izraz }.rc())
    }

    fn vrni(&mut self, vrni: &Token, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let drevo = self.drevo(izraz)?;
        let spremenljivka = self.spremenljivke.get("0_vrni")
            .ok_or(Napake::from_zaporedje(&[*vrni], E5, "nepričakovana beseda: 'vrni', uprabljena zunaj funkcije"))?.clone();

        if drevo.tip() != spremenljivka.tip() {
            return Err(Napake::from_zaporedje(izraz, E3, &format!("Ne morem vrniti spremenljivke tipa '{}' iz funkcije tipa '{}'", drevo.tip(), spremenljivka.tip())));
        }

        Ok(Vrni(Prirejanje { spremenljivka, izraz: drevo }.rc()).rc())
    }

    fn pogojni_stavek(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let (pogoj, _, izraz) = loči_spredaj(izraz, &["{"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '{'"))??;

        let (resnica, _, izraz) = loči_spredaj(izraz, &["}"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '}'"))??;

        let laž = match loči_spredaj(izraz, &["čene"]) {
            Some(Ok((_, _, d))) => match d {
                [ Rezerviranka("če", ..), .. ] | [ Ločilo("{", ..), .. ]  => Ok(d),
                _ => Err(Napake::from_zaporedje(d, E5, "Pričakovan 'čene' ali '{'"))
            },
            Some(Err(napaka)) => Err(napaka),
            None => Ok([].as_slice()),
        }?;

        let drevo = self.drevo(pogoj)?;
        if drevo.tip() != Tip::Bool {
            return Err(Napake::from_zaporedje(pogoj, E6, "Pogoj mora biti Boolova vrednost"))
        }

        Ok(PogojniStavek {
            pogoj:   drevo,
            resnica: self.okvir(resnica)?,
            laž:     self.stavek(laž)?,
        }.rc())
    }

    fn zanka_dokler(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let (pogoj_izraz, _, izraz) = loči_spredaj(izraz, &["{"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '{'"))??;

        let (telo_izraz, _, _) = loči_zadaj(izraz, &["}"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '}'"))??;

        let pogoj = self.drevo(pogoj_izraz)?;
        if pogoj.tip() != Tip::Bool {
            return Err(Napake::from_zaporedje(pogoj_izraz, E6, "Pogoj mora biti Boolova vrednost"));
        }

        self.spremenljivke_stack.push(HashMap::new());
        let telo = self.zaporedje(telo_izraz)?;
        let št_spr = self.spremenljivke_stack.pop().unwrap().values().map(|s| s.sprememba_stacka() as usize).sum::<usize>();

        Ok(Okvir { zaporedje: Zanka { pogoj, telo }.rc(), št_spr }.rc())
    }

    fn funkcija(&mut self, ime: &Token, izraz: &[Token]) -> Result<Rc<Vozlišče>, Napake> {
        let (_, _, izraz) = loči_spredaj(izraz, &["("])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '('"))??;
        let (parametri_izraz, _, izraz) = loči_spredaj(izraz, &[")"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan ')'"))??;
        let (tip_izraz, oklepaj, izraz) = loči_spredaj(izraz, &["{"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '{'"))??;

        let tip = match tip_izraz {
            [] => Ok(Tip::Brez),
            [Ločilo("->", ..)] => Err(Napake::from_zaporedje(&[*oklepaj], E5, "Za '->' pričakovan tip")),
            [Ločilo("->", ..), ostanek @ ..] => Tip::from(ostanek),
            _ =>  Err(Napake::from_zaporedje(tip_izraz, E5, "Pričakovan '-> <tip>'")),
        }?;

        let (telo, _, prazno) = loči_zadaj(izraz, &["}"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '}'"))??;

        if prazno != [] {
            return Err(Napake::from_zaporedje(prazno, E3, "Izraz funkcije se mora zaključiti z '}'"));
        }
        
        let vrni = Spremenljivka { tip: tip.clone(), ime: "0_vrni".to_string(), naslov: 0, z_odmikom: true }.rc();
        let pc   = Spremenljivka { tip: Tip::Celo, ime: "0_PC".to_string(), naslov: vrni.sprememba_stacka() as u32, z_odmikom: true }.rc();

        let mut spr_funkcije = HashMap::from([
            ("0_vrni".to_string(), vrni.clone()),
            ("0_PC".to_string(), pc.clone()),
        ]);

        let mut naslov_nove = (vrni.sprememba_stacka() + pc.sprememba_stacka()) as u32;

        let mut parametri = Vec::new(); 
        let mut napake = Napake::new();

        for parameter in parametri_izraz.split(|p| if let Ločilo(",", ..) = p { true } else { false }) {
            // imena parametrov, ločena z vejicami
            match parameter {
                [] => break,
                [Ime(..)] => (),
                _ => _ = napake.add_napaka(Napaka::from_zaporedje(parameter, E3, "Neveljavno ime parametra")),
            }

            let (ime, dvopičje, tip) = loči_spredaj(parameter, &[":"])
                .ok_or(Napake::from_zaporedje(parameter, E5, "Pričakovano ':'"))??;

            if tip == [] {
                return Err(Napake::from_zaporedje(&[*dvopičje], E5, "Za ':' pričakovan tip"))
            }

            let ime = &ime[0];
            let tip = Tip::from(tip)?;

            if spr_funkcije.contains_key(ime.as_str()) {
                return Err(Napake::from_zaporedje(&[*ime], E7, "Imena parametrov morajo biti unikatna"))
            }
            else {
                let spr = Spremenljivka { tip: tip.clone(), ime: ime.to_string(), naslov: naslov_nove, z_odmikom: true }.rc();
                spr_funkcije.insert(ime.to_string(), spr.clone());
                parametri.push(spr);
                naslov_nove += tip.sprememba_stacka() as u32;
            }
        }

        let podpis_funkcije = format!("{}({})", ime.as_str(), parametri.iter().map(|p| p.tip().to_string()).collect::<Vec<String>>().join(", "));
        spr_funkcije.insert("0_OF".to_string(), Spremenljivka { tip: Tip::Celo, ime: "0_OF".to_string(), naslov: naslov_nove, z_odmikom: true }.rc());

        let mut okolje_funkcije = Parser {
            spremenljivke_stack: self.spremenljivke_stack.clone(),
            funkcije_stack: self.funkcije_stack.clone(),
            reference_stack: self.reference_stack.clone(),
            spremenljivke: self.spremenljivke.clone(),
            funkcije: self.funkcije.clone(),
            reference: self.reference.clone(),
            znotraj_funkcije: true,
        };

        okolje_funkcije.spremenljivke_stack.push(spr_funkcije.clone());
        okolje_funkcije.spremenljivke.extend(spr_funkcije);
        okolje_funkcije.funkcije.insert(podpis_funkcije.clone(), Funkcija { 
            tip: tip.clone(),
            ime: podpis_funkcije.clone(),
            parametri: parametri.clone(),
            telo: Prazno.rc(),
            prostor: 0,
            št_klicev: 0,
        }.rc());

        let telo = okolje_funkcije.zaporedje(telo)?;
        let spr_funkcije = okolje_funkcije.spremenljivke_stack.last().unwrap();
        let prostor = spr_funkcije.values().map(|s| s.sprememba_stacka() as usize).sum::<usize>()
            - spr_funkcije["0_vrni"].sprememba_stacka() as usize
            - spr_funkcije["0_PC"].sprememba_stacka() as usize
            - parametri.iter().map(|p| p.sprememba_stacka() as usize).sum::<usize>()
            - spr_funkcije["0_OF"].sprememba_stacka() as usize;
        let fun = Funkcija { tip, ime: podpis_funkcije.clone(), parametri, telo, prostor, št_klicev: 0 }.rc();

        self.funkcije_stack.last_mut().unwrap().insert(podpis_funkcije.clone(), fun.clone());
        self.funkcije.insert(podpis_funkcije, fun.clone());
        Ok(fun)
    }

    fn funkcijski_klic<'b>(&mut self, ime: &Token, argumenti: &'b[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let Argumenti { tipi, spremenljivke, argumenti } = self.argumenti(argumenti)?;
        let podpis_funkcije = format!("{}({})", ime.as_str(), tipi.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(", "));

        let funkcija = self.funkcije.get(&podpis_funkcije)
            .ok_or(Napake::from_zaporedje(&[*ime], E2, &format!("Funkcija '{podpis_funkcije}' ne obstaja")))?
            .clone();

        if let Funkcija { tip, ime, parametri, telo, prostor, št_klicev } = &*funkcija {
            self.funkcije.insert(podpis_funkcije, Funkcija {
                tip: tip.clone(),
                ime: ime.clone(),
                parametri: parametri.clone(),
                telo: telo.clone(),
                prostor: *prostor,
                št_klicev: št_klicev + 1 }.rc());
        }

        Ok(FunkcijskiKlic { funkcija, spremenljivke: Zaporedje(spremenljivke).rc(), argumenti: Zaporedje(argumenti).rc() }.rc())
    }

    fn funkcijski_klic_zavrzi_izhod(&mut self, ime: &Token, argumenti: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let klic = self.funkcijski_klic(ime, argumenti)?;
        let velikost = klic.tip().sprememba_stacka();

        Ok(Zaporedje(vec![
            klic,
            Pop(velikost as usize).rc(),
        ]).rc())
    }

    fn multi_klic<'b>(&mut self, ime: &'b Token<'a>, argumenti_izraz: &'b [Token<'a>]) -> Result<Rc<Vozlišče>, Napake> where 'a: 'b {
        let Argumenti { tipi, spremenljivke, argumenti } = self.argumenti(argumenti_izraz)?;
        let mut funkcijski_klici: Vec<Rc<Vozlišče>> = Vec::new();
        let mut napake = Napake::new();

        for (tip, (spremenljivka, argument)) in iter::zip(tipi, iter::zip(spremenljivke, argumenti)) {
            let podpis_funkcije = format!("{}({})", ime.as_str(), tip);
            let funkcija = self.funkcije.get(&podpis_funkcije)
                .ok_or(Napake::from_zaporedje(argumenti_izraz, E2, &format!("Funkcija '{podpis_funkcije}' ne obstaja")))?
                .clone();

            if let Funkcija { tip, .. } = &*funkcija {
                if *tip == Tip::Brez {
                    funkcijski_klici.push(FunkcijskiKlic {
                        funkcija,
                        spremenljivke: Zaporedje(vec![spremenljivka.rc()]).rc(),
                        argumenti: Zaporedje(vec![argument.rc()]).rc(),
                    }.rc());
                }
                else {
                    napake.add_napaka(Napaka::from_zaporedje(&[*ime], E8,
                                      &format!("{podpis_funkcije} -> {tip}: Funkcije, ki jih vključuje multifunkcijski klic, ne smejo ničesar vračati")));
                }
            }
        }

        if napake.prazno() {
            Ok(Zaporedje(funkcijski_klici).rc())
        }
        else {
            Err(napake)
        }
    }

    fn drevo(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        self.logični(izraz)
    }

    // logični izrazi (razen negacije, ki je pri osnovnih)
    fn logični(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, &["||"]) {
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.logični(l_izraz)?;
                let d = self.logični(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Bool, Tip::Bool) => Ok(Disjunkcija(l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => match loči_zadaj(izraz, &["&&"]) {
                Some(Ok((l_izraz, op, d_izraz))) => {
                    let l = self.logični(l_izraz)?;
                    let d = self.logični(d_izraz)?;
                    match (l.tip(), d.tip()) {
                        (Tip::Bool, Tip::Bool) => Ok(Konjunkcija(l, d).rc()),
                        _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                    }
                },
                Some(Err(napaka)) => Err(napaka),
                None => self.bitni(izraz),
            }
        }
    }

    // izrazi bitne manipulacije
    fn bitni(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, &["|"]) {
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.bitni(l_izraz)?;
                let d = self.bitni(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => Ok(bitni_op(op.as_str())(l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => match loči_zadaj(izraz, &["^"]) {
                Some(Ok((l_izraz, op, d_izraz))) => {
                    let l = self.bitni(l_izraz)?;
                    let d = self.bitni(d_izraz)?;
                    match (l.tip(), d.tip()) {
                        (Tip::Celo, Tip::Celo) => Ok(bitni_op(op.as_str())(l, d).rc()),
                        _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                    }
                },
                Some(Err(napaka)) => Err(napaka),
                None => match loči_zadaj(izraz, &["&"]) {
                    Some(Ok((l_izraz, op, d_izraz))) => {
                        let l = self.bitni(l_izraz)?;
                        let d = self.bitni(d_izraz)?;
                        match (l.tip(), d.tip()) {
                            (Tip::Celo, Tip::Celo) => Ok(bitni_op(op.as_str())(l, d).rc()),
                            _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                        }
                    },
                    Some(Err(napaka)) => Err(napaka),
                    None => match loči_zadaj(izraz, &["<<", ">>"]) {
                        Some(Ok((l_izraz, op, d_izraz))) => {
                            let l = self.bitni(l_izraz)?;
                            let d = self.bitni(d_izraz)?;
                            match (l.tip(), d.tip()) {
                                (Tip::Celo, Tip::Celo) => Ok(bitni_op(op.as_str())(l, d).rc()),
                                _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                            }
                        },
                        Some(Err(napaka)) => Err(napaka),
                        None => self.primerjalni(izraz),
                    }
                }
            }
        }
    }

    // primerjalni izrazi
    fn primerjalni(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, PRIMERJALNI_OP.as_slice()) {
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.primerjalni(l_izraz)?;
                let d = self.primerjalni(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => Ok(primerjalni_op(op.as_str()).unwrap()(Tip::Celo, l, d).rc()),
                    (Tip::Real, Tip::Real) => Ok(primerjalni_op(op.as_str()).unwrap()(Tip::Real, l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Nekompatibilna tipa: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => self.aditivni(izraz)
        }
    }

    // aritmetični izrazi

    fn aditivni(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, &["+", "-"]) {
            // "-" kot unarni operator
            Some(Ok(([], Operator("-", ..), ..))) => self.aritmetični(izraz),
            Some(Ok((pred @ [.., Operator(..)], minus @ Operator("-", ..), za @ [..]))) =>
                self.aditivni([pred, [Ločilo("(", 0, 0), *minus].as_slice(), za, [Ločilo(")", 0, 0)].as_slice()].concat().as_slice()),

            // "-" kot binarni operator
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.aritmetični(l_izraz)?;
                let d = self.aritmetični(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => Ok(aritmetični_op(op.as_str())(Tip::Celo, l, d).rc()),
                    (Tip::Real, Tip::Real) => Ok(aritmetični_op(op.as_str())(Tip::Real, l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Nekompatibilna tipa: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => self.aritmetični(izraz),
        }
    }

    fn aritmetični(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, &["*", "/", "%"]) {
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.aritmetični(l_izraz)?;
                let d = self.aritmetični(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => Ok(aritmetični_op(op.as_str())(Tip::Celo, l, d).rc()),
                    (Tip::Real, Tip::Real) => Ok(aritmetični_op(op.as_str())(Tip::Real, l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Nekompatibilna tipa: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => match loči_zadaj(izraz, &["**"]) {
                Some(Ok((l_izraz, op, d_izraz))) => {
                    let l = self.aritmetični(l_izraz)?;
                    let d = self.aritmetični(d_izraz)?;
                    match (l.tip(), d.tip()) {
                        (Tip::Celo, Tip::Celo) => Ok(aritmetični_op(op.as_str())(Tip::Celo, l, d).rc()),
                        (Tip::Real, Tip::Real) => Ok(aritmetični_op(op.as_str())(Tip::Real, l, d).rc()),
                        _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Nekompatibilna tipa: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                    }
                },
                Some(Err(napaka)) => Err(napaka),
                None => self.osnovni(izraz),
            }
        }
    }

    fn osnovni(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match izraz {
            // bool
            [ Literal(L::Bool("resnica", ..)) ] => Ok(Resnica.rc()),
            [ Literal(L::Bool("laž", ..)) ] => Ok(Laž.rc()),
            // števila
            [ Literal(L::Celo(število, ..)) ] => Ok(Vozlišče::Celo(število.replace("_", "").parse().unwrap()).rc()),
            [ Literal(L::Real(število, ..)) ] => Ok(Vozlišče::Real(število.replace("_", "").parse().unwrap()).rc()),
            [ Operator("-", ..), Literal(L::Celo(str, ..)) ] => Ok(Vozlišče::Celo(-str.replace("_", "").parse::<i32>().unwrap()).rc()),
            [ Operator("-", ..), Literal(L::Real(str, ..)) ] => Ok(Vozlišče::Real(-str.replace("_", "").parse::<f32>().unwrap()).rc()),
            // niz
            [ Literal(L::Niz(niz, ..)) ] => Ok(Vozlišče::Niz(interpoliraj_niz(&niz[1..niz.len()-1])).rc()),
            // izraz v oklepaju
            [ Ločilo("(", ..), ostanek @ .., Ločilo(")", ..) ] => self.drevo(ostanek),
            // klic funkcije
            [ ime @ Ime(..), Ločilo("(", ..), argumenti @ .., Ločilo(")", ..) ] => self.funkcijski_klic(ime, argumenti),
            // pretvorba tipa 
            [ izraz @ .., Operator("kot", ..), tip @ Tip(..) ] => self.pretvorba(izraz, tip),
            // zanikanje
            [ Operator("!", ..), ostanek @ .. ] => {
                let drevo = self.drevo(ostanek)?;
                match drevo.tip() {
                    Tip::Bool => Ok(Zanikaj(drevo).rc()),
                    _ => Err(Napake::from_zaporedje(izraz, E9, "Zanikati je mogoče samo Boolove vrednosti"))
                }
            },
            // negacija
            [ Operator("-", ..), ostanek @ .. ] => {
                let drevo = self.drevo(ostanek)?;
                match drevo.tip() {
                    Tip::Celo => Ok(Odštevanje(Tip::Celo, Celo(0).rc(), drevo).rc()),
                    Tip::Real => Ok(Odštevanje(Tip::Real, Celo(0).rc(), drevo).rc()),
                    _ => Err(Napake::from_zaporedje(ostanek, E9, "Izraza ni mogoče negirati"))
                }
            },
            // spremenljivka
            [ ime @ Ime(..) ] => Ok(self.spremenljivke.get(ime.as_str())
                                    .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?.clone()),

            // referenciraj
            [ Operator("@", ..), ime @ Ime(..) ] =>
                Ok(Referenca(self.spremenljivke.get(ime.as_str())
                             .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?.clone()).rc()),

            // dereferenciraj
            deref @ [ ime @ Ime(..), Operator("@", ..) ] => {
                let referenca = self.spremenljivke.get(ime.as_str())
                    .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?;

                match &**referenca {
                    Spremenljivka { tip, .. } => match &*tip {
                        Tip::Referenca(..) => Ok(Dereferenciraj(referenca.clone()).rc()),
                        _ => Err(Napake::from_zaporedje(deref, E2, "Dereferenciramo lahko samo referenco.")),
                    },
                    _ => Err(Napake::from_zaporedje(deref, E2, "Dereferenciramo lahko samo spremenljivko.")),
                }
            }

            [ neznano @ Neznano(..) ] => Err(Napake::from_zaporedje(&[*neznano], E1, "Neznana beseda")),
            [] => Ok(Prazno.rc()),
            _ => Err(Napake::from_zaporedje(izraz, E1, &format!("Neznan izraz")))
        }
    }

    fn pretvorba(&mut self, izraz: &[Token<'a>], tip_ven_izraz: &Token) -> Result<Rc<Vozlišče>, Napake> {
        let drevo = self.drevo(izraz)?.rc();
        let tip_noter = drevo.tip();
        let tip_ven = Tip::from(&[*tip_ven_izraz])?;

        match (tip_noter.clone(), tip_ven.clone()) {
            (Tip::Real, Tip::Celo) => Ok(RealVCelo(drevo).rc()),
            (Tip::Celo, Tip::Real) => Ok(CeloVReal(drevo).rc()),
            (a, b) if a == b => Ok(drevo),
            _ => Err(Napake::from_zaporedje(&[*tip_ven_izraz], E1,
                    &format!("Tipa {} ni mogoče pretvoriti v {}", tip_noter, tip_ven)))
        }
    }


    fn argumenti<'b>(&mut self, izraz: &'b[Token<'a>]) -> Result<Argumenti, Napake> where 'a: 'b {
        let mut napake = Napake::new();

        let mut tipi = Vec::new();
        let mut spremenljivke = Vec::new();
        let mut argumenti = Vec::new();
        let razdeljeno = razdeli(izraz, &[","])?;

        for argument in razdeljeno {
            match argument {
                [] => {
                    tipi.push(Tip::Brez);
                    argumenti.push(Prazno.rc());
                    spremenljivke.push(Prazno.rc());
                },
                [ Operator("@", ..), Literal(..) ] => {
                    match self.drevo(&argument[1..]) {
                        Ok(drevo) => {
                            let tip = drevo.tip();
                            let spr = self.dodaj_spremenljivko(self.naključno_ime(25), tip.clone());
                            let prirejanje = Prirejanje { spremenljivka: spr.clone(), izraz: drevo }.rc();

                            tipi.push(Tip::Referenca(Box::new(tip)));
                            spremenljivke.push(prirejanje);
                            argumenti.push(Referenca(spr).rc());
                        },
                        Err(n) => napake.razširi(n),
                    }
                },
                [ .. ] => {
                    match self.drevo(argument) {
                        Ok(drevo) => {
                            tipi.push(drevo.tip());
                            argumenti.push(drevo);
                            spremenljivke.push(Prazno.rc());
                        },
                        Err(n) => napake.razširi(n),
                    }
                },
            }
        }

        if napake.prazno() {
            Ok(Argumenti{ tipi, spremenljivke, argumenti })
        }
        else {
            Err(napake)
        }
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

#[cfg(test)]
mod testi {
    use std::rc::Rc;

    use crate::parser::tokenizer::Tokenize;

    use super::*;

    #[test]
    fn poišči() {
        assert_eq!(loči_spredaj("{}".tokenize().as_slice(), &["{"]), Some(Ok(([].as_slice(), &Ločilo("{", 1, 1), [Ločilo("}", 1, 2)].as_slice()))));
    }

    #[test]
    fn osnovni() {
        let mut parser = Parser::new();
        assert_eq!(parser.osnovni([ Literal(L::Bool("resnica", 1, 1))].as_slice()).unwrap(), Resnica.rc());
        assert_eq!(parser.osnovni([ Literal(L::Bool("laž", 1, 1))].as_slice()).unwrap(), Laž.rc());
        assert_eq!(parser.osnovni([ Operator("!", 1, 1), Literal(L::Bool("laž", 1, 2))].as_slice()).unwrap(), Zanikaj(Laž.rc()).rc());
        assert_eq!(parser.osnovni([ Ločilo("(", 1, 1), Literal(L::Bool("laž", 1, 2)), Ločilo(")", 1, 5)].as_slice()).unwrap(), Laž.rc());
        assert_eq!(parser.osnovni([ Literal(L::Celo("3", 1, 1))].as_slice()).unwrap(), Celo(3).rc());
        assert_eq!(parser.osnovni([ Literal(L::Real("3.125", 1, 1))].as_slice()).unwrap(), Real(3.125).rc());
        assert_eq!(parser.osnovni([ Literal(L::Celo("1_000", 1, 1))].as_slice()).unwrap(), Celo(1000).rc());
        assert_eq!(parser.osnovni([ Literal(L::Niz("\"angleščina\\n\"", 1, 1))].as_slice()).unwrap(), Niz("angleščina\n".to_string()).rc());

        parser.funkcije.insert("fun()".to_string(), Funkcija {
                tip: Tip::Real,
                ime: "fun".to_string(),
                parametri: vec![],
                telo: Zaporedje(vec![
                                Vrni(Prirejanje {
                                    spremenljivka: Spremenljivka { tip: Tip::Real, ime: "vrni".to_string(), naslov: 0, z_odmikom: true }.rc(),
                                    izraz: Real(1.0).rc(),
                                }.rc()).rc()
                ]).rc(),
                prostor: 0,
                št_klicev: 0,
            }.rc());
        assert_eq!(parser.osnovni([ Ime("fun", 1, 1), Ločilo("(", 1, 4), Ločilo(")", 1, 5)].as_slice()).unwrap(), FunkcijskiKlic { 
            funkcija: parser.funkcije["fun()"].clone(),
            spremenljivke: Zaporedje(vec![]).rc(),
            argumenti: Zaporedje([].to_vec()).rc(),
        }.rc());

        parser.spremenljivke.insert("a".to_string(), Rc::new(Spremenljivka { tip: Tip::Celo, ime: "a".to_string(), naslov: 0, z_odmikom: false }));
        assert_eq!(parser.osnovni([ Ime("a", 1, 1)].as_slice()).unwrap(), parser.spremenljivke["a"].clone());
    }

    #[test]
    fn pretvorba() {
        let mut parser = Parser::new();
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("kot", 1, 3), Tip("real", 1, 7) ].as_slice()).unwrap(),
            CeloVReal(Celo(3).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Real("3.0", 1, 1)), Operator("kot", 1, 3), Tip("celo", 1, 7) ].as_slice()).unwrap(),
            RealVCelo(Real(3.0).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Real("3.0", 1, 1)), Operator("kot", 1, 3), Tip("real", 1, 7) ].as_slice()).unwrap(),
            Real(3.0).rc());
    }

    #[test]
    fn aritmetični() {
        let mut parser = Parser::new();
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("+", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Seštevanje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("-", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Odštevanje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("*", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Množenje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("/", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Deljenje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("%", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Modulo(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("**", 1, 2), Literal(L::Celo("2", 1, 4)) ].as_slice()).unwrap(),
            Potenca(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());

        assert_eq!(parser.drevo("-(3-4)".tokenize().as_slice()).unwrap(), Odštevanje(Tip::Celo, Celo(0).rc(), Odštevanje(Tip::Celo, Celo(3).rc(), Celo(4).rc()).rc()).rc());
        assert_eq!(parser.drevo("-3".tokenize().as_slice()).unwrap(), Celo(-3).rc());
        assert_eq!(parser.drevo("-3 * 2".tokenize().as_slice()).unwrap(), Množenje(Tip::Celo, Celo(-3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo("3 * -2".tokenize().as_slice()).unwrap(), Množenje(Tip::Celo, Celo(3).rc(), Celo(-2).rc()).rc());
        assert_eq!(parser.drevo("--1".tokenize().as_slice()).unwrap(), Odštevanje(Tip::Celo, Celo(0).rc(), Celo(-1).rc()).rc());
        assert_eq!(parser.drevo("2 + -1".tokenize().as_slice()).unwrap(), Seštevanje(Tip::Celo, Celo(2).rc(), Celo(-1).rc()).rc());
    }

    #[test]
    fn primerjalni() {
        let mut parser = Parser::new();
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("==", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Enako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("!=", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            NiEnako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("<=", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            ManjšeEnako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator(">=", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            VečjeEnako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("<", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Manjše(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator(">", 1, 2), Literal(L::Celo("2", 1, 4)) ].as_slice()).unwrap(),
            Večje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
    }
}
