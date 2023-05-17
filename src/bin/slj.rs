use std::{env, fs};

use slj::parser::{tokenizer::Tokenize, Parse};
use slj::program::ToProgram;

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

    let datoteka = fs::read_to_string(&args[1])
        .expect("Napaka: ne morem odpreti datoteke");

    let vrstice: Vec<&str> = datoteka.split('\n').collect();
    let drevo = datoteka
        .as_str()
        .tokenize()
        .parse();

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
            napake.izpiši(&vrstice);
        }
    }

}

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
        println!("Ukaz: {ukaz} [možnosti] <pot>");
        println!("[možnosti]:");
        println!("\t-p, --pomoč: izpiši to pomoč,");
        println!("\t-d, --debug: namesto izhoda programa izpisuj ukaze in stanje stacka pa vsakem ukazu.");
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

