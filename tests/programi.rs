use slj::{parser::{tokenizer::Tokenize, Parse}, program::ToProgram};

#[test]
fn natisni_niz() {
    let mut izhod: Box<Vec<u8>> = Box::new(Vec::new());
    let program = r#"natisni("zver")"#.to_string();
    program.tokenize().parse().to_program().zaženi_preusmeri_izhod(&mut izhod);
    assert_eq!(String::from_utf8(*izhod).unwrap(), "zver");
}

#[test]
fn natisni_število() {
    let mut izhod: Box<Vec<u8>> = Box::new(Vec::new());
    let program = r#"natisni(3.14)"#.to_string();
    program.tokenize().parse().to_program().zaženi_preusmeri_izhod(&mut izhod);
    assert_eq!(String::from_utf8(*izhod).unwrap(), "3.14");
}

#[test]
fn natisni_izraz() {
    let mut izhod: Box<Vec<u8>> = Box::new(Vec::new());
    let program = r#"natisni(3+2*4**2)"#.to_string();
    program.tokenize().parse().to_program().zaženi_preusmeri_izhod(&mut izhod);
    assert_eq!(String::from_utf8(*izhod).unwrap(), "35");
}

#[test]
fn one_liner() {
    let mut izhod: Box<Vec<u8>> = Box::new(Vec::new());
    let program = r#"x=1;če x-1==0{natisni("x=1")}else{natisni("x!=1")}"#.to_string();
    program.tokenize().parse().to_program().zaženi_preusmeri_izhod(&mut izhod);
    assert_eq!(String::from_utf8(*izhod).unwrap(), "x=1");
}

#[test]
fn preveč_vrstic() {
    let mut izhod: Box<Vec<u8>> = Box::new(Vec::new());
    let program = r#"
        x = 1;
        če x - 1 == 0
        {
            natisni
            (
                "x=1"
            )
        }
        čene
        {
            natisni(
                "x!=1"
            )
        }
    "#.to_string();
    program.tokenize().parse().to_program().zaženi_preusmeri_izhod(&mut izhod);
    assert_eq!(String::from_utf8(*izhod).unwrap(), "x=1");
}

#[test]
fn praštevil_do_1000() {
    let mut izhod: Box<Vec<u8>> = Box::new(Vec::new());
    let program = r#"
        MEJA = 1000
        praštevil = 2 # [2, 3]
        kandidat = 5

        dokler kandidat <= MEJA {
            praštevilo = resnica

            i = 2
            dokler i <= kandidat / 2 && praštevilo {
                če kandidat % i == 0 {
                    praštevilo = laž
                }
                i += 1
            }
            kandidat += 2

            če praštevilo {
                praštevil += 1
            }
        }

        natisni("praštevil do ", MEJA, ": ", praštevil, "\n")
    "#.to_string();
    program.tokenize().parse().to_program().zaženi_preusmeri_izhod(&mut izhod);
    assert_eq!(String::from_utf8(*izhod).unwrap(), "praštevil do 1000: 168\n");
}

#[test]
fn rekurzija() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"
        funkcija faktoriela(a) {
            če a <= 1 {
                vrni 1
            }
            vrni a * faktoriela(a - 1)
        }
        natisni("7! = ", faktoriela(7), "\n")
    "#.to_string();
    program.tokenize().parse().to_program().zaženi_preusmeri_izhod(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "7! = 5040\n");
}

