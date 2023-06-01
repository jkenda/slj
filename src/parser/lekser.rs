use std::{fmt::Debug, hash::Hash};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Žeton<'a> {
    Ločilo      (&'a str, usize, usize),
    Operator    (&'a str, usize, usize),
    Rezerviranka(&'a str, usize, usize),
    Ime         (&'a str, usize, usize),
    Tip         (&'a str, usize, usize),
    Literal     (L<'a>),
    Neznano     (&'a str, usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum L<'a> {
    Bool(&'a str, usize, usize),
    Celo(&'a str, usize, usize),
    Real(&'a str, usize, usize),
    Znak(&'a str, usize, usize),
    Niz (&'a str, usize, usize),
}

fn bool<'a>(val: &'a str, v: usize, z: usize) -> Žeton {
    use Žeton::*;
    use L::*;
    Literal(Bool(val, v, z))
}

fn celo<'a>(val: &'a str, v: usize, z: usize) -> Žeton {
    use Žeton::*;
    use L::*;
    Literal(Celo(val, v, z))
}

fn real<'a>(val: &'a str, v: usize, z: usize) -> Žeton {
    use Žeton::*;
    use L::*;
    Literal(Real(val, v, z))
}

fn znak<'a>(val: &'a str, v: usize, z: usize) -> Žeton {
    use Žeton::*;
    use L::*;
    Literal(Znak(val, v, z))
}

fn niz<'a>(val: &'a str, v: usize, z: usize) -> Žeton {
    use Žeton::*;
    use L::*;
    Literal(Niz(val, v, z))
}

impl<'a> Žeton<'a> {
    pub fn lokacija(&self) -> (usize, usize) {
        use Žeton::*;
        use L::*;
        match self {
            Operator(_, v, z) | Ločilo(_, v, z) | Rezerviranka(_, v, z) | Ime(_, v, z) | Tip(_, v, z) | Neznano(_, v, z) => (*v, *z),
            Žeton::Literal(literal) => match literal {
                Bool(_, v, z) | Celo(_, v, z) | Real(_, v, z) | Znak(_, v, z) | Niz(_, v, z) => (*v, *z),
            }
        }
    }

    pub fn lokacija_str(&self) -> String {
        use Žeton::*;
        use L::*;
        match self {
            Operator(.., v, z) | Ločilo(.., v, z) | Rezerviranka(.., v, z) | Ime(.., v, z) | Tip(.., v, z) | Neznano(.., v, z) => format!("[{v}, {z})"),
            Žeton::Literal(literal) => match literal {
                Bool(.., v, z) | Celo(.., v, z) | Real(.., v, z) | Znak(.., v, z) | Niz(.., v, z) => format!("[{v}, {z})"),
            }
        }
    }

    pub fn len(&self) -> usize {
        use Žeton::*;
        use L::*;
        match self {
            Operator(val, ..) | Ločilo(val, ..) | Rezerviranka(val, ..) | Ime(val, ..) | Tip(val, ..) | Neznano(val, ..) => val.len(),
            Žeton::Literal(literal) => match literal {
                Bool(val, ..) | Celo(val, ..) | Real(val, ..) | Znak(val, ..) | Niz(val, ..) => val.len(),
            }
        }
    }

    pub fn as_str(&self) -> &'a str {
        use Žeton::*;
        use L::*;
        match self {
            Operator(val, ..) | Ločilo(val, ..) | Rezerviranka(val, ..) | Ime(val, ..) | Tip(val, ..) | Neznano(val, ..) => val,
            Žeton::Literal(literal) => match literal {
                Bool(val, ..) | Celo(val, ..) | Real(val, ..) | Znak(val, ..) | Niz(val, ..) => val,
            }
        }
    }
}

impl ToString for Žeton<'_> {
    fn to_string(&self) -> String {
        use Žeton::*;
        use L::*;
        match self {
            Operator(val, ..) | Ločilo(val, ..) | Rezerviranka(val, ..) | Ime(val, ..) | Tip(val, ..) | Neznano(val, ..) => val.to_string(),
            Žeton::Literal(literal) => match literal {
                Bool(val, ..) | Celo(val, ..) | Real(val, ..) | Znak(val, ..) | Niz(val, ..) => val.to_string(),
            }
        }
    }
}

impl Hash for Žeton<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Žeton::*;
        use L::*;
        match self {
            Operator(val, ..) | Ločilo(val, ..) | Rezerviranka(val, ..) | Ime(val, ..) | Tip(val, ..) | Neznano(val, ..) => val.hash(state),
            Žeton::Literal(literal) => match literal {
                Bool(val, ..) | Celo(val, ..) | Real(val, ..) | Znak(val, ..) | Niz(val, ..) => val.hash(state),
            }
        }
    }
}

pub struct Tokenizer;

pub trait Razčleni<'a> {
    fn razčleni(&'a self) -> Vec<Žeton<'a>>;
}

impl<'a> Razčleni<'a> for &str {
    fn razčleni(&'a self) -> Vec<Žeton<'a>> {
        Tokenizer::tokenize(self)
    }
}


impl<'a> Tokenizer {

    pub fn tokenize(tekst: &'a str) -> Vec<Žeton> {
        use Žeton::*;

        const ZADNJA_MEJA: &str = r#"(?:["=<>|&!+\-*/%^@,;:#\n(){}\[\]]|\b|$)"#;
        const PRESLEDEK: &str = r"([^\S\n]*)";

        let regexi: Vec<(Regex, fn(&'a str, usize, usize) -> Žeton<'a>)> = vec![
            (Regex::new(&format!(r"^{PRESLEDEK}(naj|spr|kons|čene|če|dokler|za|funkcija|vrni|prekini){ZADNJA_MEJA}")).unwrap(), Rezerviranka),
            (Regex::new(&format!(r"^{PRESLEDEK}(brez|bool|celo|real|znak){ZADNJA_MEJA}")).unwrap(), Tip),
            (Regex::new(&format!(r"^{PRESLEDEK}(resnica|laž){ZADNJA_MEJA}")).unwrap(), bool),
            (Regex::new(&format!(r"^{PRESLEDEK}('(.|\\[\\nrt'])')")).unwrap(), znak),
            (Regex::new(&format!( "^{PRESLEDEK}(\"[^\n\"]*\")")).unwrap(), niz),
            (Regex::new(&format!(r"^{PRESLEDEK}(\d+\.\d+|\d{{1,3}}(_\d{{3}})+\.(\d{{3}}_)+\d{{1,3}}){ZADNJA_MEJA}")).unwrap(), real),
            (Regex::new(&format!(r"^{PRESLEDEK}(\d+|\d{{1,3}}(_\d{{3}})+){ZADNJA_MEJA}")).unwrap(), celo),
            (Regex::new(&format!(r"^{PRESLEDEK}([.,;:#\n(){{}}\[\]]|->)")).unwrap(), Ločilo),
            (Regex::new(&format!(r"^{PRESLEDEK}(?x)(
                        # pretvorba
                            kot |
                        # zamik
                            <<= | >>= | << | >> |
                        # primerjava
                            == | != | <= | >= | < | > |
                        # spreminjanje vrednosti
                            \*\*= | \|\|= | &&= | [+\-*/%|&^]= |
                        # Boolovi operatorji
                            \|\| | && | ! |
                        # aritmetika
                            \*\* | [+\-*/%] |
                        # binarna aritmetika
                            [|&^] |
                        # prirejanje
                            = |
                        # referenca
                            @
                        )")).unwrap(), Operator),
            (Regex::new(&format!(r"^{PRESLEDEK}([_\p{{Letter}}][\w\d]*){ZADNJA_MEJA}")).unwrap(), Ime),
            (Regex::new(&format!(r"^{PRESLEDEK}(\S*){ZADNJA_MEJA}")).unwrap(), Neznano),
        ];

        let mut tokeni: Vec<Žeton> = Vec::new();
        let mut vrstica = 1;
        let mut znak = 1;

        let mut i: usize = 0;

        while i < tekst.len() {
            match najdi_token(&regexi, &tekst[i..], vrstica, znak) {
                Some((token, dolžina)) => {
                    match token {
                        Neznano("", ..) => (),
                        _ => tokeni.push(token),
                    }
                    if let Ločilo("\n", ..) = token {
                        vrstica += 1;
                        znak = 1;
                    }
                    else {
                        // vzamemo chars().count() namesto len(),
                        // saj je važno število znakov in ne bajtov
                        znak += tekst[i..i+dolžina].chars().count();
                    }
                    i += dolžina;
                },
                None => (),
            };
        }

        tokeni
    }

}

fn najdi_token<'a>(regexi: &[(Regex, fn(&'a str, usize, usize) -> Žeton<'a>)], beseda: &'a str, vrstica: usize, znak: usize) -> Option<(Žeton<'a>, usize)> {
    let (regex, token) = &regexi[0];

    match regex.captures(beseda) {
        Some(skupine) => {
            // 1. skupina je zadetek
            let presledek = skupine.get(1).unwrap();
            let zadetek = skupine.get(2).unwrap();
            let velikost_presledka = presledek.as_str().chars().count();
            Some((token(zadetek.as_str(), vrstica, znak + velikost_presledka), zadetek.end()))
        },
        None =>
            if regexi.len() > 1 {
                najdi_token(&regexi[1..], beseda, vrstica, znak)
            }
            else {
                None
            },
    }
}

#[cfg(test)]
mod testi {
    use super::{Žeton::*, L::*, Razčleni, *};

    #[test]
    fn rezervirane_besede() {
        assert_eq!("naj".razčleni(), [Rezerviranka("naj", 1, 1)]);
        assert_eq!("čene".razčleni(), [Rezerviranka("čene", 1, 1)]);
        assert_eq!("če".razčleni(), [Rezerviranka("če", 1, 1)]);
        assert_eq!("dokler".razčleni(), [Rezerviranka("dokler", 1, 1)]);
        assert_eq!("za".razčleni(), [Rezerviranka("za", 1, 1)]);
        assert_eq!("funkcija".razčleni(), [Rezerviranka("funkcija", 1, 1)]);
        assert_eq!("vrni".razčleni(), [Rezerviranka("vrni", 1, 1)]);
        assert_eq!("prekini".razčleni(), [Rezerviranka("prekini", 1, 1)]);
    }

    #[test]
    fn literali() {
        assert_eq!("resnica".razčleni(), [Literal(Bool("resnica", 1, 1))]);
        assert_eq!("laž".razčleni(), [Literal(Bool("laž", 1, 1))]);

        assert_eq!("'a'".razčleni(), [Literal(Znak("'a'", 1, 1))]);
        assert_eq!("'đ'".razčleni(), [Literal(Znak("'đ'", 1, 1))]);
        assert_eq!(r"'\n'".razčleni(), [Literal(Znak(r"'\n'", 1, 1))]);
        assert_eq!(r"'\\'".razčleni(), [Literal(Znak(r"'\\'", 1, 1))]);
        assert_eq!(r"'\r'".razčleni(), [Literal(Znak(r"'\r'", 1, 1))]);
        assert_eq!(r"'\f'".razčleni(), [Neznano(r"'\f'", 1, 1)]);

        assert_eq!("\"\"".razčleni(), [Literal(Niz("\"\"", 1, 1))]);
        assert_eq!("\"niz\"".razčleni(), [Literal(Niz("\"niz\"", 1, 1))]);
        assert_eq!("\"3.14\"".razčleni(), [Literal(Niz("\"3.14\"", 1, 1))]);
        assert_eq!("\"{}\\n\"".razčleni(), [Literal(Niz("\"{}\\n\"", 1, 1))]);
        assert_eq!("\"{}\\n\" \"smola\"".razčleni(), [Literal(Niz("\"{}\\n\"", 1, 1)), Literal(Niz("\"smola\"", 1, 8))]);

        assert_eq!("0".razčleni(), [Literal(Celo("0", 1, 1))]);
        assert_eq!("13".razčleni(), [Literal(Celo("13", 1, 1))]);
        assert_eq!("1_000_000".razčleni(), [Literal(Celo("1_000_000", 1, 1))]);

        assert_eq!("0.5".razčleni(), [Literal(Real("0.5", 1, 1))]);
        assert_eq!("3.14".razčleni(), [Literal(Real("3.14", 1, 1))]);
    }

    #[test]
    fn ime() {
        assert_eq!("a".razčleni(), [Ime("a", 1, 1)]);
        assert_eq!("švajs".razčleni(), [Ime("švajs", 1, 1)]);
        assert_eq!("švajs  mašina".razčleni(), [Ime("švajs", 1, 1), Ime("mašina", 1, 8)]);
        assert_eq!("__groot__".razčleni(), [Ime("__groot__", 1, 1)]);
        assert_eq!("kamelskaTelewizje".razčleni(), [Ime("kamelskaTelewizje", 1, 1)]);
        assert_eq!("RabeljskoJezero123".razčleni(), [Ime("RabeljskoJezero123", 1, 1)]);
        assert_eq!("0cyka".razčleni(), [Neznano("0cyka", 1, 1)]);
    }

    #[test]
    fn operatorji() {
        assert_eq!("a kot real".razčleni(), [Ime("a", 1, 1), Operator("kot", 1, 3), Tip("real", 1, 7)]);

        assert_eq!("a<<=b".razčleni(), [Ime("a", 1, 1), Operator("<<=", 1, 2), Ime("b", 1, 5)]);
        assert_eq!("a<< b".razčleni(), [Ime("a", 1, 1), Operator("<<", 1, 2), Ime("b", 1, 5)]);
        assert_eq!("a>>=b".razčleni(), [Ime("a", 1, 1), Operator(">>=", 1, 2), Ime("b", 1, 5)]);
        assert_eq!("a >>b".razčleni(), [Ime("a", 1, 1), Operator(">>", 1, 3), Ime("b", 1, 5)]);

        assert_eq!("a==b".razčleni(), [Ime("a", 1, 1), Operator("==", 1, 2), Ime("b", 1, 4)]);
        assert_eq!("a!=b".razčleni(), [Ime("a", 1, 1), Operator("!=", 1, 2), Ime("b", 1, 4)]);
        assert_eq!("a<=b".razčleni(), [Ime("a", 1, 1), Operator("<=", 1, 2), Ime("b", 1, 4)]);
        assert_eq!("a>=b".razčleni(), [Ime("a", 1, 1), Operator(">=", 1, 2), Ime("b", 1, 4)]);
        assert_eq!("a<b".razčleni(), [Ime("a", 1, 1), Operator("<", 1, 2), Ime("b", 1, 3)]);
        assert_eq!("a>b".razčleni(), [Ime("a", 1, 1), Operator(">", 1, 2), Ime("b", 1, 3)]);

        assert_eq!("a+b".razčleni(), [Ime("a", 1, 1), Operator("+", 1, 2), Ime("b", 1, 3)]);
        assert_eq!("a-b".razčleni(), [Ime("a", 1, 1), Operator("-", 1, 2), Ime("b", 1, 3)]);
        assert_eq!("a*b".razčleni(), [Ime("a", 1, 1), Operator("*", 1, 2), Ime("b", 1, 3)]);
        assert_eq!("a/b".razčleni(), [Ime("a", 1, 1), Operator("/", 1, 2), Ime("b", 1, 3)]);
        assert_eq!("a%b".razčleni(), [Ime("a", 1, 1), Operator("%", 1, 2), Ime("b", 1, 3)]);

        assert_eq!("3+2".razčleni(), [Literal(Celo("3", 1, 1)), Operator("+", 1, 2), Literal(Celo("2", 1, 3))]);
        assert_eq!("3-2".razčleni(), [Literal(Celo("3", 1, 1)), Operator("-", 1, 2), Literal(Celo("2", 1, 3))]);
        assert_eq!("3*2".razčleni(), [Literal(Celo("3", 1, 1)), Operator("*", 1, 2), Literal(Celo("2", 1, 3))]);
        assert_eq!("3/2".razčleni(), [Literal(Celo("3", 1, 1)), Operator("/", 1, 2), Literal(Celo("2", 1, 3))]);
        assert_eq!("3%2".razčleni(), [Literal(Celo("3", 1, 1)), Operator("%", 1, 2), Literal(Celo("2", 1, 3))]);
    }

    #[test]
    fn ločila() {
        assert_eq!("a,b".razčleni(),  [Ime("a", 1, 1), Ločilo(",",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a;b".razčleni(),  [Ime("a", 1, 1), Ločilo(";",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a:b".razčleni(),  [Ime("a", 1, 1), Ločilo(":",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a#b".razčleni(),  [Ime("a", 1, 1), Ločilo("#",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a(b".razčleni(),  [Ime("a", 1, 1), Ločilo("(",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a)b".razčleni(),  [Ime("a", 1, 1), Ločilo(")",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a)b".razčleni(),  [Ime("a", 1, 1), Ločilo(")",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a{b".razčleni(),  [Ime("a", 1, 1), Ločilo("{",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a}b".razčleni(),  [Ime("a", 1, 1), Ločilo("}",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a[b".razčleni(),  [Ime("a", 1, 1), Ločilo("[",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a]b".razčleni(),  [Ime("a", 1, 1), Ločilo("]",  1, 2), Ime("b", 1, 3)]);
        assert_eq!("a\nb".razčleni(), [Ime("a", 1, 1), Ločilo("\n", 1, 2), Ime("b", 2, 1)]);
    }

    #[test]
    fn preprost() {
        assert_eq!("če čene".razčleni(), [Rezerviranka("če", 1, 1), Rezerviranka("čene", 1, 4)]);
    }

    #[test]
    fn napreden() {
        assert_eq!(
            "če\nresnica{dokler laž{natisni(\"nemogoče\")}}".razčleni(),
            [ Rezerviranka("če", 1, 1), Ločilo("\n", 1, 3), 
              bool("resnica", 2, 1), Ločilo("{", 2, 8), Rezerviranka("dokler", 2, 9), bool("laž", 2, 16),
              Ločilo("{", 2, 19), Ime("natisni", 2, 20), Ločilo("(", 2, 27), niz("\"nemogoče\"", 2, 28), Ločilo(")", 2, 38),
              Ločilo("}", 2, 39), Ločilo("}", 2, 40) ]
        );
    }

}