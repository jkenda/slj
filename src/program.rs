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

impl PartialEq for Podatek {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.i == other.i }
    }
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
    NEG,
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


const RESNICA: Podatek = Podatek { i: 1 };
const LAŽ    : Podatek = Podatek { i: 0 };

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
                .map(|znak| Osnovni(PUSH(Podatek { c: znak })))
                .collect::<Vec<UkazPodatekRelative>>(),
            Število(število) => [Osnovni(PUSH(Podatek { f: *število }))].to_vec(),
            Spremenljivka{ ime: _, naslov, z_odmikom } => [if *z_odmikom { Osnovni(LDOF(*naslov)) } else { Osnovni(LOAD(*naslov)) }].to_vec(),

            Resnica => [Osnovni(PUSH(RESNICA))].to_vec(),
            Laž     => [Osnovni(PUSH(LAŽ))].to_vec(),

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

            Zanikaj(vozlišče) => [
                vozlišče.prevedi().as_slice(),
                [Osnovni(NEG)].as_slice(),
            ].concat(),
            Konjunkcija(l, d) => Množenje(l.clone(), d.clone()).prevedi(),
            Disjunkcija(l, d) => [
                Seštevanje(l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),

            Enako(l, d) => [
                Odštevanje(l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(ZERO)].as_slice(),
            ].concat(),
            NiEnako(l, d) => Zanikaj(Enako(l.clone(), d.clone()).rc()).prevedi(),

            Večje(l, d) => [
                Odštevanje(l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),
            Manjše(l, d)      => Večje(d.clone(), l.clone()).prevedi(),
            VečjeEnako(l, d)  => Zanikaj(Manjše(l.clone(), d.clone()).rc()).prevedi(),
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
                            .replace(r"\\", "\\")
                                .replace(r"\n", "\n")
                                .replace(r"\t", "\t")
                                .replace(r"\r", "\r")
                                .replace(r#"\"""#, "\"")
                                .replace(r"\'", "\'")
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
    pub fn run(&self) {
        use UkazPodatek::*;

        let mut pc = 0;
        let mut addroff = 0;
        let mut stack: Vec<Podatek> = Vec::new();
        stack.reserve(512);

        while (pc as usize) < self.ukazi.len() {
            let ukaz_podatek = &self.ukazi[pc as usize];

            pc = unsafe {
                match ukaz_podatek {
                    NOOP => pc + 1,
                    JUMP(naslov) => naslov.clone(),
                    JMPD => stack.pop().unwrap().i as u32,
                    JMPC(naslov) => if stack.pop().unwrap() != LAŽ { naslov.clone() } else { pc + 1 },
                    PUSH(podatek) => { stack.push(*podatek); pc + 1 },
                    POP => { stack.pop(); pc + 1 },
                    POS => { stack.last_mut().unwrap().i  = if stack.last().unwrap().f  > 0.0 { RESNICA.i } else { LAŽ.i }; pc + 1 },
                    ZERO => { stack.last_mut().unwrap().i = if stack.last().unwrap().f == 0.0 { RESNICA.i } else { LAŽ.i }; pc + 1 },
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
                    NEG  => { stack.last_mut().unwrap().i = 1 - stack.pop().unwrap().i; pc + 1 },
                }
            };
        }

    }

    pub fn to_assembler(&self) -> String {
        let mut str = String::new();

        for ukaz_podatek in &self.ukazi {
            str += &match ukaz_podatek {
                NOOP          => "NOOP\n".to_string(),
                JUMP(naslov)  => format!("JUMP #{}\n", naslov),
                JMPD          => "JMPD\n".to_string(),
                JMPC(naslov)  => format!("JMPC #{}\n", naslov),
                PUSH(podatek) => format!("PUSH #{}\n", unsafe { podatek.f }),
                POP           => "POP \n".to_string(),
                POS           => "POS \n".to_string(),
                ZERO          => "ZERO\n".to_string(),
                LOAD(naslov)  => format!("LOAD @{}\n", naslov),
                LDOF(naslov)  => format!("LDOF @{}\n", naslov),
                STOR(naslov)  => format!("STOR @{}\n", naslov),
                STOF(naslov)  => format!("STOF @{}\n", naslov),
                TOP(odmik)    => format!("TOP  {}{}\n", if *odmik > 0 { "+" } else { "" }, odmik),
                SOFF          => "SOFF\n".to_string(),
                LOFF          => "LOFF\n".to_string(),
                PRTN          => "PRTN\n".to_string(),
                PRTC          => "PRTC\n".to_string(),
                ADD           => "ADD \n".to_string(),
                SUB           => "SUB \n".to_string(),
                MUL           => "MUL \n".to_string(),
                DIV           => "DIV \n".to_string(),
                MOD           => "MOD \n".to_string(),
                POW           => "POW \n".to_string(),
                NEG           => "NEG \n".to_string(),
            }
        }

        str
    }

    pub unsafe fn to_bytes(&self) -> &[UkazPodatek]  {
        slice::from_raw_parts(self.ukazi.as_ptr(), self.ukazi.len() * size_of::<UkazPodatek>())
    }

}
