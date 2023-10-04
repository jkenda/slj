use super::*;

const PRE: &str = "include 'ukazi.asm'\n\n";
const POST: &str = "\n\texit 0";

impl ToFasmX86 for Vec<UkazPodatekRelative> {
    fn v_fasm_x86(&self) -> String {

        self.iter()
            .fold(PRE.to_string(), |str, ukaz_podatek| {
                str
                + if let Oznaka(_) = ukaz_podatek { "" } else { "\t" }
                + &match ukaz_podatek {
                    PUSHI(število)       => format!("PUSH {število}"),
                    PUSHF(število)       => format!("PUSH {število}"),
                    PUSHC(znak)          => format!("PUSH 0x{:0X}", v_utf8(*znak)),
                    JUMPRelative(oznaka) => format!("JUMP {}", formatiraj_oznako(oznaka)),
                    JMPCRelative(oznaka) => format!("JMPC {}", formatiraj_oznako(oznaka)),
                    CALL(oznaka)         => format!("CALL {}", formatiraj_oznako(oznaka)),
                    Oznaka(oznaka)       => format!("{}:",     formatiraj_oznako(oznaka)),
                    PC(i)                => format!("PC {i}"),

                    Osnovni(ALOC(mem))  => format!("ALOC {mem}"),
                    Osnovni(LOAD(addr)) => format!("LOAD {addr}"), // load normal
                    Osnovni(LDOF(addr)) => format!("LDOF {addr}"), // load w/ offset
                    Osnovni(LDDY(addr)) => format!("LDDY {addr}"), // load dynamic
                    Osnovni(STOR(addr)) => format!("STOR {addr}"), // store normal
                    Osnovni(STOF(addr)) => format!("STOF {addr}"), // store w/ offset
                    Osnovni(STDY(addr)) => format!("STDY {addr}"), // store dynamic
                    Osnovni(TOP(addr))  => format!("TOP  {addr}"),
                    Osnovni(instruction) => format!("{instruction:?}"),
                }
                + "\n"
            })
        + POST
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
    use std::thread;
    use std::{fs::File, io::Write};
    use std::process::{Command, Stdio};

    use super::*;
    use crate::parser::lekser::Razčleni;
    use crate::parser::Parse;
    use crate::parser::drevo::{Drevo, Vozlišče};
    use crate::parser::tip::Tip;
    use Vozlišče::*;

    fn test(fasm: &str, input: &str, expected: &str, bytes: bool) -> Result<(), io::Error> {
        // transform AST into native x86_64 assembly
        let thread_id = format!("{:?}", thread::current().id().to_owned());
        let thread_id = thread_id
            .split("(").nth(1).unwrap()
            .split(")").nth(0).unwrap();

        let program_filename = format!("fasm/_main__{thread_id}");

        // write assembly to file
        File::create(format!("{program_filename}.asm"))?
            .write_all(fasm.as_bytes())?;

        // compile with FASM
        let output = Command::new("fasm")
            .arg(format!("{program_filename}.asm"))
            .output()
            .expect("Failed to execute fasm");

        if !output.status.success() {
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "compilation failed"));
        }

        // run compiled binary
        let mut proces = Command::new(program_filename)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to execute main");

        let mut child_stdin = proces.stdin
            .take()
            .expect("Failed to open stdin");

        child_stdin.write_all(input.as_bytes())?;

        let output = proces
            .wait_with_output()
            .expect("Failed to wait on main");

