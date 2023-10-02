use super::*;

const INCLUDE: &str = "include 'ukazi.asm'\n\n";

impl ToFasmX86 for Vec<UkazPodatekRelative> {
    fn v_fasm_x86(&self) -> String {

        self.iter()
            .fold(INCLUDE.to_string(), |str, ukaz_podatek| {
                str
                + if let Oznaka(_) = ukaz_podatek { "" } else { "\t" }
                + &match ukaz_podatek {
                    PUSHI(število) => format!("PUSH {število}"),
                    PUSHF(število) => format!("PUSH {število}"),
                    PUSHC(znak) => format!("PUSH {}", v_utf8(*znak)),
                    JUMPRelative(oznaka) => format!("JUMP {}", formatiraj_oznako(oznaka)),
                    JMPCRelative(oznaka) => format!("JMPC {}", formatiraj_oznako(oznaka)),
                    Oznaka(oznaka) => format!("{}:", formatiraj_oznako(oznaka)),
                    PC(i) => format!("PC {i}"),

                    Osnovni(ALOC(mem))  => format!("ALOC {mem}"),
                    Osnovni(LOAD(addr)) => format!("LOAD {addr}"), // load normal
                    Osnovni(LDOF(addr)) => format!("LOAD {addr}"), // load w/ offset
                    Osnovni(LDDY(addr)) => format!("LOAD {addr}"), // load dynamic
                    Osnovni(STOR(addr)) => format!("LOAD {addr}"), // store normal
                    Osnovni(STOF(addr)) => format!("LOAD {addr}"), // store w/ offset
                    Osnovni(STDY(addr)) => format!("LOAD {addr}"), // store dynamic
                    Osnovni(TOP(addr))  => format!("TOP {addr}"),
                    Osnovni(instruction) => format!("{instruction:?}"),
                }
                + "\n"
            })
        + "\n\texit 0\n"
    }
}

fn v_utf8(znak: char) -> u32 {
    let mut buf = [0u8; 4];
    znak.encode_utf8(&mut buf);
    buf.iter().rev()
        .fold(0, |acc, b| acc << 8 | *b as u32)
}

fn formatiraj_oznako(oznaka: &str) -> String {
    format!(".{}", oznaka
        .replace("(", "8")
        .replace(")", "9")
        .replace("[", "F")
        .replace("]", "G")
        .replace("@", "V")
        .replace(", ", "__")
        )
}

#[cfg(test)]
mod testi {
    use std::collections::HashMap;
    use std::{fs::File, io::Write};
    use std::process::Command;

    use super::*;
    use crate::parser::drevo::{Drevo, Vozlišče};
    use crate::parser::tip::Tip;
    use Vozlišče::*;

    fn test(drevo: Drevo, expected: &str) -> Result<(), io::Error> {
        // transform AST into native x86_64 assembly
        let fasm = drevo
            .v_fasm_x86();

        // write assembly to file
        File::create("fasm/_main.asm")?
            .write_all(fasm.as_bytes())?;

        // compile with FASM
        let output = Command::new("fasm")
            .arg("fasm/_main.asm")
            .output()
            .expect("Failed to execute fasm");

        if !output.status.success() {
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "FASM failed"));
        }

        // run compiled binary
        let output = Command::new("fasm/_main")
            .output()
            .expect("Failed to execute main");

        if !output.status.success() {
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "program failed"));
        }

        let output = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn plus_minus_krat_deljeno_tiskaj() -> Result<(), io::Error> {
        let drevo = Drevo {
            funkcije: vec![],
            št_klicev: HashMap::new(),
            main: Zaporedje(vec![
                Natisni(CeloVZnak(Seštevanje(Tip::Celo, Celo(48).rc(), Celo(1).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(Seštevanje(Tip::Celo, Celo(48).rc(), Celo(3).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(Odštevanje(Tip::Celo, Celo(58).rc(), Celo(10).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(Množenje(Tip::Celo, Celo(15).rc(), Celo(4).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(Deljenje(Tip::Celo, Celo(100).rc(), Celo(2).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(Modulo(Tip::Celo, Celo(553).rc(), Celo(100).rc()).rc()).rc()).rc(),
                Natisni(Znak('ž').rc()).rc(),
            ]).rc()
        };

        test(drevo, "130<25ž")
    }

}

