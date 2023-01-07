use std::{rc::Rc, fmt::Display};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum OdmikIme {
    Odmik(isize),
    Ime(String),
}

impl ToString for OdmikIme {
    fn to_string(&self) -> String {
        match self {
            OdmikIme::Odmik(odmik) => odmik.to_string(),
            OdmikIme::Ime(ime) => ime.clone(),
        }
    }
}

pub struct Drevo {
    pub root: Rc<Vozlišče>,
}

impl Drevo {
    pub fn new(root: Rc<Vozlišče>) -> Drevo {
        Drevo { root }
    }
}

impl Display for Drevo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root.drevo(0))
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Vozlišče {
    Prazno,

    Push(usize),
    Pop(usize),
    Vrh(i32),

    ShraniOdmik,
    NaložiOdmik,

    Niz(String),
    Število(f32),
    Spremenljivka{ ime: String, naslov: u32, z_odmikom: bool },

    Resnica,
    Laž,

    Seštevanje(Rc<Vozlišče>, Rc<Vozlišče>),
    Odštevanje(Rc<Vozlišče>, Rc<Vozlišče>),
    Množenje(Rc<Vozlišče>, Rc<Vozlišče>),
    Deljenje(Rc<Vozlišče>, Rc<Vozlišče>),
    Modulo(Rc<Vozlišče>, Rc<Vozlišče>),
    Potenca(Rc<Vozlišče>, Rc<Vozlišče>),

    Zanikaj(Rc<Vozlišče>),
    Konjunkcija(Rc<Vozlišče>, Rc<Vozlišče>),
    Disjunkcija(Rc<Vozlišče>, Rc<Vozlišče>),
    Enako(Rc<Vozlišče>, Rc<Vozlišče>),
    NiEnako(Rc<Vozlišče>, Rc<Vozlišče>),
    Večje(Rc<Vozlišče>, Rc<Vozlišče>),
    VečjeEnako(Rc<Vozlišče>, Rc<Vozlišče>),
    Manjše(Rc<Vozlišče>, Rc<Vozlišče>),
    ManjšeEnako(Rc<Vozlišče>, Rc<Vozlišče>),

    ProgramskiŠtevec(i32),
    Skok(OdmikIme),
    DinamičniSkok,
    PogojniSkok(Rc<Vozlišče>, i32),

    PogojniStavek{ pogoj: Rc<Vozlišče>, resnica: Rc<Vozlišče>, laž: Rc<Vozlišče> },
    Zanka{ pogoj: Rc<Vozlišče>, telo: Rc<Vozlišče> },

    Prirejanje{ spremenljivka: Rc<Vozlišče>, izraz: Rc<Vozlišče>, z_odmikom: bool },

    Vrni(Rc<Vozlišče>),
    Zaporedje(Vec<Rc<Vozlišče>>),
    Okvir{ zaporedje: Rc<Vozlišče>, št_spr: usize },

    Funkcija{ ime: String, parametri: Vec<Rc<Vozlišče>>, telo: Rc<Vozlišče>, prostor: usize },
    FunkcijskiKlic{ funkcija: Rc<Vozlišče>, argumenti: Rc<Vozlišče> },

    Natisni(Vec<Rc<Vozlišče>>),
}

use Vozlišče::*;

impl ToString for Vozlišče {
    fn to_string(&self) -> String {
        match self {
            Prazno => "()".to_owned(),

            Niz(niz) => "\"".to_owned() 
                    + &niz
                    .replace("\n", r"\n")
                    .replace("\t", r"\t")
                    .replace("\r", r"\r")
                    .replace("\"", r#"\""#)
                    .replace("\'", r"\'")
                    + "\"",

            Število(število) => število.to_string(),
            Spremenljivka{ ime, naslov, z_odmikom } => format!("{} ({}{})", ime,
                if *z_odmikom { "+" } else { "@" }, naslov),

            Resnica => "resnica".to_owned(),
            Laž     => "laž".to_owned(),

            Seštevanje(..) => "+".to_owned(),
            Odštevanje(..) => "-".to_owned(),
            Množenje(..)   => "*".to_owned(),
            Deljenje(..)   => "/".to_owned(),
            Modulo(..)     => "mod".to_owned(),
            Potenca(..)    => "^".to_owned(),

            Zanikaj(..)     => "ne".to_owned(),
            Konjunkcija(..) => "in".to_owned(),
            Disjunkcija(..) => "ali".to_owned(),
            Enako(..)       => "==".to_owned(),
            NiEnako(..)     => "!=".to_owned(),
            Večje(..)       => ">".to_owned(),
            VečjeEnako(..)  => ">=".to_owned(),
            Manjše(..)      => "<".to_owned(),
            ManjšeEnako(..) => "<=".to_owned(),

            PogojniStavek{..} => "če".to_owned(),
            Zanka{..}         => "dokler".to_owned(),

            Prirejanje{ spremenljivka, .. } => spremenljivka.to_string() + " = ",
            Vrni(_) => "vrni".to_owned(),

            Funkcija{ ime, parametri, .. } => {
                let parametri = parametri.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(", ");
                format!("{}({})", ime, parametri)
            },
            FunkcijskiKlic{ funkcija, .. } => format!("{}(", if let Funkcija { ime, parametri: _, telo: _, prostor: _ } = &**funkcija { ime } else { "" }),
            _ => "".to_owned(),
        }
    }
}

impl Vozlišče {

