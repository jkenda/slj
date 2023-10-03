use std::{rc::Rc, fmt::Display, mem::{discriminant, self}, collections::HashMap};
use super::{tip::Tip, napaka::{Napake, OznakaNapake::*}, lekser::Žeton};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum OdmikIme {
    Odmik(i32),
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
    pub main: Rc<Vozlišče>,
    pub funkcije: Vec<Rc<Vozlišče>>,
    pub št_klicev: HashMap<String, usize>,
}

impl Display for Drevo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.main.drevo(0))
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

    Push(i32),
    Pop(i32),
    Vrh(i32),

    ShraniOdmik,
    NaložiOdmik,

    Celo(i32),
    Real(f32),
    Znak(char),
    Niz(String),

    Spremenljivka{ tip: Tip, ime: String, naslov: i32, z_odmikom: bool, spremenljiva: bool },
    Referenca(Rc<Vozlišče>),
    RefSeznama(Rc<Vozlišče>),

    Dereferenciraj(Rc<Vozlišče>),
    Indeksiraj{ seznam_ref: Rc<Vozlišče>, indeks: Rc<Vozlišče> },
    Dolžina(Rc<Vozlišče>),

    Resnica,
    Laž,

    Seštevanje(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Odštevanje(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Množenje(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Deljenje(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Modulo(Tip, Rc<Vozlišče>, Rc<Vozlišče>),
    Potenca(Tip, Rc<Vozlišče>, Rc<Vozlišče>),

    CeloVReal(Rc<Vozlišče>),
    RealVCelo(Rc<Vozlišče>),
    CeloVZnak(Rc<Vozlišče>),
    ZnakVCelo(Rc<Vozlišče>),

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
    Skok(String),
    Klic(String),
    DinamičniSkok,
    PogojniSkok(Rc<Vozlišče>, String),

    PogojniStavek{ pogoj: Rc<Vozlišče>, resnica: Rc<Vozlišče>, laž: Rc<Vozlišče> },
    Zanka{ pogoj: Rc<Vozlišče>, telo: Rc<Vozlišče> },

    Prirejanje{ spremenljivka: Rc<Vozlišče>, izraz: Rc<Vozlišče> },
    PrirejanjeRef{ referenca: Rc<Vozlišče>, indeks: Option<Rc<Vozlišče>>, izraz: Rc<Vozlišče> },

    Vrni(Rc<Vozlišče>),
    Zaporedje(Vec<Rc<Vozlišče>>),
    Okvir{ zaporedje: Rc<Vozlišče>, št_spr: i32 },

    Funkcija{ tip: Tip, ime: String, parametri: Vec<Rc<Vozlišče>>, telo: Rc<Vozlišče>, prostor: i32 },
    FunkcijskiKlic{ funkcija: Rc<Vozlišče>, spremenljivke: Rc<Vozlišče>, argumenti: Rc<Vozlišče> },

    Natisni(Rc<Vozlišče>),
    Preberi,
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

            Spremenljivka{ tip, ime, naslov, z_odmikom, .. } => format!("{ime}: {tip} ({}{naslov})", if *z_odmikom { "+" } else { "@" }),
            Referenca(spremenljivka) | RefSeznama(spremenljivka) => "@".to_string() + &spremenljivka.to_string(),

            Dereferenciraj(spremenljivka) => spremenljivka.to_string() + &"@".to_string(),
            Dolžina(spr) => format!("{}.dolžina", spr.to_string()),

            Resnica => "resnica".to_owned(),
            Laž     => "laž".to_owned(),

            Seštevanje(..) => "+".to_owned(),
            Odštevanje(..) => "-".to_owned(),
            Množenje(..)   => "*".to_owned(),
            Deljenje(..)   => "/".to_owned(),
            Modulo(..)     => "%".to_owned(),
            Potenca(..)    => "**".to_owned(),

            CeloVReal(..)  => "kot".to_owned(),

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
            PrirejanjeRef{ referenca, .. } => referenca.to_string() + " = ",

            Vrni(_) => "vrni".to_owned(),

            Funkcija{ tip, ime, parametri, .. } => {
                let parametri = parametri.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(", ");
                format!("funkcija {}({}) -> {}", ime, parametri, tip)
            },
            FunkcijskiKlic{ funkcija, .. } => if let Funkcija { tip: _, ime, .. } = &**funkcija { ime.clone() } else { "".to_string() },
            Natisni(znak) => format!("natisni({znak})"),
            Preberi => "preberi()".to_string(),
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

            (Spremenljivka{ tip: lt, ime: li, naslov: ln, z_odmikom: lz, spremenljiva: ls },
             Spremenljivka{ tip: dt, ime: di, naslov: dn, z_odmikom: dz, spremenljiva: ds }) =>
                lt == dt && li == di && ln == dn && lz == dz && ls == ds,
            (Referenca(l), Referenca(d)) => l == d,
            (Dereferenciraj(l), Dereferenciraj(d)) => l == d,

            (Resnica, Resnica) => true,
            (Laž, Laž) => true,

            (Seštevanje(lt, ll, ld), Seštevanje(dt, dl, dd)) |
            (Odštevanje(lt, ll, ld), Odštevanje(dt, dl, dd)) |
            (Množenje(lt, ll, ld), Množenje(dt, dl, dd)) |
            (Deljenje(lt, ll, ld), Deljenje(dt, dl, dd)) |
            (Modulo(lt, ll, ld), Modulo(dt, dl, dd)) |
            (Potenca(lt, ll, ld), Potenca(dt, dl, dd)) => lt == dt && ll == dl && ld == dd,

            (CeloVReal(l), CeloVReal(d)) => l == d,
            (RealVCelo(l), RealVCelo(d)) => l == d,

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
            (PrirejanjeRef{ referenca: lr, indeks: lin, izraz: li }, PrirejanjeRef{ referenca: dr, indeks: din, izraz: di }) =>
                lr == dr && lin == din && li == di,

            (Vrni(l), Vrni(d)) => l == d,
            (Zaporedje(l), Zaporedje(d)) => l == d,
            (Okvir{ zaporedje: lz, št_spr: lš }, Okvir{ zaporedje: dz, št_spr: dš }) => lz == dz && lš == dš,

            (l @ Funkcija{ .. }, d @ Funkcija{ .. }) =>
                mem::discriminant(l) == mem::discriminant(d),

            (l @ FunkcijskiKlic{ .. }, d @ FunkcijskiKlic{ .. }) =>
                mem::discriminant(l) == mem::discriminant(d),

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
                | ProgramskiŠtevec(_) | Skok(_) | Klic(_) | PogojniSkok(..) | DinamičniSkok =>
                "".to_string(),

            Niz(_) | Celo(_) | Real(_) | Znak(_) | Resnica | Laž
                | Spremenljivka {..} | Referenca(..) | RefSeznama(..) | Dereferenciraj(..) | Dolžina(..) =>
                "  ".repeat(globina) + &self.to_string() + "\n",

            Indeksiraj { seznam_ref, indeks } => match &**seznam_ref {
                Spremenljivka { tip: Tip::Seznam(..) | Tip::RefSeznama(..), ime, .. } =>
                    " ".repeat(globina) + &format!("{ime}[")
                    + &indeks.drevo(globina + 1)
                    + &" ".repeat(globina) + "]\n",
                vozl @ _ => unreachable!("Referenca mora vsebovati spremenljivko, ne pa {:?}", vozl),
            }

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

            CeloVReal(vozlišče) =>
                vozlišče.drevo(globina) 
                + " kot real\n",
            RealVCelo(vozlišče) =>
                vozlišče.drevo(globina) 
                + " kot celo\n",
            CeloVZnak(vozlišče) =>
                vozlišče.drevo(globina) 
                + " kot znak\n",
            ZnakVCelo(vozlišče) =>
                vozlišče.drevo(globina) 
                + " kot celo\n",


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


            Prirejanje{ izraz, .. } | PrirejanjeRef{ izraz, .. } => 
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

            Funkcija { tip: _, ime: _, parametri: _, telo, .. } =>
                "  ".repeat(globina) + &self.to_string() + " {\n"
                + &telo.drevo(globina + 1)
                + &"  ".repeat(globina) + "}\n",

            FunkcijskiKlic { argumenti, .. } =>
                "  ".repeat(globina) + &self.to_string() + "(\n"
                + &argumenti.drevo(globina + 1)
                + &"  ".repeat(globina) + ")\n",

            Natisni(znak) => 
                "  ".repeat(globina) + &znak.to_string() + "\n",
            Preberi => " ".repeat(globina) + &self.to_string(),
        }
    }

    pub fn eval(&self, izraz: &[Žeton]) -> Result<Vozlišče, Napake> {
        match self {
            Celo(_) | Real(_) | Znak(_) | Niz(_) | Resnica | Laž => Ok(self.clone()),

            Spremenljivka{ ime, tip, .. } => Err(Napake::from_zaporedje(izraz, E2, &format!("Vrednost spremenljivke {ime}: {tip} ni znana vnaprej."))),
            Referenca(spr) => spr.eval(izraz),
            RefSeznama(spr) => spr.eval(izraz),

            Dereferenciraj(spr) => spr.eval(izraz),
            Indeksiraj{ seznam_ref, .. } => seznam_ref.eval(izraz),
            Dolžina(spr) => match spr.tip() {
                Tip::Seznam(_, dolžina) => Ok(Celo(dolžina)),
                Tip::RefSeznama(_) => match &**spr {
                    Spremenljivka { tip, ime, .. } => Err(Napake::from_zaporedje(izraz, E2, &format!("Dolžina seznama {ime}: {tip} ni znana vnaprej."))),
                    _ => unreachable!(),
                }
                _ => unreachable!(),
            },

            Seštevanje(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                    (Celo(l), Celo(d)) => Ok(Celo(l + d)),
                    (Real(l), Real(d)) => Ok(Real(l + d)),
                    _ => unreachable!(),
            },
            Odštevanje(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                    (Celo(l), Celo(d)) => Ok(Celo(l - d)),
                    (Real(l), Real(d)) => Ok(Real(l - d)),
                    _ => unreachable!(),
            },
            Množenje(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                    (Celo(l), Celo(d)) => Ok(Celo(l * d)),
                    (Real(l), Real(d)) => Ok(Real(l * d)),
                    _ => unreachable!(),
            },
            Deljenje(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                    (Celo(l), Celo(d)) => Ok(Celo(l / d)),
                    (Real(l), Real(d)) => Ok(Real(l / d)),
                    _ => unreachable!(),
            },
            Modulo(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(Celo(l % d)),
                (Real(l), Real(d)) => Ok(Real(l % d)),
                _ => unreachable!(),
            },
            Potenca(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(Celo(l.pow(d as u32))),
                (Real(l), Real(d)) => Ok(Real(l.powf(d))),
                _ => unreachable!(),
            },

            CeloVReal(št) => match št.eval(izraz)? {
                Celo(št) => Ok(Real(št as f32)),
                _ => unreachable!(),
            },
            RealVCelo(št) => match št.eval(izraz)? {
                Real(št) => Ok(Celo(št as i32)),
                _ => unreachable!(),
            },
            CeloVZnak(št) => match št.eval(izraz)? {
                Celo(št) => Ok(Znak(char::from_u32(št as u32).ok_or(Napake::from_zaporedje(izraz, E2, &format!("Ne morem pretovriti št. {št} v znak.")))?)),
                _ => unreachable!(),
            },
            ZnakVCelo(št) => match št.eval(izraz)? {
                Znak(št) => Ok(Celo(št as i32)),
                _ => unreachable!(),
            },

            Zanikaj(bool) => match bool.eval(izraz)? {
                Resnica => Ok(Laž),
                Laž => Ok(Resnica),
                _ => unreachable!(),
            },
            Konjunkcija(l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Laž, Laž)         => Ok(Laž),
                (Laž, Resnica)     => Ok(Laž),
                (Resnica, Laž)     => Ok(Laž),
                (Resnica, Resnica) => Ok(Resnica),
                _ => unreachable!(),
            },
            Disjunkcija(l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Laž, Laž)         => Ok(Laž),
                (Laž, Resnica)     => Ok(Resnica),
                (Resnica, Laž)     => Ok(Resnica),
                (Resnica, Resnica) => Ok(Resnica),
                _ => unreachable!(),
            },

            BitniAli(l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(Celo(l | d)),
                _ => unreachable!(),
            },
            BitniXor(l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(Celo(l ^ d)),
                _ => unreachable!(),
            },
            BitniIn(l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(Celo(l & d)),
                _ => unreachable!(),
            },
            BitniPremikLevo(l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(Celo(l << d)),
                _ => unreachable!(),
            },
            BitniPremikDesno(l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(Celo(l >> d)),
                _ => unreachable!(),
            },

            Enako(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(if l == d { Resnica } else { Laž }),
                (Real(l), Real(d)) => Ok(if l == d { Resnica } else { Laž }),
                _ => unreachable!(),
            },
            NiEnako(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(if l != d { Resnica } else { Laž }),
                (Real(l), Real(d)) => Ok(if l != d { Resnica } else { Laž }),
                _ => unreachable!(),
            },
            Večje(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(if l > d { Resnica } else { Laž }),
                (Real(l), Real(d)) => Ok(if l > d { Resnica } else { Laž }),
                _ => unreachable!(),
            },
            VečjeEnako(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(if l >= d { Resnica } else { Laž }),
                (Real(l), Real(d)) => Ok(if l >= d { Resnica } else { Laž }),
                _ => unreachable!(),
            },
            Manjše(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(if l < d { Resnica } else { Laž }),
                (Real(l), Real(d)) => Ok(if l < d { Resnica } else { Laž }),
                _ => unreachable!(),
            },
            ManjšeEnako(_, l, d) => match (l.eval(izraz)?, d.eval(izraz)?) {
                (Celo(l), Celo(d)) => Ok(if l <= d { Resnica } else { Laž }),
                (Real(l), Real(d)) => Ok(if l <= d { Resnica } else { Laž }),
                _ => unreachable!(),
            },

            _ => unreachable!(),
        }
    }

    pub fn rc(&self) -> Rc<Self> {
        Rc::new(self.clone())
    }

    pub fn sprememba_stacka(&self) -> i32 {
        match self {
            Prazno => 0,

            Push(krat) => *krat,
            Pop(krat)  => -(*krat),
            Vrh(_)     => 0,

            ShraniOdmik => -1,
            NaložiOdmik => 1,

            Celo(_) => 1,
            Real(_) => 1,
            Znak(_) => 1,
            Niz(niz) => niz.chars().count() as i32,

            Spremenljivka{ tip, .. } => tip.sprememba_stacka(),
            Referenca(_) | RefSeznama(_) => 1,

            Dereferenciraj(spr) => spr.sprememba_stacka(),
            Indeksiraj { seznam_ref, .. } => match &**seznam_ref {
                Spremenljivka { tip, .. } => tip.sprememba_stacka(),
                Referenca(spr) => match &**spr {
                    Spremenljivka { tip, .. } => tip.sprememba_stacka(),
                    _ => unreachable!("Vedno indeksiramo referenco na seznam."),
                },
                _ => unreachable!("Vedno indeksiramo referenco na seznam."),
            }
            Dolžina(..) => 1,

            Resnica => 1,
            Laž     => 1,

            Seštevanje(_, l, d) | Odštevanje(_, l, d) | Množenje(_, l, d) | Deljenje(_, l, d) | Modulo(_, l, d) | Potenca(_, l, d) |
                Enako(_, l, d) | NiEnako(_, l, d) | Večje(_, l, d) | VečjeEnako(_, l, d) | Manjše(_, l, d) | ManjšeEnako(_, l, d)
                => l.sprememba_stacka() + d.sprememba_stacka() - 1,

            CeloVReal(..) | RealVCelo(..) | CeloVZnak(..) | ZnakVCelo(..) => 0,

            Zanikaj(izraz)
                => izraz.sprememba_stacka(),

            Konjunkcija(l, d) | Disjunkcija(l, d) |
                BitniAli(l, d) | BitniXor(l, d) | BitniIn(l, d) |
                BitniPremikLevo(l, d) | BitniPremikDesno(l, d)
                => l.sprememba_stacka() + d.sprememba_stacka() - 1,

            ProgramskiŠtevec(_)     => 1,
            Skok(_) | Klic(_)                 => 0,
            DinamičniSkok           => -1,
            PogojniSkok(pogoj, _)   => pogoj.sprememba_stacka() - 1,

            PogojniStavek{ pogoj, resnica, laž }    => pogoj.sprememba_stacka() - 1 + resnica.sprememba_stacka().max(laž.sprememba_stacka()),
            Zanka{ pogoj, telo }                    => pogoj.sprememba_stacka() - 1 + telo.sprememba_stacka(),

            Prirejanje{ izraz, .. } | PrirejanjeRef{ izraz, .. } => izraz.sprememba_stacka() - 1,

            Vrni(_)             => 0,
            Zaporedje(izrazi)   => izrazi.iter().map(|i| i.sprememba_stacka()).sum(),
            Okvir{ .. }         => 0,

            Funkcija{ .. } => 0,
            FunkcijskiKlic{ .. } => 1,

            Natisni(_) => 0,
            Preberi => 1,
        }
    }

    pub fn tip(&self) -> Tip {
        match self {
            Prazno => Tip::Brez,

            Push(_) => Tip::Celo,
            Pop(_)  => Tip::Brez,
            Vrh(_)  => Tip::Celo,

            ShraniOdmik => Tip::Brez,
            NaložiOdmik => Tip::Celo,

            Celo(_) => Tip::Celo,
            Real(_) => Tip::Real,
            Znak(_) => Tip::Znak,
            Niz(niz)  => Tip::Seznam(Box::new(Tip::Znak), niz.chars().count() as i32),
            
            Spremenljivka{ tip, .. } => tip.clone(),
            Referenca(vozlišče) => Tip::Referenca(Box::new(vozlišče.tip())),
            RefSeznama(vozlišče) => Tip::RefSeznama(Box::new(vozlišče.tip().vsebuje_tip())),

            Dereferenciraj(vozlišče) => match &**vozlišče {
                Spremenljivka { tip: Tip::Referenca(element), .. } => *element.clone(),
                Spremenljivka { tip: Tip::RefSeznama(element), .. } => *element.clone(),
                _ => unreachable!("Dereferencirati je mogoče samo referenco."),
            },
            Indeksiraj { seznam_ref, .. } => {
                match seznam_ref.tip() {
                    Tip::Seznam(tip, ..) => *tip.clone(),
                    Tip::RefSeznama(tip, ..) => *tip.clone(),
                    _ => unreachable!("Vedno indeksiramo referenco na seznam."),
                }
            },
            Dolžina(..) => Tip::Celo,

            Resnica | Laž => Tip::Bool,
            Zanikaj(..) | Konjunkcija(..) | Disjunkcija(..) => Tip::Bool,
            BitniAli(..) | BitniXor(..) | BitniIn(..) | BitniPremikLevo(..) | BitniPremikDesno(..) => Tip::Celo,
            Enako(..) | NiEnako(..) | Večje(..) | VečjeEnako(..) | Manjše(..) | ManjšeEnako(..) => Tip::Bool,

            Seštevanje(tip, ..) | Odštevanje(tip, ..) | Množenje(tip, ..) | Deljenje(tip, ..) | Modulo(tip, ..) | Potenca(tip,..) => tip.clone(),

            CeloVReal(..) => Tip::Real,
            RealVCelo(..) => Tip::Celo,
            CeloVZnak(..) => Tip::Znak,
            ZnakVCelo(..) => Tip::Celo,

            ProgramskiŠtevec(..) => Tip::Celo,
            Skok(..) => Tip::Brez,
            Klic(..) => Tip::Brez,
            DinamičniSkok => Tip::Brez,
            PogojniSkok(..) => Tip::Brez,

            PogojniStavek{ .. } => Tip::Brez,
            Zanka{ .. } => Tip::Brez,
            Prirejanje{ .. } | PrirejanjeRef { ..  } => Tip::Brez,

            Vrni(vozlišče) => vozlišče.tip(),
            Zaporedje(..) => Tip::Brez,
            Okvir{ .. } => Tip::Brez,

            Funkcija{ .. } => Tip::Brez,
            FunkcijskiKlic{ funkcija, .. } => if let Funkcija { tip, .. } = &**funkcija { tip.clone() } else { Tip::Brez },

            Natisni(..) => Tip::Brez,
            Preberi => Tip::Znak,
        }
    }

    pub fn lahko_vrinemo(&self) -> bool {
        //const MEJA: usize = 7;

        match self {
            Funkcija { telo, .. } => {
                // rekurzivne funkcije ne moremo vriniti
                !telo.vsebuje(self)
            },
            _ => unreachable!(),
        }
    }

    fn vsebuje(&self, other: &Vozlišče) -> bool {
        if self == other {
            true
        }
        else {
            match self {
                Referenca(vozlišče) => vozlišče.vsebuje(other),

                Zanikaj(a) => a.vsebuje(other),
                Konjunkcija(a, b) | Disjunkcija(a, b) => a.vsebuje(other) || b.vsebuje(other),
                BitniAli(a, b) | BitniXor(a, b) | BitniIn(a, b) | BitniPremikLevo(a, b) | BitniPremikDesno(a, b)
                    => a.vsebuje(other) || b.vsebuje(other),
                Enako(_, a, b) | NiEnako(_, a, b) | Večje(_, a, b) | VečjeEnako(_, a, b) | Manjše(_, a, b) | ManjšeEnako(_, a, b) 
                    => a.vsebuje(other) || b.vsebuje(other),

                Seštevanje(_, a, b) | Odštevanje(_, a, b) | Množenje(_, a, b) | Deljenje(_, a, b) 
                    | Modulo(_, a, b) | Potenca(_, a, b) => a.vsebuje(other) || b.vsebuje(other),

                CeloVReal(a) | RealVCelo(a) => a.vsebuje(other),

                PogojniStavek { pogoj, resnica, laž } => pogoj.vsebuje(other) || resnica.vsebuje(other) || laž.vsebuje(other),
                Zanka { pogoj, telo } => pogoj.vsebuje(other) || telo.vsebuje(other),
                Prirejanje { spremenljivka: _, izraz } => izraz.vsebuje(other),

                Vrni(a) => a.vsebuje(other),
                Zaporedje(a) => a.iter().any(|s| s.vsebuje(other)),
                Okvir { zaporedje, št_spr: _ } => zaporedje.vsebuje(other),

                Funkcija { tip: _, ime: _, parametri: _, telo, .. } => telo.vsebuje(other),
                FunkcijskiKlic { funkcija, argumenti, .. } =>
                    &**funkcija == if let FunkcijskiKlic { funkcija, .. } = other {
                        &**funkcija
                    } else { unreachable!() } || argumenti.vsebuje(other),

                _ => false,
            }
        }
    }

}

