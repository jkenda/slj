use std::fmt::Debug;

use super::lekser::Žeton;


#[derive(Debug, Clone, PartialEq)]
pub struct Napake {
    napake: Vec<Napaka>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Napaka {
    pub oznaka: OznakaNapake,
    pub sporočilo: String,
    pub začetek: (usize, usize),
    pub konec: (usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OznakaNapake {
    E1,
    E2,
    E3,
    E4,
    E5,
    E6,
    E7,
    E8,
    E9,
}

impl Napaka {
    pub fn from_zaporedje(zaporedje: &[Žeton], oznaka: OznakaNapake, sporočilo: &str) -> Napaka {
        let začetek = zaporedje[0].lokacija();
        let konec = {
            let žeton = zaporedje.last().unwrap();
            let (vrstica, znak) = žeton.lokacija();
            (vrstica, znak + žeton.as_str().chars().count())
        };
        Napaka { oznaka, sporočilo: sporočilo.to_string(), začetek, konec }
    }
}

impl Napake {
    pub fn new() -> Napake {
        Napake { napake: Vec::new() }
    }

    pub fn from_zaporedje(zaporedje: &[Žeton], oznaka: OznakaNapake, sporočilo: &str) -> Napake {
        Self::new().add_napaka(Napaka::from_zaporedje(zaporedje, oznaka, sporočilo))
    }

    pub fn add_napaka(&mut self, napaka: Napaka) -> Napake {
        self.napake.push(napaka);
        self.clone()
    }

    pub fn prazno(&self) -> bool {
         self.napake.is_empty()
    }

    pub fn razširi(&mut self, other: Self) {
        self.napake.extend(other.napake)
    }
    
    pub fn izpiši(&self, vrstice: &Vec<&str>) {
        for napaka in &self.napake {
            let (prva_vrstica, prvi_znak) = napaka.začetek;
            let (zadnja_vrstica, zadnji_znak) = napaka.konec;

            println!("Napaka {:?}: {} ({prva_vrstica}, {prvi_znak})", napaka.oznaka, napaka.sporočilo);
            if prva_vrstica > 1 {
                println!("{:zamik$} | {}", prva_vrstica-1, vrstice[prva_vrstica-2], zamik=log10(zadnja_vrstica+2));
            }

            for i in prva_vrstica-1..zadnja_vrstica {
                println!("{:zamik$} | {}", i+1, vrstice[i], zamik=log10(zadnja_vrstica+2));
            }

            println!("{:zamik$} | {}{}", 
                "", 
                " ".repeat(usize::min(prvi_znak, zadnji_znak) - 1), 
                "^".repeat(usize::abs_diff(prvi_znak, zadnji_znak)), zamik=log10(zadnja_vrstica+2));

            println!("{:zamik$} | {}\n", 
                zadnja_vrstica+1, 
                if zadnja_vrstica != vrstice.len()-1  { vrstice[zadnja_vrstica] } else { "" }, zamik=log10(zadnja_vrstica+2));
        }

        let št_napak = self.napake.len().to_string();
        let zadnji_znak = št_napak.chars().last().unwrap();

        println!("{} {}, ne morem nadaljevati", št_napak,
                 if zadnji_znak == '1' {
                     "napaka"
                 }
                 else if zadnji_znak == '2' {
                     "napaki"
                 }
                 else if zadnji_znak == '3' || zadnji_znak == '4' {
                     "napake"
                 }
                 else {
                     "napak"
                 });
    }
}

fn log10(x: usize) -> usize {
    (x as f64).log10().ceil() as usize
}
