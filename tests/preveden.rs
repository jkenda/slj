use slj::{parser::{Parse, lekser::{Razčleni, Žeton::*, L}}, program::ToFasmX86};
use std::{io::{Write, self}, thread, fs::File, process::{Command, Stdio}};

fn test(src: &str, input: &str) -> String {
    let fasm = src
        .razčleni("[test]")
        .analiziraj()
        .unwrap()
        .v_fasm_x86(2);

    // transform AST into native x86_64 assembly
    let thread_id = format!("{:?}", thread::current().id().to_owned());
    let thread_id = thread_id
        .split("(").nth(1).unwrap()
        .split(")").nth(0).unwrap();

    let program_filename = format!("fasm/_main__{thread_id}");

    // write assembly to file
    File::create(format!("{program_filename}.asm")).unwrap()
        .write_all(fasm.as_bytes()).unwrap();

    // compile with FASM
    let output = Command::new("fasm")
        .arg(format!("{program_filename}.asm"))
        .output()
        .expect("Failed to execute fasm");

    if !output.status.success() {
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        return String::from_utf8_lossy(&output.stderr).to_string();
    }

    // run compiled binary
    let mut proces = Command::new(program_filename)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute main");

    let mut child_stdin = proces.stdin
        .take()
        .expect("Failed to open stdin");

    child_stdin.write_all(input.as_bytes()).unwrap();

    let output = proces
        .wait_with_output()
        .expect("Failed to wait on main");

    if !output.status.success() {
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        return String::from_utf8_lossy(&output.stderr).to_string();
    }

    String::from_utf8_lossy(&output.stdout).to_string()
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
fn natisni_bool() {
    let program = r#"natisni(resnica)"#;
    assert_eq!(test(program, ""), "resnica");

    let program = r#"natisni(laž)"#;
    assert_eq!(test(program, ""), "laž");
}

#[test]
fn natisni_niz() {
    let program = r#"natisni("zver")"#;
    assert_eq!(test(program, ""), "zver");
}

#[test]
fn natisni_število() {
    let program = r#"natisni(42)"#;
    assert_eq!(program.razčleni("[test]"), [Ime("natisni", 1, 1, "[test]"), Ločilo("(", 1, 8, "[test]"), Literal(L::Celo("42", 1, 9, "[test]")), Ločilo(")", 1, 11, "[test]")]);
    assert_eq!(test(program, ""), "42");

    let program = r#"natisni(0)"#;
    assert_eq!(program.razčleni("[test]"), [Ime("natisni", 1, 1, "[test]"), Ločilo("(", 1, 8, "[test]"), Literal(L::Celo("0", 1, 9, "[test]")), Ločilo(")", 1, 10, "[test]")]);
    assert_eq!(test(program, ""), "0");

    let program = r#"natisni(0.0)"#;
    assert_eq!(test(program, ""), "0.0");

    let program = r#"natisni(0.02)"#;
    assert_eq!(test(program, ""), "0.02");

    let program = r#"natisni(0.5)"#;
    assert_eq!(test(program, ""), "0.5");

    let program = r#"natisni(3.141592653589793)"#;
    assert_eq!(test(program, ""), "3.14159");
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
        spr praštevila: [celo; 1000]
        spr praštevil = 2
        praštevila[0] = 2
        praštevila[1] = 3

        za kandidat = 5, kandidat <= praštevila.dolžina, kandidat += 2 {
            spr praštevilo = resnica

            za i = 0, praštevila[i]**2 <= kandidat && praštevilo, i += 1 {
                če kandidat % praštevila[i] == 0 {
                    praštevilo = laž
                }
            }

            če praštevilo {
                praštevila[praštevil] = kandidat
                praštevil += 1
            }
        }

        natisni!("praštevil do ", praštevila.dolžina, ": ", praštevil, "\n")
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
        spr št = init()
        funkcija inkrement() -> brez {
            št += 1
        }
        spr i = 0; dokler i < 3 {
            inkrement()
            i += 1
        }
        natisni(št)
    "#;
    assert_eq!(test(program, ""), "3");
}

#[test]
fn multi_funkcija() {
    let program = r#"
        spr a = 0; spr b = 0.0
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
        spr seznam: [real; 3]
        naj ref = @seznam
        spr dolžina = 0
    
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
        za i = 0, i < seznam.dolžina, i += 1 {
            seznam[i] = (ref.dolžina - i) kot real
            natisni!(ref[i], " ")
        }
    "#;
    assert_eq!(test(program, ""), "1.0 2.0 3.0\n3.0 2.0 1.0 ");
}

#[test]
fn fake_natisni() {
    let program = r#"
        funkcija _natisni(niz: @[znak]) {
            naj dolžina = niz.dolžina
            spr i = 0; dokler i < dolžina {
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
        spr i = 1; dokler i <= 3 {
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
        spr i = 1
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

    let program = r#"
        spr medp: [znak; 128]
        naj dolžina = preberi(@medp)
        natisni(@medp, dolžina)
    "#;
    assert_eq!(test(program, "🥝Hard liquor mixed with a bit of intellect🥝\n"), "🥝Hard liquor mixed with a bit of intellect🥝\n");
}

#[test]
fn konstante() {
    let program = r#"
        kons FPI = 3.125
        kons R = 13
        kons OBSEG = 2.0 * FPI * R kot real
        natisni(OBSEG)
    "#;
    assert_eq!(test(program, ""), "81.25");
}

#[test]
fn natisni() {
    let program = r#"
        natisni('a')
        natisni("bcd")
        natisni!("efg", 2 + 3)
    "#;
    assert_eq!(test(program, ""), "abcdefg5");
}

#[test]
fn fn_fn() {
    let asm = r#"
        funkcija a() {
            natisni('c')
        }
        funkcija b() {
            a()
        }
        b()
    "#;

    assert_eq!(test(asm, ""), "c");
}

