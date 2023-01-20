use super::tokenizer::Token;


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
}

impl Napake {
    pub fn new() -> Napake {
        Napake { napake: Vec::new() }
    }

    pub fn from_zaporedje(zaporedje: &[Token], oznaka: OznakaNapake, sporočilo: &str) -> Napake {
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
}

impl Napaka {
    pub fn from_zaporedje(zaporedje: &[Token], oznaka: OznakaNapake, sporočilo: &str) -> Napaka {
        let začetek = zaporedje[0].lokacija();
        let konec = {
            let žeton = zaporedje.last().unwrap();
            let (vrstica, znak) = žeton.lokacija();
            (vrstica, znak + žeton.as_str().chars().count())
        };
        Napaka { oznaka, sporočilo: sporočilo.to_string(), začetek, konec }
    }
}

impl IntoIterator for Napake {
    type IntoIter = NapakeIntoIter;
    type Item = Napaka;

    fn into_iter(self) -> Self::IntoIter {
        NapakeIntoIter {
            napake: self,
            index: 0,
        }
    }
}

pub struct NapakeIntoIter {
    napake: Napake,
    index: usize,
}

impl Iterator for NapakeIntoIter {
    type Item = Napaka;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.napake.napake.get(self.index)?.clone();
        self.index += 1;
        Some(item)
    }
}

