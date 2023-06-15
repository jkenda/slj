mod prevedi;
mod postprocesiraj;
mod from_assembler;
mod v_assembler;
mod v_cpp;
mod zazeni;

use std::collections::HashMap;
use std::{mem::size_of, fmt::Debug};
use std::{fmt, io};

use crate::parser::{drevo::Drevo, tip::Tip};
use crate::parser::drevo::{OdmikIme, Vozlišče::{*, self}};
use self::{UkazPodatek::*, UkazPodatekRelative::*};

pub trait ToProgram {
    fn v_program(&self) -> Program;
    fn v_cpp(&self) -> String;
}

trait Prevedi {
    fn prevedi(&self) -> Vec<UkazPodatekRelative>;
    fn len(&self) -> usize;
}

trait Postprocesiraj {
    fn postprocesiraj(&self) -> (Vec<UkazPodatek>, Vec<Tip>);
}

#[derive(Clone, Copy)]
pub union Podatek {
    i: i32,
    f: f32,
    c: char,
}

impl PartialEq for Podatek {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.i == other.i }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum UkazPodatek
{
    NOOP,
    JUMP(i32),
    JMPD,
    JMPC(i32),
    PUSH(Podatek),
    ALOC(i32),
    POS,
    ZERO,
    LOAD(i32), // load normal
    LDOF(i32), // load w/ offset
    LDDY(i32), // load dynamic
    STOR(i32), // store normal
    STOF(i32), // store w/ offset
    STDY(i32), // store dynamic
    TOP(i32),
    SOFF,
    LOFF,
    PUTC,
    GETC,
    ADDF,
    SUBF,
    MULF,
    DIVF,
    MODF,
    POWF,
    ADDI,
    SUBI,
    MULI,
    DIVI,
    MODI,
    POWI,
    BOR,
    BXOR,
    BAND,
    BSLL,
    BSLD,
    FTOI,
    ITOF,
}

#[derive(Debug, Clone, PartialEq)]
enum UkazPodatekRelative {
    Osnovni(UkazPodatek),
    PUSHI(i32),
    PUSHF(f32),
    PUSHC(char),
    JUMPRelative(OdmikIme),
    JMPCRelative(i32),
    PC(i32),
    Oznaka(String)
}

#[derive(Debug, PartialEq)]
pub struct Program {
    push_tipi: Vec<Tip>,
    ukazi: Vec<UkazPodatek>,
}


impl Debug for Podatek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.i })
    }
}

impl ToProgram for Drevo {
    fn v_program(&self) -> Program {
        let (ukazi, push_tipi) = self
            .prevedi()
            .postprocesiraj();

        Program { 
            push_tipi,
            ukazi,
        }
    }

    fn v_cpp(&self) -> String {
        dbg!(self.št_klicev.clone());

        "#include <cwchar>\n".to_string()
        + "#include <vector>\n\n"
        + "union Podatek\n{\n\tint i;\n\tfloat f;\n\twchar_t c;\n};\n\n"
        + "static const Podatek LAŽ = { .i = 0 };\n"
        + "static const Podatek RESNICA = { .i = 1 };\n\n"
        + "int addroff = 0;\n"
        + "int dynaddr = 0;\n"
        + "std::vector<Podatek> stack;\n\n"
        + &self.funkcije.iter().fold(String::new(), |str, funkcija| {
            str + &funkcija.v_cpp_funkcija(&self.št_klicev)
        })
        + "int main()\n{\n"
        + &self.main.v_cpp_main(&self.št_klicev)
            .lines()
            .fold(String::new(), |str, l| str + if !l.ends_with(':') { "\t" } else { "" } + l + "\n")
        + "}\n"
    }
}


const RESNICA: Podatek = Podatek { i: 1 };
const LAŽ    : Podatek = Podatek { i: 0 };
const NIČ    : Podatek = Podatek { i: 0 };


impl Program {
    pub unsafe fn to_bytes(&self) -> (*const u8, usize)  {
        (self.ukazi.as_ptr().cast::<u8>(), size_of::<UkazPodatek>() * self.ukazi.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_assembler_from_assembler() {
        let program = Program {
            push_tipi: vec![
                Tip::Real,
                Tip::Real,
                Tip::Celo,
                Tip::Znak,
                Tip::Znak,
            ],
            ukazi: [
                NOOP,
                JUMP(23),
                JMPD,
                JMPC(18),
                PUSH(Podatek { f: 3.14159268 }),
                PUSH(Podatek { f: 0.0 }),
                PUSH(Podatek { i: 42 }),
                PUSH(Podatek { c: 'c' }),
                PUSH(Podatek { c: '\n' }),
                ALOC(-12),
                POS,
                ZERO,
                LOAD(13),
                LDOF(42),
                STOR(256),
                STOF(200),
                TOP(13),
                TOP(-13),
                SOFF,
                LOFF,
                PUTC,
                GETC,
                ADDF,
                SUBF,
                MULF,
                DIVF,
                MODF,
                POWF,
                ADDI,
                SUBI,
                MULI,
                DIVI,
                MODI,
                POWI,
                FTOI,
                ITOF,
                BOR,
                BXOR,
                BAND,
            ].to_vec(),
        };

        assert_eq!(program, Program::from(program.v_assembler()));
    }
}
