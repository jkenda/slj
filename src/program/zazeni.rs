use std::io::Write;

use console::Term;
use unsafe_unwrap::UnsafeUnwrap;

use super::*;

impl Program {
    pub fn zaženi(&self) {
        self.zaženi_z_izhodom(&mut io::stdout())
    }

    pub fn zaženi_debug(&self) {
        let mut pc: i32 = 0;
        let mut addroff: i32 = 0;
        let mut stack: Vec<Podatek> = Vec::with_capacity(32_768);

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
        assert!(stack.len() == 0);
    }

    pub fn zaženi_z_izhodom(&self, izhod: &mut impl io::Write) {
        let mut pc: i32 = 0;
        let mut addroff: i32 = 0;
        let mut stack: Vec<Podatek> = Vec::with_capacity(32_768);

        while (pc as usize) < self.ukazi.len() {
            Program::korak(&self.ukazi[pc as usize], &mut stack, &mut pc, &mut addroff, izhod);
        }
        assert!(stack.len() == 0);
    }

    #[inline]
    fn korak(ukaz_podatek: &UkazPodatek, stack: &mut Vec<Podatek>, pc: &mut i32, addroff: &mut i32, izhod: &mut impl io::Write) {
        *pc = unsafe {
            match ukaz_podatek {
                NOOP => *pc + 1,

                JUMP(naslov) => *naslov,
                JMPD => stack.pop().unsafe_unwrap().i,
                JMPC(naslov) => if stack.pop().unsafe_unwrap() != LAŽ { *naslov } else { *pc + 1 },

                PUSH(podatek) => { stack.push(*podatek); *pc + 1 },
                ALOC(razlika) => { stack.resize((stack.len() as i32 + razlika) as usize, NIČ); *pc + 1 }

                POS  => { *stack.last_mut().unsafe_unwrap() = if stack.last().unsafe_unwrap().f  > 0.0 { RESNICA } else { LAŽ }; *pc + 1 },
                ZERO => { *stack.last_mut().unsafe_unwrap() = if stack.last().unsafe_unwrap().f == 0.0 { RESNICA } else { LAŽ }; *pc + 1 },

                LOAD(naslov) => { stack.push(*stack.get(*naslov as usize).unsafe_unwrap()); *pc + 1 },
                LDOF(naslov) => { stack.push(*stack.get((*addroff + *naslov) as usize).unsafe_unwrap()); *pc + 1 },
                LDDY(naslov) => {
                    let dynaddr = stack.pop().unsafe_unwrap().i;
                    stack.push(*stack.get((*naslov + dynaddr) as usize).unsafe_unwrap());
                    *pc + 1
                },

                STOR(naslov) => { *stack.get_mut(*naslov as usize).unsafe_unwrap() = stack.pop().unsafe_unwrap(); *pc + 1 },
                STOF(naslov) => { *stack.get_mut(*addroff as usize + *naslov as usize).unsafe_unwrap() = stack.pop().unsafe_unwrap(); *pc + 1 },
                STDY(naslov) => {
                    let dynaddr = stack.pop().unsafe_unwrap().i;
                    *stack.get_mut(*naslov as usize + dynaddr as usize).unsafe_unwrap() = stack.pop().unsafe_unwrap();
                    *pc + 1
                }

                TOP (naslov) => { *addroff = stack.len() as i32 + naslov; *pc + 1 },

                SOFF => { *addroff = stack.pop().unsafe_unwrap().i;   *pc + 1 },
                LOFF => { stack.push(Podatek { i: *addroff }); *pc + 1 },

                PRTC => { write!(izhod, "{}", stack.pop().unsafe_unwrap().c).unsafe_unwrap(); *pc + 1 },
                GETC => {
                    let mut term = Term::stdout();
                    let c = term.read_char().unsafe_unwrap();
                    let _ = term.write_all(c.to_string().as_bytes());
                    stack.push(Podatek { c });
                    *pc + 1
                }

                ADDF => { stack.last_mut().unsafe_unwrap().f = stack.get(stack.len() - 2).unsafe_unwrap().f    + stack.pop().unsafe_unwrap().f;  *pc + 1 },
                SUBF => { stack.last_mut().unsafe_unwrap().f = stack.get(stack.len() - 2).unsafe_unwrap().f    - stack.pop().unsafe_unwrap().f;  *pc + 1 },
                MULF => { stack.last_mut().unsafe_unwrap().f = stack.get(stack.len() - 2).unsafe_unwrap().f    * stack.pop().unsafe_unwrap().f;  *pc + 1 },
                DIVF => { stack.last_mut().unsafe_unwrap().f = stack.get(stack.len() - 2).unsafe_unwrap().f    / stack.pop().unsafe_unwrap().f;  *pc + 1 },
                MODF => { stack.last_mut().unsafe_unwrap().f = stack.get(stack.len() - 2).unsafe_unwrap().f    % stack.pop().unsafe_unwrap().f;  *pc + 1 },
                POWF => { stack.last_mut().unsafe_unwrap().f = stack.get(stack.len() - 2).unsafe_unwrap().f.powf(stack.pop().unsafe_unwrap().f); *pc + 1 },

                ADDI => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i.wrapping_add(stack.pop().unsafe_unwrap().i);        *pc + 1 },
                SUBI => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i.wrapping_sub(stack.pop().unsafe_unwrap().i);        *pc + 1 },
                MULI => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i.wrapping_mul(stack.pop().unsafe_unwrap().i);        *pc + 1 },
                DIVI => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i.wrapping_div(stack.pop().unsafe_unwrap().i);        *pc + 1 },
                MODI => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i.wrapping_rem(stack.pop().unsafe_unwrap().i);        *pc + 1 },
                POWI => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i.wrapping_pow(stack.pop().unsafe_unwrap().i as u32); *pc + 1 },

                BOR  => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i | stack.pop().unsafe_unwrap().i;  *pc + 1 },
                BXOR => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i ^ stack.pop().unsafe_unwrap().i;  *pc + 1 },
                BAND => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i & stack.pop().unsafe_unwrap().i;  *pc + 1 },

                BSLL => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i << stack.pop().unsafe_unwrap().i;  *pc + 1 },
                BSLD => { stack.last_mut().unsafe_unwrap().i = stack.get(stack.len() - 2).unsafe_unwrap().i >> stack.pop().unsafe_unwrap().i;  *pc + 1 },

                FTOI => { stack.last_mut().unsafe_unwrap().i = stack.last().unsafe_unwrap().f as i32; *pc + 1 },
                ITOF => { stack.last_mut().unsafe_unwrap().f = stack.last().unsafe_unwrap().i as f32; *pc + 1 },
            }
        };
    }

    #[inline]
    fn korak_debug(ukaz_podatek: &UkazPodatek, stack: &mut Vec<Podatek>, pc: &mut i32, addroff: &mut i32, izhod: &mut impl io::Write) -> Option<()> {
        *pc = unsafe {
            match ukaz_podatek {
                NOOP => *pc + 1,

                JUMP(naslov) => *naslov,
                JMPD => stack.pop()?.i,
                JMPC(naslov) => if stack.pop()? != LAŽ { *naslov } else { *pc + 1 },

                PUSH(podatek) => { stack.push(*podatek); *pc + 1 },
                ALOC(razlika) => { stack.set_len((stack.len() as i32 + razlika) as usize); *pc + 1 }

                POS  => { *stack.last_mut()? = if stack.last()?.f  > 0.0 { RESNICA } else { LAŽ }; *pc + 1 },
                ZERO => { *stack.last_mut()? = if stack.last()?.f == 0.0 { RESNICA } else { LAŽ }; *pc + 1 },

                LOAD(naslov) => { stack.push(*stack.get(*naslov as usize)?); *pc + 1 },
                LDOF(naslov) => { stack.push(*stack.get(*addroff as usize + *naslov as usize)?); *pc + 1 },
                LDDY(naslov) => {
                    let dynaddr = stack.pop()?.i;
                    stack.push(*stack.get(*naslov as usize + dynaddr as usize)?);
                    *pc + 1
                },

                STOR(naslov) => { *stack.get_mut(*naslov as usize)? = stack.pop()?; *pc + 1 },
                STOF(naslov) => { *stack.get_mut(*addroff as usize + *naslov as usize)? = stack.pop()?; *pc + 1 },
                STDY(naslov) => {
                    let dynaddr = stack.pop()?.i;
                    *stack.get_mut(*naslov as usize + dynaddr as usize)? = stack.pop()?;
                    *pc + 1
                }

                TOP (naslov) => { *addroff = stack.len() as i32 + naslov; *pc + 1 },

                SOFF => { *addroff = stack.pop()?.i;   *pc + 1 },
                LOFF => { stack.push(Podatek { i: *addroff as i32 }); *pc + 1 },

                PRTC => { write!(izhod, "{}", stack.pop()?.c).ok()?; *pc + 1 },
                GETC => {
                    let mut term = Term::stdout();
                    let c = term.read_char().ok()?;
                    let _ = term.write_all(c.to_string().as_bytes());
                    stack.push(Podatek { c });
                    *pc + 1
                }

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
                BXOR => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i ^ stack.pop()?.i;  *pc + 1 },
                BAND => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i & stack.pop()?.i;  *pc + 1 },

                BSLL => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i << stack.pop()?.i;  *pc + 1 },
                BSLD => { stack.last_mut()?.i = stack.get(stack.len() - 2)?.i >> stack.pop()?.i;  *pc + 1 },

                FTOI => { stack.last_mut()?.i = stack.last()?.f as i32; *pc + 1 },
                ITOF => { stack.last_mut()?.f = stack.last()?.i as f32; *pc + 1 },
            }
        };
        Some(())
    }
}

#[cfg(test)]
mod testi {
    use super::*;

    #[test]
    fn zaženi() {
        let mut pc: i32 = 0;
        let mut addroff: i32 = 0;
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
        Program::korak(&ALOC(-1), &mut stack, &mut pc, &mut addroff, &mut stdout);
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

        stack = vec![Podatek { i: 1234 }, Podatek { i: 5678 }];

        // BAND
        Program::korak(&BAND, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { i: 1234 & 5678 }]);
        assert_eq!(pc, 36);
        assert_eq!(addroff, 0);

        stack = vec![Podatek { i: 1234 }, Podatek { i: 5678 }];

        // BXOR
        Program::korak(&BXOR, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { i: 1234 ^ 5678 }]);
        assert_eq!(pc, 37);
        assert_eq!(addroff, 0);

        stack = vec![Podatek { i: 1234 }, Podatek { i: 5678 }];

        // BOR
        Program::korak(&BOR, &mut stack, &mut pc, &mut addroff, &mut stdout);
        assert_eq!(stack, [Podatek { i: 1234 | 5678 }]);
        assert_eq!(pc, 38);
        assert_eq!(addroff, 0);
    }

}
