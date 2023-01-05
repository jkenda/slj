use core::slice;
use std::collections::HashMap;
use std::iter;
use std::{mem::size_of, fmt::Debug};
use std::fmt;

use crate::parser::drevo::Drevo;
use crate::parser::drevo::{OdmikIme, Vozlišče::{*, self}};
use self::{UkazPodatek::*, UkazPodatekRelative::*};

#[derive(Clone, Copy)]
pub union Podatek {
    i: i32,
    f: f32,
    c: char,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
enum UkazPodatekRelative {
    Osnovni(UkazPodatek),
    JUMPRelative(OdmikIme),
    JMPCRelative(i32),
    PC(i32),
    Oznaka(String)
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

pub struct Program {
    pc: u32,
    addroff: u32,
    stack: Vec<Podatek>,
    ukazi: Vec<UkazPodatek>,
}


pub trait ToProgram {
    fn to_program(&self) -> Program;
}

trait Prevedi {
    fn prevedi(&self) -> Vec<UkazPodatekRelative>;
    fn len(&self) -> usize;
}

pub trait Postprocesiraj {
    fn postprocesiraj(&self) -> Vec<UkazPodatek>;
}

impl Debug for Podatek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.i })
    }
}

impl Prevedi for Drevo {
    fn prevedi(&self) -> Vec<UkazPodatekRelative> {
        self.root.prevedi()
    }

    fn len(&self) -> usize {
        self.root.len()
    }
}

impl ToProgram for Drevo {
    fn to_program(&self) -> Program {
        Program::from(self)
    }
}


const RESNICA: f32 = 1.0;
const LAŽ    : f32 = 0.0;

impl Prevedi for Vozlišče {
    fn prevedi(&self) -> Vec<UkazPodatekRelative> {
        match self {
            Prazno => [].to_vec(),

            Push(krat) => iter::repeat(Osnovni(PUSH(Podatek { i: 0 }))).take(*krat).collect(),
            Pop(krat) => iter::repeat(Osnovni(POP).clone()).take(*krat).collect(),
            Vrh(odmik) => [Osnovni(TOP(*odmik as i32))].to_vec(),

            ShraniOdmik => [Osnovni(SOFF)].to_vec(),
            NaložiOdmik => [Osnovni(LOFF)].to_vec(),

            Niz(niz) => niz.chars().rev()
                .map(|c| Osnovni(PUSH(Podatek { c })))
                .collect::<Vec<UkazPodatekRelative>>(),

            Število(število) => [Osnovni(PUSH(Podatek { f: *število }))].to_vec(),
            Spremenljivka{ ime: _, naslov, z_odmikom } => [if *z_odmikom { Osnovni(LDOF(*naslov)) } else { Osnovni(LOAD(*naslov)) }].to_vec(),

            Resnica => [Osnovni(PUSH(Podatek { f: RESNICA }))].to_vec(),
            Laž     => [Osnovni(PUSH(Podatek { f: LAŽ }))].to_vec(),

            Seštevanje(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(ADD)].as_slice(),
            ].concat(),

            Odštevanje(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(SUB)].as_slice(),
            ].concat(),

