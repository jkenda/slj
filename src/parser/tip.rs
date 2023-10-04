use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::rc::Rc;
use Tip::*;

use crate::parser::lekser::L;
use crate::parser::napaka::OznakaNapake::*;

use super::drevo::Vozlišče;
use super::napaka::{Napake, Napaka};
use super::lekser::Žeton;
use super::loci::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Tip {
    Brez,
    Bool,
    Celo,
    Real,
    Znak,
    Seznam(Box<Tip>, i32),
    Strukt(BTreeMap<String, Box<Tip>>),
    Referenca(Box<Tip>),
    RefSeznama(Box<Tip>),
}

impl Tip {
    pub fn from(izraz: &[Žeton], konstante: &HashMap<String, Rc<Vozlišče>>) -> Result<Self, Napake> {
        use Žeton::{Ločilo, Operator};
        match izraz {
            [ Žeton::Tip("brez", ..) ] => Ok(Tip::Brez),
            [ Žeton::Tip("bool", ..) ] => Ok(Tip::Bool),
            [ Žeton::Tip("celo", ..) ] => Ok(Tip::Celo),
            [ Žeton::Tip("real", ..) ] => Ok(Tip::Real),
            [ Žeton::Tip("znak", ..) ] => Ok(Tip::Znak),
            [ Ločilo("[", ..), tip @ .., Ločilo(";", ..), žeton @ Žeton::Literal(L::Celo(len, ..)) , Ločilo("]", ..) ] => 
                Ok(Tip::Seznam(Box::new(Tip::from(tip, konstante)?), 
                        match len.replace("_", "").parse() {
                            Ok(len) => Ok(len),
                            Err(err) => Err(Napake::from_zaporedje(&[*žeton], E1,
                                    &format!("Iz vrednosti ni mogoče ustvariti števila: {err} {}", žeton.lokacija_str())))
                        }?)),
            [ Ločilo("[", ..), tip @ .., Ločilo(";", ..), žeton @ Žeton::Ime(ime, ..) , Ločilo("]", ..) ] => 
                Ok(Tip::Seznam(Box::new(Tip::from(tip, konstante)?), 
                        match konstante.get(*ime) {
                            Some(v) => match &**v {
                                Vozlišče::Celo(len) => Ok(*len),
                                _ => Err(Napake::from_zaporedje(&[*žeton], E1,
                                        &format!("'{ime}' ni tipa 'celo': {}", žeton.lokacija_str())))
                            },
                            None => Err(Napake::from_zaporedje(&[*žeton], E1,
                                    &format!("Konstanta '{ime}' ne obstaja: {}", žeton.lokacija_str())))
                        }?)),
            [ Ločilo("{", ..), vmes @ .., Ločilo("}", ..) ] => Ok(Tip::Strukt(zgradi_tip_strukta(vmes)?)),
            [ Operator("@", ..), Ločilo("[", ..), ostanek @ .. , Ločilo("]", ..)] => Ok(RefSeznama(Box::new(Tip::from(ostanek, konstante)?))),
            [ Operator("@", ..), ostanek @ .. ] => Ok(Referenca(Box::new(Tip::from(ostanek, konstante)?))),
            _ => Err(Napake::from_zaporedje(izraz, E1, 
                    &format!("Neznan tip: '{}'", izraz.iter().map(|t| t.as_str()).collect::<Vec<&str>>().join("")))),
        }
    }

    pub fn dolžina(&self) -> i32 {
        match self {
            Seznam(_, len) => *len,
            _ => unreachable!("Tip {self} nima dolžine.")
        }
    }

    pub fn sprememba_stacka(&self) -> i32 {
        match self {
            Brez => 0,
            Bool | Celo | Real | Znak => 1,
            Seznam(tip, len) => (tip.sprememba_stacka() * len) + 1,
            Strukt(polja) => polja.values().map(|p| p.sprememba_stacka()).sum(),
            Referenca(_) => 1,
            RefSeznama(_) => 1,
        }
    }

