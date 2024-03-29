use std::{mem::transmute, fmt::Display};

use super::{
    ToFasmX86,
    UkazPodatekRelative::{self, *},
    UkazPodatek::*,
};

#[cfg(debug_assertions)]
const HEADER: &str = r#"
include "header.asm"

"#;
#[cfg(debug_assertions)]
const FOOTER: &str = r#"
include "footer.asm"
"#;

#[cfg(not(debug_assertions))]
const HEADER: &str = include_str!("../../fasm/header.asm");
#[cfg(not(debug_assertions))]
const FOOTER: &str = include_str!("../../fasm/footer.asm");

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum Instr {
    Label(String),
    Macro(&'static str),
    Aloc(i32),

    Nop,
    Ret,

    Mov(Op, Op),
    Lea(R, Op),
    Push(Op),
    Pop(Op),

    ArOp(ArO, R, Op),
    IDiv(R),
    Cdq,

    Inc(Op),
    Dec(Op),

    Cmp(Op, Op),
    Setg(R),
    Sete(R),
    Setne(R),

    Jmp(String),
    Jne(String),
    Je(String),
    Jl(String),
    Call(String),
    Syscall,

    Fld(Op),
    Fild(Op),
    Fstp(Op),
    Fistp(Op),
    Fadd(Op),
    Fsub(Op),
    Fmul(Op),
    Fdiv(Op),
    Fprem,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum Op {
    UImm(u32),
    SImm(i32),
    Reg(R),
    Deref(Size, R, i32)
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum Size {
    Dword,
    Qword,
}

use Instr::*;
use Op::*;
use Size::*;
use super::{
    ArO,
    R::{self, *}
};

impl Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label(label) => write!(f, "{label}:\n"),
            Macro(text)  => write!(f, "\t{text}\n"),
            Aloc(mem)    => write!(f, "\taloc {mem}\n"),

            Nop => "\tnop\n".fmt(f),
            Ret => "\tret\n".fmt(f),

            Mov(a, b)   => write!(f, "\tmov  {a}, {b}\n"),
            Lea(r, a)   => write!(f, "\tlea  {r}, {a}\n"),
            Push(data)  => write!(f, "\tpush {data}\n"),
            Pop(data)   => write!(f, "\tpop  {data}\n"),

            Inc(a)      => write!(f, "\tinc  {a}\n"),
            Dec(a)      => write!(f, "\tdec  {a}\n"),

            ArOp(ar_op, a, b)  => write!(f, "\t{ar_op} {a}, {b}\n"),
            IDiv(r)      => write!(f, "\tidiv {r}\n"),
            Cdq         => write!(f, "\tcdq\n"),

            Cmp(a, b)   => write!(f, "\tcmp  {a}, {b}\n"),
            Setg(r)     => write!(f, "\tsetg {r}\n"),
            Sete(r)     => write!(f, "\tsete {r}\n"),
            Setne(r)    => write!(f, "\tsetne {r}\n"),

            Jmp(label)  => write!(f, "\tjmp  {label}\n"),
            Jne(label)  => write!(f, "\tjne  {label}\n"),
            Je(label)   => write!(f, "\tje   {label}\n"),
            Jl(label)   => write!(f, "\tjl   {label}\n"),
            Call(label) => write!(f, "\tcall {label}\n"),
            Syscall     => write!(f, "\tsyscall\n"),

            Fld(op)     => write!(f, "\tfld  {op}\n"),
            Fild(op)    => write!(f, "\tfild {op}\n"),
            Fstp(op)    => write!(f, "\tfstp {op}\n"),
            Fistp(op)   => write!(f, "\tfistp {op}\n"),
            Fadd(op)    => write!(f, "\tfadd {op}\n"),
            Fsub(op)    => write!(f, "\tfsub {op}\n"),
            Fmul(op)    => write!(f, "\tfmul {op}\n"),
            Fdiv(op)    => write!(f, "\tfdiv {op}\n"),
            Fprem       => write!(f, "\tfprem\n"),
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UImm(data)           => write!(f, "0x{data:X}"),
            SImm(data)           => write!(f, "{data}"),
            Reg(r)               => write!(f, "{r}"),
            Deref(size, r, off)  => write!(f, "{size} [{r}{}{}]",
                if *off == 0 { "" } else if *off > 0 { " + " } else { " - " },
                if *off == 0 { "".to_string() } else { format!("0x{:X}", off.abs()) }),
        }
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dword => write!(f, "dword"),
            Qword => write!(f, "qword"),
        }
    }
}