            Množenje(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(MUL)].as_slice(),
            ].concat(),

            Deljenje(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(DIV)].as_slice(),
            ].concat(),

            Modulo(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(MOD)].as_slice(),
            ].concat(),

            Potenca(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(POW)].as_slice(),
            ].concat(),

            Zanikaj(vozlišče) => Odštevanje(Število(1.0).rc(), vozlišče.clone()).prevedi(),
            Konjunkcija(l, d) => Množenje(l.clone(), d.clone()).prevedi(),
            Disjunkcija(l, d) => [
                Seštevanje(l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),

            Enako(l, d) => [
                Odštevanje(l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(ZERO)].as_slice(),
            ].concat(),

            Večje(l, d) => [
                Odštevanje(l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),

            Manjše(l, d)      => Večje(d.clone(), l.clone()).prevedi(),
            VečjeEnako(l, d)  => Manjše(d.clone(), l.clone()).prevedi(),
            ManjšeEnako(l, d) => VečjeEnako(d.clone(), l.clone()).prevedi(),

            ProgramskiŠtevec(odmik) => [PC(*odmik)].to_vec(),
            Skok(odmik_ime) => [JUMPRelative(odmik_ime.clone())].to_vec(),

            DinamičniSkok => [Osnovni(JMPD).to_owned()].to_vec(),
            PogojniSkok(pogoj, skok) => [
                pogoj.prevedi().as_slice(),
                [JMPCRelative(*skok)].as_slice(),
            ].concat(),

            PogojniStavek{ pogoj, resnica, laž } => {
                let skok = Skok(OdmikIme::Odmik((resnica.len() + 1) as isize)).rc();
                Zaporedje(vec![
                          PogojniSkok(pogoj.clone(), (laž.len() + skok.len() + 1) as i32).rc(),
                          laž.clone(),
                          skok,
                          resnica.clone(),
                ]).prevedi()
            },

            Zanka{ pogoj, telo } => PogojniStavek{
                    pogoj: pogoj.clone(),
                    resnica: Zaporedje(vec![
                        telo.clone(),
                        Skok(OdmikIme::Odmik(-(telo.len() as isize) - pogoj.len() as isize - 2)).rc()
                    ]).rc(),
                    laž: Prazno.rc(),
                }.prevedi(),

            Prirejanje{ spremenljivka, izraz, z_odmikom } => {
                let naslov = if let Spremenljivka { ime: _, naslov, z_odmikom: _ } = &**spremenljivka { naslov.clone() } else { 0 };
                let shrani = if *z_odmikom { Osnovni(STOF(naslov)) } else { Osnovni(STOR(naslov)) };
                [
                    izraz.clone().prevedi().as_slice(),
                    [shrani].as_slice()
                ].concat()
            },

            Vrni(prirejanje) => [
                prirejanje.prevedi().as_slice(),
                [Oznaka("vrni".to_string())].as_slice(),
            ].concat(),

            Zaporedje(vozlišča) => vozlišča.into_iter().map(|v| v.prevedi()).flatten().collect(),
            Okvir{ zaporedje, št_spr } => Zaporedje(vec![
                Push(*št_spr).rc(),
                zaporedje.clone(),
                Pop(*št_spr).rc()
            ]).prevedi(),

            Funkcija{ ime, parametri, telo, prostor } => {
                let pred = Zaporedje(vec![
                    NaložiOdmik.rc(),
                       Vrh((
                           -NaložiOdmik.sprememba_stacka()
                           - parametri.len() as isize
                           - ProgramskiŠtevec(0).sprememba_stacka()
                           - Push(1).sprememba_stacka()) as i32).rc(),
                       Push(*prostor).rc(),
                ]);

                let za = Zaporedje(vec![
                    Pop(*prostor).rc(),
                    ShraniOdmik.rc(),
                    Pop(parametri.len()).rc(),
                    DinamičniSkok.rc()
                ]);

                [
                    Skok(OdmikIme::Odmik((1 + pred.len() + telo.len() + za.len()) as isize)).prevedi().as_slice(),
                    [Oznaka(ime.clone())].as_slice(),
                    pred.prevedi().as_slice(),
                    telo.prevedi().as_slice(),
                    [Oznaka(format!("konec_funkcije {}", ime))].as_slice(),
                    za.prevedi().as_slice()
                ].concat()
            },

            FunkcijskiKlic{ funkcija, argumenti } => {
                let vrni = Push(1);
                let skok = Skok(OdmikIme::Ime(format!(".{}", if let Funkcija { ime, parametri: _, telo: _, prostor: _ } = &**funkcija { ime } else { "" })));
                let pc   = ProgramskiŠtevec((1 + argumenti.len() + skok.len()) as i32);

                Zaporedje(vec![
                    vrni.rc(),
                    pc.rc(),
                    argumenti.rc(),
                    skok.rc(),
                ]).prevedi()
            },

            Natisni(izrazi) => izrazi.into_iter()
                .map(|izraz| [
                    izraz.prevedi().as_slice(),
                    match &**izraz {
                        Niz(_) => iter::repeat(Osnovni(PRTC)).take(izraz.sprememba_stacka() as usize).collect(),
                        _ => [Osnovni(PRTN)].to_vec(),
                    }.as_slice()
                ].concat()).flatten().collect(),
        }
    }

    fn len(&self) -> usize {
        match self {
            _ => {
                self.prevedi()
                    .iter()
                    .filter(|u| if let Oznaka(..) = u { false } else { true })
                    .count()
            },
        }
    }

}

impl From<&Drevo> for Program {
    fn from(drevo: &Drevo) -> Self {
        Program { 
            pc: 0,
            addroff: 0,
            stack: { let mut s = Vec::new(); s.reserve(512); s },
            ukazi: drevo.prevedi().postprocesiraj(),
        }
    }
}

impl From<String> for Program {
    fn from(assembler: String) -> Self {
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

        let mut program = Program { pc: 0, addroff: 0, stack: Vec::new(), ukazi };
        program.stack.reserve(512);
        program
    }
}

