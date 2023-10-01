use super::*;

impl ToFasmX86 for Vec<UkazPodatekRelative> {
    fn v_fasm_x86(&self) -> String {

        self.iter()
            .fold(String::new(), |str, ukaz_podatek| {
                str
                + if let Oznaka(_) = ukaz_podatek { "" } else { "\t" }
                + &match ukaz_podatek {
                    PUSHI(število) => format!("PUSH, {število}"),
                    PUSHF(število) => format!("PUSH, {število}"),
                    PUSHC(znak) => format!("PUSH, '{znak}'"),
                    JUMPRelative(oznaka) => format!("JUMP, {oznaka}"),
                    JMPCRelative(oznaka) => format!("JMPC, {oznaka}"),
                    Oznaka(oznaka) => format!(".{oznaka}:"),
                    PC(i) => format!("PC, {i}"),

                    Osnovni(NOOP) => "nop".to_string(),
                    Osnovni(ALOC(mem)) => format!("ALOC, {mem}"),
                    Osnovni(POS) => todo!(),
                    Osnovni(ZERO) => "ZERO".to_string(),
                    Osnovni(LOAD(addr)) => format!("LOAD, {addr}"), // load normal
                    Osnovni(LDOF(addr)) => format!("LOAD, {addr}"), // load w/ offset
                    Osnovni(LDDY(addr)) => format!("LOAD, {addr}"), // load dynamic
                    Osnovni(STOR(addr)) => format!("LOAD, {addr}"), // store normal
                    Osnovni(STOF(addr)) => format!("LOAD, {addr}"), // store w/ offset
                    Osnovni(STDY(addr)) => format!("LOAD, {addr}"), // store dynamic
                    Osnovni(TOP(addr))  => format!("TOP, {addr}"),
                    instruction => format!("{instruction:?}"),
                }
                + "\n"
            })
    }
}
