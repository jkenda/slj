use std::{fmt::Debug, hash::Hash};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Žeton<'a> {
    Ločilo      (&'a str, usize, usize, &'a str),
    Operator    (&'a str, usize, usize, &'a str),
    Rezerviranka(&'a str, usize, usize, &'a str),
    Ime         (&'a str, usize, usize, &'a str),
    Tip         (&'a str, usize, usize, &'a str),
    Literal     (L<'a>),
    Neznano     (&'a str, usize, usize, &'a str),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum L<'a> {
    Bool(&'a str, usize, usize, &'a str),
    Celo(&'a str, usize, usize, &'a str),
    Real(&'a str, usize, usize, &'a str),
    Znak(&'a str, usize, usize, &'a str),
    Niz (&'a str, usize, usize, &'a str),
}

fn bool<'a>(val: &'a str, v: usize, z: usize, f: &'a str) -> Žeton<'a> {
    use Žeton::*;
    use L::*;
    Literal(Bool(val, v, z, f))
}

fn celo<'a>(val: &'a str, v: usize, z: usize, f: &'a str) -> Žeton<'a> {
    use Žeton::*;
    use L::*;
    Literal(Celo(val, v, z, f))
}

fn real<'a>(val: &'a str, v: usize, z: usize, f: &'a str) -> Žeton<'a> {
    use Žeton::*;
    use L::*;
    Literal(Real(val, v, z, f))
}

fn znak<'a>(val: &'a str, v: usize, z: usize, f: &'a str) -> Žeton<'a> {
    use Žeton::*;
    use L::*;
    Literal(Znak(val, v, z, f))
}

fn niz<'a>(val: &'a str, v: usize, z: usize, f: &'a str) -> Žeton<'a> {
    use Žeton::*;
    use L::*;
    Literal(Niz(val, v, z, f))
}

impl<'a> Žeton<'a> {
    pub fn lokacija(&self) -> (usize, usize) {
        use Žeton::*;
        use L::*;
        match self {
            Operator(_, v, z, ..) | Ločilo(_, v, z, ..) | Rezerviranka(_, v, z, ..) | Ime(_, v, z, ..) | Tip(_, v, z, ..) | Neznano(_, v, z, ..) => (*v, *z),
            Žeton::Literal(literal) => match literal {
                Bool(_, v, z, ..) | Celo(_, v, z, ..) | Real(_, v, z, ..) | Znak(_, v, z, ..) | Niz(_, v, z, ..) => (*v, *z),
            }
        }
    }

    pub fn datoteka(&self) -> String {
        use Žeton::*;
        use L::*;
        match self {
            Operator(.., f) | Ločilo(.., f) | Rezerviranka(.., f) | Ime(.., f) | Tip(.., f) | Neznano(.., f) => f.to_string(),
            Žeton::Literal(literal) => match literal {
                Bool(.., f) | Celo(.., f) | Real(.., f) | Znak(.., f) | Niz(.., f) => f.to_string(),
            }
        }
    }

    pub fn lokacija_str(&self) -> String {
        use Žeton::*;
        use L::*;
        match self {
            Operator(.., v, z, f) | Ločilo(.., v, z, f) | Rezerviranka(.., v, z, f) | Ime(.., v, z, f) | Tip(.., v, z, f) | Neznano(.., v, z, f) => format!("{f}: [{v}, {z})"),
            Žeton::Literal(literal) => match literal {
                Bool(.., v, z, f) | Celo(.., v, z, f) | Real(.., v, z, f) | Znak(.., v, z, f) | Niz(.., v, z, f) => format!("[{f}: {v}, {z})"),
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

pub struct Lekser<'a> {
    ime: &'a str,
    tekst: &'a str,
}

pub trait Razčleni<'a> {
    fn razčleni<'b: 'a>(&'b self, ime: &'b str) -> Vec<Žeton<'b>>;
}

impl<'a> Razčleni<'a> for str {
    fn razčleni<'b: 'a>(&'b self, ime: &'b str) -> Vec<Žeton<'b>>
    {
        let lekser = Lekser::<'b>::new(ime, self);
        lekser.razčleni()
    }
}


impl<'a> Lekser<'a> {

    pub const fn new<'b>(ime: &'b str, tekst: &'b str) -> Self
        where 'b: 'a
    {
        Lekser { ime, tekst }
    }

    pub fn razčleni(&self) -> Vec<Žeton<'a>> {
        use Žeton::*;

        const ZADNJA_MEJA: &str = r#"(?:["=<>|&!+\-*/%^@,;:#\n(){}\[\]]|\b|$)"#;
        const PRESLEDEK: &str = r"([^\S\n]*)";

        let regexi: Vec<(Regex, fn(&'a str, usize, usize, &'a str) -> Žeton<'a>)> = vec![
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

        while i < self.tekst.len() {
            match self.najdi_token(&regexi, &self.tekst[i..], vrstica, znak) {
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
                        znak += self.tekst[i..i+dolžina].chars().count();
                    }
                    i += dolžina;
                },
                None => (),
            };
        }

        tokeni
    }