impl Postprocesiraj for Vec<UkazPodatekRelative> {
    fn postprocesiraj(&self) -> Vec<UkazPodatek> {
        let mut postproc1 = self.clone();
        let mut postproc: Vec<UkazPodatek> = Vec::new();
        let mut oznake_vrstic: HashMap<String, u32> = HashMap::new();

        // nadomesti "vrni" z JUMP x
        let mut i: usize = 0;
        while i < postproc1.len() {
            if let Oznaka(oznaka) = &postproc1[i] {
                if oznaka == "vrni" {
                    // poišči oznako za konec funkcije
                    // nadomesti oznako "vrni" z relativnim skokom
                    let mut konec = i + 1;
                    loop {
                        match &postproc1[konec] {
                            Oznaka(oznaka) => if oznaka.starts_with("konec_funkcije") {
                                postproc1[konec] = JUMPRelative(OdmikIme::Ime(oznaka.split_whitespace().last().unwrap().to_string()));
                                break;
                            },
                            _ => konec += 1,
                        }
                    }
                }
            }
            i += 1;
        }

        // preberi oznake vrstic in jih odstrani
        let mut i: usize = 0;
        while i < postproc1.len() {
            match &postproc1[i] {
                Oznaka(oznaka) => {
                    oznake_vrstic.insert(oznaka.clone(), i as u32);
                    postproc.remove(i);
                }
                _ => i += 1,
            }
        }

        // relativni skok -> absolutni skok
        for (št_vrstice, ukaz_podatek) in postproc1.iter().enumerate() {
            postproc.push(match ukaz_podatek {
                Osnovni(osnovni_ukaz) => osnovni_ukaz.clone(),
                JUMPRelative(odmik_ime) => match odmik_ime {
                    OdmikIme::Odmik(rel_skok) => JUMP((št_vrstice as isize + rel_skok) as u32),
                    OdmikIme::Ime(ime)        => JUMP(oznake_vrstic[ime]),
                },
                JMPCRelative(rel_skok) => JMPC((št_vrstice as i32 + rel_skok) as u32),
                PC(odmik) => PUSH(Podatek { i: št_vrstice as i32 + odmik }),
                Oznaka(_) => NOOP,
            });
        }

        postproc
    }
}

impl Program {
    pub fn run(&mut self) {
        use UkazPodatek::*;

        while (self.pc as usize) < self.ukazi.len() {
            let ukaz_podatek = &self.ukazi[self.pc as usize];

            self.pc = unsafe {
                match ukaz_podatek {
                    NOOP => self.pc + 1,
                    JUMP(naslov) => naslov.clone(),
                    JMPD => self.stack.pop().unwrap().i as u32,
                    JMPC(naslov) => if self.stack.pop().unwrap().f != LAŽ { naslov.clone() } else { self.pc + 1 },
                    PUSH(podatek) => { self.stack.push(*podatek); self.pc + 1 },
                    POP => { self.stack.pop(); self.pc + 1 },
                    POS => { self.stack.last_mut().unwrap().f = if self.stack.last().unwrap().f > 0.0 { RESNICA } else { LAŽ }; self.pc + 1 },
                    ZERO => { self.stack.last_mut().unwrap().f = if self.stack.last().unwrap().f == 0.0 { RESNICA } else { LAŽ }; self.pc + 1 },
                    LOAD(podatek) => { self.stack.push(self.stack[podatek.clone() as usize]); self.pc + 1 },
                    LDOF(podatek) => { self.stack.push(self.stack[self.addroff as usize + podatek.clone() as usize]); self.pc + 1 },
                    STOR(podatek) => { self.stack[podatek.clone() as usize] = self.stack.pop().unwrap(); self.pc + 1 },
                    STOF(podatek) => { self.stack[self.addroff as usize + podatek.clone() as usize] = self.stack.pop().unwrap(); self.pc + 1 },
                    TOP(podatek)  => { self.addroff = (self.stack.len() as i32 + podatek) as u32; self.pc + 1 },
                    SOFF => { self.addroff = self.stack.pop().unwrap().i as u32; self.pc + 1 },
                    LOFF => { self.stack.push(Podatek { i: self.addroff as i32 }); self.pc + 1 },
                    PRTN => { print!("{}", self.stack.pop().unwrap().f); self.pc + 1 },
                    PRTC => { print!("{}", self.stack.pop().unwrap().c); self.pc + 1 },
                    ADD  => { self.stack.last_mut().unwrap().f = self.stack.pop().unwrap().f + self.stack.pop().unwrap().f; self.pc + 1 },
                    SUB  => { self.stack.last_mut().unwrap().f = self.stack.pop().unwrap().f - self.stack.pop().unwrap().f; self.pc + 1 },
                    MUL  => { self.stack.last_mut().unwrap().f = self.stack.pop().unwrap().f * self.stack.pop().unwrap().f; self.pc + 1 },
                    DIV  => { self.stack.last_mut().unwrap().f = self.stack.pop().unwrap().f / self.stack.pop().unwrap().f; self.pc + 1 },
                    MOD  => { self.stack.last_mut().unwrap().f = self.stack.pop().unwrap().f % self.stack.pop().unwrap().f; self.pc + 1 },
                    POW  => { self.stack.last_mut().unwrap().f = self.stack.pop().unwrap().f.powf(self.stack.pop().unwrap().f); self.pc + 1 },
                }
            };
        }
    }

    pub unsafe fn to_bytes(&self) -> &[UkazPodatek]  {
        slice::from_raw_parts(self.ukazi.as_ptr(), self.ukazi.len() * size_of::<UkazPodatek>())
    }

}
