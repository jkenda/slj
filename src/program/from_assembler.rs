use super::*;

impl From<String> for Program {
    fn from(assembler: String) -> Self {
        use UkazPodatek::*;

        let mut ukazi: Vec<UkazPodatek> = Vec::new();
        let mut push_tipi = Vec::new();
        let vrstice = assembler.split('\n');

        for vrstica in vrstice {
            if vrstica.len() == 0 { continue; }
            let besede: Vec<&str> = vrstica.split_whitespace().collect();

            ukazi.push(match besede[1] {
                "PUSH" => {
                    if besede[2].chars().nth(0).unwrap() == '#' {
                        if besede[2].contains('.') {
                            push_tipi.push(Tip::Real);
                            PUSH(Podatek { f:  besede[2][1..].parse().unwrap() })
                        }
                        else {
                            push_tipi.push(Tip::Celo);
                            PUSH(Podatek { i:  besede[2][1..].parse().unwrap() })
                        }
                    }
                    else {
                        push_tipi.push(Tip::Znak);
                        PUSH(Podatek { c: besede[2][1..besede[1].len()-1]
                            .replace(r"\\", "\\")
                                .replace(r"\n", "\n")
                                .replace(r"\t", "\t")
                                .replace(r"\r", "\r")
                                .replace(r#"\""#, "\"")
                                .replace(r"\'", "\'")
                                .chars()
                                .next()
                                .unwrap() })
                    }
                },
                "ALOC" => ALOC(besede[2][0..].parse().unwrap()),
                "JUMP" => JUMP(besede[2][1..].parse().unwrap()),
                "JMPC" => JMPC(besede[2][1..].parse().unwrap()),
                "LOAD" => LOAD(besede[2][1..].parse().unwrap()),
                "LDOF" => LDOF(besede[2][1..].parse().unwrap()),
                "LDDY" => LDDY(besede[2][1..].parse().unwrap()),
                "STOR" => STOR(besede[2][1..].parse().unwrap()),
                "STOF" => STOF(besede[2][1..].parse().unwrap()),
                "STDY" => STDY(besede[2][1..].parse().unwrap()),
                "TOP"  => TOP(besede[2][0..].parse().unwrap()),
                "JMPD" => JMPD,
                "POS"  => POS,
                "ZERO" => ZERO,
                "LOFF" => LOFF,
                "SOFF" => SOFF,
                "PRTC" => PRTC,
                "GETC" => GETC,
                "ADDF" => ADDF,
                "SUBF" => SUBF,
                "MULF" => MULF,
                "DIVF" => DIVF,
                "MODF" => MODF,
                "POWF" => POWF,
                "ADDI" => ADDI,
                "SUBI" => SUBI,
                "MULI" => MULI,
                "DIVI" => DIVI,
                "MODI" => MODI,
                "POWI" => POWI,
                "BOR"  => BOR,
                "BXOR" => BXOR,
                "BAND" => BAND,
                "FTOI" => FTOI,
                "ITOF" => ITOF,
                _      => NOOP,
            });
        }

        Program { push_tipi, ukazi }
    }
}

