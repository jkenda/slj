use slj::{parser::{tokenizer::{Tokenize, Token::*, L}, Parse}, program::ToProgram};
use std::io::Cursor;

fn test(program: &str, vhod: &str) -> String {
    let mut izhod = Vec::<u8>::new();

    program.tokenize().parse().unwrap().to_program().zaženi_z_io(&mut Cursor::new(vhod), &mut izhod);
    return String::from_utf8(izhod).unwrap();
}

#[test]
fn natisni_znak() {
    let program = r#"
        natisni('z')
        natisni('v')
        natisni('e')
        natisni('r')
        "#;

    assert_eq!(test(program, ""), "zver");
}

#[test]
fn natisni_niz() {
    let program = r#"natisni("zver")"#;
    assert_eq!(test(program, ""), "zver");
}

#[test]
fn natisni_število() {
    let program = r#"natisni(42)"#;
    assert_eq!(program.tokenize(), [Ime("natisni", 1, 1), Ločilo("(", 1, 8), Literal(L::Celo("42", 1, 9)), Ločilo(")", 1, 11)]);
    assert_eq!(test(program, ""), "42");

    let program = r#"natisni(0)"#;
    assert_eq!(program.tokenize(), [Ime("natisni", 1, 1), Ločilo("(", 1, 8), Literal(L::Celo("0", 1, 9)), Ločilo(")", 1, 10)]);
    assert_eq!(test(program, ""), "0");

    let program = r#"natisni(0.02)"#;
    assert_eq!(test(program, ""), "0.02");

    let program = r#"natisni(0.5)"#;
    assert_eq!(test(program, ""), "0.5");

    let program = r#"natisni(3.141592653589793)"#;
    assert_eq!(test(program, ""), "3.141592");
}

#[test]
fn natisni_izraz() {
    let program = r#"natisni(3+2*4**2)"#;
    assert_eq!(test(program, ""), "35");
}

#[test]
fn one_liner() {
    let program = r#"naj x=1;če x-1==0{natisni("x=1")}else{natisni("x!=1")}"#;
    assert_eq!(test(program, ""), "x=1");
}

#[test]
fn preveč_vrstic() {
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
    assert_eq!(test(program, ""), "x=1");
}

#[test]
fn praštevil_do_1000() {
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
    assert_eq!(test(program, ""), "praštevil do 1000: 168\n");
}

#[test]
fn rekurzija() {
    let program = r#"
        funkcija faktoriela(a: celo) -> celo {
            če a <= 1 {
                vrni 1
            }
            vrni a * faktoriela(a - 1)
        }
        natisni!("7! = ", faktoriela(7), "\n")
    "#;
    assert_eq!(test(program, ""), "7! = 5040\n");
}

#[test]
fn spr_pred_funkcijo() {
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
    assert_eq!(test(program, ""), "3");
}

#[test]
fn multi_funkcija() {
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
    assert_eq!(test(program, ""), "42, 3.14");
}

#[test]
fn referenca() {
    let program = r#"
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
    assert_eq!(test(program, ""), "13 3.14 42 3.14 13");

    let program = r#"
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
    assert_eq!(test(program, ""), "7 13 17");
}

#[test]
fn indeksiranje() {
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
    assert_eq!(test(program, ""), "1 2 3\n3 2 1 ");
}

#[test]
fn fake_natisni() {
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
    assert_eq!(test(program, ""), "žibje");
}

#[test]
fn zanke() {
    let program = r#"
        naj i = 1; dokler i <= 3 {
            natisni(i)
            i += 1
        }
    "#;
    assert_eq!(test(program, ""), "123");

    let program = r#"
        za i = 1, i <= 3, i += 1 {
            natisni(i)
        }
        naj i = 123
    "#;
    assert_eq!(test(program, ""), "123");

    let program = r#"
        naj i = 1
        za , i <= 3, i += 1 {
            natisni(i)
        }
        i = 1
        za , i <= 3, {
            natisni(i)
            i += 1
        }
    "#;
    assert_eq!(test(program, ""), "123123");
}

#[test]
fn pretvorbe() {
    let program = r#"
        natisni(v_celo("1312"))
    "#;
    assert_eq!(test(program, ""), "1312");
}

#[test]
fn vhod() {
    let program = r#"
        natisni(preberi())
        natisni(preberi())
        natisni(preberi())
        natisni(preberi())
    "#;
    assert_eq!(test(program, "zver"), "zver");
}

