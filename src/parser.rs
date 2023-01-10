pub mod drevo;
pub mod tokenizer;

use std::{collections::HashMap, rc::Rc};

use self::{drevo::{Drevo, Vozlišče::{*, self}}, tokenizer::{Token::{*, self}, L::{*, self}}};

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

const PRIREDITVENI: [&str; 6] = [
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "^=",
];

const PRIMERJALNI: [&str; 6] = [
    "==",
    "!=",
    ">",
    ">=",
    "<",
    "<=",
];

fn prireditveni_op(op: &str) -> fn(Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
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

fn primerjalni_op(op: &str) -> fn(Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
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

    fn predprocesiran<'b>(izraz: &'b[Token<'a>]) -> Vec<Token<'a>> 
        where 'a : 'b
    {
        let mut predproc = Vec::new();
        let mut znotraj_komentarja = false;

        // predproc = izraz brez komentarjev
        for i in 0..izraz.len() {
            match izraz[i] {
                Ločilo("#", ..) => znotraj_komentarja = true,
                Ločilo("\n", ..) => { if znotraj_komentarja { znotraj_komentarja = false }; predproc.push(izraz[i]) },
                _ => if !znotraj_komentarja { predproc.push(izraz[i]) },
            }
        }

        let mut i = 0;

        // odstrani razna zaporedja "{", "}" in "\n",
        // da se pravilno prevede
        while i < predproc.len() - 1 {
            i += match predproc[i..] {
                [ Ločilo("{", ..),  Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 0 },
                [ Ločilo("\n", ..), Ločilo("}", ..),  .. ] => { predproc.remove(i+0); 0 },
                [ Ločilo("\n", ..), Ločilo("\n", ..), .. ] => { predproc.remove(i+0); 0 }
                _ => 1,
            };
        }

        predproc
    }

    fn parse<'b>(&mut self, izraz: &'b[Token<'a>]) -> Drevo 
        where 'a: 'b
    {
        Drevo::new(self.okvir(&Parser::predprocesiran(izraz)))
    }

    fn okvir<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        self.spremenljivke_stack.push(HashMap::new());
        self.funkcije_stack.push(HashMap::new());

        let zaporedje = self.zaporedje(izraz);

        let št_spr = self.spremenljivke_stack.last().unwrap()
            .values().map(|s| s.sprememba_stacka() as usize).sum();

        for (ime, _) in self.spremenljivke_stack.pop().unwrap() {
            self.spremenljivke.remove(&ime);
        }
        for (ime, _) in self.funkcije_stack.pop().unwrap() {
            self.funkcije.remove(&ime);
        }

        Okvir { zaporedje, št_spr }.rc()
    }

    // zaporedje izrazov, ločeno z ";" in "\n"
    fn zaporedje<'b>(&mut self, mut izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        let mut izrazi: Vec<Rc<Vozlišče>> = Vec::new();
        let mut ločeno = poišči_spredaj(izraz, &[";", "\n"]);

        while ločeno.is_some() {
            let (_, prvi_stavek, ostanek) = ločeno.unwrap();
            izrazi.push(self.stavek(prvi_stavek));

            izraz = ostanek;
            ločeno = poišči_spredaj(izraz, &[";", "\n"]);
        }
        izrazi.push(self.stavek(izraz));

        Zaporedje(izrazi).rc()
    }

    fn stavek<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        match izraz {
            [ Ime(ime, ..), Operator("=", ..), ostanek @ .. ] => {
                Prirejanje {
                    spremenljivka: match self.spremenljivke.get(ime) {
                        Some(spr) => spr.clone(),
                        None => {
                            let spr = Spremenljivka {
                                ime: ime.to_string(),
                                naslov: self.spremenljivke.len() as u32,
                                z_odmikom: self.znotraj_funkcije
                            }.rc();
                            self.spremenljivke_stack.last_mut().unwrap().insert(ime, spr.clone());
                            self.spremenljivke.insert(ime, spr.clone());
                            spr
                        }
                    },
                    izraz: self.drevo(ostanek),
                }.rc()
            },

            [ Ime(ime_l, ..), Operator(operator, ..), ostanek @ .. ] => {
                if !PRIREDITVENI.contains(operator) {
                    panic!("Neznan operator: {:?}", izraz[1]);
                }
                let operator = prireditveni_op(&operator);
                Prirejanje {
                    spremenljivka: self.spremenljivke[ime_l].clone(),
                    izraz: operator(self.spremenljivke[ime_l].clone(), self.drevo(ostanek)).rc(),
                }.rc()
            },

            [ Ločilo("{", ..), vmes @ .., Ločilo("}", ..) ] => self.okvir(vmes),
            [ Ime("natisni", ..), Ločilo("(", ..), vmes @ .., Ločilo(")", ..)] => Natisni(self.argumenti(vmes)).rc(),
            [ Rezerviranka("če", ..), .. ] => self.pogojni_stavek(izraz),
            [ Rezerviranka("dokler", ..), .., Ločilo("}", ..) ] => self.zanka_dokler(izraz),
            [ Rezerviranka("funkcija", ..), .., Ločilo("}", ..) ] => self.funkcija(izraz),
            [ Rezerviranka("vrni", ..), .. ] => Vrni(Prirejanje {
                spremenljivka: match self.spremenljivke.get("0_vrni") {
                    Some(spr) => spr.clone(),
                    None => panic!("nepričakovana beseda: 'vrni', uprabljena zunaj funkcije: {}", izraz[0].lokacija_str()),
                },
                izraz: self.drevo(&izraz[1..]),
            }.rc()).rc(),
            [  ] => Prazno.rc(),
            _ => panic!("Neznan stavek: {:?}", izraz),
        }
    }

    fn pogojni_stavek<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        let (_, _, izraz) = poišči_spredaj(izraz, &["če"])
            .expect(&format!("Pričakovan 'če': {}", izraz[0].lokacija_str()));

        let (_, pogoj, izraz) = poišči_spredaj(izraz, &["{"])
            .expect(&format!("Pričanovan '{}': {}", "{", izraz[1].lokacija_str()));

        let (_, resnica, izraz) = poišči_spredaj(izraz, &["}"])
            .expect(&format!("Pričakovan '{}': {}", "}", izraz[1].lokacija_str()));

        let laž = match poišči_spredaj(izraz, &["čene"]) {
            Some((_, _, d)) => match d {
                [ Rezerviranka("če", ..), .. ] | [ Ločilo("{", ..), .. ]  => d,
                _ => panic!("Pričakovana 'če' ali '{}': {}", "{", d[0].lokacija_str()),
            },
            None => &[],
        };

        PogojniStavek {
            pogoj:   self.drevo(pogoj),
            resnica: self.okvir(resnica),
            laž:     self.stavek(laž),
        }.rc()
    }

    fn zanka_dokler<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        let (_, _, izraz) = poišči_spredaj(izraz, &["dokler"])
            .expect(&format!("Pričakovan 'dokler': {}", izraz[0].lokacija_str()));

        let (_, pogoj, izraz) = poišči_spredaj(izraz, &["{"])
            .expect(&format!("Pričanovan '{}': {}", "{", izraz[1].lokacija_str()));

        let (_, telo, _) = poišči_zadaj(izraz, &["}"])
            .expect(&format!("Pričakovan '{}': {}", "}", izraz[1].lokacija_str()));

        let pogoj = self.drevo(pogoj);
        self.spremenljivke_stack.push(HashMap::new());
        let telo = self.zaporedje(telo);
        let št_spr = self.spremenljivke_stack.pop().unwrap().len();

        Okvir { zaporedje: Zanka { pogoj, telo }.rc(), št_spr }.rc()
    }

    fn funkcija<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        let prejšnje_spr = self.spremenljivke.clone();
        let mut spr_funkcije = HashMap::from([
            ("0_vrni", Spremenljivka { ime: "vrni".to_owned(), naslov: 0, z_odmikom: true }.rc()),
            ("0_PC", Spremenljivka { ime: "0_PC".to_owned(), naslov: 1, z_odmikom: true }.rc()),
        ]);

        let (_, _, izraz) = poišči_spredaj(izraz, &["funkcija"])
            .expect(&format!("Pričakovana 'funkcija': {}", izraz[0].lokacija_str()));

        let (_, ime, izraz) = poišči_spredaj(izraz, &["("])
            .expect(&format!("Pričakovan '(': {}", izraz[0].lokacija_str()));

        let (_, parametri_izraz, izraz) = poišči_spredaj(izraz, &[")"])
            .expect(&format!("Pričakovan ')': {}", izraz[0].lokacija_str()));

        let (_, _, izraz) = poišči_spredaj(izraz, &["{"])
            .expect(&format!("Pričakovan '{}' {}", "{", izraz[0].lokacija_str()));

        let (_, telo, _) = poišči_zadaj(izraz, &["}"])
            .expect(&format!("Pričakovan '{}' {}", "}", izraz[0].lokacija_str()));

        let ime_funkcije = ime.first()
            .expect(&format!("Manjkajoče ime funkcije."))
            .as_str();

        let mut parametri: Vec<Rc<Vozlišče>> = Vec::new(); 

        for parameter in parametri_izraz.split(|p| if let Ločilo(",", ..) = p { true } else { false }) {
            if parameter.len() != 1 {
                panic!("Neveljavno ime parametra: {}", parameter.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(" "));
            }
            let parameter = &parameter[0];

            if spr_funkcije.contains_key(parameter.to_string().as_str()) {
                panic!("Imena parametrov morajo biti unikatna: {}", parameter.to_string());
            }
            else {
                let naslov = spr_funkcije.len() as u32;
                spr_funkcije.insert(parameter.as_str(), Spremenljivka { ime: parameter.to_string(), naslov, z_odmikom: true }.rc());
                parametri.push(spr_funkcije[parameter.as_str()].clone());
            }
        }

        spr_funkcije.insert("0_OF", Spremenljivka { ime: "0_OF".to_owned(), naslov: spr_funkcije.len() as u32, z_odmikom: true }.rc());

        let fun  = Funkcija { ime: ime_funkcije.to_owned(), parametri: parametri.clone(), telo: Vozlišče::Število(0.0).rc(), prostor: 0 }.rc();
        self.funkcije_stack.last_mut().unwrap().insert(ime_funkcije, fun.clone());
        self.funkcije.insert(ime_funkcije, fun.clone());

        self.spremenljivke = spr_funkcije.clone();
        self.spremenljivke_stack.push(spr_funkcije.clone());
        self.znotraj_funkcije = true;
        let telo = self.zaporedje(telo);
        self.znotraj_funkcije = false;
        self.spremenljivke_stack.pop();
        self.spremenljivke = prejšnje_spr;

        let prostor = parametri.len();
        Funkcija { ime: ime_funkcije.to_string(), parametri, telo, prostor }.rc()
    }

    fn funkcijski_klic(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        let (_, ime, izraz) = poišči_spredaj(izraz, &["("])
             .expect(&format!("Pročakovan '(': {}", izraz[0].lokacija_str()));

        let (_, argumenti, _) = poišči_spredaj(izraz, &[")"])
            .expect(&format!("Pričakovan ')': {}", izraz.last().unwrap().lokacija_str()));

        let ime = ime[0].to_string();
        let funkcija = self.funkcije[ime.as_str()].clone();

        let argumenti = self.argumenti(argumenti);
        FunkcijskiKlic { funkcija, argumenti: Zaporedje(argumenti).rc() }.rc()
    }

    fn drevo(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        self.disjunktivni(izraz)
    }

    // LOGIČNI IZRAZI (razen negacije, ki je pri osnovnih)

    fn disjunktivni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["||"]) {
            Some((_, l, d)) => Disjunkcija(
                self.disjunktivni(l),
                self.konjunktivni(d)
            ).rc(),
            None => self.konjunktivni(izraz)
        }
    }

    fn konjunktivni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["&&"]) {
            Some((_, l, d)) => Konjunkcija(
                self.konjunktivni(l),
                self.primerjalni(d)
            ).rc(),
            None => self.primerjalni(izraz)
        }
    }

    // BITNI OPERATORJI

    fn bitni_ali(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["|"]) {
            None => self.potenčni(izraz),
            Some(("|", l, d)) => BitniAli(self.bitni_ali(l), self.bitni_xor(d)).rc(),
            _ => panic!("Neveljaven izraz: {}", izraz.iter().map(|t| t.as_str()).collect::<Vec<&str>>().join(" "))
        }
    }

    fn bitni_xor(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["&"]) {
            None => self.potenčni(izraz),
            Some(("|", l, d)) => BitniXor(self.bitni_xor(l), self.bitni_in(d)).rc(),
            _ => panic!("Neveljaven izraz: {}", izraz.iter().map(|t| t.as_str()).collect::<Vec<&str>>().join(" "))
        }
    }

    fn bitni_in(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["^"]) {
            None => self.potenčni(izraz),
            Some(("|", l, d)) => BitniIn(self.bitni_in(l), self.primerjalni(d)).rc(),
            _ => panic!("Neveljaven izraz: {}", izraz.iter().map(|t| t.as_str()).collect::<Vec<&str>>().join(" "))
        }
    }

    // PRIMERJALNI IZRAZI

    fn primerjalni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, PRIMERJALNI.as_slice()) {
            Some((op, l, d)) => primerjalni_op(op)(
                    self.primerjalni(l),
                    self.aditivni(d)
                ).rc(),
            None => self.aditivni(izraz)
        }
    }

    // ARITMETIČNI IZRAZI

    fn aditivni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["+", "-"]) {
            None => self.multiplikativni(izraz),
            Some(("+", l, d)) => Seštevanje(self.aditivni(l), self.multiplikativni(d)).rc(),
            Some(("-", l, d)) => match l {
                // [.., Operator(..)] => ,
                _ => Odštevanje(self.aditivni(l), self.aditivni(d)).rc()
            },
            _ => panic!("Neveljaven izraz: {}", izraz.iter().map(|t| t.as_str()).collect::<Vec<&str>>().join(" "))
        }
    }

    fn multiplikativni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["*", "/", "%"]) {
            None => self.potenčni(izraz),
            Some(("*", l, d)) => Množenje(self.multiplikativni(l), self.potenčni(d)).rc(),
            Some(("/", l, d)) => Deljenje(self.multiplikativni(l), self.potenčni(d)).rc(),
            Some(("%", l, d)) => Modulo  (self.multiplikativni(l), self.potenčni(d)).rc(),
            _ => panic!("Neveljaven izraz: {}", izraz.iter().map(|t| t.as_str()).collect::<Vec<&str>>().join(" "))
        }
    }

    // potenčni aritmetični izraz - zadnji ima prednost
    fn potenčni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match poišči_zadaj(izraz, &["**"]) {
            None => self.osnovni(izraz),
            Some((_, l, d)) => Potenca(self.potenčni(l), self.osnovni(d)).rc()
        }
    }

    fn osnovni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        match izraz {
            [ Literal(L::Bool("resnica", ..)) ] => Resnica.rc(),
            [ Literal(L::Bool("laž", ..)) ] => Laž.rc(),
            [ Operator("!", ..), ostanek @ .. ] => Zanikaj(self.drevo(ostanek)).rc(),
            [ Ločilo("(", ..), ostanek @ .., Ločilo(")", ..) ] => self.drevo(ostanek),
            [ Literal(L::Število(število, ..)) ] => Vozlišče::Število(match število.replace("_", "").parse() {
                Ok(število) => število,
                Err(_) => panic!("Vredosti {število} ni mogoče pretvoriti v število"),
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
            [ Ime(ime, ..) ] => match self.spremenljivke.get(ime) {
                Some(spr) => spr.clone(),
                None => panic!("Neznana spremenljivka: {}", ime),
            },
            [  ] => Prazno.rc(),
            _ => panic!("Neveljaven izraz: {:?}", izraz),
        }
    }

    fn argumenti(&self, mut izraz: &'a[Token<'a>]) -> Vec<Rc<Vozlišče>> {
        let mut argumenti: Vec<Rc<Vozlišče>> = Vec::new();
        let mut razdeljeno = poišči_spredaj(izraz, &[","]);

        while razdeljeno.is_some() {
            let (_, argument, ostanek) = razdeljeno.unwrap();
            argumenti.push(self.drevo(argument));
            izraz = ostanek;
            razdeljeno = poišči_spredaj(izraz, &[","]);
        }

        argumenti.push(self.drevo(izraz));
        argumenti
    }

}

