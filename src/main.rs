mod prevajanik;
mod binary;
mod parser;

use std::{env, fs};

use parser::{tokenizer::Tokenize, Parse};

fn main() {
    let args: Vec<String> = env::args().collect();
    let pot = &args[1];

    let datoteka = fs::read_to_string(pot).unwrap_or("{}".to_owned());

    println!("{}", datoteka
             .tokenize()
             .parse());

/* KO BO KONČANO:
    datoteka
        .tokenize()
        .parse()
        .as_program()
        .run()
*/
}
