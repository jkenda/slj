use std::io;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::{fs::File, io::Write};
use std::process::Command;

use slj::parser::{lekser::Razčleni, Parse};
use slj::program::ToFasmX86;

fn main() -> std::io::Result<()> {
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

    let drevo = datoteka
        .as_str()
        .razčleni(ime)
        .analiziraj();

    match drevo {
        Ok(drevo) => {
            // transform AST into native x86_64 assembly
            let fasm = drevo
                .v_fasm_x86(2);

            let filename = Path::new(ime)
                .file_stem()
                .expect("file has no name")
                .to_str()
                .expect("file name has no valid utf-8");

            fs::create_dir_all(format!("bin/"))?;
            let asm_filename = format!("fasm/{filename}.asm");
            let filename = format!("bin/{filename}");

            // write assembly to file
            File::create(&asm_filename)?
                .write_all(fasm.as_bytes())?;

            // compile with FASM
            let output = Command::new("fasm")
                .arg(asm_filename)
                .arg(&filename)
                .output()
                .expect("Failed to execute fasm");

            if !output.status.success() {
                io::stdout().write_all(&output.stdout)?;
                io::stderr().write_all(&output.stderr)?;
                return Err(io::Error::new(io::ErrorKind::Other, "compilation failed"));
            }

            if !možnosti.zaženi {
                return Ok(());
            }

            // run compiled binary
            let status = Command::new(filename)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .expect("Failed to execute main");

            match status.code() {
                Some(0) => Ok(()),
                Some(code) => Err(io::Error::new(io::ErrorKind::Other, format!("program failed with exit code {code}"))),
                None => Err(io::Error::new(io::ErrorKind::Other, "program failed")),
            }
        },
        Err(napake) => {
            napake.izpiši();
            Ok(())
        }
    }

}

struct Možnosti {
    pomoč: bool,
    zaženi: bool,
}

impl Možnosti {
    fn new() -> Možnosti {
        Možnosti {
            pomoč: false,
            zaženi: false
        }
    }
}

fn pomoč(ukaz: &String) {
        println!("Ukaz: {ukaz} [možnosti] <pot>");
        println!("[možnosti]:");
        println!("\t-p, --pomoč: izpiši to pomoč,");
        println!("\t-r, --run: po prevajanju zeženi program.");
}

fn analiziraj_možnosti(args: &[String]) -> Možnosti {
    let mut možnosti = Možnosti::new();
    for arg in args {
        match arg.as_str() {
            "--pomoč" => možnosti.pomoč = true,
            "--zaženi" => možnosti.zaženi = true,
            _ => if arg.starts_with("--") {
                panic!("Neznana možnost: '{arg}'");
            }
            else if arg.starts_with('-') {
                for znak in &arg.chars().collect::<Vec<char>>()[1..] {
                    if *znak == 'p' { možnosti.pomoč = true }
                    else if *znak == 'z' { možnosti.zaženi = true }
                    else { panic!("Neznana možnost: '{znak}'") };
                }
            },
        }
    }
    možnosti
}