impl Display for super::ArO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ArO::*;
        match self {
            Add  => write!(f, "add "),
            Sub  => write!(f, "sub "),
            IMul => write!(f, "imul"),
            Or   => write!(f, "or  "),
            Xor  => write!(f, "xor "),
            And  => write!(f, "and "),
            Shl  => write!(f, "shl "),
            Shr  => write!(f, "shr "),
        }
    }
}

impl Display for super::R {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rax => write!(f, "rax"),
            Eax => write!(f, "eax"),
            Al  => write!(f, "al"),
            Rbx => write!(f, "rbx"),
            Ebx => write!(f, "ebx"),
            Bl  => write!(f, "bl"),
            Rcx => write!(f, "rcx"),
            Cl  => write!(f, "cl"),
            Ecx => write!(f, "ecx"),
            Rdx => write!(f, "rdx"),
            Rsi => write!(f, "rsi"),
            Rdi => write!(f, "rdi"),
            R8  => write!(f, "r8"),
            R9  => write!(f, "r9"),
            Rsp => write!(f, "rsp"),
        }
    }
}

impl ToFasmX86 for Vec<UkazPodatekRelative> {
    fn v_fasm_x86(self, level: u32) -> String {
        use Instr::*;
        use Op::*;
        use super::{
            R::*,
            ArO::*,
        };

        let len = self.len() * 2;
        let opti = if level >= 1 { opti1(self) } else { self };

        let asm = opti.into_iter()
            .fold(Vec::with_capacity(len), |mut seq, ukaz_podatek| {
                seq.append(&mut match ukaz_podatek {
                    // PUSH
                    PUSHI(število) =>
                        push(unsafe { transmute::<i32, u32>(število) }),
                    PUSHF(število) =>
                        push(unsafe { transmute::<f32, u32>(število) }),
                    PUSHC(znak) =>
                        push(unsafe { transmute::<i32, u32>(v_utf8(znak)) }),
                    PUSHREF(addr, true) => vec![
                            Lea(Rdi, Deref(Qword, R9, -addr * 8)),
                            Push(Reg(Rdi))],
                    PUSHREF(addr, false) => vec![
                            Lea(Rdi, Deref(Qword, R8, -addr * 8)),
                            Push(Reg(Rdi))],
                    // JUMP
                    JUMPRel(oznaka) => vec![
                        Jmp(formatiraj_oznako(&oznaka))],
                    JMPCRel(oznaka) => vec![
                        Pop(Reg(Rax)),
                        Cmp(Reg(Eax), SImm(0)),
                        Jne(formatiraj_oznako(&oznaka))],
                    PC(..) =>
                        vec![],
                    CALL(oznaka) => vec![
                        Call(formatiraj_oznako(&oznaka))],
                    Osnovni(JMPD) =>
                        vec![Ret],
                    Oznaka(oznaka) => vec![
                        Label(formatiraj_oznako(&oznaka))],

                    // combined LOAD/STORE/OPERATION
                    STIMM(data, addr, reg) => vec![
                        Mov(Deref(Dword, reg, -addr * 8), UImm(data))],
                    LDOP(op, r1, r2, addr1, addr2) => vec![
                        Mov(Reg(Eax), Deref(Dword, r1, -addr1 * 8)),
                        ArOp(op, Eax, Deref(Dword, r2, -addr2 * 8)),
                        Push(Reg(Rax))],
                    LDST(r1, r2, src, dst) => vec![
                        Mov(Reg(Eax), Deref(Dword, r1, -src * 8)),
                        Mov(Deref(Dword, r2, -dst * 8), Reg(Eax))],

                    Osnovni(NOOP) =>
                        vec![Nop],
                    Osnovni(POS) => vec![
                        Pop(Reg(Rax)),
                        Cmp(Reg(Eax), SImm(0)),
                        Mov(Reg(Eax), SImm(0)),
                        Setg(Al),
                        Push(Reg(Rax))
                    ],
                    Osnovni(ZERO) => vec![
                        Pop(Reg(Rax)),
                        Cmp(Reg(Eax), SImm(0)),
                        Mov(Reg(Eax), SImm(0)),
                        Sete(Al),
                        Push(Reg(Rax))],

                    // load
                    Osnovni(LOAD(addr)) => vec![
                        Push(Deref(Qword, R8, -addr * 8))],
                    Osnovni(LDOF(addr)) => vec![
                        Push(Deref(Qword, R9, -addr * 8))],
                    Osnovni(LDDY(offs)) => vec![
                        Pop(Reg(Rdi)),
                        Push(Deref(Qword, Rdi, -offs * 8))],
                    LDINDEXED => vec![
                        Pop(Reg(Rbx)),
                        Pop(Reg(Rax)),
                        ArOp(IMul, Rbx, SImm(8)),
                        ArOp(Sub, Rax, Reg(Rbx)),
                        Push(Deref(Qword, Rax, 0))],

                    // store
                    Osnovni(STOR(addr)) => vec![
                        Pop(Deref(Qword, R8, -addr * 8))],
                    Osnovni(STOF(addr)) => vec![
                        Pop(Deref(Qword, R9, -addr * 8))],
                    Osnovni(STDY(offs)) => vec![
                        Pop(Reg(Rdi)),
                        Pop(Deref(Qword, Rdi, -offs * 8))],
                    STINDEXED => vec![
                        Pop(Reg(Rbx)),
                        Pop(Reg(Rax)),
                        ArOp(IMul, Rbx, SImm(8)),
                        ArOp(Sub, Rax, Reg(Rbx)),
                        Pop(Deref(Qword, Rax, 0))],

                    // handle address offset for load/store in functions
                    Osnovni(TOP(offs)) => vec![
                        Lea(R9, Deref(Qword, Rsp, -offs * 8 - 8))],
                    Osnovni(LOFF) => vec![
                        Push(Reg(R9))],
                    Osnovni(SOFF) => vec![
                        Pop(Reg(R9))],

                    // arithmetic operations
                    Osnovni(op @ (ADDI | SUBI | MULI)) => vec![
                        Pop(Reg(Rbx)),
                        Pop(Reg(Rax)),
                        ArOp(match op {
                            ADDI => Add,
                            SUBI => Sub,
                            MULI => IMul,
                            _ => unreachable!()
                        }, Rax, Reg(Rbx)),
                        Push(Reg(Rax))],
                    Osnovni(op @ (DIVI | MODI)) => vec![
                        Pop(Reg(Rbx)),
                        Pop(Reg(Rax)),
                        Cdq,
                        IDiv(Ebx),
                        Push(Reg(match op {
                            DIVI => Rax,
                            MODI => Rdx,
                            _ => unreachable!()
                        }))],
                    Osnovni(POWI) => vec![
                        Pop(Reg(Rcx)),
                        Pop(Reg(Rbx)),
                        Mov(Reg(Rax), SImm(1)),
                        Call("_powi".to_string()),
                        Push(Reg(Rax))],
                    Osnovni(op @ (BOR | BXOR | BAND | BSLL | BSLR)) => vec![
                        Pop(Reg(Rcx)),
                        Pop(Reg(Rax)),
                        match op {
                            BOR  => ArOp(Or,  Eax, Reg(Ecx)),
                            BXOR => ArOp(Xor, Eax, Reg(Ecx)),
                            BAND => ArOp(And, Eax, Reg(Ecx)),
                            BSLL => ArOp(Shl, Eax, Reg(Cl)),
                            BSLR => ArOp(Shr, Eax, Reg(Cl)),
                            _ => unreachable!()
                        },
                        Push(Reg(Rax))],

                    Osnovni(op @ (ADDF | SUBF | MULF | DIVF)) => vec![
                        Fld(Deref(Dword, Rsp, 8)),
                        match op {
                            ADDF => Fadd(Deref(Dword, Rsp, 0)),
                            SUBF => Fsub(Deref(Dword, Rsp, 0)),
                            MULF => Fmul(Deref(Dword, Rsp, 0)),
                            DIVF => Fdiv(Deref(Dword, Rsp, 0)),
                            _ => unreachable!()
                        },
                        Pop(Deref(Qword, Rsp, -8)),
                        Fstp(Deref(Dword, Rsp, 0))],
                    Osnovni(MODF) => vec![
                        Fld(Deref(Dword, Rsp, 0)),
                        Fld(Deref(Dword, Rsp, 8)),
                        Fprem,
                        Pop(Deref(Qword, Rsp, -8)),
                        Fstp(Deref(Dword, Rsp, 0))],
                    Osnovni(POWF) => vec![
                        Macro("powf"),],

                    Osnovni(FTOI) => vec![
                        Fld  (Deref(Dword, Rsp, 0)),
                        Fistp(Deref(Dword, Rsp, 0))],
                    Osnovni(ITOF) => vec![
                        Fild(Deref(Dword, Rsp, 0)),
                        Fstp(Deref(Dword, Rsp, 0))],

                    Osnovni(PUTC) => vec![
                        Pop(Reg(Rax)),
                        Call("_putc".to_string())],
                    Osnovni(GETC) => vec![
                        Call("_getc".to_string()),
                        Push(Reg(Rax))],
                    Osnovni(FLUSH) => vec![
                        Call("_flush".to_string())],

                    Osnovni(ALOC(mem)) => vec![
                        Aloc(mem)],

                    _ => unreachable!()
                });
                seq
            });

        let asm = if level >= 2 { opti2(asm) } else { asm };

        asm.into_iter()
        .fold(HEADER.to_string(), |str, repr| str + &repr.to_string())
        + FOOTER
    }
}

