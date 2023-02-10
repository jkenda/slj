use slj::{parser::{tokenizer::{Tokenize, Token::*, L}, drevo::Vozlišče::{*, self}, Parse}, program::ToProgram};

#[test]
fn natisni_niz() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"natisni("zver")"#.to_string();
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "zver");
}

#[test]
fn natisni_število() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"natisni(3.14)"#.to_string();
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "3.14");

    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"natisni(42)"#.to_string();
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(program.tokenize(), [Ime("natisni", 1, 1), Ločilo("(", 1, 8), Literal(L::Celo("42", 1, 9)), Ločilo(")", 1, 11)]);
    assert_eq!(program.tokenize().parse().unwrap().root, Okvir{ zaporedje: Zaporedje(vec![Natisni(vec![Vozlišče::Celo(42).rc()]).rc()]).rc(), št_spr: 0 }.rc());
    assert_eq!(String::from_utf8(izhod).unwrap(), "42");
}

#[test]
fn natisni_izraz() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"natisni(3+2*4**2)"#.to_string();
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "35");
}

#[test]
fn one_liner() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"naj x=1;če x-1==0{natisni("x=1")}else{natisni("x!=1")}"#.to_string();
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "x=1");
}

#[test]
fn preveč_vrstic() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"
        naj x = 1
        ;

        če x - 1 == 0
        {
            natisni
            (
                "x=1"
            )
        }

        čene
        {
            natisni
            (

                "x!=1"
            )
        }
    "#.to_string();
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "x=1");
}

#[test]
fn praštevil_do_1000() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"
        naj MEJA = 1000
        naj praštevil = 2 # [2, 3]
        naj kandidat = 5

        dokler kandidat <= MEJA {
            naj praštevilo = resnica

            naj i = 2; dokler i <= kandidat / 2 && praštevilo {
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
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "praštevil do 1000: 168\n");
}

#[test]
fn rekurzija() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"
        funkcija faktoriela(a: celo) -> celo {
            če a <= 1 {
                vrni 1
            }
            vrni a * faktoriela(a - 1)
        }
        natisni("7! = ", faktoriela(7), "\n")
    "#.to_string();
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "7! = 5040\n");
}

#[test]
fn spr_pred_funkcijo() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"
        funkcija init() -> celo {
            vrni 0
        }
        naj spr = init()
        funkcija inkrement() -> brez {
            spr += 1
        }
        naj i = 0; dokler i < 3 {
            naj _ = inkrement()
            i += 1
        }
        natisni(spr)
    "#.to_string();
    println!("{}", program.tokenize().parse().unwrap().to_string());
    println!("{}", program.tokenize().parse().unwrap().to_program().to_assembler());
    program.tokenize().parse().unwrap().to_program().zaženi_debug();
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "3");
}

#[test]
fn makro_funkcija() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"
        naj a = 0; naj b = 0.0
        funkcija prištej(x: celo) -> brez {
            a += x
        }
        funkcija prištej(x: real) -> brez {
            b += x
        }

        prištej!(42, 3.14)
        natisni(a, ", ", b)
    "#.to_string();
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "42, 3.14");
}

