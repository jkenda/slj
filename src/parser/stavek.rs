use super::{*, argumenti::Argumenti};

impl<'a> Parser<'a> {
    pub fn stavek<'b>(&mut self, izraz: &'b[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> where 'a: 'b {
        match izraz {
            // multifunkcijski klic
            [ ime @ Ime(..), Operator("!", ..), Ločilo("(", ..), argumenti @ .., Ločilo(")", ..) ] => self.multi_klic(ime, argumenti),
            // deklaracija
            [ Rezerviranka("spr", ..), ime @ Ime(..), Ločilo(":", ..), tip @ .. ] => self.deklaracija(ime, tip),
            // inicializacija
            [ Rezerviranka("spr", ..),  ime @ Ime(..), Operator("=", ..), ostanek @ .. ] => self.inicializacija(ime, None, ostanek, true),
            [ Rezerviranka("naj", ..),  ime @ Ime(..), Operator("=", ..), ostanek @ .. ] => self.inicializacija(ime, None, ostanek, false),
            [ Rezerviranka("kons", ..), ime @ Ime(..), Operator("=", ..), ostanek @ .. ] => self.konstanta(ime, None, ostanek),
            // prirejanje referenci
            [ ime @ Ime(..), Operator("@", ..), Operator("=", ..), ostanek @ .. ] => self.prirejanje_ref(ime, ostanek),
            // prirejanje
            [ ime @ Ime(..), Operator("=", ..), ostanek @ .. ] => self.prirejanje(ime, ostanek),
            // kombinirano prirejanje referenci (+=, -=, *= ...)
            [ ime @ Ime(..), Operator("@", ..), operator @ Operator(op, ..), ostanek @ .. ] => {
                match prireditveni_op(op) {
                    Brez => Err(Napake::from_zaporedje(izraz, E1, "Neznan izraz")),
                    _ => self.kombinirano_prirejanje_ref(ime, operator, ostanek),
                }
            },
            // kombinirano prirejanje (+=, -=, *= ...)
            [ ime @ Ime(..), operator @ Operator(op, ..), ostanek @ .. ] => {
                match prireditveni_op(op) {
                    Brez => Err(Napake::from_zaporedje(izraz, E1, "Neznan izraz")),
                    _ => self.kombinirano_prirejanje(ime, operator, ostanek),
                }
            },
            // prirejanje elementu seznama
            [ ime @ Ime(..), Ločilo("[", ..), ostanek @ .. ] => {
                let (indeks, _, ostanek) = loči_spredaj(ostanek, &["]"])
                    .ok_or(Napake::from_zaporedje(ostanek, E5, "Pričakovan ']'"))??;
                let (prazno, _, izraz) = loči_spredaj(ostanek, &["="])
                    .ok_or(Napake::from_zaporedje(ostanek, E5, "Pričakovan '='"))??;
                
                match prazno {
                    [] => self.prirejanje_seznamu(ime, indeks, izraz),
                    _ => Err(Napake::from_zaporedje(prazno, E5, "Pričakovan '='")),
                }
            },
            // okvir
            [ Ločilo("{", ..), vmes @ .., Ločilo("}", ..) ] => self.okvir(vmes),
            // funkcija natisni (zaenkrat še posebna funkcija)
            [ ime @ Ime("natisni", ..), Ločilo("(", ..), argumenti @ .., Ločilo(")", ..) ] => self.natisni(ime, argumenti),
            // funkcija natisni (zaenkrat še posebna funkcija)
            [ Ime("splakni", ..), Ločilo("(", ..), Ločilo(")", ..) ] => Ok(Splakni.rc()),
            // funkcijski klic
            [ ime @ Ime(..), Ločilo("(", ..), argumenti @ .., Ločilo(")", ..) ] => self.funkcijski_klic_zavrzi_izhod(ime, argumenti),
            // pogojni stavek
            [ Rezerviranka("če", ..), ostanek @ .. ] => self.pogojni_stavek(ostanek),
            // zanka dokler (while loop)
            [ Rezerviranka("dokler", ..), ostanek @ .. ] => self.zanka_dokler(ostanek),
            // zanka za (for loop)
            [ Rezerviranka("za", ..), ostanek @ .. ] => self.zanka_za(ostanek),
            // deklaracija funkcije
            [ Rezerviranka("funkcija", ..), ime @ Ime(..), ostanek @ .. ] => self.funkcija(ime, ostanek),
            // vrni (return)
            [ vrni @ Rezerviranka("vrni", ..), ostanek @ .. ] => self.vrni(vrni, ostanek),
            // prazen stavek
            [  ] => Ok(Prazno.rc()),
            // neznan stavek (noben od zgornjih)
            _ => Err(Napake::from_zaporedje(izraz, E1, "Neznan izraz")),
        }
    }

    fn deklaracija(&mut self, ime: &Žeton<'a>, tip: &[Žeton]) -> Result<Rc<Vozlišče>, Napake> {
        let tip = Tip::from(tip, &self.konstante)?;
        let spremenljivka = match self.spremenljivke.get(ime.as_str()) {
            Some(_) => Err(Napake::from_zaporedje(&[*ime], E2, "Spremenljivka že obstaja")),
            None => Ok(self.dodaj_spremenljivko(ime.as_str(), tip.clone(), true)),
        }?;
        
        match tip {
            Tip::Seznam(_, dolžina) => Ok(Prirejanje { spremenljivka, izraz: Celo(dolžina).rc() }.rc()),
            _ => Ok(Prazno.rc()),
        }
    }

    fn inicializacija(&mut self, ime: &Žeton<'a>, tip_izraza: Option<&[Žeton]>, izraz: &[Žeton<'a>], spremenljiva: bool) -> Result<Rc<Vozlišče>, Napake> {
        let izraz = self.drevo(izraz)?;
        let tip_spr = match tip_izraza {
            Some(tip) => Tip::from(tip, &self.konstante)?,
            None => izraz.tip(),
        };
        let spremenljivka = match self.spremenljivke.get(ime.as_str()) {
            Some(_) => Err(Napake::from_zaporedje(&[*ime], E2, "Spremenljivka že obstaja")),
            None => Ok(self.dodaj_spremenljivko(ime.as_str(), tip_spr.clone(), spremenljiva))
        }?;

        if tip_spr == izraz.tip() {
            Ok(Prirejanje { spremenljivka, izraz }.rc())
        }
        else {
            Err(Napake::from_zaporedje(tip_izraza.unwrap(), E3,
                    &format!("Izraza tipa '{}' ni mogoče prirediti spremenljivki tipa '{}'", izraz.tip(), tip_spr)))
        }
    }