    pub fn vsebuje_tip(&self) -> Self {
        match self {
            Tip::Seznam(tip, _) => (**tip).clone(),
            _ => unreachable!("Samo seznami vsebujejo tipe"),
        }
    }
}

impl Display for Tip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Tip::*;
        write!(f, "{}", &match self {
            Brez => "brez".to_string(),
            Bool => "bool".to_string(),
            Celo => "celo".to_string(),
            Real => "real".to_string(),
            Znak => "znak".to_string(),
            Seznam(tip, len) => format!("[{tip}; {len}]"),
            Strukt(polja) => {
                let mut str = "{\n".to_string();
                for (ime, tip) in polja {
                    str += &format!("{ime}: {tip},\n");
                }
                str += "}";
                str
            },
            Referenca(tip) => format!("@{tip}"),
            RefSeznama(tip) => format!("@[{tip}]"),
        })
    }
}

fn zgradi_tip_strukta<'a: 'b, 'b>(mut izraz: &'b [Žeton<'a>]) -> Result<BTreeMap<String, Box<Tip>>, Napake> {
    let mut polja = BTreeMap::new();
    let mut napake = Napake::new();

    let mut ločeno = loči_spredaj(izraz, &[","]);
    while ločeno.is_some() {
        let (polje, _, ostanek) = ločeno.unwrap()?;

        match polje {
            [ ime @ Žeton::Ime(..), Žeton::Ločilo(":", ..), tip @ .. ] => {
                match Tip::from(tip, &HashMap::new()) {
                    Ok(tip) => match polja.insert(ime.to_string(), Box::new(tip)) {
                        Some(..) => _ = napake.add_napaka(Napaka::from_zaporedje(&[*ime], E1, "Polje s tem imenom že obstaja")),
                        None => (),
                    },
                    Err(n)  => _ = napake.razširi(n),
                }
            },
            _ => _ = napake.add_napaka(Napaka::from_zaporedje(polje, E1, "Neveljavno polje")),
        };

        izraz = ostanek;
        ločeno = loči_spredaj(izraz, &[";", "\n"]);
    }
    if izraz != &[] {
        match izraz {
            [ ime @ Žeton::Ime(..), Žeton::Ločilo(":", ..), tip @ .. ] => {
                match Tip::from(tip, &HashMap::new()) {
                    Ok(tip) => match polja.insert(ime.to_string(), Box::new(tip)) {
                        Some(..) => _ = napake.add_napaka(Napaka::from_zaporedje(&[*ime], E1, "Polje s tem imenom že obstaja")),
                        None => (),
                    },
                    Err(n)  => _ = napake.razširi(n),
                }
            },
            _ => _ = napake.add_napaka(Napaka::from_zaporedje(izraz, E1, "Neveljavno polje")),
        };
    }

    if napake.prazno() {
        Ok(polja)
    }
    else {
        Err(napake)
    }
}

#[cfg(test)]
mod testi {
    use crate::parser::lekser::Razčleni;

    use super::*;

    #[test]
    fn from_string_to_string() {
        assert_eq!(Tip::from("brez".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "brez");
        assert_eq!(Tip::from("bool".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "bool");
        assert_eq!(Tip::from("celo".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "celo");
        assert_eq!(Tip::from("real".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "real");
        assert_eq!(Tip::from("znak".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "znak");

        assert_eq!(Tip::from("[celo; 6]".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "[celo; 6]");
        assert_eq!(Tip::from("[[celo; 3]; 6]".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "[[celo; 3]; 6]");

        assert_eq!(Tip::from("{ x: real, y: real }".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "{\nx: real,\ny: real,\n}");
        assert_eq!(Tip::from("{ _arr: [celo; 128], len: celo }".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "{\n_arr: [celo; 128],\nlen: celo,\n}");

        assert_eq!(Tip::from("@celo".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "@celo");
        assert_eq!(Tip::from("@[real]".razčleni("[test]").as_slice(), &HashMap::new()).unwrap().to_string(), "@[real]");
    }
}
