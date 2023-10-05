use std::mem::transmute;

use crate::parser::loci::Escape;

use super::*;

const PRE: &str = "include 'ukazi.asm'\n\n";
const POST: &str = "
    exit 0
";

impl ToFasmX86 for Vec<UkazPodatekRelative> {
    fn v_fasm_x86(&self) -> String {

        let mut opti = self.clone();
        let mut i = 0;

        // optimiziraj

        while i < opti.len() - 2 {
            let sub = &opti[i..];
            //println!("sub: {:?}", &sub[0..3]);
            i = match sub {
                [
                    a @ (PUSHI(..) | PUSHC(..)),
                    b @ (PUSHI(..) | PUSHC(..)),
                    Osnovni(op @ (ADDI | SUBI | MULI | DIVF)),
                    ..
                ] => {
                    let op = match op {
                        ADDI => i32::wrapping_add,
                        SUBI => i32::wrapping_sub,
                        MULI => i32::wrapping_mul,
                        DIVF => i32::wrapping_div,
                        _ => unreachable!()
                    };
                    opti[i] = match (a, b) {
                        (PUSHI(a), PUSHI(b)) => PUSHI(op(*a, *b)),
                        (PUSHI(a), PUSHC(b)) => PUSHC(unsafe {transmute::<i32, char>(op(*a, v_utf8(*b))) }),
                        (PUSHC(a), PUSHI(b)) => PUSHC(unsafe {transmute::<i32, char>(op(v_utf8(*a), *b)) }),
                        (PUSHC(a), PUSHC(b)) => PUSHC(unsafe {transmute::<i32, char>(op(v_utf8(*a), v_utf8(*b))) }),
                        _ => unreachable!(),
                    };
                    opti.remove(i + 1); opti.remove(i + 1);
                    i
                },

                [PUSHF(a), PUSHF(b), Osnovni(op @ (ADDF | SUBF | MULF | DIVF))] => {
                    let result = match op {
                        ADDF => a + b,
                        SUBF => a - b,
                        MULF => a * b,
                        DIVF => a / b,
                        _ => unreachable!(),
                    };
                    PUSHOPT(unsafe { std::mem::transmute::<f32, i32>(result) });
                    opti.remove(i + 1); opti.remove(i + 1);
                    i
                }

                [
                    Osnovni(ld1 @ (LOAD(src) | LDOF(src))),
                    Osnovni(ld2 @ (LOAD(dst) | LDOF(dst))),
                    Osnovni(op @ (ADDI | SUBI | MULI)),
                    ..
                ] => {
                    let op = match op {
                        ADDI => "add", SUBI => "sub",
                        MULI => "imul", DIVI => "idiv",
                        _ => unreachable!(),
                    };
                    let off1 = match ld1 {
                        LOAD(..) => "r8",
                        LDOF(..) => "r9",
                        _ => unreachable!(),
                    };
                    let off2 = match ld2 {
                        LOAD(..) => "r8",
                        LDOF(..) => "r9",
                        _ => unreachable!(),
                    };

                    opti[i] = LDOP(op, off1, off2, *src, *dst);
                    opti.remove(i + 1); opti.remove(i + 1);
                    i
                },

                [
                    push @ (PUSHI(..) | PUSHC(..) | PUSHF(..)),
                    Osnovni(stor @ (STOR(dst) | STOF(dst))),
                    ..
                ] => {
                    let reg = match stor {
                        STOR(..) => "r8",
                        STOF(..) => "r9",
                        _ => unreachable!(),
                    };
                    let data = match push {
                        PUSHI(data) => *data,
                        PUSHC(data) => v_utf8(*data),
                        PUSHF(data) => unsafe { transmute::<f32, i32>(*data) },
                        _ => unreachable!(),
                    };
                    opti[i] = STIMM(data, *dst, reg);
                    opti.remove(i + 1);
                    i - 1
                },

                [
                    Osnovni(load @ (LOAD(src) | LDOF(src))),
                    Osnovni(stor @ (STOR(dst) | STOF(dst))),
                    ..
                ] => {
                    let reg1 = match load {
                        LOAD(..) => "r8",
                        LDOF(..) => "r9",
                        _ => unreachable!(),
                    };
                    let reg2 = match stor {
                        STOR(..) => "r8",
                        STOF(..) => "r9",
                        _ => unreachable!(),
                    };

                    opti[i] = LDST(reg1, reg2, *src, *dst);
                    opti.remove(i + 1);
                    i
                },

                [
                    STIMM(..),
                    Osnovni(stor @ (STOR(dst) | STOF(dst))),
                    ..
                ] => {
                    // potuj nazaj do PUSHa
                    let mut j = i;

                    let data = loop {
                        match &opti[j] {
                            PUSHI(data) => break Some(*data),
                            PUSHC(data) => break Some(v_utf8(*data)),
                            PUSHF(data) => break Some(unsafe { transmute::<f32, i32>(*data) }),
                            STIMM(..) => j -= 1,
                            _ => break None,
                        };
                        if i == 0 { break None }
                    };

                    match data {
                        Some(data) => {
                            let reg = match stor {
                                STOR(..) => "r8",
                                STOF(..) => "r9",
                                _ => unreachable!(),
                            };

                            let dst = *dst;
                            opti.remove(j);
                            opti.remove(i);
                            opti.insert(i, STIMM(data, dst, reg));
                            i
                        },
                        None => i + 1
                    }
                },

                _ => i + 1,
            }
        }

        opti.iter()
            .fold(PRE.to_string(), |str, ukaz_podatek| {
                str
                + if let Oznaka(_) = ukaz_podatek { "" } else { "\t" }
                + &match ukaz_podatek {
                    PUSHI(število)  => format!("PUSH 0x{število:X} ; {število:?}: celo"),
                    PUSHF(število)  => format!("PUSH 0x{:X} ; {število:?}: real", unsafe { std::mem::transmute::<f32, u32>(*število) }),
                    PUSHC(znak)     => format!("PUSH 0x{:X} ; '{}': char", v_utf8(*znak), znak.to_string().escape()),
                    JUMPRel(oznaka) => format!("JUMP {}", formatiraj_oznako(oznaka)),
                    JMPCRel(oznaka) => format!("JMPC {}", formatiraj_oznako(oznaka)),
                    CALL(oznaka)    => format!("CALL {}", formatiraj_oznako(oznaka)),
                    Oznaka(oznaka)  => format!("{}:",     formatiraj_oznako(oznaka)),
                    PC(i)           => format!("PC {i}"),

                    PUSHOPT(data) => format!("PUSH_OPT 0x{data:X}"),
                    STIMM(data, addr, reg) => format!("ST_IMM 0x{data:X}, {addr}, {reg}"),
                    LDOP(op, ld1, ld2, addr1, addr2) => format!("LD_OP {op}, {ld1}, {ld2}, {addr1}, {addr2}"),
                    LDST(off1, off2, src, dst) => format!("LD_ST {off1}, {off2}, {src}, {dst}"),

                    Osnovni(ALOC(mem))  => format!("ALOC {mem}"),
                    Osnovni(LOAD(addr)) => format!("LOAD 0x{addr:0X}"), // load normal
                    Osnovni(LDOF(addr)) => format!("LDOF 0x{addr:0X}"), // load w/ offset
                    Osnovni(LDDY(offs)) => format!("LDDY {offs}"), // load dynamic
                    Osnovni(STOR(addr)) => format!("STOR 0x{addr:0X}"), // store normal
                    Osnovni(STOF(addr)) => format!("STOF 0x{addr:0X}"), // store w/ offset
                    Osnovni(STDY(offs)) => format!("STDY {offs}"), // store dynamic
                    Osnovni(TOP(offs))  => format!("TOP  {offs}"),
                    Osnovni(instruction) => format!("{instruction:?}"),
                }
                + "\n"
            })
        + POST
    }
}

