mod prevedi;
mod postprocesiraj;

use std::collections::HashMap;
use std::{mem::size_of, fmt::Debug};
use std::{fmt, io};

use crate::parser::drevo::Drevo;
use crate::parser::drevo::{OdmikIme, Vozlišče::{*, self}};
use self::{UkazPodatek::*, UkazPodatekRelative::*};

pub trait ToProgram {
    fn to_program(&self) -> Program;
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
enum Tip {
    CELO,
    REAL,
    ZNAK,
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
    fn to_program(&self) -> Program {
        Program::from(self)
    }
}


const RESNICA: Podatek = Podatek { i: 1 };
const LAŽ    : Podatek = Podatek { i: 0 };


impl From<&Drevo> for Program {
    fn from(drevo: &Drevo) -> Self {
        let (ukazi, push_tipi) = drevo.prevedi().postprocesiraj();

        Program { 
            push_tipi,
            ukazi,
        }
    }
}

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
                            push_tipi.push(Tip::REAL);
                            PUSH(Podatek { f:  besede[2][1..].parse().unwrap() })
                        }
                        else {
                            push_tipi.push(Tip::CELO);
                            PUSH(Podatek { i:  besede[2][1..].parse().unwrap() })
                        }
                    }
                    else {
                        push_tipi.push(Tip::ZNAK);
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
                "JUMP" => JUMP(besede[2][1..].parse().unwrap()),
                "JMPC" => JMPC(besede[2][1..].parse().unwrap()),
                "LOAD" => LOAD(besede[2][1..].parse().unwrap()),
                "LDOF" => LDOF(besede[2][1..].parse().unwrap()),
                "STOR" => STOR(besede[2][1..].parse().unwrap()),
                "STOF" => STOF(besede[2][1..].parse().unwrap()),
                "TOP"  => TOP(besede[2][0..].parse().unwrap()),
                "JMPD" => JMPD,
                "POP"  => POP,
                "POS"  => POS,
                "ZERO" => ZERO,
                "LOFF" => LOFF,
                "SOFF" => SOFF,
                "PRTN" => PRTN,
                "PRTC" => PRTC,
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

impl Program {
    pub fn zaženi(&self) {
        let mut pc: u32 = 0;
        let mut addroff: u32 = 0;
        let mut stack: Vec<Podatek> = Vec::new();
        stack.reserve(512);

        let mut stdout = Box::new(io::stdout());

        while (pc as usize) < self.ukazi.len() {
            Program::korak(&self.ukazi[pc as usize], &mut stack, &mut pc, &mut addroff, &mut stdout);
        }
    }

    pub fn zaženi_debug(&self) {
        let mut pc: u32 = 0;
        let mut addroff: u32 = 0;
        let mut stack: Vec<Podatek> = Vec::new();
        stack.reserve(512);

        let mut stdout = io::stdout();

        while (pc as usize) < self.ukazi.len() {
            let ukaz = &self.ukazi[pc as usize];

            print!("{addroff}, {pc}, {ukaz:?}: ");
            match Program::korak_debug(ukaz, &mut stack, &mut pc, &mut addroff, &mut stdout) {
                Some(_) => (),
                None => panic!("Napaka v ukazu #{pc}: {:?}", ukaz),
            }
            println!("{stack:?}");
        }
    }

    pub fn zaženi_preusmeri_izhod(&self, izhod: &mut impl io::Write) {
        let mut pc: u32 = 0;
        let mut addroff: u32 = 0;
        let mut stack: Vec<Podatek> = Vec::new();
        stack.reserve(512);

        while (pc as usize) < self.ukazi.len() {
            Program::korak(&self.ukazi[pc as usize], &mut stack, &mut pc, &mut addroff, izhod);
        }
    }

    pub fn to_assembler(&self) -> String {
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
                        Tip::REAL => format!("PUSH #{:?}\n", unsafe { podatek.f }),
                        Tip::CELO => format!("PUSH #{}\n",   unsafe { podatek.i }),
                        Tip::ZNAK => format!("PUSH '{}'\n",  unsafe { podatek.c
                            .to_string()
                            .replace("\\", r"\\")
                                .replace("\n", r"\n")
                                .replace("\t", r"\t")
                                .replace("\r", r"\r")
                                .replace("\"", r#"\""#)
                                .replace("\'", r"\'")
                        }),
                    }
                },
                POP           => "POP \n".to_string(),
                POS           => "POS \n".to_string(),
                ZERO          => "ZERO\n".to_string(),
                LOAD(naslov)  => format!("LOAD @{naslov}\n"),
                LDOF(naslov)  => format!("LDOF +{naslov}\n"),
                STOR(naslov)  => format!("STOR @{naslov}\n"),
                STOF(naslov)  => format!("STOF +{naslov}\n"),
                TOP(odmik)    => format!("TOP  {}{odmik}\n", if *odmik > 0 { "+" } else { "" }),
                SOFF          => "SOFF\n".to_string(),
                LOFF          => "LOFF\n".to_string(),
                PRTN          => "PRTN\n".to_string(),
                PRTC          => "PRTC\n".to_string(),
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
                FTOI          => "FTOI\n".to_string(),
                ITOF          => "ITOF\n".to_string(),
            }
        }

        str
    }

    pub unsafe fn to_bytes(&self) -> (*const u8, usize)  {
        (self.ukazi.as_ptr().cast::<u8>(), size_of::<UkazPodatek>() * self.ukazi.len())
    }

    fn korak(ukaz_podatek: &UkazPodatek, stack: &mut Vec<Podatek>, pc: &mut u32, addroff: &mut u32, izhod: &mut impl io::Write) {
        *pc = unsafe {
            match ukaz_podatek {
                NOOP => *pc + 1,

                JUMP(naslov) => naslov.clone(),
                JMPD => stack.pop().unwrap().i as u32,
                JMPC(naslov) => if stack.pop().unwrap() != LAŽ { naslov.clone() } else { *pc + 1 },

                PUSH(podatek) => { stack.push(*podatek); *pc + 1 },
                POP => { stack.pop(); *pc + 1 },

                POS  => { *stack.last_mut().unwrap() = if stack.last().unwrap().f  > 0.0 { RESNICA } else { LAŽ }; *pc + 1 },
                ZERO => { *stack.last_mut().unwrap() = if stack.last().unwrap().f == 0.0 { RESNICA } else { LAŽ }; *pc + 1 },

                LOAD(naslov) => { stack.push(*stack.get(naslov.clone() as usize).unwrap()); *pc + 1 },
                LDOF(naslov) => { stack.push(*stack.get(*addroff as usize + naslov.clone() as usize).unwrap()); *pc + 1 },
                STOR(naslov) => { *stack.get_mut(naslov.clone() as usize).unwrap() = stack.pop().unwrap(); *pc + 1 },
                STOF(naslov) => { *stack.get_mut(*addroff as usize + naslov.clone() as usize).unwrap() = stack.pop().unwrap(); *pc + 1 },
                TOP (naslov) => { *addroff = (stack.len() as i32 + naslov) as u32; *pc + 1 },

                SOFF => { *addroff = stack.pop().unwrap().i as u32;   *pc + 1 },
                LOFF => { stack.push(Podatek { i: *addroff as i32 }); *pc + 1 },

                PRTN => { write!(izhod, "{}", stack.pop().unwrap().f).unwrap(); *pc + 1 },
                PRTC => { write!(izhod, "{}", stack.pop().unwrap().c).unwrap(); *pc + 1 },

                ADDF => { stack.last_mut().unwrap().f = stack.get(stack.len() - 2).unwrap().f    + stack.pop().unwrap().f;  *pc + 1 },
                SUBF => { stack.last_mut().unwrap().f = stack.get(stack.len() - 2).unwrap().f    - stack.pop().unwrap().f;  *pc + 1 },
                MULF => { stack.last_mut().unwrap().f = stack.get(stack.len() - 2).unwrap().f    * stack.pop().unwrap().f;  *pc + 1 },
                DIVF => { stack.last_mut().unwrap().f = stack.get(stack.len() - 2).unwrap().f    / stack.pop().unwrap().f;  *pc + 1 },
                MODF => { stack.last_mut().unwrap().f = stack.get(stack.len() - 2).unwrap().f    % stack.pop().unwrap().f;  *pc + 1 },
                POWF => { stack.last_mut().unwrap().f = stack.get(stack.len() - 2).unwrap().f.powf(stack.pop().unwrap().f); *pc + 1 },

                ADDI => { stack.last_mut().unwrap().i = stack.get(stack.len() - 2).unwrap().i   + stack.pop().unwrap().i;         *pc + 1 },
                SUBI => { stack.last_mut().unwrap().i = stack.get(stack.len() - 2).unwrap().i   - stack.pop().unwrap().i;         *pc + 1 },
                MULI => { stack.last_mut().unwrap().i = stack.get(stack.len() - 2).unwrap().i   * stack.pop().unwrap().i;         *pc + 1 },
                DIVI => { stack.last_mut().unwrap().i = stack.get(stack.len() - 2).unwrap().i   / stack.pop().unwrap().i;         *pc + 1 },
                MODI => { stack.last_mut().unwrap().i = stack.get(stack.len() - 2).unwrap().i   % stack.pop().unwrap().i;         *pc + 1 },
                POWI => { stack.last_mut().unwrap().i = stack.get(stack.len() - 2).unwrap().i.pow(stack.pop().unwrap().i as u32); *pc + 1 },

                BOR  => { stack.last_mut().unwrap().i = stack.get(stack.len() - 2).unwrap().i | stack.pop().unwrap().i;  *pc + 1 },
                BXOR => { stack.last_mut().unwrap().i = stack.get(stack.len() - 2).unwrap().i | stack.pop().unwrap().i;  *pc + 1 },
                BAND => { stack.last_mut().unwrap().i = stack.get(stack.len() - 2).unwrap().i | stack.pop().unwrap().i;  *pc + 1 },

                FTOI => { stack.last_mut().unwrap().i = stack.last().unwrap().f as i32; *pc + 1 },
                ITOF => { stack.last_mut().unwrap().f = stack.last().unwrap().i as f32; *pc + 1 },
            }
        };
    }

    fn korak_debug(ukaz_podatek: &UkazPodatek, stack: &mut Vec<Podatek>, pc: &mut u32, addroff: &mut u32, izhod: &mut impl io::Write) -> Option<()> {
        *pc = unsafe {
            match ukaz_podatek {
                NOOP => *pc + 1,

                JUMP(naslov) => naslov.clone(),
                JMPD => stack.pop()?.i as u32,
                JMPC(naslov) => if stack.pop()? != LAŽ { naslov.clone() } else { *pc + 1 },

                PUSH(podatek) => { stack.push(*podatek); *pc + 1 },
                POP => { stack.pop(); *pc + 1 },

                POS  => { *stack.last_mut()? = if stack.last()?.f  > 0.0 { RESNICA } else { LAŽ }; *pc + 1 },
                ZERO => { *stack.last_mut()? = if stack.last()?.f == 0.0 { RESNICA } else { LAŽ }; *pc + 1 },

                LOAD(naslov) => { stack.push(*stack.get(naslov.clone() as usize)?); *pc + 1 },
                LDOF(naslov) => { stack.push(*stack.get(*addroff as usize + naslov.clone() as usize)?); *pc + 1 },
                STOR(naslov) => { *stack.get_mut(naslov.clone() as usize)? = stack.pop()?; *pc + 1 },
                STOF(naslov) => { *stack.get_mut(*addroff as usize + naslov.clone() as usize)? = stack.pop()?; *pc + 1 },
                TOP (naslov) => { *addroff = (stack.len() as i32 + naslov) as u32; *pc + 1 },

                SOFF => { *addroff = stack.pop()?.i as u32;   *pc + 1 },
                LOFF => { stack.push(Podatek { i: *addroff as i32 }); *pc + 1 },

                PRTN => { write!(izhod, "{}", stack.pop()?.f).ok()?; *pc + 1 },
                PRTC => { write!(izhod, "{}", stack.pop()?.c).ok()?; *pc + 1 },

                ADDF => { stack.last_mut()?.f = stack.get(stack.len() - 2)?.f    + stack.pop()?.f;  *pc + 1 },
                SUBF => { stack.last_mut()?.f = stack.get(stack.len() - 2)?.f    - stack.pop()?.f;  *pc + 1 },
                MULF => { stack.last_mut()?.f = stack.get(stack.len() - 2)?.f    * stack.pop()?.f;  *pc + 1 },
                DIVF => { stack.last_mut()?.f = stack.get(stack.len() - 2)?.f    / stack.pop()?.f;  *pc + 1 },
                MODF => { stack.last_mut()?.f = stack.get(stack.len() - 2)?.f    % stack.pop()?.f;  *pc + 1 },
                POWF => { stack.last_mut()?.f = stack.get(stack.len() - 2)?.f.powf(stack.pop()?.f); *pc + 1 },

                ADDI => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i   + stack.pop()?.i;         *pc + 1 },
                SUBI => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i   - stack.pop()?.i;         *pc + 1 },
                MULI => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i   * stack.pop()?.i;         *pc + 1 },
                DIVI => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i   / stack.pop()?.i;         *pc + 1 },
                MODI => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i   % stack.pop()?.i;         *pc + 1 },
                POWI => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i.pow(stack.pop()?.i as u32); *pc + 1 },

                BOR  => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i | stack.pop()?.i;  *pc + 1 },
                BXOR => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i | stack.pop()?.i;  *pc + 1 },
                BAND => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i | stack.pop()?.i;  *pc + 1 },

                FTOI => { stack.last_mut()?.i = stack.last()?.f as i32; *pc + 1 },
                ITOF => { stack.last_mut()?.f = stack.last()?.i as f32; *pc + 1 },
            }
        };
        Some(())
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_assembler_from_assembler() {
        let program = Program {
            push_tipi: vec![
                Tip::REAL,
                Tip::REAL,
                Tip::CELO,
                Tip::ZNAK,
                Tip::ZNAK,
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
                POP,
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
                PRTN,
                PRTC,
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

        assert_eq!(program, Program::from(program.to_assembler()));
    }

    #[test]
    fn zaženi() {
        let mut pc: u32 = 0;
        let mut addroff: u32 = 0;
        let mut stack: Vec<Podatek> = Vec::new();

        assert_eq!(stack, []);
        assert_eq!(pc, 0);
        assert_eq!(addroff, 0);

        let mut stdout = Box::new(Vec::new());

        // x (@0)
        Program::korak(&PUSH(Podatek { f: 1.0 }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }]);
        assert_eq!(pc, 1);
        assert_eq!(addroff, 0);

        // y (@1)
        Program::korak(&PUSH(Podatek { f: 3.14 }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }]);
        assert_eq!(pc, 2);
        assert_eq!(addroff, 0);

        // LOAD y
        Program::korak(&LOAD(1), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { f: 3.14 }]);
        assert_eq!(pc, 3);
        assert_eq!(addroff, 0);

        // LOAD x
        Program::korak(&LOAD(0), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { f: 3.14 }, Podatek { f: 1.0 }]);
        assert_eq!(pc, 4);
        assert_eq!(addroff, 0);

        // y - x
        Program::korak(&SUBF, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { f: 2.14 }]);
        assert_eq!(pc, 5);
        assert_eq!(addroff, 0);

        // y > x (y - x > 0 <=> y > x)
        Program::korak(&POS, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { i: 1 }]);
        assert_eq!(pc, 6);
        assert_eq!(addroff, 0);

        // NOOP
        Program::korak(&NOOP, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { i: 1 }]);
        assert_eq!(pc, 7);
        assert_eq!(addroff, 0);

        // JMPC #0
        Program::korak(&JMPC(0), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }]);
        assert_eq!(pc, 0);
        assert_eq!(addroff, 0);

        // PUSH #8
        Program::korak(&PUSH(Podatek { i: 8 }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { i: 8 }]);
        assert_eq!(pc, 1);
        assert_eq!(addroff, 0);

        // JMPD
        Program::korak(&JMPD, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }]);
        assert_eq!(pc, 8);
        assert_eq!(addroff, 0);

        // JUMP #13
        Program::korak(&JUMP(13), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }]);
        assert_eq!(pc, 13);
        assert_eq!(addroff, 0);

        // PUSH #0.0
        Program::korak(&PUSH(Podatek { f: 0.0 }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { f: 0.0 }]);
        assert_eq!(pc, 14);
        assert_eq!(addroff, 0);

        // ZERO (0.0 == 0.0)
        Program::korak(&ZERO, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { i: 1 }]);
        assert_eq!(pc, 15);
        assert_eq!(addroff, 0);

        // PUSH 'c'
        Program::korak(&PUSH(Podatek { c: '\n' }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { i: 1 }, Podatek { c: '\n' }]);
        assert_eq!(pc, 16);
        assert_eq!(addroff, 0);

        // PUSH '\n'
        Program::korak(&PUSH(Podatek { c: 'c' }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { i: 1 }, Podatek { c: '\n' }, Podatek { c: 'c' }]);
        assert_eq!(pc, 17);
        assert_eq!(addroff, 0);

        // PRTC
        // PRTC
        Program::korak(&PRTC, &mut stack, &mut pc, &mut addroff, &mut stdout);
        Program::korak(&PRTC, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { i: 1 }]);
        assert_eq!(pc, 19);
        assert_eq!(addroff, 0);

        // POP
        Program::korak(&POP, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }]);
        assert_eq!(pc, 20);
        assert_eq!(addroff, 0);

        // PUSH #1.0
        Program::korak(&PUSH(Podatek { f: 1.0 }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { f: 1.0 }]);
        assert_eq!(pc, 21);
        assert_eq!(addroff, 0);

        // PUSH #0.0
        Program::korak(&PUSH(Podatek { f: 0.0 }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { f: 1.0 }, Podatek { f: 0.0 }]);
        assert_eq!(pc, 22);
        assert_eq!(addroff, 0);

        // MUL (0.0 * 1.0) = 0.0
        Program::korak(&MULF, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 1.0 }, Podatek { f: 3.14 }, Podatek { f: 0.0 }]);
        assert_eq!(pc, 23);
        assert_eq!(addroff, 0);

        // STOR @0 (x = 0.0)
        Program::korak(&STOR(0), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.14 }]);
        assert_eq!(pc, 24);
        assert_eq!(addroff, 0);

        // LOFF
        Program::korak(&LOFF, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.14 }, Podatek { i: 0 }]);
        assert_eq!(pc, 25);
        assert_eq!(addroff, 0);

        // PUSH #3.01
        Program::korak(&PUSH(Podatek { f: 3.01 }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.14 }, Podatek { i: 0 }, Podatek { f: 3.01 }]);
        assert_eq!(pc, 26);
        assert_eq!(addroff, 0);

        // TOP -3
        Program::korak(&TOP(-3), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.14 }, Podatek { i: 0 }, Podatek { f: 3.01 }]);
        assert_eq!(pc, 27);
        assert_eq!(addroff, 1);

        // LDOF @0
        Program::korak(&LDOF(0), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.14 }, Podatek { i: 0 }, Podatek { f: 3.01 }, Podatek { f: 3.14 }]);
        assert_eq!(pc, 28);
        assert_eq!(addroff, 1);

        // ADD
        Program::korak(&ADDF, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.14 }, Podatek { i: 0 }, Podatek { f: 3.01 + 3.14 }]);
        assert_eq!(pc, 29);
        assert_eq!(addroff, 1);

        // PUSH 1.0
        Program::korak(&PUSH(Podatek { f: 1.0 }), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.14 }, Podatek { i: 0 }, Podatek { f: 3.01 + 3.14 }, Podatek { f: 1.0 }]);
        assert_eq!(pc, 30);
        assert_eq!(addroff, 1);

        // DIV
        Program::korak(&DIVF, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.14 }, Podatek { i: 0 }, Podatek { f: (3.01 + 3.14) / 1.0 }]);
        assert_eq!(pc, 31);
        assert_eq!(addroff, 1);

        // STOF @0
        Program::korak(&STOF(0), &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.01 + 3.14 }, Podatek { i: 0 }]);
        assert_eq!(pc, 32);
        assert_eq!(addroff, 1);

        // SOFF
        Program::korak(&SOFF, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 0.0 }, Podatek { f: 3.01 + 3.14 }]);
        assert_eq!(pc, 33);
        assert_eq!(addroff, 0);

        stack[0].f = 5.0;
        stack[1].f = 3.0;

        // MOD
        Program::korak(&MODF, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 2.0 }]);
        assert_eq!(pc, 34);
        assert_eq!(addroff, 0);

        stack.push(Podatek { f: 5.0 });

        // POW
        Program::korak(&POWF, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { f: 32.0 }]);
        assert_eq!(pc, 35);
        assert_eq!(addroff, 0);
    }

}