fn opti1(mut opti: Vec<UkazPodatekRelative>) -> Vec<UkazPodatekRelative> {
    use super::{
        R::*,
        ArO::*,
    };

    let mut i = 0;
    while i < opti.len() - 2 {
        let sub = &opti[i..];
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
                PUSHF(result);
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
                    ADDI => Add, SUBI => Sub,
                    MULI => IMul,
                    _ => unreachable!(),
                };
                let off1 = match ld1 {
                    LOAD(..) => R8,
                    LDOF(..) => R9,
                    _ => unreachable!(),
                };
                let off2 = match ld2 {
                    LOAD(..) => R8,
                    LDOF(..) => R9,
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
                    STOR(..) => R8,
                    STOF(..) => R9,
                    _ => unreachable!(),
                };
                let data = match push {
                    PUSHI(data) => *data,
                    PUSHC(data) => v_utf8(*data),
                    PUSHF(data) => unsafe { transmute::<f32, i32>(*data) },
                    _ => unreachable!(),
                };
                opti[i] = STIMM(unsafe { transmute::<i32, u32>(data) }, *dst, reg);
                opti.remove(i + 1);
                i - 1
            },

            [
                Osnovni(load @ (LOAD(src) | LDOF(src))),
                Osnovni(stor @ (STOR(dst) | STOF(dst))),
                ..
            ] => {
                let r1 = match load {
                    LOAD(..) => R8,
                    LDOF(..) => R9,
                    _ => unreachable!(),
                };
                let r2 = match stor {
                    STOR(..) => R8,
                    STOF(..) => R9,
                    _ => unreachable!(),
                };

                opti[i] = LDST(r1, r2, *src, *dst);
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
                            STOR(..) => R8,
                            STOF(..) => R9,
                            _ => unreachable!(),
                        };

                        let dst = *dst;
                        opti.remove(j);
                        opti.remove(i);
                        opti.insert(i, STIMM(unsafe { transmute::<i32, u32>(data) }, dst, reg));
                        i
                    },
                    None => i + 1
                }
            },

            _ => i + 1,
        }
    };
    opti
}

