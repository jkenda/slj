use std::{fmt::Debug, collections::HashMap, fs::read_to_string, sync::{Mutex, OnceLock}};

use super::lekser::Žeton;

unsafe fn datoteke() -> &'static mut Mutex<HashMap<String, Vec<String>>> {
    static mut DATOTEKE: OnceLock<Mutex<HashMap<String, Vec<String>>>> = OnceLock::new();
    match DATOTEKE.get_mut() {
        Some(d) => d,
        None => {
            let _ = DATOTEKE.set(Mutex::new(HashMap::new()));
            DATOTEKE.get_mut().unwrap()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Napake {
    napake: Vec<Napaka>,
    datoteke: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Napaka {
    pub oznaka: OznakaNapake,
    pub sporočilo: String,
    pub datoteka: String,
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
        let začetek = zaporedje.first().unwrap().lokacija();
        let datoteka = zaporedje.first().unwrap().datoteka();
        let sporočilo = sporočilo.to_string();
        let konec = {
            let žeton = zaporedje.last().unwrap();
            let (vrstica, znak) = žeton.lokacija();
            (vrstica, znak + žeton.as_str().chars().count())
        };
        Napaka { oznaka, datoteka, sporočilo, začetek, konec }
    }

    fn vrstice(&self) -> &[String] {
        let datoteke = unsafe { datoteke().get_mut().unwrap() };

        if !datoteke.contains_key(&self.datoteka) {
            datoteke.insert(self.datoteka.clone(),
                read_to_string(&self.datoteka)
                .unwrap()
                .lines()
                .map(String::from)
                .collect());
        }

        datoteke.get(&self.datoteka).unwrap()
    }
}

impl Napake {
    pub fn new() -> Napake {
        Napake { napake: Vec::new(), datoteke: HashMap::new() }
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
    
    pub fn izpiši(&self) {
        for napaka in &self.napake {
            let Napaka {
                oznaka,
                sporočilo,
                datoteka,
                začetek: (prva_vrstica, prvi_znak),
                konec: (zadnja_vrstica, zadnji_znak)
            } = napaka;

            let vrstice = &napaka.vrstice()[..*zadnja_vrstica + 1];

            println!("Napaka {oznaka:?}: {sporočilo} | {datoteka}:{prva_vrstica}:{prvi_znak}");

            let zamik = log10(zadnja_vrstica+2);

            if *prva_vrstica > 1 {
                let št_vrstice = prva_vrstica - 1;
                let vrstica = &vrstice[prva_vrstica-2];
                println!("{št_vrstice:zamik$} | {vrstica}");
            }

            for i in prva_vrstica-1..*zadnja_vrstica {
                let št_vrstice = i+1;
                let vrstica = &vrstice[i];
                println!("{št_vrstice:zamik$} | {vrstica}");
            }

            let razlika = usize::min(*prvi_znak, *zadnji_znak) - 1;
            let podčrtaj = "^".repeat(usize::abs_diff(*prvi_znak, *zadnji_znak));
            const PRAZNO: &str = "";
            println!("{PRAZNO:zamik$} | {PRAZNO:razlika$}{podčrtaj}");

            let št_vrstice = zadnja_vrstica + 1;
            let vrstica = vrstice.last().unwrap();
            println!("{št_vrstice:zamik$} | {vrstica}\n");
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