fn poišči_spredaj<'a, 'b>(izraz: &'b[Token<'a>], nizi: &[&'static str]) -> Option<(&'a str, &'b[Token<'a>], &'b[Token<'a>])>
    where 'a: 'b
{
    let mut navadnih: isize = 0;
    let mut zavitih:  isize = 0;
    let mut oglatih:  isize = 0;

    ////println!("Iščemo {:?}", nizi);

    for (i, tok) in izraz.iter().enumerate() {
        match tok.as_str() {
            ")" => navadnih -= 1,
            "}" => zavitih  -= 1,
            "]" => oglatih  -= 1,
            _   => ()
        }

        ////println!("{}, {}, {} - \"{}\"", navadnih, zavitih, oglatih, tok.as_str());

        if navadnih <= 0 && zavitih <= 0 && oglatih <= 0
            && nizi.iter().any(|s| *s == tok.as_str()) {
                ////println!("{:?} najden", tok);
                return Some((tok.as_str(), &izraz[..i], &izraz[i+1..]));
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

fn poišči_zadaj<'a, 'b>(izraz: &'b[Token<'a>], nizi: &[&'static str]) -> Option<(&'a str, &'b[Token<'a>], &'b[Token<'a>])> 
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
                return Some((tok.as_str(), &izraz[..i], &izraz[i+1..]));
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
        assert_eq!(poišči_spredaj("{}".to_string().tokenize().as_slice(), &["{"]), Some(("{", [].as_slice(), [Ločilo("}", 1, 2)].as_slice())));
    }

    #[test]
    fn osnovni() {
        let mut parser = Parser::new();
        assert_eq!(parser.osnovni([ Literal(Bool("resnica", 1, 1))].as_slice()), Resnica.rc());
        assert_eq!(parser.osnovni([ Literal(Bool("laž", 1, 1))].as_slice()), Laž.rc());
        assert_eq!(parser.osnovni([ Operator("!", 1, 1), Literal(Bool("laž", 1, 2))].as_slice()), Zanikaj(Laž.rc()).rc());
        assert_eq!(parser.osnovni([ Ločilo("(", 1, 1), Literal(Bool("laž", 1, 2)), Ločilo(")", 1, 5)].as_slice()), Laž.rc());
        assert_eq!(parser.osnovni([ Literal(L::Število("3", 1, 1))].as_slice()), Število(3.0).rc());
        assert_eq!(parser.osnovni([ Literal(L::Število("3.125", 1, 1))].as_slice()), Število(3.125).rc());
        assert_eq!(parser.osnovni([ Literal(L::Število("1_000", 1, 1))].as_slice()), Število(1000.0).rc());
        assert_eq!(parser.osnovni([ Literal(L::Niz("\"angleščina\\n\"", 1, 1))].as_slice()), Niz("angleščina\n".to_owned()).rc());

        parser.funkcije.insert("fun", Funkcija {
                ime: "fun".to_string(),
                parametri: vec![],
                telo: Vrni(Prirejanje {
                    spremenljivka: Spremenljivka { ime: "vrni".to_string(), naslov: 0, z_odmikom: true }.rc(),
                    izraz: Število(1.0).rc(),
                }.rc()).rc(),
                prostor: 0,
            }.rc());
        assert_eq!(parser.osnovni([ Ime("fun", 1, 1), Ločilo("(", 1, 4), Ločilo(")", 1, 5)].as_slice()), FunkcijskiKlic { 
            funkcija: parser.funkcije["fun"].clone(),
            argumenti: Zaporedje([Prazno.rc()].to_vec()).rc(),
        }.rc());

        parser.spremenljivke.insert("a", Rc::new(Spremenljivka { ime: "a".to_owned(), naslov: 0, z_odmikom: false }));
        assert_eq!(parser.osnovni([ Ime("a", 1, 1)].as_slice()), parser.spremenljivke["a"].clone());
    }

    #[test]
    fn aritmetični() {
        let parser = Parser::new();
        assert_eq!(parser.drevo([ Literal(L::Število("3", 1, 1)), Operator("+", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        Seštevanje(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Število("3", 1, 1)), Operator("-", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        Odštevanje(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Število("3", 1, 1)), Operator("*", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        Množenje(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Število("3", 1, 1)), Operator("/", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        Deljenje(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Število("3", 1, 1)), Operator("%", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        Modulo(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Število("3", 1, 1)), Operator("**", 1, 2), Literal(L::Število("2", 1, 4)) ].as_slice()),
        Potenca(Število(3.0).rc(), Število(2.0).rc()).rc());
    }

    #[test]
    fn primerjalni() {
        let parser = Parser::new();
        assert_eq!(parser.primerjalni([ Literal(L::Število("3", 1, 1)), Operator("==", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        Enako(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Število("3", 1, 1)), Operator("!=", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        NiEnako(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Število("3", 1, 1)), Operator("<=", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        ManjšeEnako(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Število("3", 1, 1)), Operator(">=", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        VečjeEnako(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Število("3", 1, 1)), Operator("<", 1, 2), Literal(L::Število("2", 1, 3)) ].as_slice()),
        Manjše(Število(3.0).rc(), Število(2.0).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Število("3", 1, 1)), Operator(">", 1, 2), Literal(L::Število("2", 1, 4)) ].as_slice()),
        Večje(Število(3.0).rc(), Število(2.0).rc()).rc());
    }
}
