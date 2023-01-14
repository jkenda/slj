use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tip {
    Brez,
    Bool,
    Celo,
    Real,
    Znak,
    Niz,
}

impl From<&str> for Tip {
    fn from(value: &str) -> Self {
        match value {
            "brez" => Tip::Brez,
            "bool" => Tip::Bool,
            "celo" => Tip::Celo,
            "real" => Tip::Real,
            "znak" => Tip::Znak,
            "niz"  => Tip::Niz,
            _ => panic!("Neznan tip: {value}"),
        }
    }
}

impl Display for Tip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Tip::*;
        write!(f, "{}", match self {
            Brez => "brez",
            Bool => "bool",
            Celo => "celo",
            Real => "real",
            Znak => "znak",
            Niz  => "niz",
        })
    }
}

