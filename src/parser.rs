pub mod drevo;
pub mod tokenizer;
pub mod tip;

use std::{collections::HashMap, rc::Rc};

use self::{drevo::{Drevo, Vozlišče::{*, self}}, tip::Tip, tokenizer::{Token::{*, self}, L}};

#[derive(Debug)]
struct Parser<'a> {
    spremenljivke_stack: Vec<HashMap<&'a str, Rc<Vozlišče>>>,
    funkcije_stack: Vec<HashMap<&'a str, Rc<Vozlišče>>>,
    spremenljivke: HashMap<&'a str, Rc<Vozlišče>>,
    funkcije: HashMap<&'a str, Rc<Vozlišče>>,
    znotraj_funkcije: bool,
}

pub trait Parse {
    fn parse(&self) -> Drevo;
}

impl Parse for Vec<Token<'_>> {
    fn parse(&self) -> Drevo {
        Parser::new().parse(self)
    }
}

const PRIREDITVENI_OP: [&str; 6] = ["+=", "-=", "*=", "/=", "%=", "^="];
fn prireditveni_op(op: &str) -> fn(Tip, Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
    match op {
        "+="  => Seštevanje,
        "-="  => Odštevanje,
        "*="  => Množenje,
        "/="  => Deljenje,
        "%="  => Modulo,
        "**=" => Potenca,
        _     => unreachable!()
    }
}