    pub fn drevo(&self, globina: usize) -> String {
        match self {
            Prazno => "  ".repeat(globina) + "()\n",

            Niz(_) | Število(_) | Spremenljivka {..} | Resnica | Laž => 
                "  ".repeat(globina) + &self.to_string() + "\n",

            Potenca(l, d) | Množenje(l, d) | Deljenje(l, d) | Modulo(l, d) | Seštevanje(l, d) | Odštevanje(l, d)
                | Konjunkcija(l, d) | Disjunkcija(l, d) 
                | Enako(l, d) | NiEnako(l, d) | Večje(l, d) | VečjeEnako(l, d) | Manjše(l, d) | ManjšeEnako(l, d) =>
                "  ".repeat(globina) + &self.to_string() + "\n"
                + &l.drevo(globina + 1) 
                + &d.drevo(globina + 1),

            Zanikaj(vozlišče) =>
                "  ".repeat(globina) + &self.to_string() + "\n"  
                + &vozlišče.drevo(globina + 1),

            PogojniStavek { pogoj, resnica, laž } =>
                "  ".repeat(globina) + "če (\n" 
                + &pogoj.drevo(globina + 1) 
                + &"  ".repeat(globina) + ") "
                + &resnica.drevo(globina).trim_start()
                + &match &**laž {
                    Prazno => "".to_owned(),
                    _ => "  ".repeat(globina) + &"čene ".to_owned() 
                        + &laž.drevo(globina).trim_start(),
                },

            Zanka { pogoj, telo } => 
                "  ".repeat(globina) + "dokler(\n"
                + &pogoj.drevo(globina + 1)
                + &"  ".repeat(globina) + ") {\n"
                + &telo.drevo(globina + 1)
                + &"  ".repeat(globina) + "}\n",

            Prirejanje{ spremenljivka: _, izraz, z_odmikom: _ } => 
                "  ".repeat(globina) + &self.to_string() + "\n" 
                + &izraz.drevo(globina + 1),

            Vrni(prirejanje) => 
                "  ".repeat(globina) + "vrni (\n"
                + &prirejanje.drevo(globina + 1)
                + &"  ".repeat(globina) + ")\n",

            Zaporedje(vozlišča) => vozlišča.into_iter().map(|v| v.drevo(globina + 1)).collect::<Vec<String>>().join(&("  ".repeat(globina) + ",\n")),

            Okvir{ zaporedje, .. } => 
                "  ".repeat(globina) + "{\n" 
                + &zaporedje.drevo(globina + 1)
                + &"  ".repeat(globina) + "}\n",

            Funkcija { ime: _, parametri: _, telo, prostor: _ } =>
                "  ".repeat(globina) + &self.to_string() + " {\n"
                + &telo.drevo(globina + 1)
                + &"  ".repeat(globina) + "}\n",

            FunkcijskiKlic { funkcija: _, argumenti } =>
                "  ".repeat(globina) + &self.to_string() + "(\n"
                + &argumenti.drevo(globina + 1)
                + ")\n",

            Natisni(zaporedje) => 
                "  ".repeat(globina) + "natisni(\n" 
                + &zaporedje
                    .into_iter()
                    .map(|v| v.drevo(globina + 1))
                    .collect::<Vec<String>>()
                    .join(&("  ".repeat(globina) + ",\n"))
                + &"  ".repeat(globina) + ")\n",

            _ => "".to_owned()
        }
    }

    pub fn rc(&self) -> Rc<Self> {
        Rc::new(self.clone())
    }

    pub fn sprememba_stacka(&self) -> isize {
        match self {
            Prazno => 0,

            Push(krat) => *krat as isize,
            Pop(krat)  => -(*krat as isize),
            Vrh(_)     => 0,

            ShraniOdmik => -1,
            NaložiOdmik => 1,

            Niz(niz) => niz.len() as isize,
            Število(_) => 1,
            Spremenljivka{ .. } => 1,

            Resnica => 1,
            Laž     => 1,

            Seštevanje(l, d) | Odštevanje(l, d) | Množenje(l, d) | Deljenje(l, d) | Modulo(l, d) | Potenca(l, d)
                => l.sprememba_stacka() + d.sprememba_stacka() - 1,

            Zanikaj(izraz)
                => izraz.sprememba_stacka(),

            Konjunkcija(l, d) | Disjunkcija(l, d) |
                Enako(l, d) | NiEnako(l, d) | Večje(l, d) | VečjeEnako(l, d) | Manjše(l, d) | ManjšeEnako(l, d)
                => l.sprememba_stacka() + d.sprememba_stacka() - 1,

            ProgramskiŠtevec(_)     => 1,
            Skok(_)                 => 0,
            DinamičniSkok           => -1,
            PogojniSkok(pogoj, _)   => pogoj.sprememba_stacka() - 1,

            PogojniStavek{ pogoj, resnica, laž }    => pogoj.sprememba_stacka() - 1 + resnica.sprememba_stacka().max(laž.sprememba_stacka()),
            Zanka{ pogoj, telo }                    => pogoj.sprememba_stacka() - 1 + telo.sprememba_stacka(),

            Prirejanje{ spremenljivka: _, izraz, .. } => izraz.sprememba_stacka() - 1,

            Vrni(_)             => 0,
            Zaporedje(izrazi)   => izrazi.iter().map(|i| i.sprememba_stacka()).sum(),
            Okvir{ .. }         => 0,

            Funkcija{ .. } => 0,
            FunkcijskiKlic{ .. } => 1,

            Natisni(_) => 0,
        }
    }

}

