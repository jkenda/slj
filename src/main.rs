mod program;
mod parser;

use std::{env, fs};

use parser::{tokenizer::Tokenize, Parse};
use program::ToProgram;

fn main() {
    let args: Vec<String> = env::args().collect();
    let pot = &args[1];

    let datoteka = fs::read_to_string(pot).unwrap_or("{}".to_owned());

    let drevo = datoteka
        .tokenize()
        .parse();

    let program = drevo.to_program();

    println!("{drevo}");
    println!("{}", program.to_assembler());

    program.run()
}
