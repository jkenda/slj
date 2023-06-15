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
                        Tip::Znak => format!("PUSH '{}'\n",  unsafe { podatek.c
                            .to_string()
                            .replace("\\", r"\\")
                                .replace("\n", r"\n")
                                .replace("\t", r"\t")
                                .replace("\r", r"\r")
                                .replace("\"", r#"\""#)
                                .replace("\'", r"\'")
                        }),
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
                SOFF          => "SOFF\n".to_string(),
                LOFF          => "LOFF\n".to_string(),
                PUTC          => "PUTC\n".to_string(),
                GETC          => "GETC\n".to_string(),
                ADDF          => "ADDF\n".to_string(),
                SUBF          => "SUBF\n".to_string(),
                MULF          => "MULF\n".to_string(),
                DIVF          => "DIVF\n".to_string(),
                MODF          => "MODF\n".to_string(),
                POWF          => "POWF\n".to_string(),
                ADDI          => "ADDI\n".to_string(),
                SUBI          => "SUBI\n".to_string(),
                MULI          => "MULI\n".to_string(),
                DIVI          => "DIVI\n".to_string(),
                MODI          => "MODI\n".to_string(),
                POWI          => "POWI\n".to_string(),
                BOR           => "BOR \n".to_string(),
                BXOR          => "BXOR\n".to_string(),
                BAND          => "BAND\n".to_string(),
                BSLL          => "BSLL\n".to_string(),
                BSLD          => "BSLD\n".to_string(),
                FTOI          => "FTOI\n".to_string(),
                ITOF          => "ITOF\n".to_string(),
            }
        }

        str
    }
}
