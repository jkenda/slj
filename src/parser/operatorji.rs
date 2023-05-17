use std::rc::Rc;
use super::drevo::VozliščeOption::{*, self};
use crate::parser::{Vozlišče::{*, self}, Tip};

pub fn prireditveni_op(op: &str) -> VozliščeOption {
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

pub const PRIMERJALNI_OP: [&str; 6] = ["==", "!=", ">", ">=", "<", "<="];
pub fn primerjalni_op(op: &str) -> Option<fn(Tip, Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče> {
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

pub fn aritmetični_op(op: &str) -> fn(Tip, Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
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

pub fn bitni_op(op: &str) -> fn(Rc<Vozlišče>, Rc<Vozlišče>) -> Vozlišče {
    match op {
        "|"  => BitniAli,
        "^"  => BitniXor,
        "&"  => BitniIn,
        "<<"  => BitniPremikLevo,
        ">>"  => BitniPremikDesno,
        _    => unreachable!()
    }
}

