use super::*;

const INCLUDE: &str = "include 'ukazi.asm'\n\n";

impl ToFasmX86 for Vec<UkazPodatekRelative> {
    fn v_fasm_x86(&self) -> String {

        self.iter()
            .fold(INCLUDE.to_string(), |str, ukaz_podatek| {
                str
                + if let Oznaka(_) = ukaz_podatek { "" } else { "\t" }
                + &match ukaz_podatek {
                    PUSHI(število) => format!("PUSH {število}"),
                    PUSHF(število) => format!("PUSH {število}"),
                    PUSHC(znak) => format!("PUSH {}", *znak as u32),
                    JUMPRelative(oznaka) => format!("JUMP {}", formatiraj_oznako(oznaka)),
                    JMPCRelative(oznaka) => format!("JMPC {}", formatiraj_oznako(oznaka)),
                    Oznaka(oznaka) => format!("{}:", formatiraj_oznako(oznaka)),
                    PC(i) => format!("PC {i}"),

                    Osnovni(ALOC(mem))  => format!("ALOC {mem}"),
                    Osnovni(LOAD(addr)) => format!("LOAD {addr}"), // load normal
                    Osnovni(LDOF(addr)) => format!("LOAD {addr}"), // load w/ offset
                    Osnovni(LDDY(addr)) => format!("LOAD {addr}"), // load dynamic
                    Osnovni(STOR(addr)) => format!("LOAD {addr}"), // store normal
                    Osnovni(STOF(addr)) => format!("LOAD {addr}"), // store w/ offset
                    Osnovni(STDY(addr)) => format!("LOAD {addr}"), // store dynamic
                    Osnovni(TOP(addr))  => format!("TOP {addr}"),
                    Osnovni(instruction) => format!("{instruction:?}"),
                }
                + "\n"
            })
    }
}

fn formatiraj_oznako(oznaka: &str) -> String {
    format!(".{}", oznaka
        .replace("(", "8")
        .replace(")", "9")
        .replace("[", "F")
        .replace("]", "G")
        .replace("@", "V")
        .replace(", ", "__")
        )
}

