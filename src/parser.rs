pub mod drevo;
pub mod tokenizer;

use std::{collections::HashMap, rc::Rc};

use self::{drevo::{Drevo, Vozlišče::{*, self}}, tokenizer::Token::{*, self}};

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

const PRIREJALNI: [&str; 6] = [
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "^=",
];

const PRIMERJALNI: [&str; 5] = [
    "==",
    ">",
    ">=",
    "<",
    "<=",
];

fn prirejalni_op(op: &str) -> fn(Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
    match op {
        "+=" => Seštevanje, 
        "-=" => Odštevanje, 
        "*=" => Množenje, 
        "/=" => Deljenje, 
        "%=" => Modulo,
        "^=" => Potenca,
        _    => unreachable!()
    }
}

fn primerjalni_op(op: &str) -> fn(Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
    match op {
        "==" => Enako,
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

        while i < predproc.len() - 1 {
            i += match predproc[i..] {
                [ Ločilo("{", ..),  Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 1 },
                [ Ločilo("\n", ..), Ločilo("}", ..),  .. ] => { predproc.remove(i+0); 1 },
                _ => 1,
            };
        }

        predproc
    }

    fn parse<'b>(&mut self, izraz: &'b[Token<'a>]) -> Drevo 
        where 'a: 'b
    {
        let predprocesiran = Parser::predprocesiran(izraz);
        Drevo::new(self.okvir(&predprocesiran))
    }

    fn okvir<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        println!("okvir: {:?}", izraz);

        self.spremenljivke_stack.push(HashMap::new());
        self.funkcije_stack.push(HashMap::new());

        let zaporedje = self.zaporedje(izraz);

        let št_spr = self.spremenljivke_stack.last().unwrap().values()
            .map(|spr| spr.sprememba_stacka())
            .count()
            + self.funkcije_stack.last().unwrap().values()
            .map(|spr| spr.sprememba_stacka())
            .count();

        for (ime, _) in self.spremenljivke_stack.pop().unwrap() {
            self.spremenljivke.remove(&ime);
        }

        Okvir { zaporedje, št_spr }.rc()
    }

    fn zaporedje<'b>(&mut self, mut izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        println!("zaporedje: {:?}", izraz);

        let mut izrazi: Vec<Rc<Vozlišče>> = Vec::new();

        let mut ločeno = poišči_spredaj(izraz, &[";", "\n"]);

        while ločeno.is_some() {
            let (_, prvi_stavek, ostanek) = ločeno.unwrap();
            izrazi.push(self.stavek(prvi_stavek));

            izraz = ostanek;
            ločeno = poišči_spredaj(izraz, &[";", "\n"]);
        }

        if izraz.len() > 0 {
            izrazi.push(self.stavek(izraz))
        }

        Rc::new(Zaporedje(izrazi))
    }

    fn stavek<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        println!("stavek: {:?}", izraz);

        match izraz {
            [ Ime(ime, ..), Operator("=", ..), ostanek @ .. ] => {
                Prirejanje {
                    spremenljivka: match self.spremenljivke.get(ime) {
                        Some(spr) => spr.clone(),
                        None => {
                            let spr = Spremenljivka { ime: ime.to_string(), naslov: self.spremenljivke.len() as i64, z_odmikom: self.znotraj_funkcije }.rc();
                            self.spremenljivke.insert(ime, spr.clone());
                            spr
                        }
                    },
                    izraz: self.drevo(ostanek),
                    z_odmikom: self.znotraj_funkcije
                }.rc()
            },

            [ Ime(ime_l, ..), Operator(operator, ..), ostanek @ .. ] => {
                let operator = prirejalni_op(&operator);
                Prirejanje {
                    spremenljivka: self.spremenljivke[ime_l].clone(),
                    izraz: operator(self.spremenljivke[ime_l].clone(), self.drevo(ostanek)).rc(),
                    z_odmikom: self.znotraj_funkcije
                }.rc()
            },

            [ Ločilo("{", ..), vmes @ .., Ločilo("}", ..) ] => self.okvir(vmes),
            [ Ime("natisni", ..), Ločilo("(", ..), vmes @ .., Ločilo(")", ..)] => Natisni(self.argumenti(vmes)).rc(),
            [ Literal("če", ..), .. ] => self.pogojni_stavek(izraz),
            [ Literal("dokler", ..), .., Ločilo("}", ..) ] => self.zanka(izraz),
            [ Literal("funkcija", ..), .., Ločilo("}", ..) ] => self.funkcija(izraz),
            [ Literal("vrni", ..), .. ] => Vrni(Prirejanje {
                spremenljivka: self.spremenljivke["vrni"].clone(),
                izraz: self.drevo(&izraz[1..]),
                z_odmikom: self.znotraj_funkcije
            }.rc()).rc(),
            [  ] => Prazno.rc(),
            _ => panic!("Neznan stavek: {:?}", izraz),
        }
    }

    fn pogojni_stavek<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        println!("pogojni_stavek: {:?}", izraz);

        const PRAZEN_IZRAZ: &[Token] = &[];

        let (_, _, izraz) = poišči_spredaj(izraz, &["če"])
            .expect(&format!("Pričakovan 'če': {}", izraz[0].lokacija_str()));

        let (_, pogoj, izraz) = poišči_spredaj(izraz, &["{"])
            .expect(&format!("Pričanovan '{}': {}", "{", izraz[1].lokacija_str()));

        let (_, resnica, izraz) = poišči_spredaj(izraz, &["}"])
            .expect(&format!("Pričakovan '{}': {}", "}", izraz[1].lokacija_str()));

        let laž = match poišči_spredaj(izraz, &["čene"]) {
            Some((_, _, d)) => d,
            None => PRAZEN_IZRAZ,
        };

        PogojniStavek {
            pogoj: self.drevo(pogoj),
            resnica: self.okvir(resnica).rc(),
            laž: self.zaporedje(laž),
        }.rc()
    }

    fn zanka<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        println!("zanka: {:?}", izraz);

        let (_, _, izraz) = poišči_spredaj(izraz, &["dokler"])
            .expect(&format!("Pričakovan 'dokler': {}", izraz[0].lokacija_str()));

        let (_, pogoj, izraz) = poišči_spredaj(izraz, &["{"])
            .expect(&format!("Pričanovan '{}': {}", "{", izraz[1].lokacija_str()));

        let (_, telo, _) = poišči_zadaj(izraz, &["}"])
            .expect(&format!("Pričakovan '{}': {}", "}", izraz[1].lokacija_str()));

        let nove_spr: HashMap<String, Vozlišče> = HashMap::new();
        let pogoj = self.drevo(pogoj);
        let telo = self.zaporedje(telo);

        Okvir { zaporedje: Zanka { pogoj, telo }.rc(), št_spr: nove_spr.len() }.rc()
    }

    fn funkcija<'b>(&mut self, izraz: &'b[Token<'a>]) -> Rc<Vozlišče>
        where 'a: 'b
    {
        println!("funkcija: {:?}", izraz);
        
        let prejšnje_spr = self.spremenljivke.clone();
        let mut spr_funkcije = HashMap::from([
            ("vrni", Spremenljivka { ime: "vrni".to_owned(), naslov: 0, z_odmikom: true }.rc()),
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

        let (_, telo, _) = poišči_spredaj(izraz, &["}"])
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
                let naslov = spr_funkcije.len() as i64;
                spr_funkcije.insert(parameter.as_str(), Spremenljivka { ime: parameter.to_string(), naslov, z_odmikom: true }.rc());
                parametri.push(spr_funkcije[parameter.as_str()].clone());
            }
        }

        spr_funkcije.insert("0_OF", Spremenljivka { ime: "0_OF".to_owned(), naslov: spr_funkcije.len() as i64, z_odmikom: true }.rc());

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
        println!("funkcijski_klic: {:?}", izraz);

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
        println!("drevo: {:?}", izraz);
        self.disjunktivni(izraz)
    }

    fn disjunktivni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        println!("disjunktivni: {:?}", izraz);

        match poišči_zadaj(izraz, &["ali"]) {
            Some((_, l, d)) => Disjunkcija(
                self.disjunktivni(l),
                self.konjunktivni(d)
            ).rc(),
            None => self.konjunktivni(izraz)
        }
    }

    fn konjunktivni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        println!("konjunktivni: {:?}", izraz);

        match poišči_zadaj(izraz, &["in"]) {
            Some((_, l, d)) => Konjunkcija(
                self.konjunktivni(l),
                self.primerjalni(d)
            ).rc(),
            None => self.primerjalni(izraz)
        }
    }

    fn primerjalni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        println!("primerjalni: {:?}", izraz);

        match poišči_zadaj(izraz, PRIMERJALNI.as_slice()) {
            Some((op, l, d)) => {
                primerjalni_op(op)(
                    self.konjunktivni(l),
                    self.primerjalni(d)
                ).rc()
            },
            None => self.aditivni(izraz)
        }
    }

    fn aditivni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        println!("aditivni: {:?}", izraz);

        match poišči_zadaj(izraz, &["+", "-"]) {
            None => self.multiplikativni(izraz),
            Some(("+", l, d)) => Seštevanje(self.aditivni(l), self.multiplikativni(d)).rc(),
            Some(("-", l, d)) => Odštevanje(self.aditivni(l), self.multiplikativni(d)).rc(),
            _ => panic!("Neveljaven izraz: {}", izraz.iter().map(|t| t.as_str()).collect::<Vec<&str>>().join(" "))
        }
    }

    fn multiplikativni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        println!("multiplikativni: {:?}", izraz);

        match poišči_zadaj(izraz, &["*", "/", "%"]) {
            None => self.potenčni(izraz),
            Some(("*", l, d)) => Seštevanje(self.multiplikativni(l), self.potenčni(d)).rc(),
            Some(("/", l, d)) => Odštevanje(self.multiplikativni(l), self.potenčni(d)).rc(),
            Some(("%", l, d)) => Odštevanje(self.multiplikativni(l), self.potenčni(d)).rc(),
            _ => panic!("Neveljaven izraz: {}", izraz.iter().map(|t| t.as_str()).collect::<Vec<&str>>().join(" "))
        }
    }

    fn potenčni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        println!("potenčni: {:?}", izraz);

        match poišči_zadaj(izraz, &["^"]) {
            None => self.osnovni(izraz),
            Some((_, l, d)) => Potenca(self.potenčni(l), self.osnovni(d)).rc()
        }
    }

    fn osnovni(&self, izraz: &[Token]) -> Rc<Vozlišče> {
        println!("osnovni: {:?}", izraz);

        match izraz {
            [ Literal("resnica", ..) ] => Resnica.rc(),
            [ Literal("laž", ..) ] => Laž.rc(),
            [ Ločilo("!", ..), ostanek @ .. ] => Zanikaj(self.drevo(ostanek)).rc(),
            [ Ločilo("(", ..), ostanek @ .., Ločilo(")", ..) ] => self.drevo(ostanek),
            [ Token::Število(število, ..) ] => Vozlišče::Število(število.parse().unwrap()).rc(),
            [ Token::Niz(niz, ..) ] => Vozlišče::Niz(niz[1..niz.len()-1].to_string()).rc(),
            [ Ime(..), Ločilo("(", ..), .., Ločilo(")", ..) ] => self.funkcijski_klic(izraz),
            [ Ime(ime, ..) ] => match self.spremenljivke.get(ime) {
                Some(spr) => spr.clone(),
                None => panic!("Neznana spremenljivka"),
            },
            [  ] => Prazno.rc(),
            _ => panic!("Neveljaven izraz: {:?}", izraz),
        }
    }

    fn argumenti(&self, mut izraz: &'a[Token<'a>]) -> Vec<Rc<Vozlišče>> {
        println!("argumenti: {:?}", izraz);

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

    //println!("Iščemo {:?}", nizi);

    for (i, tok) in izraz.iter().enumerate() {
        match tok.as_str() {
            ")" => navadnih -= 1,
            "}" => zavitih  -= 1,
            "]" => oglatih  -= 1,
            _   => ()
        }

        //println!("{}, {}, {} - \"{}\"", navadnih, zavitih, oglatih, tok.as_str());

        if navadnih <= 0 && zavitih <= 0 && oglatih <= 0
            && nizi.iter().any(|s| *s == tok.as_str()) {
                //println!("{:?} najden", tok);
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