    fn konstanta(&mut self, ime: &Žeton<'a>, tip_izraza: Option<&[Žeton]>, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let drevo = self.drevo(izraz)?;
        let tip_kons = match tip_izraza {
            Some(tip) => Tip::from(tip, &self.konstante)?,
            None => drevo.tip(),
        };
        match self.konstante.get(ime.as_str()) {
            Some(_) => Err(Napake::from_zaporedje(&[*ime], E2, "Konstanta že obstaja")),
            None => Ok(self.dodaj_konstanto(ime.to_string(), drevo.eval(izraz)?.rc()))
        }?;

        if tip_kons == drevo.tip() {
            Ok(Prazno.rc())
        }
        else {
            Err(Napake::from_zaporedje(tip_izraza.unwrap(), E3,
                    &format!("Izraza tipa '{}' ni mogoče prirediti konstanti tipa '{}'", drevo.tip(), tip_kons)))
        }
    }

    fn prirejanje(&mut self, ime: &Žeton<'a>, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let izraz = self.drevo(izraz)?;
        let spremenljivka = self.poišči_spr(ime)?;

        if let Spremenljivka { spremenljiva, .. } = &*spremenljivka {
            if !spremenljiva {
                return Err(Napake::from_zaporedje(&[*ime], E3,
                        &format!("Ne morem prirediti {}: ni spremenljiva vrednost", ime.as_str())));
            }
        }

        if izraz.tip() != spremenljivka.tip() {
            return Err(Napake::from_zaporedje(&[*ime], E3,
                    &format!("Nemogoča operacija: {} = {}", spremenljivka.tip(), izraz.tip())));
        }

        Ok(Prirejanje { spremenljivka, izraz }.rc())
    }

    fn prirejanje_ref(&mut self, ime: &Žeton<'a>, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let izraz = self.drevo(izraz)?;
        let referenca = self.poišči_spr(ime)?;

        match referenca.tip() {
            Tip::Referenca(tip) => {
                if *tip == izraz.tip() {
                    Ok(PrirejanjeRef { referenca, izraz, indeks: None }.rc())
                }
                else {
                    Err(Napake::from_zaporedje(&[*ime], E3,
                            &format!("Nemogoča operacija: {} = {}", *tip, izraz.tip())))
                }
            },
            tip @ _ => Err(Napake::from_zaporedje(&[*ime], E2, 
                    &format!("V spremenljivko tipa '{}' ni mogoče Dereferencirati.", tip))),
        }

    }