const PRIMERJALNI_OP: [&str; 6] = ["==", "!=", ">", ">=", "<", "<="];
fn primerjalni_op(op: &str) -> fn(Tip, Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
    match op {
        "==" => Enako,
        "!=" => NiEnako,
        ">"  => Večje,
        ">=" => VečjeEnako,
        "<"  => Manjše,
        "<=" => ManjšeEnako,
        _    => unreachable!()
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
            spremenljivke: HashMap::new(),
            funkcije: HashMap::new(),
            znotraj_funkcije: false,
        }
    }

    fn predprocesiran<'b>(izraz: &'b[Token<'a>]) -> Vec<Token<'a>> where 'a : 'b {
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

    fn parse<'b>(&mut self, izraz: &'b[Token<'a>]) -> Drevo where 'a: 'b {
        Drevo::new(self.okvir(&Parser::predprocesiran(izraz)))
    }

    fn okvir<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče> where 'a: 'b {
        if !self.znotraj_funkcije {
            self.spremenljivke_stack.push(HashMap::new());
            self.funkcije_stack.push(HashMap::new());
        }

        let zaporedje = self.zaporedje(izraz);

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
        }

        Okvir { zaporedje, št_spr }.rc()
    }

    // zaporedje izrazov, ločeno z ";" in "\n"
    fn zaporedje<'b>(&mut self, mut izraz: &'b[Token<'a>]) -> Rc<Vozlišče> where 'a: 'b {
        let mut izrazi: Vec<Rc<Vozlišče>> = Vec::new();
        let mut ločeno = poišči_spredaj(izraz, &[";", "\n"]);

        while ločeno.is_some() {
            let (prvi_stavek, _, ostanek) = ločeno.unwrap();
            izrazi.push(self.stavek(prvi_stavek));

            izraz = ostanek;
            ločeno = poišči_spredaj(izraz, &[";", "\n"]);
        }
        izrazi.push(self.stavek(izraz));

        Zaporedje(izrazi).rc()
    }

    fn stavek<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče> where 'a: 'b {
        match izraz {
            [ Ime(ime, ..), Operator("=", ..), ostanek @ .. ] => {
                let izraz = self.drevo(ostanek);
                Prirejanje {
                    spremenljivka: match self.spremenljivke.get(ime) {
                        Some(spr) => spr.clone(),
                        None => {
                            let spr = Spremenljivka {
                                tip: izraz.tip(),
                                ime: ime.to_string(),
                                naslov: if self.znotraj_funkcije { self.spremenljivke_stack.last().unwrap().len() } else { self.spremenljivke.len() } as u32,
                                z_odmikom: self.znotraj_funkcije
                            }.rc();
                            self.spremenljivke_stack.last_mut().unwrap().insert(ime, spr.clone());
                            self.spremenljivke.insert(ime, spr.clone());
                            spr
                        }
                    },
                    izraz,
                }.rc()
            },

            [ Ime(ime_l, ..), Operator(operator, ..), ostanek @ .. ] => {
                if !PRIREDITVENI_OP.contains(operator) {
                    panic!("Neznan operator: {:?}", izraz[1]);
                }
                let operator = prireditveni_op(&operator);
                let drevo = self.drevo(ostanek);
                match self.spremenljivke.get(ime_l) {
                    Some(spremenljivka) => {
                        let tip = if let Spremenljivka { tip, .. } = &**spremenljivka { *tip } else { Tip::Brez };
                        Prirejanje {
                            spremenljivka: spremenljivka.clone(),
                            izraz: operator(tip, spremenljivka.clone(), drevo).rc()
                        }.rc()
                    },
                    None => panic!("Spremenljivka {ime_l} ne obstaja: {}", izraz[0].lokacija_str())
                }
            },

            [ Ločilo("{", ..), vmes @ .., Ločilo("}", ..) ] => self.okvir(vmes),
            [ Ime("natisni", ..), Ločilo("(", ..), vmes @ .., Ločilo(")", ..)] => Natisni(self.argumenti(vmes)).rc(),
            [ Rezerviranka("če", ..), .. ] => self.pogojni_stavek(izraz),
            [ Rezerviranka("dokler", ..), .., Ločilo("}", ..) ] => self.zanka_dokler(&izraz[1..]),
            [ Rezerviranka("funkcija", ..), .., Ločilo("}", ..) ] => self.funkcija(&izraz[1..]),
            [ Rezerviranka("vrni", ..), .. ] => {
                let drevo = self.drevo(&izraz[1..]);
                Vrni(Prirejanje {
                    spremenljivka: match self.spremenljivke.get("0_vrni") {
                        Some(spr) => spr.clone(),
                        None => panic!("nepričakovana beseda: 'vrni', uprabljena zunaj funkcije: {}", izraz[0].lokacija_str()),
                    },
                    izraz: drevo,
                }.rc()).rc()
            },
            [  ] => Prazno.rc(),
            _ => panic!("Neznan stavek: {:?}", izraz),
        }
    }

    fn pogojni_stavek<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče> where 'a: 'b {
        let (_, _, izraz) = poišči_spredaj(izraz, &["če"])
            .expect(&format!("Pričakovan 'če': {}", izraz[0].lokacija_str()));

        let (pogoj, _, izraz) = poišči_spredaj(izraz, &["{"])
            .expect(&format!("Pričanovan '{}': {}", "{", izraz[1].lokacija_str()));

        let (resnica, _, izraz) = poišči_spredaj(izraz, &["}"])
            .expect(&format!("Pričakovan '{}': {}", "}", izraz[1].lokacija_str()));

        let laž = match poišči_spredaj(izraz, &["čene"]) {
            Some((_, _, d)) => match d {
                [ Rezerviranka("če", ..), .. ] | [ Ločilo("{", ..), .. ]  => d,
                _ => panic!("Pričakovana 'če' ali '{}': {}", "{", d[0].lokacija_str()),
            },
            None => &[],
        };

        let drevo = self.drevo(pogoj);
        if drevo.tip() != Tip::Bool {
            panic!("Pogoj mora biti Boolova vrednost: {}", pogoj[0].lokacija_str());
        }

        PogojniStavek {
            pogoj: drevo,
            resnica: self.okvir(resnica),
            laž:     self.stavek(laž),
        }.rc()
    }

    fn zanka_dokler<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče> where 'a: 'b {
        let (pogoj_izraz, _, izraz) = poišči_spredaj(izraz, &["{"])
            .expect(&format!("Pričanovan '{}': {}", "{", izraz[1].lokacija_str()));

        let (telo_izraz, _, _) = poišči_zadaj(izraz, &["}"])
            .expect(&format!("Pričakovan '{}': {}", "}", izraz[1].lokacija_str()));

        let pogoj = self.drevo(pogoj_izraz);
        if pogoj.tip() != Tip::Bool {
            panic!("Pogoj mora biti Boolova vrednost: {}", pogoj_izraz[0].lokacija_str());
        }

        self.spremenljivke_stack.push(HashMap::new());
        let telo = self.zaporedje(telo_izraz);
        let št_spr = self.spremenljivke_stack.pop().unwrap().len();

        Okvir { zaporedje: Zanka { pogoj, telo }.rc(), št_spr }.rc()
    }

    fn funkcija<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče> where 'a: 'b {
        let mut okolje_funkcije = Parser {
            spremenljivke_stack: self.spremenljivke_stack.clone(),
            funkcije_stack: self.funkcije_stack.clone(),
            spremenljivke: self.spremenljivke.clone(),
            funkcije: self.funkcije.clone(),
            znotraj_funkcije: true,
        };

        let (ime, _, izraz) = poišči_spredaj(izraz, &["("])
            .expect(&format!("Pričakovan '(': {}", izraz[0].lokacija_str()));

        let (parametri_izraz, _, izraz) = poišči_spredaj(izraz, &[")"])
            .expect(&format!("Pričakovan ')': {}", izraz[0].lokacija_str()));

        let (_, _, izraz) = poišči_spredaj(izraz, &["->"])
            .expect(&format!("Pričakovan '->': {}", izraz[0].lokacija_str()));

        let (tip_izraz, _, izraz) = poišči_spredaj(izraz, &["{"])
            .expect(&format!("Pričakovan '{}' {}", "{", izraz[0].lokacija_str()));

        let (telo, _, _) = poišči_zadaj(izraz, &["}"])
            .expect(&format!("Pričakovan '{}' {}", "}", izraz[0].lokacija_str()));

        let ime_funkcije = ime.first()
            .expect(&format!("Manjkajoče ime funkcije."))
            .as_str();

        let tip = Tip::from(tip_izraz[0].as_str());

        let mut spr_funkcije = HashMap::from([
            ("0_vrni", Spremenljivka { tip, ime: "0_vrni".to_owned(), naslov: 0, z_odmikom: true }.rc()),
            ("0_PC", Spremenljivka { tip: Tip::Celo, ime: "0_PC".to_owned(), naslov: 1, z_odmikom: true }.rc()),
        ]);

        let mut parametri: Vec<Rc<Vozlišče>> = Vec::new(); 

        for parameter in parametri_izraz.split(|p| if let Ločilo(",", ..) = p { true } else { false }) {
            if parameter.is_empty() {
                break;
            }

            let (ime, _, tip) = poišči_spredaj(parameter, &[":"])
                .expect(&format!("Pričakovano ':': {}", parameter[0].lokacija_str()));
            let ime = &ime[0];
            let tip = Tip::from(tip[0].as_str());

            if spr_funkcije.contains_key(ime.as_str()) {
                panic!("Imena parametrov morajo biti unikatna: {}", ime.as_str());
            }
            else {
                let naslov = spr_funkcije.len() as u32;
                let spr = Spremenljivka { tip, ime: ime.to_string(), naslov, z_odmikom: true }.rc();
                spr_funkcije.insert(ime.as_str(), spr.clone());
                parametri.push(spr);
            }
        }

        spr_funkcije.insert("0_OF", Spremenljivka { tip: Tip::Celo, ime: "0_OF".to_owned(), naslov: spr_funkcije.len() as u32, z_odmikom: true }.rc());

        okolje_funkcije.spremenljivke_stack.push(spr_funkcije.clone());
        okolje_funkcije.spremenljivke.extend(spr_funkcije.clone());
        okolje_funkcije.funkcije.insert(ime_funkcije, Funkcija { tip, ime: ime_funkcije.to_string(), parametri: parametri.clone(), telo: Prazno.rc(), prostor: 0 }.rc());

        let telo = okolje_funkcije.zaporedje(telo);
        let prostor = okolje_funkcije.spremenljivke_stack.last().unwrap().values().map(|s| s.sprememba_stacka() as usize).sum::<usize>()
            - spr_funkcije["0_vrni"].sprememba_stacka() as usize
            - spr_funkcije["0_PC"].sprememba_stacka() as usize
            - parametri.iter().map(|p| p.sprememba_stacka() as usize).sum::<usize>()
            - NaložiOdmik.sprememba_stacka() as usize;
        let fun = Funkcija { tip, ime: ime_funkcije.to_string(), parametri, telo, prostor }.rc();

        self.funkcije_stack.last_mut().unwrap().insert(ime_funkcije, fun.clone());
        self.funkcije.insert(ime_funkcije, fun.clone());
        fun
    }

    fn funkcijski_klic(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        let (ime, izraz) = (izraz[0], &izraz[2..]);

        let (argumenti, _, _) = poišči_spredaj(izraz, &[")"])
            .expect(&format!("Pričakovan ')': {}", izraz.last().unwrap().lokacija_str()));

        let funkcija = self.funkcije.get(ime.as_str())
            .expect(&format!("Funkcija {} ne obstaja: {}", ime.as_str(), ime.lokacija_str()))
            .clone();

        let argumenti = self.argumenti(argumenti);
        FunkcijskiKlic { funkcija, argumenti: Zaporedje(argumenti).rc() }.rc()
    }

    fn drevo(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        self.logični(izraz)
    }

    // logični izrazi (razen negacije, ki je pri osnovnih)
    fn logični(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["||"]) {
            Some((l_izraz, op, d_izraz)) => {
                let l = self.logični(l_izraz);
                let d = self.logični(d_izraz);
                match (l.tip(), d.tip()) {
                    (Tip::Bool, Tip::Bool) => Disjunkcija(l, d).rc(),
                    _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                }
            },
            None => match poišči_zadaj(izraz, &["&&"]) {
                Some((l_izraz, op, d_izraz)) => {
                    let l = self.logični(l_izraz);
                    let d = self.logični(d_izraz);
                    match (l.tip(), d.tip()) {
                        (Tip::Bool, Tip::Bool) => Konjunkcija(l, d).rc(),
                        _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                    }
                },
                None => self.bitni(izraz),
            }
        }
    }

    // izrazi bitne manipulacije
    fn bitni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["|"]) {
            Some((l_izraz, op, d_izraz)) => {
                let l = self.bitni(l_izraz);
                let d = self.bitni(d_izraz);
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => bitni_op(op.as_str())(l, d).rc(),
                    _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                }
            },
            None => match poišči_zadaj(izraz, &["^"]) {
                Some((l_izraz, op, d_izraz)) => {
                    let l = self.bitni(l_izraz);
                    let d = self.bitni(d_izraz);
                    match (l.tip(), d.tip()) {
                        (Tip::Celo, Tip::Celo) => bitni_op(op.as_str())(l, d).rc(),
                        _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                    }
                },
                None => match poišči_zadaj(izraz, &["&"]) {
                    Some((l_izraz, op, d_izraz)) => {
                        let l = self.bitni(l_izraz);
                        let d = self.bitni(d_izraz);
                        match (l.tip(), d.tip()) {
                            (Tip::Celo, Tip::Celo) => bitni_op(op.as_str())(l, d).rc(),
                            _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                        }
                    },
                    None => match poišči_zadaj(izraz, &["<<", ">>"]) {
                        Some((l_izraz, op, d_izraz)) => {
                            let l = self.bitni(l_izraz);
                            let d = self.bitni(d_izraz);
                            match (l.tip(), d.tip()) {
                                (Tip::Celo, Tip::Celo) => bitni_op(op.as_str())(l, d).rc(),
                                _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                            }
                        },
                        None => self.primerjalni(izraz),
                    }
                }
            }
        }
    }

    // primerjalni izrazi
    fn primerjalni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, PRIMERJALNI_OP.as_slice()) {
            Some((l_izraz, op, d_izraz)) => {
                let l = self.primerjalni(l_izraz);
                let d = self.primerjalni(d_izraz);
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => primerjalni_op(op.as_str())(Tip::Celo, l, d).rc(),
                    (Tip::Real, Tip::Real) => primerjalni_op(op.as_str())(Tip::Real, l, d).rc(),
                    _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                }
            },
            None => self.aritmetični(izraz)
        }
    }

    // aritmetični izrazi
    fn aritmetični(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["+", "-"]) {
            // negativno število
            Some(([], Operator("-", ..), [Literal(L::Celo(str, ..))])) => Vozlišče::Celo(-str.parse::<i32>().unwrap()).rc(),
            Some(([], Operator("-", ..), [Literal(L::Real(str, ..))])) => Vozlišče::Real(-str.parse::<f32>().unwrap()).rc(),
            // "-" kot unarni operator
            Some(([], Operator("-", ..), d_izraz)) => {
                let d = self.aritmetični(d_izraz);
                match d.tip() {
                    Tip::Celo => Odštevanje(Tip::Celo, Vozlišče::Celo(0).rc(),   d).rc(),
                    Tip::Real => Odštevanje(Tip::Real, Vozlišče::Real(0.0).rc(), d).rc(),
                    _ => panic!("Neveljavna operacija: -{} {}", d_izraz[0].as_str(), izraz[0].lokacija_str()),
                }
            },
            Some((l_izraz, op, d_izraz)) => {
                let l = self.aritmetični(l_izraz);
                let d = self.aritmetični(d_izraz);
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => aritmetični_op(op.as_str())(Tip::Celo, l, d).rc(),
                    (Tip::Real, Tip::Real) => aritmetični_op(op.as_str())(Tip::Real, l, d).rc(),
                    _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                }
            },
            None => match poišči_zadaj(izraz, &["*", "/", "%"]) {
                Some((l_izraz, op, d_izraz)) => {
                    let l = self.aritmetični(l_izraz);
                    let d = self.aritmetični(d_izraz);
                    match (l.tip(), d.tip()) {
                        (Tip::Celo, Tip::Celo) => aritmetični_op(op.as_str())(Tip::Celo, l, d).rc(),
                        (Tip::Real, Tip::Real) => aritmetični_op(op.as_str())(Tip::Real, l, d).rc(),
                        _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                    }
                },
                None => match poišči_zadaj(izraz, &["**"]) {
                    Some((l_izraz, op, d_izraz)) => {
                        let l = self.aritmetični(l_izraz);
                        let d = self.aritmetični(d_izraz);
                        match (l.tip(), d.tip()) {
                            (Tip::Celo, Tip::Celo) => aritmetični_op(op.as_str())(Tip::Celo, l, d).rc(),
                            (Tip::Real, Tip::Real) => aritmetični_op(op.as_str())(Tip::Real, l, d).rc(),
                            _ => panic!("Neveljavna operacija: {} {} {} {}", l_izraz[0].as_str(), op.as_str(), d_izraz[0].as_str(), op.lokacija_str()),
                        }
                    },
                    None => self.osnovni(izraz),
                }
            }
        }
    }

    fn osnovni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match izraz {
            [ Literal(L::Bool("resnica", ..)) ] => Resnica.rc(),
            [ Literal(L::Bool("laž", ..)) ] => Laž.rc(),
            [ Operator("!", ..), ostanek @ .. ] => {
                let izraz = self.drevo(ostanek);
                match izraz.tip() {
                    Tip::Bool => Zanikaj(izraz).rc(),
                    _ => panic!("Neveljavna operacija: !{} {}", ostanek[0].as_str(), ostanek[0].lokacija_str()),
                }
            },
            [ Ločilo("(", ..), ostanek @ .., Ločilo(")", ..) ] => self.drevo(ostanek),
            [ Literal(L::Celo(število, ..)) ] => Vozlišče::Celo(match število.replace("_", "").parse() {
                Ok(število) => število,
                Err(err) => panic!("Ne morem pretvoriti {število} v število: {err}"),
            }).rc(),
            [ Literal(L::Real(število, ..)) ] => Vozlišče::Real(match število.replace("_", "").parse() {
                Ok(število) => število,
                Err(err) => panic!("Ne morem pretvoriti {število} v število: {err}"),
            }).rc(),
            [ Literal(L::Niz(niz, ..)) ] => Vozlišče::Niz(niz[1..niz.len()-1]
                                                          .to_string()
                                                          .replace(r"\\", "\\")
                                                          .replace(r"\n", "\n")
                                                          .replace(r"\t", "\t")
                                                          .replace(r"\r", "\r")
                                                          .replace(r#"\"""#, "\"")
                                                          .replace(r"\'", "\'")).rc(),
            [ Ime(..), Ločilo("(", ..), .., Ločilo(")", ..) ] => self.funkcijski_klic(izraz),
            [ Ime(ime, ..) ] => {
                let spr = match self.spremenljivke.get(ime) {
                    Some(spr) => spr.clone(),
                    None => panic!("Neznana spremenljivka: {}", ime),
                };
                spr
            },
            [] => Prazno.rc(),
            _ => panic!("Neveljaven izraz: {:?}", izraz),
        }
    }

    fn argumenti(&self, mut izraz: &'a[Token<'a>]) -> Vec<Rc<Vozlišče>> {
        let mut argumenti: Vec<Rc<Vozlišče>> = Vec::new();
        let mut razdeljeno = poišči_spredaj(izraz, &[","]);

        while razdeljeno.is_some() {
            let (argument, _, ostanek) = razdeljeno.unwrap();
            argumenti.push(self.drevo(argument));
            izraz = ostanek;
            razdeljeno = poišči_spredaj(izraz, &[","]);
        }

        argumenti.push(self.drevo(izraz));
        argumenti
    }

}

