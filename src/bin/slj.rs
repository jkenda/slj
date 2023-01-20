use std::{env, fs};

use slj::parser::{tokenizer::Tokenize, napaka::*, Parse};
use slj::program::ToProgram;

struct Možnosti {
    pomoč: bool,
    debug: bool,
}

impl Možnosti {
    fn new() -> Možnosti {
        Možnosti {
            pomoč: false,
            debug: false
        }
    }
}

fn pomoč(ukaz: &String) {
        println!("Argumenti: {ukaz} [možnosti] <pot>");
        println!("[možnosti]:");
        println!("\t-p, --pomoč: izpiši to pomoč,");
        println!("\t-s, --debug: namesto izhoda programa izpisuj ukaze in stanje stacka pa vsakem ukazu.");
}

fn analiziraj_možnosti(args: &[String]) -> Možnosti {
    let mut možnosti = Možnosti::new();
    for arg in args {
        match arg.as_str() {
            "--pomoč" => možnosti.pomoč = true,
            "--debug" => možnosti.debug = true,
            _ => if arg.starts_with("--") {
                panic!("Neznana možnost: '{arg}'");
            }
            else if arg.starts_with('-') {
                for znak in &arg.chars().collect::<Vec<char>>()[1..] {
                    if *znak == 'p' { možnosti.pomoč = true }
                    else if *znak == 'd' { možnosti.debug = true }
                    else { panic!("Neznana možnost: '{znak}'") };
                }
            },
        }
    }
    možnosti
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        pomoč(&args[0]);
        return;
    }

    let možnosti = analiziraj_možnosti(&args[1..]);

    if možnosti.pomoč {
        pomoč(&args[0]);
        return;
    }

    let pot = &args.last().unwrap();

    let datoteka = fs::read_to_string(pot)
        .expect("Napaka: ne morem odpreti datoteke");

    let vrstice: Vec<&str> = datoteka.split('\n').collect();
    let drevo = datoteka.tokenize().parse();

    match drevo {
        Ok(drevo) => {
            if možnosti.debug {
                drevo.to_program().zaženi_debug();
            }
            else {
                drevo.to_program().zaženi();
            }
        },
        Err(napake) => {
            izpiši_napake(napake, &vrstice);
        }
    }

}

fn izpiši_napake(napake: Napake, vrstice: &Vec<&str>) {
    for napaka in napake.clone() {
        let (prva_vrstica, prvi_znak) = napaka.začetek;
        let (zadnja_vrstica, zadnji_znak) = napaka.konec;

        println!("Napaka {:?}: {} ({prva_vrstica}, {prvi_znak})", napaka.oznaka, napaka.sporočilo);
        if prva_vrstica > 1 {
            println!("{:zamik$} | {}", prva_vrstica-1, vrstice[prva_vrstica-2], zamik=log10(zadnja_vrstica+2));
        }

        for i in prva_vrstica-1..zadnja_vrstica {
            println!("{:zamik$} | {}", i+1, vrstice[i], zamik=log10(zadnja_vrstica+2));
        }

        println!("{:zamik$} | {}{}", "", " ".repeat(usize::min(prvi_znak, zadnji_znak) - 1), "^".repeat(usize::abs_diff(prvi_znak, zadnji_znak)), zamik=log10(zadnja_vrstica+2));
        println!("{:zamik$} | {}\n", zadnja_vrstica+1, if zadnja_vrstica != vrstice.len()-1  { vrstice[zadnja_vrstica] } else { "" }, zamik=log10(zadnja_vrstica+2));
    }

    let št_napak = napake.into_iter().count().to_string();
    let zadnji_znak = št_napak.chars().last().unwrap();
    println!("{} {} napak, ne morem nadaljevati", št_napak,
             if zadnji_znak == '1' {
                "napaka"
             }
             else if zadnji_znak == '2' {
                 "napaki"
             }
             else if zadnji_znak == '3' || zadnji_znak == '4' {
                 "napake"
             }
             else {
                 "napak"
             });
}

fn log10(x: usize) -> usize {
    (x as f64).log10().ceil() as usize
}