        if !output.status.success() {
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "program failed"));
        }

        if bytes {
            assert_eq!(output.stdout, expected.as_bytes());
        }
        else {
            let output = String::from_utf8_lossy(&output.stdout);
            assert_eq!(output, expected);
        }

        Ok(())
    }

    #[test]
    fn putc() -> Result<(), io::Error> {
        let asm = Drevo {
            funkcije: vec![],
            št_klicev: HashMap::new(),
            main: Zaporedje(vec![
                Natisni(Znak('a').rc()).rc(),
                Natisni(Znak('ž').rc()).rc(),
                Natisni(Znak('😭').rc()).rc(),
                Natisni(Znak('\n').rc()).rc(),
            ]).rc()
        }
        .v_fasm_x86();

        test(&asm, "", "až😭\n", true)
    }

    #[test]
    fn getc() -> Result<(), io::Error> {
        let asm = Drevo {
            funkcije: vec![],
            št_klicev: HashMap::new(),
            main: Zaporedje(vec![
                Natisni(Preberi.rc()).rc(),
                Natisni(Preberi.rc()).rc(),
                Natisni(Preberi.rc()).rc(),
                Natisni(Preberi.rc()).rc(),
            ]).rc()
        }
        .v_fasm_x86();

        test(&asm, "asdf", "asdf", true)
        //test(drevo, "až😭\n", "až😭\n", true)
    }

    #[test]
    fn cele_operacije() -> Result<(), io::Error> {
        let asm = Drevo {
            funkcije: vec![],
            št_klicev: HashMap::new(),
            main: Okvir {
                zaporedje: Zaporedje(vec![
                    Natisni(CeloVZnak(Plus(Tip::Celo, Celo(48).rc(), Celo(1).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Plus(Tip::Celo, Celo(48).rc(), Celo(3).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Minus(Tip::Celo, Celo(58).rc(), Celo(10).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Krat(Tip::Celo, Celo(15).rc(), Celo(4).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Deljeno(Tip::Celo, Celo(100).rc(), Celo(2).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Modulo(Tip::Celo, Celo(553).rc(), Celo(100).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Potenca(Tip::Celo, Celo(3).rc(), Celo(4).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Minus(Tip::Celo, Celo(128).rc(), Potenca(Tip::Celo, Celo(-3).rc(), Celo(4).rc()).rc()).rc()).rc()).rc(),
                ]).rc(),
                št_spr: 11,
            }.rc()
        }
        .v_fasm_x86();

        test(&asm, "", "130<25Q/", false)
    }

    #[test]
    fn bitne_operacije() -> Result<(), io::Error> {
        let asm = vec![
            PUSHI(0b110), PUSHI(0b011), Osnovni(BOR),  Osnovni(PUTC),
            PUSHI(0b110), PUSHI(0b011), Osnovni(BXOR), Osnovni(PUTC),
            PUSHI(0b110), PUSHI(0b011), Osnovni(BAND), Osnovni(PUTC),
            PUSHI(0b011), PUSHI(0b001), Osnovni(BSLL), Osnovni(PUTC),
            PUSHI(0b110), PUSHI(0b001), Osnovni(BSLR), Osnovni(PUTC),
        ]
        .v_fasm_x86();

        test(&asm, "", &String::from_utf8_lossy(&[
                0b111u8,
                0b101u8,
                0b010u8,
                0b110u8,
                0b011u8,
        ]), false)
    }

    #[test]
    fn jump() -> Result<(), io::Error> {
        let asm = vec![
            PUSHC('0'),
            Osnovni(PUTC),
            JUMPRelative("else".to_string()),
            PUSHC('1'),
            Osnovni(PUTC),
            Oznaka("else".to_string()),
            PUSHC('2'),
            Osnovni(PUTC),
        ]
        .v_fasm_x86();

        test(&asm, "", "02", false)
    }

    #[test]
    fn jmpc() -> Result<(), io::Error> {
        let asm = vec![
            PUSHC('0'),
            Osnovni(PUTC),
            PUSHI(1),
            JMPCRelative("else1".to_string()),
            PUSHC('1'),
            Osnovni(PUTC),
            Oznaka("else1".to_string()),
            PUSHC('2'),
            Osnovni(PUTC),
            PUSHI(0),
            JMPCRelative("else2".to_string()),
            PUSHC('3'),
            Osnovni(PUTC),
            JUMPRelative("konec".to_string()),
            Oznaka("else2".to_string()),
            PUSHC('4'),
            Osnovni(PUTC),
            Oznaka("konec".to_string()),
        ]
        .v_fasm_x86();

        test(&asm, "", "023", false)
    }

    #[test]
    fn primerjave() -> Result<(), io::Error> {
        let asm = vec![
            PUSHI(1),
            Osnovni(POS),
            JMPCRelative("konec1".to_string()),
            PUSHC('1'),
            Osnovni(PUTC),
            Oznaka("konec1".to_string()),
            PUSHI(1),
            Osnovni(ZERO),
            JMPCRelative("konec2".to_string()),
            PUSHC('2'),
            Osnovni(PUTC),
            Oznaka("konec2".to_string()),
        ]
        .v_fasm_x86();

        test(&asm, "", "2", false)
    }

    #[test]
    fn load() -> Result<(), io::Error> {
        let asm = vec![
            PUSHC('1'), Osnovni(TOP(0)), PUSHC('2'), PUSHC('3'),

            Osnovni(LOAD(0)), Osnovni(PUTC),
            Osnovni(LOAD(1)), Osnovni(PUTC),
            Osnovni(LDOF(0)), Osnovni(PUTC),
            Osnovni(LDOF(1)), Osnovni(PUTC),

            Osnovni(PUTC), Osnovni(PUTC), Osnovni(PUTC),
        ]
        .v_fasm_x86();

        test(&asm, "", "1223321", false)
    }

    #[test]
    fn stor() -> Result<(), io::Error> {
        let asm = vec![
            PUSHC('1'), Osnovni(TOP(0)), PUSHC('2'), PUSHC('3'),

            Osnovni(LOAD(0)), PUSHI(1), Osnovni(SUBI), Osnovni(STOR(0)),
            Osnovni(LOAD(1)), PUSHI(1), Osnovni(SUBI), Osnovni(STOR(1)),
            Osnovni(LDOF(1)), PUSHI(1), Osnovni(SUBI), Osnovni(STOF(1)),
            Osnovni(LOAD(0)), Osnovni(PUTC),
            Osnovni(LOAD(1)), Osnovni(PUTC),
            Osnovni(LOAD(2)), Osnovni(PUTC),

            Osnovni(ALOC(-3)),
        ]
        .v_fasm_x86();

        test(&asm, "", "012", false)
    }

    #[test]
    fn loff_soff() -> Result<(), io::Error> {
        let asm = vec![
            Osnovni(LOFF), Osnovni(TOP(0)), Osnovni(LOFF),
            Osnovni(LOAD(0)), Osnovni(LOAD(1)), Osnovni(SUBI),
            PUSHC('0'), Osnovni(ADDI), Osnovni(PUTC),
        ]
        .v_fasm_x86();

        test(&asm, "", "8", false)
    }

    #[test]
    fn natisni() -> Result<(), io::Error> {
        let asm = r#"
            natisni(0)
        "#
        .razčleni("[test]")
        .analiziraj()
        .unwrap()
        .v_fasm_x86();

        test(&asm, "", "8", false)
    }

}

