use core::slice;
use std::{mem::size_of, fmt::Debug};
use std::fmt;

#[derive(Clone, Copy)]
pub union Podatek {
    i: i32,
    f: f32,
    c: char,
}

#[derive(Debug)]
pub enum UkazPodatek
{
    NOOP,
    JUMP(u32),
    JMPD,
    JMPC(u32),
    PUSH(Podatek),
    POP,
    POS,
    ZERO,
    LOAD(u32),
    LDOF(u32),
    STOR(u32),
    STOF(u32),
    TOP(i32),
    SOFF,
    LOFF,
    PRTN,
    PRTC,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    POW,
}

const IMENA: [&str; 22] = [
    "JUMP",
    "JMPD",
    "JMPC",
    "PUSH",
    "POP",
    "POS",
    "ZERO",
    "LOAD",
    "LDOF",
    "TOP ",
    "SOFF",
    "LOFF",
    "STOR",
    "STOF",
    "PRTN",
    "PRTC",
    "ADD",
    "SUB",
    "MUL",
    "DIV",
    "MOD",
    "POW",
];

impl Debug for Podatek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.i })
    }
}

pub struct Program {
    ukazi: Vec<UkazPodatek>,
}

const RESNICA: f32 = 1.0;
const LAŽ: f32     = 0.0;

impl Program {

    pub fn from(assembler: String) -> Self {
        use UkazPodatek::*;

        let mut ukazi: Vec<UkazPodatek> = Vec::new();
        let vrstice = assembler.split('\n');

        for vrstica in vrstice {
            if vrstica.len() == 0 { continue; }
            let besede: Vec<&str> = vrstica.split_whitespace().collect();

            ukazi.push(match besede[0] {
                "PUSH" => {
                    if besede[1].chars().nth(0).unwrap() == '#' {
                        if besede[1].contains('.') {
                            PUSH(Podatek { f:  besede[1][1..].parse().unwrap() })
                        }
                        else {
                            PUSH(Podatek { i:  besede[1][1..].parse().unwrap() })
                        }
                    }
                    else {
                        PUSH(Podatek { c: besede[1][1..besede[1].len()-1]
                            .replace("\\\\", "\\")
                                .replace("\\n", "\n")
                                .replace("\\t", "\t")
                                .replace("\\r", "\r")
                                .replace("\\\"", "\"")
                                .replace("\\\'", "\'")
                                .chars()
                                .next()
                                .unwrap() })
                    }
                },
                "JUMP" => JUMP(besede[1][1..].parse().unwrap()),
                "JMPC" => JMPC(besede[1][1..].parse().unwrap()),
                "LOAD" => LOAD(besede[1][1..].parse().unwrap()),
                "LDOF" => LDOF(besede[1][1..].parse().unwrap()),
                "STOR" => STOR(besede[1][1..].parse().unwrap()),
                "STOF" => STOF(besede[1][1..].parse().unwrap()),
                "TOP"  => TOP(besede[1][1..].parse().unwrap()),
                "JMPD" => JMPD,
                "POP"  => POP,
                "POS"  => POS,
                "ZERO" => ZERO,
                "LOFF" => LOFF,
                "SOFF" => SOFF,
                "PRTN" => PRTN,
                "PRTC" => PRTC,
                "ADD"  => ADD,
                "SUB"  => SUB,
                "MUL"  => MUL,
                "DIV"  => DIV,
                "MOD"  => MOD,
                "POW"  => POW,
                _      => NOOP,
            });
        }

        Program { ukazi }
    }

    pub fn run(&self) {
        use UkazPodatek::*;

        let mut pc: u32 = 0;
        let mut addroff: u32 = 0;

        let mut stack: Vec<Podatek> = Vec::new();
        stack.reserve(512);

        while (pc as usize) < self.ukazi.len() {
            let ukaz_podatek = &self.ukazi[pc as usize];

            pc = unsafe {
                match ukaz_podatek {
                    NOOP => pc + 1,
                    JUMP(naslov) => naslov.clone(),
                    JMPD => stack.pop().unwrap().i as u32,
                    JMPC(naslov) => if stack.pop().unwrap().f != LAŽ { naslov.clone() } else { pc + 1 },
                    PUSH(podatek) => { stack.push(*podatek); pc + 1 },
                    POP => { stack.pop(); pc + 1 },
                    POS => { stack.last_mut().unwrap().f = if stack.last().unwrap().f > 0.0 { RESNICA } else { LAŽ }; pc + 1 },
                    ZERO => { stack.last_mut().unwrap().f = if stack.last().unwrap().f == 0.0 { RESNICA } else { LAŽ }; pc + 1 },
                    LOAD(podatek) => { stack.push(stack[podatek.clone() as usize]); pc + 1 },
                    LDOF(podatek) => { stack.push(stack[addroff as usize + podatek.clone() as usize]); pc + 1 },
                    STOR(podatek) => { stack[podatek.clone() as usize] = stack.pop().unwrap(); pc + 1 },
                    STOF(podatek) => { stack[addroff as usize + podatek.clone() as usize] = stack.pop().unwrap(); pc + 1 },
                    TOP(podatek)  => { addroff = (stack.len() as i32 + podatek) as u32; pc + 1 },
                    SOFF => { addroff = stack.pop().unwrap().i as u32; pc + 1 },
                    LOFF => { stack.push(Podatek { i: addroff as i32 }); pc + 1 },
                    PRTN => { print!("{}", stack.pop().unwrap().f); pc + 1 },
                    PRTC => { print!("{}", stack.pop().unwrap().c); pc + 1 },
                    ADD  => { stack.last_mut().unwrap().f = stack.pop().unwrap().f + stack.pop().unwrap().f; pc + 1 },
                    SUB  => { stack.last_mut().unwrap().f = stack.pop().unwrap().f - stack.pop().unwrap().f; pc + 1 },
                    MUL  => { stack.last_mut().unwrap().f = stack.pop().unwrap().f * stack.pop().unwrap().f; pc + 1 },
                    DIV  => { stack.last_mut().unwrap().f = stack.pop().unwrap().f / stack.pop().unwrap().f; pc + 1 },
                    MOD  => { stack.last_mut().unwrap().f = stack.pop().unwrap().f % stack.pop().unwrap().f; pc + 1 },
                    POW  => { stack.last_mut().unwrap().f = stack.pop().unwrap().f.powf(stack.pop().unwrap().f); pc + 1 },
                }
            };
        }
    }

    pub unsafe fn to_bytes(&self) -> &[UkazPodatek]  {
        slice::from_raw_parts(self.ukazi.as_ptr(), self.ukazi.len() * size_of::<UkazPodatek>())
    }

}

pub trait ToProgram {
    fn to_program(&self) -> Program;
}

impl ToProgram for String {
    fn to_program(&self) -> Program {
        Program::from(self.clone())
    }
}
