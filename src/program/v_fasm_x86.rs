use super::*;

const PRE: &str = r#"
include 'ukazi.asm'

entry $
	mov [stack_0], rsp
	mov r8, rsp

"#;
const POST: &str = r#"
	exit 0
"#;

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
                    Oznaka(oznaka)       => format!("{}:",     formatiraj_oznako(oznaka)),
                    PC(i)                => format!("PC {i}"),

                    Osnovni(ALOC(mem))  => format!("ALOC {mem}"),
                    Osnovni(LOAD(addr)) => format!("LOAD {addr}"), // load normal
                    Osnovni(LDOF(addr)) => format!("LOAD {addr}"), // load w/ offset
                    Osnovni(LDDY(addr)) => format!("LOAD {addr}"), // load dynamic
                    Osnovni(STOR(addr)) => format!("LOAD {addr}"), // store normal
                    Osnovni(STOF(addr)) => format!("LOAD {addr}"), // store w/ offset
                    Osnovni(STDY(addr)) => format!("LOAD {addr}"), // store dynamic
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
    use crate::parser::drevo::{Drevo, Vozlišče};
    use crate::parser::tip::Tip;
    use Vozlišče::*;

    fn test(drevo: Drevo, input: &str, expected: &str, bytes: bool) -> Result<(), io::Error> {
        // transform AST into native x86_64 assembly
        let fasm = drevo
            .v_fasm_x86();

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
        let drevo = Drevo {
            funkcije: vec![],
            št_klicev: HashMap::new(),
            main: Zaporedje(vec![
                Natisni(Znak('a').rc()).rc(),
                Natisni(Znak('ž').rc()).rc(),
                Natisni(Znak('😭').rc()).rc(),
                Natisni(Znak('\n').rc()).rc(),
            ]).rc()
        };

        test(drevo, "", "až😭\n", true)
    }

    #[test]
    fn getc() -> Result<(), io::Error> {
        let drevo = Drevo {
            funkcije: vec![],
            št_klicev: HashMap::new(),
            main: Zaporedje(vec![
                Natisni(Preberi.rc()).rc(),
                Natisni(Preberi.rc()).rc(),
                Natisni(Preberi.rc()).rc(),
                Natisni(Preberi.rc()).rc(),
            ]).rc()
        };

        test(drevo, "asdf", "asdf", true)
        //test(drevo, "až😭\n", "až😭\n", true)
    }

    #[test]
    fn plus_minus_krat_deljeno_mod() -> Result<(), io::Error> {
        let drevo = Drevo {
            funkcije: vec![],
            št_klicev: HashMap::new(),
            main: Okvir {
                zaporedje: Zaporedje(vec![
                    Natisni(CeloVZnak(Seštevanje(Tip::Celo, Celo(48).rc(), Celo(1).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Seštevanje(Tip::Celo, Celo(48).rc(), Celo(3).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Odštevanje(Tip::Celo, Celo(58).rc(), Celo(10).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Množenje(Tip::Celo, Celo(15).rc(), Celo(4).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Deljenje(Tip::Celo, Celo(100).rc(), Celo(2).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Modulo(Tip::Celo, Celo(553).rc(), Celo(100).rc()).rc()).rc()).rc(),
                    Natisni(Znak('\n').rc()).rc(),
                ]).rc(),
                št_spr: 11,
            }.rc()
        };

        test(drevo, "", "130<25\n", false)
    }

}