    fn najdi_token(&self, regexi: &[(Regex, fn(&'a str, usize, usize, &'a str) -> Žeton<'a>)], beseda: &'a str, vrstica: usize, znak: usize) -> Option<(Žeton<'a>, usize)> {
        let (regex, token) = &regexi[0];

        match regex.captures(beseda) {
            Some(skupine) => {
                // 1. skupina je zadetek
                let presledek = skupine.get(1).unwrap();
                let zadetek = skupine.get(2).unwrap();
                let velikost_presledka = presledek.as_str().chars().count();
                Some((token(zadetek.as_str(), vrstica, znak + velikost_presledka, self.ime), zadetek.end()))
            },
            None =>
                if regexi.len() > 1 {
                    self.najdi_token(&regexi[1..], beseda, vrstica, znak)
                }
                else {
                    None
                },
        }
    }

}

#[cfg(test)]
mod testi {
    use super::{Žeton::*, L::*, Razčleni, *};

    #[test]
    fn rezervirane_besede() {
        assert_eq!("naj".razčleni("[test]"), [Rezerviranka("naj", 1, 1, "[test]")]);
        assert_eq!("čene".razčleni("[test]"), [Rezerviranka("čene", 1, 1, "[test]")]);
        assert_eq!("če".razčleni("[test]"), [Rezerviranka("če", 1, 1, "[test]")]);
        assert_eq!("dokler".razčleni("[test]"), [Rezerviranka("dokler", 1, 1, "[test]")]);
        assert_eq!("za".razčleni("[test]"), [Rezerviranka("za", 1, 1, "[test]")]);
        assert_eq!("funkcija".razčleni("[test]"), [Rezerviranka("funkcija", 1, 1, "[test]")]);
        assert_eq!("vrni".razčleni("[test]"), [Rezerviranka("vrni", 1, 1, "[test]")]);
        assert_eq!("prekini".razčleni("[test]"), [Rezerviranka("prekini", 1, 1, "[test]")]);
    }

    #[test]
    fn literali() {
        assert_eq!("resnica".razčleni("[test]"), [Literal(Bool("resnica", 1, 1, "[test]"))]);
        assert_eq!("laž".razčleni("[test]"), [Literal(Bool("laž", 1, 1, "[test]"))]);

        assert_eq!("'a'".razčleni("[test]"), [Literal(Znak("'a'", 1, 1, "[test]"))]);
        assert_eq!("'đ'".razčleni("[test]"), [Literal(Znak("'đ'", 1, 1, "[test]"))]);
        assert_eq!(r"'\n'".razčleni("[test]"), [Literal(Znak(r"'\n'", 1, 1, "[test]"))]);
        assert_eq!(r"'\\'".razčleni("[test]"), [Literal(Znak(r"'\\'", 1, 1, "[test]"))]);
        assert_eq!(r"'\r'".razčleni("[test]"), [Literal(Znak(r"'\r'", 1, 1, "[test]"))]);
        assert_eq!(r"'\f'".razčleni("[test]"), [Neznano(r"'\f'", 1, 1, "[test]")]);

        assert_eq!("\"\"".razčleni("[test]"), [Literal(Niz("\"\"", 1, 1, "[test]"))]);
        assert_eq!("\"niz\"".razčleni("[test]"), [Literal(Niz("\"niz\"", 1, 1, "[test]"))]);
        assert_eq!("\"3.14\"".razčleni("[test]"), [Literal(Niz("\"3.14\"", 1, 1, "[test]"))]);
        assert_eq!("\"{}\\n\"".razčleni("[test]"), [Literal(Niz("\"{}\\n\"", 1, 1, "[test]"))]);
        assert_eq!("\"{}\\n\" \"smola\"".razčleni("[test]"), [Literal(Niz("\"{}\\n\"", 1, 1, "[test]")), Literal(Niz("\"smola\"", 1, 8, "[test]"))]);

        assert_eq!("0".razčleni("[test]"), [Literal(Celo("0", 1, 1, "[test]"))]);
        assert_eq!("13".razčleni("[test]"), [Literal(Celo("13", 1, 1, "[test]"))]);
        assert_eq!("1_000_000".razčleni("[test]"), [Literal(Celo("1_000_000", 1, 1, "[test]"))]);

        assert_eq!("0.5".razčleni("[test]"), [Literal(Real("0.5", 1, 1, "[test]"))]);
        assert_eq!("3.14".razčleni("[test]"), [Literal(Real("3.14", 1, 1, "[test]"))]);
    }

    #[test]
    fn ime() {
        assert_eq!("a".razčleni("[test]"), [Ime("a", 1, 1, "[test]")]);
        assert_eq!("švajs".razčleni("[test]"), [Ime("švajs", 1, 1, "[test]")]);
        assert_eq!("švajs  mašina".razčleni("[test]"), [Ime("švajs", 1, 1, "[test]"), Ime("mašina", 1, 8, "[test]")]);
        assert_eq!("__groot__".razčleni("[test]"), [Ime("__groot__", 1, 1, "[test]")]);
        assert_eq!("kamelskaTelewizje".razčleni("[test]"), [Ime("kamelskaTelewizje", 1, 1, "[test]")]);
        assert_eq!("RabeljskoJezero123".razčleni("[test]"), [Ime("RabeljskoJezero123", 1, 1, "[test]")]);
        assert_eq!("0cyka".razčleni("[test]"), [Neznano("0cyka", 1, 1, "[test]")]);
    }

    #[test]
    fn operatorji() {
        assert_eq!("a kot real".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("kot", 1, 3, "[test]"), Tip("real", 1, 7, "[test]")]);

        assert_eq!("a<<=b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("<<=", 1, 2, "[test]"), Ime("b", 1, 5, "[test]")]);
        assert_eq!("a<< b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("<<",  1, 2, "[test]"), Ime("b", 1, 5, "[test]")]);
        assert_eq!("a>>=b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator(">>=", 1, 2, "[test]"), Ime("b", 1, 5, "[test]")]);
        assert_eq!("a >>b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator(">>",  1, 3, "[test]"), Ime("b", 1, 5, "[test]")]);

        assert_eq!("a==b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("==", 1, 2, "[test]"), Ime("b", 1, 4, "[test]")]);
        assert_eq!("a!=b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("!=", 1, 2, "[test]"), Ime("b", 1, 4, "[test]")]);
        assert_eq!("a<=b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("<=", 1, 2, "[test]"), Ime("b", 1, 4, "[test]")]);
        assert_eq!("a>=b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator(">=", 1, 2, "[test]"), Ime("b", 1, 4, "[test]")]);
        assert_eq!("a<b".razčleni("[test]"), [Ime("a",  1, 1, "[test]"), Operator("<",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a>b".razčleni("[test]"), [Ime("a",  1, 1, "[test]"), Operator(">",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);

        assert_eq!("a+b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("+", 1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a-b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("-", 1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a*b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("*", 1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a/b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("/", 1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a%b".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Operator("%", 1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);

        assert_eq!("3+2".razčleni("[test]"), [Literal(Celo("3", 1, 1, "[test]")), Operator("+", 1, 2, "[test]"), Literal(Celo("2", 1, 3, "[test]"))]);
        assert_eq!("3-2".razčleni("[test]"), [Literal(Celo("3", 1, 1, "[test]")), Operator("-", 1, 2, "[test]"), Literal(Celo("2", 1, 3, "[test]"))]);
        assert_eq!("3*2".razčleni("[test]"), [Literal(Celo("3", 1, 1, "[test]")), Operator("*", 1, 2, "[test]"), Literal(Celo("2", 1, 3, "[test]"))]);
        assert_eq!("3/2".razčleni("[test]"), [Literal(Celo("3", 1, 1, "[test]")), Operator("/", 1, 2, "[test]"), Literal(Celo("2", 1, 3, "[test]"))]);
        assert_eq!("3%2".razčleni("[test]"), [Literal(Celo("3", 1, 1, "[test]")), Operator("%", 1, 2, "[test]"), Literal(Celo("2", 1, 3, "[test]"))]);
    }

    #[test]
    fn ločila() {
        assert_eq!("a,b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo(",",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a;b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo(";",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a:b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo(":",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a#b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo("#",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a(b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo("(",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a)b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo(")",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a)b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo(")",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a{b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo("{",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a}b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo("}",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a[b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo("[",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a]b".razčleni("[test]"),  [Ime("a", 1, 1, "[test]"), Ločilo("]",  1, 2, "[test]"), Ime("b", 1, 3, "[test]")]);
        assert_eq!("a\nb".razčleni("[test]"), [Ime("a", 1, 1, "[test]"), Ločilo("\n", 1, 2, "[test]"), Ime("b", 2, 1, "[test]")]);
    }

    #[test]
    fn preprost() {
        assert_eq!("če čene".razčleni("[test]"), [Rezerviranka("če", 1, 1, "[test]"), Rezerviranka("čene", 1, 4, "[test]")]);
    }

    #[test]
    fn napreden() {
        assert_eq!(
            "če\nresnica{dokler laž{natisni(\"nemogoče\")}}".razčleni("[test]"),
            [ Rezerviranka("če", 1, 1, "[test]"), Ločilo("\n", 1, 3, "[test]"), 
              bool("resnica", 2, 1, "[test]"), Ločilo("{", 2, 8, "[test]"), Rezerviranka("dokler", 2, 9, "[test]"), bool("laž", 2, 16, "[test]"),
              Ločilo("{", 2, 19, "[test]"), Ime("natisni", 2, 20, "[test]"), Ločilo("(", 2, 27, "[test]"), niz("\"nemogoče\"", 2, 28, "[test]"), Ločilo(")", 2, 38, "[test]"),
              Ločilo("}", 2, 39, "[test]"), Ločilo("}", 2, 40, "[test]") ]
        );
    }

}