fn poišči_spredaj<'a, 'b>(izraz: &'b[Token<'a>], nizi: &[&'static str]) -> Option<(&'b[Token<'a>], &'b Token<'a>, &'b[Token<'a>])>
    where 'a: 'b
{
    let mut navadnih: isize = 0;
    let mut zavitih:  isize = 0;
    let mut oglatih:  isize = 0;

    for (i, tok) in izraz.iter().enumerate() {
        match tok.as_str() {
            ")" => navadnih -= 1,
            "}" => zavitih  -= 1,
            "]" => oglatih  -= 1,
            _   => ()
        }

        if navadnih <= 0 && zavitih <= 0 && oglatih <= 0
            && nizi.iter().any(|s| *s == tok.as_str()) {
                ////println!("{:?} najden", tok);
                return Some((&izraz[..i], tok, &izraz[i+1..]));
            }

        if navadnih < 0 || zavitih < 0 || oglatih < 0 {
            panic!("Neujemajoč oklepaj: {:?}", tok)
        }

        match tok.as_str() {
            "(" => navadnih += 1,
            "{" => zavitih  += 1,
            "[" => oglatih  += 1,
            _   => ()
        }
    }

    None
}

fn poišči_zadaj<'a, 'b>(izraz: &'b[Token<'a>], nizi: &[&'static str]) -> Option<(&'b[Token<'a>], &'b Token<'a>, &'b[Token<'a>])>
    where 'a: 'b
{
    let mut navadnih: isize = 0;
    let mut zavitih:  isize = 0;
    let mut oglatih:  isize = 0;

    for (i, tok) in izraz.iter().rev().enumerate() {
        // obrni i, drugače ima zadnji element seznama i = 0, predzadnji 1 ...
        let i = izraz.len() - 1 - i;

        match tok.as_str() {
            "(" => navadnih -= 1,
            "{" => zavitih -= 1,
            "[" => oglatih -= 1,
            _ => ()
        }

        if navadnih == 0 && zavitih == 0 && oglatih == 0
            && nizi.iter().any(|s| *s == tok.as_str()) {
                return Some((&izraz[..i], tok, &izraz[i+1..]));
            }

        if navadnih < 0 || zavitih < 0 || oglatih < 0 {
            panic!("Neujemajoč oklepaj: {:?}", tok)
        }

        match tok.as_str() {
            ")" => navadnih += 1,
            "}" => zavitih += 1,
            "]" => oglatih += 1,
            _ => ()
        }
    }

    if navadnih != 0 || zavitih != 0 || oglatih != 0 {
        panic!("Oklepaji se ne ujemajo");
    }

    None
}

#[cfg(test)]
mod testi {
    use std::rc::Rc;

    use crate::parser::tokenizer::Tokenize;

    use super::*;

    #[test]
    fn poišči() {
        assert_eq!(poišči_spredaj("{}".to_string().tokenize().as_slice(), &["{"]), Some(([].as_slice(), &Ločilo("{", 1, 1), [Ločilo("}", 1, 2)].as_slice())));
    }

    #[test]
    fn osnovni() {
        let mut parser = Parser::new();
        assert_eq!(parser.osnovni([ Literal(L::Bool("resnica", 1, 1))].as_slice()), Resnica.rc());
        assert_eq!(parser.osnovni([ Literal(L::Bool("laž", 1, 1))].as_slice()), Laž.rc());
        assert_eq!(parser.osnovni([ Operator("!", 1, 1), Literal(L::Bool("laž", 1, 2))].as_slice()), Zanikaj(Laž.rc()).rc());
        assert_eq!(parser.osnovni([ Ločilo("(", 1, 1), Literal(L::Bool("laž", 1, 2)), Ločilo(")", 1, 5)].as_slice()), Laž.rc());
        assert_eq!(parser.osnovni([ Literal(L::Celo("3", 1, 1))].as_slice()), Celo(3).rc());
        assert_eq!(parser.osnovni([ Literal(L::Real("3.125", 1, 1))].as_slice()), Real(3.125).rc());
        assert_eq!(parser.osnovni([ Literal(L::Celo("1_000", 1, 1))].as_slice()), Celo(1000).rc());
        assert_eq!(parser.osnovni([ Literal(L::Niz("\"angleščina\\n\"", 1, 1))].as_slice()), Niz("angleščina\n".to_owned()).rc());

        parser.funkcije.insert("fun", Funkcija {
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
            }.rc());
        assert_eq!(parser.osnovni([ Ime("fun", 1, 1), Ločilo("(", 1, 4), Ločilo(")", 1, 5)].as_slice()), FunkcijskiKlic { 
            funkcija: parser.funkcije["fun"].clone(),
            argumenti: Zaporedje([Prazno.rc()].to_vec()).rc(),
        }.rc());

        parser.spremenljivke.insert("a", Rc::new(Spremenljivka { tip: Tip::Celo, ime: "a".to_owned(), naslov: 0, z_odmikom: false }));
        assert_eq!(parser.osnovni([ Ime("a", 1, 1)].as_slice()), parser.spremenljivke["a"].clone());
    }

    #[test]
    fn aritmetični() {
        let parser = Parser::new();
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("+", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            Seštevanje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("-", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            Odštevanje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("*", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            Množenje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("/", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            Deljenje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("%", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            Modulo(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("**", 1, 2), Literal(L::Celo("2", 1, 4)) ].as_slice()),
            Potenca(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());

        assert_eq!(parser.drevo("-(3-4)".to_string().tokenize().as_slice()), Odštevanje(Tip::Celo, Celo(0).rc(), Odštevanje(Tip::Celo, Celo(3).rc(), Celo(4).rc()).rc()).rc());
        assert_eq!(parser.drevo("-3".to_string().tokenize().as_slice()), Celo(-3).rc());
    }

    #[test]
    fn primerjalni() {
        let parser = Parser::new();
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("==", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            Enako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("!=", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            NiEnako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("<=", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            ManjšeEnako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator(">=", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            VečjeEnako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("<", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()),
            Manjše(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator(">", 1, 2), Literal(L::Celo("2", 1, 4)) ].as_slice()),
            Večje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
    }
}
