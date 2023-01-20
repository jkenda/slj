use std::{rc::Rc, fmt::Display, mem::discriminant};
use super::tip::Tip;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
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

pub enum VozliščeOption {
    Aritmetični(fn(Tip, Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče),
    Logični(fn(Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče),
    Bitni(fn(Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče),
    Brez,
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

    Celo(i32),
    Real(f32),
    Znak(char),
    Niz(String),
    Referenca(Rc<Vozlišče>),

    Spremenljivka{ tip: Tip, ime: String, naslov: u32, z_odmikom: bool },

    Resnica,
    Laž,

    Seštevanje(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Odštevanje(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Množenje(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Deljenje(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Modulo(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Potenca(Tip, Rc<Vozlišče>, Rc<Vozlišče>),

    Zanikaj(Rc<Vozlišče>),
    Konjunkcija(Rc<Vozlišče>, Rc<Vozlišče>),
    Disjunkcija(Rc<Vozlišče>, Rc<Vozlišče>),

    BitniAli(Rc<Vozlišče>, Rc<Vozlišče>),
    BitniXor(Rc<Vozlišče>, Rc<Vozlišče>),
    BitniIn(Rc<Vozlišče>, Rc<Vozlišče>),
    BitniPremikLevo(Rc<Vozlišče>, Rc<Vozlišče>),
    BitniPremikDesno(Rc<Vozlišče>, Rc<Vozlišče>),

    Enako(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    NiEnako(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Večje(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    VečjeEnako(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Manjše(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    ManjšeEnako(Tip, Rc<Vozlišče>, Rc<Vozlišče>),

    ProgramskiŠtevec(i32),
    Skok(OdmikIme),
    DinamičniSkok,
    PogojniSkok(Rc<Vozlišče>, i32),

    PogojniStavek{ pogoj: Rc<Vozlišče>, resnica: Rc<Vozlišče>, laž: Rc<Vozlišče> },
    Zanka{ pogoj: Rc<Vozlišče>, telo: Rc<Vozlišče> },

    Prirejanje{ spremenljivka: Rc<Vozlišče>, izraz: Rc<Vozlišče> },

    Vrni(Rc<Vozlišče>),
    Zaporedje(Vec<Rc<Vozlišče>>),
    Okvir{ zaporedje: Rc<Vozlišče>, št_spr: usize },

    Funkcija{ tip: Tip, ime: String, parametri: Vec<Rc<Vozlišče>>, telo: Rc<Vozlišče>, prostor: usize },
    FunkcijskiKlic{ funkcija: Rc<Vozlišče>, argumenti: Rc<Vozlišče> },

    Natisni(Vec<Rc<Vozlišče>>),
}

use Vozlišče::*;

impl Display for Vozlišče {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Prazno => "()".to_owned(),

            Niz(niz) => "\"".to_owned() 
                    + &niz
                    .replace("\n", r"\n")
                    .replace("\t", r"\t")
                    .replace("\r", r"\r")
                    .replace("\"", r#"\""#)
                    .replace("\'", r"\'")
                    + "\"",

            Celo(število) => število.to_string(),
            Real(število) => število.to_string(),
            Znak(znak)    => znak.to_string(),
            Referenca(spremenljivka) => "@".to_string() + &spremenljivka.to_string(),
            Spremenljivka{ tip, ime, naslov, z_odmikom } => format!("{ime} ({}{naslov}): {tip}", if *z_odmikom { "+" } else { "@" }),

            Resnica => "resnica".to_owned(),
            Laž     => "laž".to_owned(),

            Seštevanje(..) => "+".to_owned(),
            Odštevanje(..) => "-".to_owned(),
            Množenje(..)   => "*".to_owned(),
            Deljenje(..)   => "/".to_owned(),
            Modulo(..)     => "%".to_owned(),
            Potenca(..)    => "**".to_owned(),

            Zanikaj(..)     => "!".to_owned(),
            Konjunkcija(..) => "&&".to_owned(),
            Disjunkcija(..) => "||".to_owned(),

            BitniAli(..) => "|".to_owned(),
            BitniXor(..) => "^".to_owned(),
            BitniIn(..)  => "&".to_owned(),

            BitniPremikLevo(..)  => "<<".to_owned(),
            BitniPremikDesno(..) => ">>".to_owned(),

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
                format!("funkcija {}({})", ime, parametri)
            },
            FunkcijskiKlic{ funkcija, .. } => if let Funkcija { tip: _, ime, .. } = &**funkcija { ime.clone() } else { "".to_string() },
            _ => "".to_owned(),
        })
    }
}

impl PartialEq for Vozlišče {
    fn eq(&self, other: &Self) -> bool {
        if discriminant(self) != discriminant(other) {
            return false;
        }

        match (self, other) {
            (Prazno, Prazno) => true,

            (Push(l), Push(d)) => l == d,
            (Pop(l), Pop(d)) => l == d,
            (Vrh(l), Vrh(d)) => l == d,

            (ShraniOdmik, ShraniOdmik) => true,
            (NaložiOdmik, NaložiOdmik) => true,

            (Celo(l), Celo(d)) => l == d,
            (Real(l), Real(d)) => l == d,
            (Znak(l), Znak(d)) => l == d,
            (Niz(l), Niz(d)) => l == d,
            (Spremenljivka{ tip: lt, ime: li, naslov: ln, z_odmikom: lz }, Spremenljivka{ tip: dt, ime: di, naslov: dn, z_odmikom: dz }) =>
                lt == dt && li == di && ln == dn && lz == dz,

            (Resnica, Resnica) => true,
            (Laž, Laž) => true,

            (Seštevanje(lt, ll, ld), Seštevanje(dt, dl, dd)) |
            (Odštevanje(lt, ll, ld), Odštevanje(dt, dl, dd)) |
            (Množenje(lt, ll, ld), Množenje(dt, dl, dd)) |
            (Deljenje(lt, ll, ld), Deljenje(dt, dl, dd)) |
            (Modulo(lt, ll, ld), Modulo(dt, dl, dd)) |
            (Potenca(lt, ll, ld), Potenca(dt, dl, dd)) => lt == dt && ll == dl && ld == dd,

            (Zanikaj(l), Zanikaj(d)) => l == d,
            (Konjunkcija(ll, ld), Konjunkcija(dl, dd)) |
            (Disjunkcija(ll, ld), Disjunkcija(dl, dd)) => ll == dl && ld == dd,

            (BitniAli(ll, ld), BitniAli(dl, dd)) |
            (BitniXor(ll, ld), BitniXor(dl, dd)) |
            (BitniIn(ll, ld), BitniIn(dl, dd)) => ll == dl && ld == dd,

            (BitniPremikLevo(ll, ld), BitniPremikLevo(dl, dd)) |
            (BitniPremikDesno(ll, ld), BitniPremikDesno(dl, dd)) => ll == dl && ld == dd,

            (Enako(lt, ll, ld), Enako(dt, dl, dd)) |
            (NiEnako(lt, ll, ld), NiEnako(dt, dl, dd)) |
            (Večje(lt, ll, ld), Večje(dt, dl, dd)) |
            (VečjeEnako(lt, ll, ld), VečjeEnako(dt, dl, dd)) |
            (Manjše(lt, ll, ld), Manjše(dt, dl, dd)) |
            (ManjšeEnako(lt, ll, ld), ManjšeEnako(dt, dl, dd)) => lt == dt && ll == dl && ld == dd,

            (PogojniSkok(ll, ld), PogojniSkok(dl, dd)) => ll == dl && ld == dd,

            (PogojniStavek{ pogoj: lp, resnica: lr, laž: ll }, PogojniStavek{ pogoj: dp, resnica: dr, laž: dl }) =>
                lp == dp && lr == dr && ll == dl,

            (Zanka{ pogoj: lp, telo: lt }, Zanka{ pogoj: dp, telo: dt }) =>
                lp == dp && lt == dt,

            (Prirejanje{ spremenljivka: ls, izraz: li }, Prirejanje{ spremenljivka: ds, izraz: di }) =>
                ls == ds && li == di,

            (Vrni(l), Vrni(d)) => l == d,
            (Zaporedje(l), Zaporedje(d)) => l == d,
            (Okvir{ zaporedje: lz, št_spr: lš }, Okvir{ zaporedje: dz, št_spr: dš }) => lz == dz && lš == dš,

            (Funkcija{ tip: ltip, ime: li, parametri: lp, telo: lt, prostor: lpr }, Funkcija{ tip: dtip, ime: di, parametri: dp, telo: dt, prostor: dpr }) =>
                ltip == dtip && li == di && lp == dp && lt == dt && lpr == dpr,

            (FunkcijskiKlic{ funkcija: lf, argumenti: la }, FunkcijskiKlic{ funkcija: df, argumenti: da }) =>
                lf == df && la == da,

            (Natisni(l), Natisni(d)) => l == d,

            _ => false
        }
    }
}

impl Vozlišče {

    pub fn drevo(&self, globina: usize) -> String {
        match self {
            Prazno => "  ".repeat(globina) + "()\n",

            Push(_) | Pop(_) | Vrh(_) | ShraniOdmik | NaložiOdmik
                | ProgramskiŠtevec(_) | Skok(_) | PogojniSkok(..) | DinamičniSkok =>
                "".to_string(),

            Niz(_) | Celo(_) | Real(_) | Znak(_) | Spremenljivka {..} | Resnica | Laž | Referenca(..) => 
                "  ".repeat(globina) + &self.to_string() + "\n",

            Konjunkcija(l, d) | Disjunkcija(l, d) | BitniAli(l, d) | BitniXor(l, d) | BitniIn(l, d)
                | BitniPremikLevo(l, d) | BitniPremikDesno(l, d) =>
                "  ".repeat(globina) + &self.to_string() + "\n"
                + &l.drevo(globina + 1) 
                + &d.drevo(globina + 1),

            Potenca(_, l, d) | Množenje(_, l, d) | Deljenje(_, l, d) | Modulo(_, l, d) | Seštevanje(_, l, d) | Odštevanje(_, l, d)
                | Enako(_, l, d) | NiEnako(_, l, d) | Večje(_, l, d) | VečjeEnako(_, l, d) | Manjše(_, l, d) | ManjšeEnako(_, l, d) =>
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


            Prirejanje{ spremenljivka: _, izraz } => 
                "  ".repeat(globina) + &self.to_string() + "\n" 
                + &izraz.drevo(globina + 1),

            Vrni(prirejanje) => 
                "  ".repeat(globina) + "vrni (\n"
                + &if let Prirejanje { spremenljivka: _, izraz } = &**prirejanje { izraz.drevo(globina + 1) } else { "".to_string() }
                + &"  ".repeat(globina) + ")\n",

            Zaporedje(vozlišča) => vozlišča.into_iter().map(|v| v.drevo(globina + 1)).collect::<Vec<String>>().join(&("  ".repeat(globina) + ",\n")),

            Okvir{ zaporedje, .. } => 
                "  ".repeat(globina) + "{\n" 
                + &zaporedje.drevo(globina + 1)
                + &"  ".repeat(globina) + "}\n",

            Funkcija { tip: _, ime: _, parametri: _, telo, prostor: _ } =>
                "  ".repeat(globina) + &self.to_string() + " {\n"
                + &telo.drevo(globina + 1)
                + &"  ".repeat(globina) + "}\n",

            FunkcijskiKlic { funkcija: _, argumenti } =>
                "  ".repeat(globina) + &self.to_string() + "(\n"
                + &argumenti.drevo(globina + 1)
                + &"  ".repeat(globina) + ")\n",

            Natisni(zaporedje) => 
                "  ".repeat(globina) + "natisni(\n" 
                + &zaporedje
                    .into_iter()
                    .map(|v| v.drevo(globina + 1))
                    .collect::<Vec<String>>()
                    .join(&("  ".repeat(globina) + ",\n"))
                + &"  ".repeat(globina) + ")\n",
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

            Celo(_) => 1,
            Real(_) => 1,
            Znak(_) => 1,
            Niz(niz) => niz.chars().count() as isize,
            Referenca(_) => 1,
            Spremenljivka{ .. } => 1,

            Resnica => 1,
            Laž     => 1,

            Seštevanje(_, l, d) | Odštevanje(_, l, d) | Množenje(_, l, d) | Deljenje(_, l, d) | Modulo(_, l, d) | Potenca(_, l, d) |
                Enako(_, l, d) | NiEnako(_, l, d) | Večje(_, l, d) | VečjeEnako(_, l, d) | Manjše(_, l, d) | ManjšeEnako(_, l, d)
                => l.sprememba_stacka() + d.sprememba_stacka() - 1,

            Zanikaj(izraz)
                => izraz.sprememba_stacka(),

            Konjunkcija(l, d) | Disjunkcija(l, d) |
                BitniAli(l, d) | BitniXor(l, d) | BitniIn(l, d) |
                BitniPremikLevo(l, d) | BitniPremikDesno(l, d)
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

    pub fn tip(&self) -> Tip {
        match self {
            Prazno => Tip::Brez,

            Push(_) => Tip::Celo,
            Pop(_)  => Tip::Brez,
            Vrh(_)    => Tip::Celo,

            ShraniOdmik => Tip::Brez,
            NaložiOdmik => Tip::Celo,

            Celo(_) => Tip::Celo,
            Real(_) => Tip::Real,
            Znak(_) => Tip::Znak,
            Referenca(vozlišče) => Tip::Referenca(Box::new(vozlišče.tip())),
            Niz(niz)  => Tip::Seznam(Box::new(Tip::Znak), niz.len()),
            Spremenljivka{ tip, .. } => tip.clone(),

            Resnica | Laž => Tip::Bool,
            Zanikaj(..) | Konjunkcija(..) | Disjunkcija(..) => Tip::Bool,
            BitniAli(..) | BitniXor(..) | BitniIn(..) | BitniPremikLevo(..) | BitniPremikDesno(..) => Tip::Celo,
            Enako(..) | NiEnako(..) | Večje(..) | VečjeEnako(..) | Manjše(..) | ManjšeEnako(..) => Tip::Bool,

            Seštevanje(tip, ..) | Odštevanje(tip, ..) | Množenje(tip, ..) | Deljenje(tip, ..) | Modulo(tip, ..) | Potenca(tip,..) => tip.clone(),

            ProgramskiŠtevec(..) => Tip::Celo,
            Skok(..) => Tip::Brez,
            DinamičniSkok => Tip::Brez,
            PogojniSkok(..) => Tip::Brez,

            PogojniStavek{ .. } => Tip::Brez,
            Zanka{ .. } => Tip::Brez,
            Prirejanje{ .. } => Tip::Brez,

            Vrni(vozlišče) => vozlišče.tip(),
            Zaporedje(..) => Tip::Brez,
            Okvir{ .. } => Tip::Brez,

            Funkcija{ .. } => Tip::Brez,
            FunkcijskiKlic{ funkcija, .. } => if let Funkcija { tip, .. } = &**funkcija { tip.clone() } else { Tip::Brez },

            Natisni(..) => Tip::Brez,
        }
    }

}

