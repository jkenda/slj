use slj::{parser::{tokenizer::{Tokenize, Token::*, L}, Parse}, program::ToProgram};

#[test]
fn natisni_znak() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"
        natisni('z')
        natisni('v')
        natisni('e')
        natisni('r')
        "#;
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "zver");
}

#[test]
fn natisni_niz() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"natisni("zver")"#;
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "zver");
}

#[test]
fn natisni_število() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"natisni(3.14159268)"#;
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "3.14159268");

    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"natisni(42)"#;
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(program.tokenize(), [Ime("natisni", 1, 1), Ločilo("(", 1, 8), Literal(L::Celo("42", 1, 9)), Ločilo(")", 1, 11)]);
    assert_eq!(String::from_utf8(izhod).unwrap(), "42");

    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"natisni(0)"#;
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(program.tokenize(), [Ime("natisni", 1, 1), Ločilo("(", 1, 8), Literal(L::Celo("42", 1, 9)), Ločilo(")", 1, 11)]);
    assert_eq!(String::from_utf8(izhod).unwrap(), "0");
}

#[test]
fn natisni_izraz() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"natisni(3+2*4**2)"#;
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "35");
}

#[test]
fn one_liner() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"naj x=1;če x-1==0{natisni("x=1")}else{natisni("x!=1")}"#;
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
    "#;
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

        natisni!("praštevil do ", MEJA, ": ", praštevil, "\n")
    "#;
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
        natisni!("7! = ", faktoriela(7), "\n")
    "#;
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
            inkrement()
            i += 1
        }
        natisni(spr)
    "#;
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "3");
}

#[test]
fn multi_funkcija() {
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
        natisni!(a, ", ", b)
    "#;
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "42, 3.14");
}

#[test]
fn referenca() {
    let mut izhod: Vec<u8> = Vec::new();
    let mut program = r#"
        funkcija naloži(ref: @real) {
            natisni!(ref@, " ")
        }
        funkcija naloži(ref: @celo) {
            natisni!(ref@, " ")
            naloži(@3.14)
        }

        naj a = 13
        naj b = @a
        naloži(@a)
        naloži(@42)
        natisni(b@)
        "#;
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod.clone()).unwrap(), "13 3.14 42 3.14 13");

    izhod.clear();
    program = r#"
        funkcija spremeni(ref: @celo, val: celo) {
            ref@ = val;
        }
        funkcija povečaj(ref: @celo, val: celo) {
            ref@ += val
        }

        naj a = 7
        natisni!(a, " ")
        spremeni(@a, 13)
        natisni!(a, " ")
        povečaj(@a, 4)
        natisni(a)
        "#;
    let parsed = program.tokenize().parse().unwrap();
    let program = parsed.to_program();
    println!("{}", parsed.to_string());
    println!("{}", program.to_assembler());
    program.zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod.clone()).unwrap(), "7 13 17");
}

#[test]
fn indeksiranje() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"
        naj seznam: [real; 3]
        naj ref = @seznam
        naj dolžina = 0
    
        funkcija dodaj(seznam: @[real], št: real) {
            seznam[dolžina] = št
            dolžina += 1
        }

        funkcija dodaj(seznam: @[real], št: @real) {
            seznam[dolžina] = št@
            dolžina += 1
        }

        dodaj(@seznam, @1.0)
        dodaj(@seznam, 2.0)
        dodaj(ref, 3.0)

        natisni!(seznam[0], " ", seznam[1], " ", ref[2], "\n")
        naj i = 0; dokler i < seznam.dolžina {
            seznam[i] = (ref.dolžina - i) kot real
            natisni!(ref[i], " ")
            i += 1
        }
    "#;
    println!("{}", program.tokenize().parse().unwrap().to_program().to_assembler());
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "1 2 3\n3 2 1 ");
}

#[test]
fn fake_natisni() {
    let mut izhod: Vec<u8> = Vec::new();
    let program = r#"
        funkcija _natisni(niz: @[znak]) {
            naj dolžina = niz.dolžina
            naj i = 0; dokler i < dolžina {
                natisni(niz[i])
                i += 1
            }
        }

        _natisni(@"žibje")
    "#;
    println!("{}", program.tokenize().parse().unwrap().to_program().to_assembler());
    program.tokenize().parse().unwrap().to_program().zaženi_z_izhodom(&mut izhod);
    assert_eq!(String::from_utf8(izhod).unwrap(), "žibje");
}

