use super::*;

pub struct Argumenti {
    pub tipi: Vec<Tip>,
    pub spremenljivke: Vec<Rc<Vozlišče>>,
    pub argumenti: Vec<Rc<Vozlišče>>,
}

const ŠTEVILKE: &[&str] = &[
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", 
    "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", 
    "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", 
    "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", 
    "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", 
    "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", 
    "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", 
    "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", 
    "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", 
    "90", "91", "92", "93", "94", "95", "96", "97", "98", "99", 
];

impl<'a> Parser<'a> {
    pub fn argumenti<'b>(&mut self, izraz: &'b[Žeton<'a>]) -> Result<Argumenti, Napake> where 'a: 'b {
        let mut napake = Napake::new();

        let mut tipi = Vec::new();
        let mut spremenljivke = Vec::new();
        let mut argumenti = Vec::new();
        let razdeljeno = razdeli(izraz, &[","])?;

        for argument in razdeljeno {
            match argument {
                [] => {
                    tipi.push(Tip::Brez);
                    argumenti.push(Prazno.rc());
                    spremenljivke.push(Prazno.rc());
                },
                [ Operator("@", ..), literal @ Literal(..) ]
                    | [ literal @ Literal(L::Niz(..)) ] => {
                    match self.drevo(&[*literal]) {
                        Ok(drevo) => {
                            let tip = drevo.tip();

                            let mut i = 0;
                            let ime = loop {
                                if !self.spremenljivke.contains_key(ŠTEVILKE[i]) {
                                    break ŠTEVILKE[i];
                                }
                                i += 1;
                            };

                            let spr = self.dodaj_spremenljivko(ime, tip.clone(), false);
                            let prirejanje = Prirejanje { spremenljivka: spr.clone(), izraz: drevo }.rc();

                            let referenca = match tip {
                                Tip::Seznam(..) => RefSeznama(spr).rc(),
                                _ => Referenca(spr).rc()
                            };

                            tipi.push(referenca.tip());
                            spremenljivke.push(prirejanje);
                            argumenti.push(referenca);
                        },
                        Err(n) => napake.razširi(n),
                    }
                },
                [ .. ] => {
                    match self.drevo(argument) {
                        Ok(drevo) => {
                            tipi.push(drevo.tip());
                            argumenti.push(drevo);
                            spremenljivke.push(Prazno.rc());
                        },
                        Err(n) => napake.razširi(n),
                    }
                },
            }
        }

        if napake.prazno() {
            Ok(Argumenti{ tipi, spremenljivke, argumenti })
        }
        else {
            Err(napake)
        }
    }
}