fn opti2(mut asm: Vec<Instr>) -> Vec<Instr> {
    use Instr::*;
    use Op::*;
    use super::{
        R::*,
        ArO::*,
    };

    let mut i = 0;
    while i < asm.len() - 2 {
        i = match &asm[i..] {
            [Push(r1), Pop(r2), ..] if r1 == r2 => {
                asm.remove(i);
                asm.remove(i);
                (i as isize - 3).max(0) as usize
            },
            [Push(val), Pop(reg @ Reg(..)), ..] => {
                asm[i] = Mov(*reg, *val);
                asm.remove(i + 1);
                (i as isize - 3).max(0) as usize
            },
            [
                Mov(Reg(a), val1 @ (SImm(..) | UImm(..) | Deref(..))),
                Mov(Reg(b), val2 @ (SImm(..) | UImm(..) | Deref(..))),
                ArOp(Add, c, Reg(d)),
                ..
            ] if a == d && b == c => {
                let b = *b;
                let val2 = *val2;
                asm[i] = Mov(Reg(b), *val1);
                asm[i + 1] = ArOp(Add, b, val2);
                asm.remove(i + 2);
                (i as isize - 3).max(0) as usize
            },
            [ArOp(Add, a, UImm(1) | SImm(1)), ..] => {
                asm[i] = Inc(Reg(*a));
                (i as isize - 3).max(0) as usize
            },
            [ArOp(Sub, a, UImm(1) | SImm(1)), ..] => {
                asm[i] = Dec(Reg(*a));
                (i as isize - 3).max(0) as usize
            },
            [
                Mov(a, imm @ (UImm(..) | SImm(..))),
                Mov(b, c),
                ..
            ] if a == c => {
                asm[i] = Mov(*b, *imm);
                asm.remove(i + 1);
                (i as isize - 3).max(0) as usize
            },
            [Jne(..), Jmp(b), ..] => {
                asm[i] = Je(b.clone());
                asm.remove(i + 1);
                (i as isize - 3).max(0) as usize
            },
            [
                a @ Mov(Reg(Rbx), ..),
                b @ Mov(Reg(Rax), ..),
                ..
            ] => {
                let a = a.clone();
                asm[i] = b.clone();
                asm[i + 1] = a;
                (i as isize - 3).max(0) as usize
            },
            [
                Mov(Reg(Rax), UImm(..) | SImm(..)),
                Mov(Reg(Rbx), c),
                ArOp(op, Rax, Reg(Rbx)),
                ..
            ] => {
                asm[i + 1] = ArOp(*op, Rax, *c);
                asm.remove(i + 2);
                (i as isize - 3).max(0) as usize
            },
            [Mov(Reg(..), ..), Pop(dst @ Reg(..)), ..] => {
                // potuj nazaj do PUSHa
                let mut j = i;

                let data = loop {
                    match &asm[j] {
                        Push(data @ (SImm(..) | UImm(..))) => break Some(*data),
                        Mov(Reg(reg), ..) if *reg != R8 && *reg != R9 => j -= 1,
                        _ => break None,
                    };
                    if j == 0 { break None }
                };

                match data {
                    Some(data) => {
                        let dst = *dst;
                        asm.remove(j);
                        asm.remove(i);
                        asm.insert(i, Mov(dst, data));
                        (j as isize - 3).max(0) as usize
                    },
                    None => i + 1
                }
            },
            /* TODO: preglej, zakaj je ta optimizacija nevarna
            [
                Pop(dst),
                ..
            ] if i != 0 => {
                // potuj nazaj do PUSHa
                let mut j = i - 1;

                let data = loop {
                    match &asm[j] {
                        Push(data @ (SImm(..) | UImm(..))) => break Some(*data),
                        instr => match instr {
                            Label(..) => break None,
                            Push(..) => break None,
                            Pop(..) => break None,
                            Mov(a, ..) if a == dst => break None,
                            _ => j -= 1,
                        }
                    };
                    if j == 0 { break None }
                };

                match data {
                    Some(data) => {
                        let dst = *dst;
                        asm[i] = Mov(dst, data);
                        asm.remove(j);
                        j
                    },
                    None => i + 1
                }
            },
            */

            _ => i + 1
        }
    };
    asm
}

