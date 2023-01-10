use std::{env, fs};

use slj::parser::{tokenizer::Tokenize, Parse};
use slj::program::ToProgram;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Argumenti: {} <pot>", args[0]);
        return;
    }

    let pot = &args[1];

    match fs::read_to_string(pot) {
        Err(napaka)  => println!("Ne morem odpreti datoteke: {napaka}."),
        Ok(datoteka) => datoteka
            .tokenize()
            .parse()
            .to_program()
            .zaženi(),
    }
}
