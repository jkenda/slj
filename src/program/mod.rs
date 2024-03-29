mod prevedi;
mod postprocesiraj;
mod from_assembler;
mod v_assembler;
mod v_fasm_x86;
mod zazeni;

use std::collections::HashMap;
use std::{mem::size_of, fmt::Debug};
use std::{fmt, io};

use crate::parser::{drevo::Drevo, tip::Tip};
use crate::parser::drevo::Vozlišče::{*, self};
use self::{UkazPodatek::*, UkazPodatekRelative::*};

pub trait ToProgram {
    fn v_program(self) -> Program;
}

trait Prevedi {
    fn prevedi(self) -> Vec<UkazPodatekRelative>;
    fn len(&self) -> usize;
}

trait Postprocesiraj {
    fn vrni_v_oznake(self) -> Vec<UkazPodatekRelative>;
    fn postprocesiraj(self) -> (Vec<UkazPodatek>, Vec<Tip>);
}

pub trait ToFasmX86 {
    fn v_fasm_x86(self, opti: u32) -> String;
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
    JMPC(i32),
    JMPD,
    ALOC(i32),
    PUSH(Podatek),
    LOAD(i32), // load normal
    LDOF(i32), // load w/ offset
    LDDY(i32), // load dynamic
    STOR(i32), // store normal
    STOF(i32), // store w/ offset
    STDY(i32), // store dynamic
    TOP(i32),
    POS,
    ZERO,
    SOFF,
    LOFF,
    PUTC,
    GETC,
    FLUSH,
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
    BSLR,
    FTOI,
    ITOF,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum ArO {
    Add,
    Sub,
    IMul,
    Or,
    Xor,
    And,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum R {
    Rax, Eax, Al,
    Rbx, Ebx, Bl,
    Rcx, Ecx, Cl,
    Rdx,
    Rdi,
    Rsi,
    R8,
    R9,
    Rsp,
}

#[derive(Debug, Clone, PartialEq)]
enum UkazPodatekRelative {
    Osnovni(UkazPodatek),
    PUSHI(i32),
    PUSHF(f32),
    PUSHC(char),
    JUMPRel(String),
    JMPCRel(String),
    CALL(String),
    PC(i32),
    Oznaka(String),

    STIMM(u32, i32, R),
    LDOP(ArO, R, R, i32, i32),
    LDST(R, R, i32, i32),
    PUSHREF(i32, bool),
    LDINDEXED,
    STINDEXED,
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
    fn v_program(self) -> Program {
        let (ukazi, push_tipi) = self
            .prevedi()
            .vrni_v_oznake()
            .postprocesiraj();

        Program { 
            push_tipi,
            ukazi,
        }
    }
}

impl ToFasmX86 for Drevo {
    fn v_fasm_x86(self, opti: u32) -> String {
        self
            .prevedi()
            .vrni_v_oznake()
            .v_fasm_x86(opti)
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