#[cfg(test)]
mod testi {
    use super::*;
    use crate::parser::{lekser::{Razčleni, L}, Parse, drevo::{Prazno, FunkcijskiKlic, Zaporedje}};

    #[test]
    fn eq() {
    }

    #[test]
    #[ignore]
    fn vsebuje() {
        let rekurzivna_f = if let Zaporedje(stavki) = &*r#"funkcija f() {
            f()
        }"#.razčleni("[test]").analiziraj().unwrap().main.clone() {
            stavki[0].clone()
        }
        else {
            Prazno.rc()
        };

        assert_eq!(rekurzivna_f.vsebuje(&FunkcijskiKlic{
            funkcija: rekurzivna_f.clone(),
            spremenljivke: Zaporedje(vec![]).rc(),
            argumenti: Zaporedje(vec![]).rc() }), true);
    }

    #[test]
    fn eval() {
        assert_eq!(Seštevanje(Tip::Celo, Celo(7).rc(), Celo(6).rc()).eval(&[]).unwrap(), Celo(13));
        assert_eq!(Celo(42).eval(&[]).unwrap(), Celo(42));
        assert_eq!(Real(3.14).eval(&[]).unwrap(), Real(3.14));
        assert_eq!(Znak('ć').eval(&[]).unwrap(), Znak('ć'));
        assert_eq!(Niz("ribič".to_string()).eval(&[]).unwrap(), Niz("ribič".to_string()));

        assert_eq!(Dolžina(Spremenljivka { tip: Tip::Seznam(Box::new(Tip::Brez), 14), ime: "".to_string(), naslov: 0, z_odmikom: false, spremenljiva: true }.rc()).eval(&[]).unwrap(), Celo(14));

        assert_eq!(Seštevanje(Tip::Celo, Celo(7).rc(), Celo(6).rc()).eval(&[]).unwrap(), Celo(13));
        assert_eq!(Seštevanje(Tip::Real, Real(13.0).rc(), Real(29.0).rc()).eval(&[]).unwrap(), Real(42.0));
        assert_eq!(Odštevanje(Tip::Celo, Celo(7).rc(), Celo(6).rc()).eval(&[]).unwrap(), Celo(1));
        assert_eq!(Odštevanje(Tip::Real, Real(13.0).rc(), Real(29.0).rc()).eval(&[]).unwrap(), Real(-16.0));
        assert_eq!(Množenje(Tip::Celo, Celo(7).rc(), Celo(6).rc()).eval(&[]).unwrap(), Celo(42));
        assert_eq!(Množenje(Tip::Real, Real(13.0).rc(), Real(29.0).rc()).eval(&[]).unwrap(), Real(377.0));
        assert_eq!(Deljenje(Tip::Celo, Celo(16).rc(), Celo(6).rc()).eval(&[]).unwrap(), Celo(2));
        assert_eq!(Deljenje(Tip::Real, Real(13.0).rc(), Real(4.0).rc()).eval(&[]).unwrap(), Real(3.25));
        assert_eq!(Modulo(Tip::Celo, Celo(16).rc(), Celo(6).rc()).eval(&[]).unwrap(), Celo(4));
        assert_eq!(Modulo(Tip::Real, Real(13.75).rc(), Real(0.5).rc()).eval(&[]).unwrap(), Real(0.25));
        assert_eq!(Potenca(Tip::Celo, Celo(2).rc(), Celo(6).rc()).eval(&[]).unwrap(), Celo(64));
        assert_eq!(Potenca(Tip::Real, Real(4.0).rc(), Real(0.5).rc()).eval(&[]).unwrap(), Real(2.0));

        assert_eq!(CeloVReal(Celo(13).rc()).eval(&[]).unwrap(), Real(13.0));
        assert_eq!(RealVCelo(Real(13.0).rc()).eval(&[]).unwrap(), Celo(13));
        assert_eq!(RealVCelo(Real(3.14).rc()).eval(&[]).unwrap(), Celo(3));
        assert_eq!(CeloVZnak(Celo(32).rc()).eval(&[Žeton::Literal(L::Celo("32", 1, 1, "[test]"))]).unwrap(), Znak(' '));
        assert_eq!(ZnakVCelo(Znak('\n').rc()).eval(&[]).unwrap(), Celo(10));

        assert_eq!(Zanikaj(Resnica.rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Zanikaj(Laž.rc()).eval(&[]).unwrap(), Resnica);

        assert_eq!(Konjunkcija(Laž.rc(), Laž.rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Konjunkcija(Laž.rc(), Resnica.rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Konjunkcija(Resnica.rc(), Laž.rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Konjunkcija(Resnica.rc(), Resnica.rc()).eval(&[]).unwrap(), Resnica);

        assert_eq!(Disjunkcija(Laž.rc(), Laž.rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Disjunkcija(Laž.rc(), Resnica.rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(Disjunkcija(Resnica.rc(), Laž.rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(Disjunkcija(Resnica.rc(), Resnica.rc()).eval(&[]).unwrap(), Resnica);

        assert_eq!(BitniAli(Celo(0b10).rc(), Celo(0b01).rc()).eval(&[]).unwrap(), Celo(0b11));
        assert_eq!(BitniAli(Celo(0b00).rc(), Celo(0b01).rc()).eval(&[]).unwrap(), Celo(0b01));

        assert_eq!(BitniIn(Celo(0b10).rc(), Celo(0b01).rc()).eval(&[]).unwrap(), Celo(0b00));
        assert_eq!(BitniIn(Celo(0b11).rc(), Celo(0b01).rc()).eval(&[]).unwrap(), Celo(0b01));

        assert_eq!(BitniXor(Celo(0b10).rc(), Celo(0b01).rc()).eval(&[]).unwrap(), Celo(0b11));
        assert_eq!(BitniXor(Celo(0b11).rc(), Celo(0b01).rc()).eval(&[]).unwrap(), Celo(0b10));

        assert_eq!(BitniPremikLevo(Celo(0b10).rc(), Celo(1).rc()).eval(&[]).unwrap(), Celo(0b100));
        assert_eq!(BitniPremikLevo(Celo(0b11).rc(), Celo(2).rc()).eval(&[]).unwrap(), Celo(0b1100));

        assert_eq!(BitniPremikDesno(Celo(0b10).rc(), Celo(1).rc()).eval(&[]).unwrap(), Celo(0b1));
        assert_eq!(BitniPremikDesno(Celo(0b1100).rc(), Celo(2).rc()).eval(&[]).unwrap(), Celo(0b11));

        assert_eq!(Enako(Tip::Celo, Celo(12).rc(), Celo(12).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(Enako(Tip::Celo, Celo(13).rc(), Celo(14).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Enako(Tip::Real, Real(3.14).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(Enako(Tip::Real, Real(3.14).rc(), Real(3.14159268).rc()).eval(&[]).unwrap(), Laž);

        assert_eq!(NiEnako(Tip::Celo, Celo(12).rc(), Celo(12).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(NiEnako(Tip::Celo, Celo(13).rc(), Celo(14).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(NiEnako(Tip::Real, Real(3.14).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(NiEnako(Tip::Real, Real(3.14).rc(), Real(3.14159268).rc()).eval(&[]).unwrap(), Resnica);

        assert_eq!(Večje(Tip::Celo, Celo(12).rc(), Celo(12).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Večje(Tip::Celo, Celo(14).rc(), Celo(13).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(Večje(Tip::Real, Real(3.14).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Večje(Tip::Real, Real(3.14159268).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Resnica);

        assert_eq!(Večje(Tip::Celo, Celo(12).rc(), Celo(12).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Večje(Tip::Celo, Celo(14).rc(), Celo(13).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(Večje(Tip::Real, Real(3.14).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Večje(Tip::Real, Real(3.14159268).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Resnica);

        assert_eq!(VečjeEnako(Tip::Celo, Celo(12).rc(), Celo(12).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(VečjeEnako(Tip::Celo, Celo(14).rc(), Celo(13).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(VečjeEnako(Tip::Real, Real(3.14).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(VečjeEnako(Tip::Real, Real(3.14159268).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(VečjeEnako(Tip::Real, Real(3.14).rc(), Real(3.14159268).rc()).eval(&[]).unwrap(), Laž);

        assert_eq!(Manjše(Tip::Celo, Celo(12).rc(), Celo(12).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Manjše(Tip::Celo, Celo(13).rc(), Celo(14).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(Manjše(Tip::Real, Real(3.14).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Manjše(Tip::Real, Real(3.14159268).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(Manjše(Tip::Real, Real(3.14).rc(), Real(3.14159268).rc()).eval(&[]).unwrap(), Resnica);

        assert_eq!(ManjšeEnako(Tip::Celo, Celo(12).rc(), Celo(12).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(ManjšeEnako(Tip::Celo, Celo(13).rc(), Celo(14).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(ManjšeEnako(Tip::Real, Real(3.14).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Resnica);
        assert_eq!(ManjšeEnako(Tip::Real, Real(3.14159268).rc(), Real(3.14).rc()).eval(&[]).unwrap(), Laž);
        assert_eq!(ManjšeEnako(Tip::Real, Real(3.14).rc(), Real(3.14159268).rc()).eval(&[]).unwrap(), Resnica);
    }

}
