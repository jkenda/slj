use std::{env, fs};

use slj::parser::{lekser::Razčleni, Parse};
use slj::program::ToProgram;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        pomoč(&args[0]);
        return;
    }

    let datoteka = fs::read_to_string(&args[1])
        .expect("Napaka: ne morem odpreti datoteke");

    let vrstice: Vec<&str> = datoteka
        .split('\n')
        .collect();

    let drevo = datoteka
        .as_str()
        .razčleni()
        .analiziraj();


    match drevo {
        Ok(drevo) => {
            println!("{}", drevo
                .to_string());
            println!("{}", drevo
                .to_program()
                .to_assembler());
        },
        Err(napake) => {
            napake.izpiši(&vrstice);
        }
    }
}

fn pomoč(ukaz: &String) {
    println!("Ukaz: {ukaz} <pot>");
}

