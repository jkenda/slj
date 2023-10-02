use std::collections::HashMap;
use std::io;
use std::{fs::File, io::Write};
use std::process::Command;

use slj::parser::drevo::{Drevo, Vozlišče};
use slj::parser::napaka::Napake;
use slj::parser::tip::Tip;
use slj::parser::{lekser::Razčleni, Parse};
use slj::program::ToFasmX86;

fn main() -> std::io::Result<()> {
    /*
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        pomoč(&args[0]);
        return Ok(());
    }

    let možnosti = analiziraj_možnosti(&args[1..]);

    if možnosti.pomoč {
        pomoč(&args[0]);
        return Ok(());
    }

    let ime = &args[1];

    let datoteka = fs::read_to_string(ime)
        .expect("Napaka: ne morem odpreti datoteke");

    let drevo = r#"natisni("pozdravljen svet")"#
        .razčleni("[prazno]")
        .analiziraj();
    */

    use Vozlišče::*;

    let drevo = Drevo {
        funkcije: vec![],
        št_klicev: HashMap::new(),
        main: Zaporedje(vec![
            Natisni(CeloVZnak(Seštevanje(Tip::Celo, Celo(48).rc(), Celo(1).rc()).rc()).rc()).rc(),
            Natisni(CeloVZnak(Seštevanje(Tip::Celo, Celo(48).rc(), Celo(3).rc()).rc()).rc()).rc(),
            Natisni(Znak('\n').rc()).rc(),
            Natisni(CeloVZnak(Odštevanje(Tip::Celo, Celo(58).rc(), Celo(10).rc()).rc()).rc()).rc(),
            Natisni(Znak('\n').rc()).rc(),
            Natisni(CeloVZnak(Množenje(Tip::Celo, Celo(15).rc(), Celo(4).rc()).rc()).rc()).rc(),
            Natisni(Znak('\n').rc()).rc(),
            Natisni(CeloVZnak(Deljenje(Tip::Celo, Celo(100).rc(), Celo(2).rc()).rc()).rc()).rc(),
            Natisni(Znak('\n').rc()).rc(),
            Natisni(CeloVZnak(Modulo(Tip::Celo, Celo(553).rc(), Celo(100).rc()).rc()).rc()).rc(),
            Natisni(Znak('\n').rc()).rc(),
        ]).rc()
    };
    let drevo: Result<Drevo, Napake> = Ok(drevo);

    match drevo {
        Ok(drevo) => {
            // transform AST into native x86_64 assembly
            let fasm = drevo
                .v_fasm_x86();

            // write assembly to file
            File::create("fasm/_main.asm")?
                .write_all(fasm.as_bytes())?;

            // compile with FASM
            let output = Command::new("fasm")
                .arg("fasm/_main.asm")
                .output()
                .expect("Failed to execute fasm");

            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
            if !output.status.success() {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "FASM failed"));
            }

            // run compiled binary
            let output = Command::new("fasm/_main")
                .output()
                .expect("Failed to execute main");

            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
            Ok(())
        },
        Err(napake) => {
            napake.izpiši();
            Ok(())
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

