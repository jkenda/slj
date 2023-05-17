use super::*;

pub struct Argumenti {
    pub tipi: Vec<Tip>,
    pub spremenljivke: Vec<Rc<Vozlišče>>,
    pub argumenti: Vec<Rc<Vozlišče>>,
}

impl<'a> Parser<'a> {
    pub fn argumenti<'b>(&mut self, izraz: &'b[Token<'a>]) -> Result<Argumenti, Napake> where 'a: 'b {
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
                [ Operator("@", ..), Literal(..) ] => {
                    match self.drevo(&argument[1..]) {
                        Ok(drevo) => {
                            let tip = drevo.tip();
                            let spr = self.dodaj_spremenljivko(self.naključno_ime(25), tip.clone());
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