fn push(data: u32) -> Vec<Instr> {
    use Instr::*;
    use Op::*;
    use super::R::*;
    if data > 0xFFFF {
        vec![Mov(Reg(Rax), UImm(data)),
        Push(Reg(Rax))]
    } else {
        vec![Push(UImm(data))]
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
    format!("_{}", oznaka
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
    use std::{thread, io};
    use std::{fs::File, io::Write};
    use std::process::{Command, Stdio};

    use super::*;
    use crate::parser::drevo::{Drevo, Vozlišče};
    use crate::parser::tip::Tip;
    use Vozlišče::*;

    fn test(fasm: &str, input: &str) -> Result<String, io::Error> {
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
            return Err(io::Error::new(io::ErrorKind::Other, format!("exit code: {}", output.status.code().unwrap())));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
            ]).rc(),
            prostor: 0,
        }
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?, "až😭\n");
        Ok(())
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
            ]).rc(),
            prostor: 0,
        }
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "asdf")?, "asdf");
        //test(drevo, "až😭\n", "až😭\n", true)
        Ok(())
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
            }.rc(),
            prostor: 0,
        }
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?, "130<>25Q/");
        Ok(())
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
            prostor: 0,
        }
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?, "0123");
        Ok(())
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
            prostor: 0,
        }
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?, "130<>25");
        Ok(())
    }

    #[test]
    fn bitne_operacije() -> Result<(), io::Error> {
        let asm = vec![
            Oznaka("main".to_string()),
            PUSHI(0b110), PUSHI(0b011), Osnovni(BOR),  Osnovni(PUTC),
            PUSHI(0b110), PUSHI(0b011), Osnovni(BXOR), Osnovni(PUTC),
            PUSHI(0b110), PUSHI(0b011), Osnovni(BAND), Osnovni(PUTC),
            PUSHI(0b011), PUSHI(0b001), Osnovni(BSLL), Osnovni(PUTC),
            PUSHI(0b110), PUSHI(0b001), Osnovni(BSLR), Osnovni(PUTC),
        ]
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?.as_bytes(), &[
                0b111u8,
                0b101u8,
                0b010u8,
                0b110u8,
                0b011u8,
        ]);
        Ok(())
    }

    #[test]
    fn jump() -> Result<(), io::Error> {
        let asm = vec![
            Oznaka("main".to_string()),
            PUSHC('0'),
            Osnovni(PUTC),
            JUMPRel("else".to_string()),
            PUSHC('1'),
            Osnovni(PUTC),
            Oznaka("else".to_string()),
            PUSHC('2'),
            Osnovni(PUTC),
        ]
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?, "02");
        Ok(())
    }

    #[test]
    fn jmpc() -> Result<(), io::Error> {
        let asm = vec![
            Oznaka("main".to_string()),
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
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?, "023");
        Ok(())
    }

    #[test]
    fn primerjave() -> Result<(), io::Error> {
        let asm = vec![
            Oznaka("main".to_string()),
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
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?, "2");
        Ok(())
    }

    #[test]
    fn load() -> Result<(), io::Error> {
        let asm = vec![
            Oznaka("main".to_string()),
            PUSHC('1'), Osnovni(TOP(0)), PUSHC('2'), PUSHC('3'),

            Osnovni(LOAD(0)), Osnovni(PUTC),                    // 1
            PUSHREF(0, false), Osnovni(LDDY(0)), Osnovni(PUTC), // 1
            PUSHREF(0, true),  Osnovni(LDDY(0)), Osnovni(PUTC), // 2
            Osnovni(LDOF(0)), Osnovni(PUTC),                    // 2
            Osnovni(LDOF(1)), Osnovni(PUTC),                    // 3
            PUSHREF(0, false), Osnovni(LDDY(2)), Osnovni(PUTC), // 3

            Osnovni(PUTC), Osnovni(PUTC), Osnovni(PUTC), // 3 2 1
            Osnovni(TOP(0)),
        ]
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?, "112233321");
        Ok(())
    }

    #[test]
    fn stor() -> Result<(), io::Error> {
        let asm = vec![
            Oznaka("main".to_string()),
            PUSHC('1'), Osnovni(TOP(0)), PUSHC('2'), PUSHC('3'),

            Osnovni(LOAD(0)), PUSHI(1), Osnovni(SUBI), Osnovni(STOR(0)),
            Osnovni(LOAD(1)), PUSHI(1), Osnovni(SUBI), PUSHREF(1, false), Osnovni(STDY(0)),
            Osnovni(LDOF(1)), PUSHI(1), Osnovni(SUBI), Osnovni(STOF(1)),
            Osnovni(LOAD(0)), Osnovni(PUTC),
            Osnovni(LOAD(1)), Osnovni(PUTC),
            Osnovni(LOAD(2)), Osnovni(PUTC),

            Osnovni(ALOC(-3)),
            Osnovni(TOP(0)),
        ]
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?, "012");
        Ok(())
    }

    #[test]
    fn loff_soff() -> Result<(), io::Error> {
        let asm = vec![
            Oznaka("main".to_string()),
            Osnovni(LOFF), Osnovni(TOP(1)), Osnovni(LOFF),
            Osnovni(LOAD(0)), Osnovni(LOAD(1)), Osnovni(SUBI), Osnovni(PUTC),
            Osnovni(ALOC(-2)),
            Osnovni(TOP(0)),
        ]
        .v_fasm_x86(0);

        assert_eq!(test(&asm, "")?.as_bytes(), [16]);
        Ok(())
    }

}

