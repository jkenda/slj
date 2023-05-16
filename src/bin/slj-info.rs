use std::{env, fs};

use slj::parser::{tokenizer::Tokenize, Parse};
use slj::program::ToProgram;

fn main() {
    let args: Vec<String> = env::args().collect();
    let pot = args.last().unwrap();

    let datoteka = fs::read_to_string(pot).unwrap_or("{}".to_owned());

    let vrstice: Vec<&str> = datoteka.split('\n').collect();
    let drevo = datoteka
        .as_str()
        .tokenize()
        .parse();


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
