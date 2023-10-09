use crate::parser::loci::Escape;

use super::*;

impl Program {
    pub fn v_assembler(&self) -> String {
        let mut str = String::new();
        let mut j = 0;

        for (i, ukaz_podatek) in self.ukazi.iter().enumerate() {
            str += &format!("{i:3} ");
            str += &match ukaz_podatek {
                NOOP          => "NOOP\n".to_string(),
                JUMP(naslov)  => format!("JUMP #{naslov}\n"),
                JMPD          => "JMPD\n".to_string(),
                JMPC(naslov)  => format!("JMPC #{naslov}\n"),
                PUSH(podatek) => {
                    j += 1;
                    match self.push_tipi[j - 1] {
                        Tip::Real => format!("PUSH #{:?}\n", unsafe { podatek.f }),
                        Tip::Celo => format!("PUSH #{}\n",   unsafe { podatek.i }),
                        Tip::Znak => format!("PUSH '{}'\n",  unsafe { podatek.c.to_string().escape() }),
                        _ => unreachable!()
                    }
                },
                ALOC(razlika) => format!("ALOC {}{razlika}\n", if *razlika > 0 { "+" } else { "" }),
                POS           => "POS \n".to_string(),
                ZERO          => "ZERO\n".to_string(),
                LOAD(naslov)  => format!("LOAD @{naslov}\n"),
                LDOF(naslov)  => format!("LDOF +{naslov}\n"),
                LDDY(naslov)  => format!("LDDY +{naslov}\n"),
                STOR(naslov)  => format!("STOR @{naslov}\n"),
                STOF(naslov)  => format!("STOF +{naslov}\n"),
                STDY(naslov)  => format!("STDY +{naslov}\n"),
                TOP(odmik)    => format!("TOP  {}{odmik}\n", if *odmik > 0 { "+" } else { "" }),
                _             => format!("{ukaz_podatek:?}\n"),
            }
        }

        str
    }
}