fn v_utf8(znak: char) -> i32 {
    let mut buf = [0u8; 4];
    znak.encode_utf8(&mut buf);
    let n = buf.iter().rev()
        .fold(0, |acc, b| acc << 8 | *b as u32);
    unsafe { transmute::<u32, i32>(n) }
}

fn formatiraj_oznako(oznaka: &str) -> String {
    format!(".{}", oznaka
        .replace("(", "8")
        .replace(")", "9")
        .replace("[", "F")
        .replace("]", "G")
        .replace("@", "V")
        .replace(", ", "__"))
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
            println!("{program_filename}.asm");
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "compilation failed"));
        }

        // run compiled binary
        let mut proces = Command::new(&program_filename)
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
            println!("{program_filename}.asm");
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "program failed"));
        }

        if bytes {
            assert_eq!(output.stdout, expected.as_bytes(), "{program_filename}");
        }
        else {
            let output = String::from_utf8_lossy(&output.stdout);
            assert_eq!(output, expected, "{program_filename}");
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
                    Natisni(CeloVZnak(Add(Tip::Celo, Celo(48).rc(), Celo(1).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Add(Tip::Celo, Celo(48).rc(), Celo(3).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Sub(Tip::Celo, Celo(58).rc(), Celo(10).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Mul(Tip::Celo, Celo(15).rc(), Celo(4).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Mul(Tip::Celo, Celo(-62).rc(), Celo(-1).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Div(Tip::Celo, Celo(100).rc(), Celo(2).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Mod(Tip::Celo, Celo(553).rc(), Celo(100).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Pow(Tip::Celo, Celo(3).rc(), Celo(4).rc()).rc()).rc()).rc(),
                    Natisni(CeloVZnak(Sub(Tip::Celo, Celo(128).rc(), Pow(Tip::Celo, Celo(-3).rc(), Celo(4).rc()).rc()).rc()).rc()).rc(),
                ]).rc(),
                št_spr: 11,
            }.rc()
        }
        .v_fasm_x86();

        test(&asm, "", "130<>25Q/", false)
    }

    #[test]
    fn itof_ftoi() -> Result<(), io::Error> {
        let asm = Drevo {
            funkcije: vec![],
            št_klicev: HashMap::new(),
            main: Zaporedje(vec![
                Natisni(RealVCelo(CeloVReal(ZnakVCelo(Znak('0').rc()).rc()).rc()).rc()).rc(),
                Natisni(RealVCelo(CeloVReal(ZnakVCelo(Znak('1').rc()).rc()).rc()).rc()).rc(),
                Natisni(RealVCelo(CeloVReal(ZnakVCelo(Znak('2').rc()).rc()).rc()).rc()).rc(),
                Natisni(RealVCelo(CeloVReal(ZnakVCelo(Znak('3').rc()).rc()).rc()).rc()).rc(),
            ]).rc(),
        }
        .v_fasm_x86();

        test(&asm, "", "0123", false)
    }

    #[test]
    fn realne_operacije() -> Result<(), io::Error> {
        let asm = Drevo {
            funkcije: vec![],
            št_klicev: HashMap::new(),
            main: Zaporedje(vec![
                Natisni(CeloVZnak(RealVCelo(Add(Tip::Real, Real( 48.0).rc(), Real(  1.0).rc()).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(RealVCelo(Add(Tip::Real, Real( 48.0).rc(), Real(  3.0).rc()).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(RealVCelo(Sub(Tip::Real, Real( 58.0).rc(), Real( 10.0).rc()).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(RealVCelo(Mul(Tip::Real, Real( 15.0).rc(), Real(  4.0).rc()).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(RealVCelo(Mul(Tip::Real, Real(-62.0).rc(), Real( -1.0).rc()).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(RealVCelo(Div(Tip::Real, Real(100.0).rc(), Real(  2.0).rc()).rc()).rc()).rc()).rc(),
                Natisni(CeloVZnak(RealVCelo(Mod(Tip::Real, Real(553.0).rc(), Real(100.0).rc()).rc()).rc()).rc()).rc(),
                //Natisni(CeloVZnak(RealVCelo(Pow(Tip::Real, Real(3.0).rc(),   Real(  4.0).rc()).rc()).rc()).rc()).rc(),
                //Natisni(CeloVZnak(RealVCelo(Sub(Tip::Real, Real(128.0).rc(), Pow(Tip::Real, Real(-3.0).rc(), Real(4.0).rc()).rc()).rc()).rc()).rc()).rc(),
            ]).rc(),
        }
        .v_fasm_x86();

        test(&asm, "", "130<>25", false)
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
            JUMPRel("else".to_string()),
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
            JMPCRel("else1".to_string()),
            PUSHC('1'),
            Osnovni(PUTC),
            Oznaka("else1".to_string()),
            PUSHC('2'),
            Osnovni(PUTC),
            PUSHI(0),
            JMPCRel("else2".to_string()),
            PUSHC('3'),
            Osnovni(PUTC),
            JUMPRel("konec".to_string()),
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
            JMPCRel("konec1".to_string()),
            PUSHC('1'),
            Osnovni(PUTC),
            Oznaka("konec1".to_string()),
            PUSHI(1),
            Osnovni(ZERO),
            JMPCRel("konec2".to_string()),
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
            PUSHI(1), Osnovni(LDDY(0)), Osnovni(PUTC),
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
            Osnovni(LOAD(1)), PUSHI(1), Osnovni(SUBI), PUSHI(1), Osnovni(STDY(0)),
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
            natisni('a')
            natisni("bcd")
            natisni!("efg", 2 + 3)
        "#
        .razčleni("[test]")
        .analiziraj()
        .unwrap()
        .v_fasm_x86();

        test(&asm, "", "abcdefg5", false)
    }

}