    fn prirejanje_seznamu(&mut self, ime: &Žeton<'a>, indeks: &[Žeton<'a>], izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let izraz = self.drevo(izraz)?;
        let indeks = Some(self.drevo(indeks)?);
        let tip_indeksa = indeks.clone().unwrap().tip();
        let spr = self.poišči_spr(ime)?;

        if let Tip::Seznam(tip, _) | Tip::RefSeznama(tip) = spr.tip() {
            if *tip != izraz.tip() {
                return Err(Napake::from_zaporedje(&[*ime], E3,
                        &format!("Nemogoča operacija: {} = {}", *tip, izraz.tip())));
            }
        }
        if tip_indeksa != Tip::Celo {
            return Err(Napake::from_zaporedje(&[*ime], E3,
                    &format!("Neveljaven tip indeksa: '{}'", tip_indeksa)));
        }

        match spr.tip() {
            Tip::RefSeznama(..) => Ok(PrirejanjeRef { referenca: spr, indeks, izraz }.rc()),
            Tip::Seznam(..) => Ok(PrirejanjeRef { referenca: RefSeznama(spr).rc(), indeks, izraz }.rc()),
            tip @ _ => Err(Napake::from_zaporedje(&[*ime], E2, 
                    &format!("V spremenljivko tipa '{}' ni mogoče indeksirati.", tip))),
        }
    }

    fn kombinirano_prirejanje(&mut self, ime: &Žeton, operator: &Žeton, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let spremenljivka = self.poišči_spr(ime)?;

        if let Spremenljivka { spremenljiva, .. } = &*spremenljivka {
            if !spremenljiva {
                return Err(Napake::from_zaporedje(&[*ime], E3,
                        &format!("Ne morem prirediti {}: ni spremenljiva vrednost", ime.as_str())));
            }
        }

        let drevo = self.drevo(izraz)?;
        let izraz = Self::prirejanje_v_kombinirano(spremenljivka.clone(), operator, drevo)?;

        Ok(Prirejanje { spremenljivka, izraz }.rc())
    }

    fn kombinirano_prirejanje_ref(&mut self, ime: &Žeton, operator: &Žeton, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let referenca = self.poišči_spr(ime)?;
        let spremenljivka = match referenca.tip() {
            Tip::Referenca(_) => Dereferenciraj(referenca.clone()).rc(),
            _ => Err(Napake::from_zaporedje(&[*ime, *operator], E3, "Dereferencirati je mogoče samo referenco."))?
        };
        let drevo = self.drevo(izraz)?;
        let izraz = Self::prirejanje_v_kombinirano(spremenljivka, operator, drevo)?;

        Ok(PrirejanjeRef { referenca, izraz, indeks: None }.rc())
    }

    fn prirejanje_v_kombinirano(spremenljivka: Rc<Vozlišče>, operator: &Žeton, drevo: Rc<Vozlišče>) -> Result<Rc<Vozlišče>, Napake> {
        match prireditveni_op(operator.as_str()) {
            Aritmetični(op) => match (spremenljivka.tip(), drevo.tip()) {
                (Tip::Celo, Tip::Celo) => Ok(op(Tip::Celo, spremenljivka, drevo).rc()),
                (Tip::Real, Tip::Real) => Ok(op(Tip::Real, spremenljivka, drevo).rc()),
                _ => Err(Napake::from_zaporedje(&[*operator], E3,
                        &format!("Nemogoča operacija: {} {} {}", spremenljivka.tip(), operator.as_str(), drevo.tip()))),
            },
            Logični(op) => match (spremenljivka.tip(), drevo.tip()) {
                (Tip::Bool, Tip::Bool) => Ok(op(spremenljivka, drevo).rc()),
                _ => Err(Napake::from_zaporedje(&[*operator], E3,
                        &format!("Nemogoča operacija: {} {} {}", spremenljivka.tip(), operator.as_str(), drevo.tip()))),
            }
            Bitni(op) => match (spremenljivka.tip(), drevo.tip()) {
                (Tip::Celo, Tip::Celo) => Ok(op(spremenljivka, drevo).rc()),
                _ => Err(Napake::from_zaporedje(&[*operator], E3,
                        &format!("Nemogoča operacija: {} {} {}", spremenljivka.tip(), operator.as_str(), drevo.tip()))),
            }
            Brez => Err(Napake::from_zaporedje(&[*operator], E4, "Neznan operator"))
        }
    }

    fn natisni(&mut self, ime: &Žeton, argumenti_izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let Argumenti { tipi, argumenti, .. } = self.argumenti(argumenti_izraz)?;

        match tipi.as_slice() {
            [ Tip::Znak ] => Ok(Natisni(argumenti[0].clone()).rc()),
            _ => self.funkcijski_klic(ime, argumenti_izraz)
        }
    }

    fn vrni(&mut self, vrni: &Žeton, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let drevo = self.drevo(izraz)?;
        let spremenljivka = self.spremenljivke.get("0_vrni")
            .ok_or(Napake::from_zaporedje(&[*vrni], E5, "nepričakovana beseda: 'vrni', uporabljena zunaj funkcije"))?.clone();

        if drevo.tip() != spremenljivka.tip() {
            return Err(Napake::from_zaporedje(izraz, E3, &format!("Ne morem vrniti spremenljivke tipa '{}' iz funkcije tipa '{}'", drevo.tip(), spremenljivka.tip())));
        }

        Ok(Vrni(Prirejanje { spremenljivka, izraz: drevo }.rc()).rc())
    }

    fn pogojni_stavek(&mut self, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let (pogoj, _, izraz) = loči_spredaj(izraz, &["{"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '{'"))??;

        let (resnica, _, izraz) = loči_spredaj(izraz, &["}"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '}'"))??;

        let laž = match loči_spredaj(izraz, &["čene"]) {
            Some(Ok((_, _, d))) => match d {
                [ Rezerviranka("če", ..), .. ] | [ Ločilo("{", ..), .. ]  => Ok(d),
                _ => Err(Napake::from_zaporedje(d, E5, "Pričakovan 'čene' ali '{'"))
            },
            Some(Err(napaka)) => Err(napaka),
            None => Ok([].as_slice()),
        }?;

        let drevo = self.drevo(pogoj)?;
        if drevo.tip() != Tip::Bool {
            return Err(Napake::from_zaporedje(pogoj, E6, "Pogoj mora biti Boolova vrednost"))
        }

        Ok(PogojniStavek {
            pogoj:   drevo,
            resnica: self.okvir(resnica)?,
            laž:     self.stavek(laž)?,
        }.rc())
    }

    fn zanka_dokler(&mut self, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let (pogoj_izraz, _, izraz) = loči_spredaj(izraz, &["{"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '{'"))??;

        let (telo_izraz, _, _) = loči_zadaj(izraz, &["}"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '}'"))??;

        let pogoj = self.drevo(pogoj_izraz)?;
        if pogoj.tip() != Tip::Bool {
            return Err(Napake::from_zaporedje(pogoj_izraz, E6, "Pogoj mora biti Boolova vrednost"));
        }

        self.v_okvir();
        // {
        let telo = self.zaporedje(telo_izraz)?;
        // }
        self.iz_okvirja();
        Ok(Zanka { pogoj, telo }.rc())
    }

    fn zanka_za(&mut self, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let (prirejanje_izraz, _, izraz) = loči_spredaj(izraz, &[","])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan ','"))??;

        let (pogoj_izraz, _, izraz) = loči_spredaj(izraz, &[","])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan ','"))??;

        let (za_izraz, _, izraz) = loči_spredaj(izraz, &["{"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '{'"))??;

        let (telo_izraz, _, _) = loči_zadaj(izraz, &["}"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '}'"))??;

        let telo_izraz = [
            telo_izraz,
            [Ločilo(";", 1, 1, "")].as_slice(),
            za_izraz,
        ].concat();

        self.v_okvir();
        // {
        let prirejanje = match loči_spredaj(prirejanje_izraz, &["="]) {
            Some(Ok(([ime @ Ime(..)], _, izraz))) => self.inicializacija(ime, None, izraz, true),
            Some(Ok(_)) => Err(Napake::from_zaporedje(prirejanje_izraz, E5, "Pričakovan '='")),
            Some(Err(err)) => Err(err),
            None => Ok(Prazno.rc()),
        }?;
        let pogoj = self.drevo(pogoj_izraz)?;
        let telo = self.okvir(telo_izraz.as_slice())?;
        // }
        self.iz_okvirja();
        
        let zaporedje = Zaporedje(vec![
            prirejanje,
            Zanka { pogoj, telo }.rc(),
        ]).rc();

        Ok(zaporedje)
    }

}
