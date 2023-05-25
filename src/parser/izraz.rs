use super::*;

impl<'a> Parser<'a> {
    pub fn drevo(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        self.logični(izraz)
    }

    // logični izrazi (razen negacije, ki je pri osnovnih)
    fn logični(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, &["||"]) {
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.logični(l_izraz)?;
                let d = self.logični(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Bool, Tip::Bool) => Ok(Disjunkcija(l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => match loči_zadaj(izraz, &["&&"]) {
                Some(Ok((l_izraz, op, d_izraz))) => {
                    let l = self.logični(l_izraz)?;
                    let d = self.logični(d_izraz)?;
                    match (l.tip(), d.tip()) {
                        (Tip::Bool, Tip::Bool) => Ok(Konjunkcija(l, d).rc()),
                        _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                    }
                },
                Some(Err(napaka)) => Err(napaka),
                None => self.bitni(izraz),
            }
        }
    }

    // izrazi bitne manipulacije
    fn bitni(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, &["|"]) {
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.bitni(l_izraz)?;
                let d = self.bitni(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => Ok(bitni_op(op.as_str())(l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => match loči_zadaj(izraz, &["^"]) {
                Some(Ok((l_izraz, op, d_izraz))) => {
                    let l = self.bitni(l_izraz)?;
                    let d = self.bitni(d_izraz)?;
                    match (l.tip(), d.tip()) {
                        (Tip::Celo, Tip::Celo) => Ok(bitni_op(op.as_str())(l, d).rc()),
                        _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                    }
                },
                Some(Err(napaka)) => Err(napaka),
                None => match loči_zadaj(izraz, &["&"]) {
                    Some(Ok((l_izraz, op, d_izraz))) => {
                        let l = self.bitni(l_izraz)?;
                        let d = self.bitni(d_izraz)?;
                        match (l.tip(), d.tip()) {
                            (Tip::Celo, Tip::Celo) => Ok(bitni_op(op.as_str())(l, d).rc()),
                            _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                        }
                    },
                    Some(Err(napaka)) => Err(napaka),
                    None => match loči_zadaj(izraz, &["<<", ">>"]) {
                        Some(Ok((l_izraz, op, d_izraz))) => {
                            let l = self.bitni(l_izraz)?;
                            let d = self.bitni(d_izraz)?;
                            match (l.tip(), d.tip()) {
                                (Tip::Celo, Tip::Celo) => Ok(bitni_op(op.as_str())(l, d).rc()),
                                _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Neveljavna tipa za operacijo: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                            }
                        },
                        Some(Err(napaka)) => Err(napaka),
                        None => self.primerjalni(izraz),
                    }
                }
            }
        }
    }

    // primerjalni izrazi
    pub fn primerjalni(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, PRIMERJALNI_OP.as_slice()) {
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.primerjalni(l_izraz)?;
                let d = self.primerjalni(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => Ok(primerjalni_op(op.as_str()).unwrap()(Tip::Celo, l, d).rc()),
                    (Tip::Znak, Tip::Znak) => Ok(primerjalni_op(op.as_str()).unwrap()(Tip::Celo, ZnakVCelo(l).rc(), ZnakVCelo(d).rc()).rc()),
                    (Tip::Real, Tip::Real) => Ok(primerjalni_op(op.as_str()).unwrap()(Tip::Real, l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Nemogoča operacija: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => self.aditivni(izraz)
        }
    }

    // aritmetični izrazi

    fn aditivni(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, &["+", "-"]) {
            // "-" kot unarni operator
            Some(Ok(([], Operator("-", ..), ..))) => self.aritmetični(izraz),
            Some(Ok((pred @ [.., Operator(..)], minus @ Operator("-", ..), za @ [..]))) =>
                self.aditivni([pred, [Ločilo("(", 0, 0), *minus].as_slice(), za, [Ločilo(")", 0, 0)].as_slice()].concat().as_slice()),

            // "-" kot binarni operator
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.aritmetični(l_izraz)?;
                let d = self.aritmetični(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => Ok(aritmetični_op(op.as_str())(Tip::Celo, l, d).rc()),
                    (Tip::Real, Tip::Real) => Ok(aritmetični_op(op.as_str())(Tip::Real, l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Nemogoča operacija: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => self.aritmetični(izraz),
        }
    }

    fn aritmetični(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match loči_zadaj(izraz, &["*", "/", "%"]) {
            Some(Ok((l_izraz, op, d_izraz))) => {
                let l = self.aritmetični(l_izraz)?;
                let d = self.aritmetični(d_izraz)?;
                match (l.tip(), d.tip()) {
                    (Tip::Celo, Tip::Celo) => Ok(aritmetični_op(op.as_str())(Tip::Celo, l, d).rc()),
                    (Tip::Real, Tip::Real) => Ok(aritmetični_op(op.as_str())(Tip::Real, l, d).rc()),
                    _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Nemogoča operacija: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                }
            },
            Some(Err(napaka)) => Err(napaka),
            None => match loči_zadaj(izraz, &["**"]) {
                Some(Ok((l_izraz, op, d_izraz))) => {
                    let l = self.aritmetični(l_izraz)?;
                    let d = self.aritmetični(d_izraz)?;
                    match (l.tip(), d.tip()) {
                        (Tip::Celo, Tip::Celo) => Ok(aritmetični_op(op.as_str())(Tip::Celo, l, d).rc()),
                        (Tip::Real, Tip::Real) => Ok(aritmetični_op(op.as_str())(Tip::Real, l, d).rc()),
                        _ => Err(Napake::from_zaporedje(&[*op], E5, &format!("Nemogoča operacija: {} {} {}", l.tip(), op.as_str(), d.tip()))),
                    }
                },
                Some(Err(napaka)) => Err(napaka),
                None => self.osnovni(izraz),
            }
        }
    }

    pub fn osnovni(&mut self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        match izraz {
            // bool
            [ Literal(L::Bool("resnica", ..)) ] => Ok(Resnica.rc()),
            [ Literal(L::Bool("laž", ..)) ] => Ok(Laž.rc()),
            // števila
            [ Literal(L::Celo(število, ..)) ] => Ok(Vozlišče::Celo(število.replace("_", "").parse().unwrap()).rc()),
            [ Literal(L::Real(število, ..)) ] => Ok(Vozlišče::Real(število.replace("_", "").parse().unwrap()).rc()),
            [ Operator("-", ..), Literal(L::Celo(str, ..)) ] => Ok(Vozlišče::Celo(-str.replace("_", "").parse::<i32>().unwrap()).rc()),
            [ Operator("-", ..), Literal(L::Real(str, ..)) ] => Ok(Vozlišče::Real(-str.replace("_", "").parse::<f32>().unwrap()).rc()),
            // znak
            [ Literal(L::Znak(str, ..)) ] => Ok(Vozlišče::Znak(interpoliraj_niz(&str[1..str.len()-1]).chars().nth(0).unwrap()).rc()),
            // niz
            [ Literal(L::Niz(niz, ..)) ] => Ok(Vozlišče::Niz(interpoliraj_niz(&niz[1..niz.len()-1])).rc()),
            // izraz v oklepaju
            [ Ločilo("(", ..), ostanek @ .., Ločilo(")", ..) ] => self.drevo(ostanek),
            // funkcija asm(str)
            [ Ime("asm", ..), Ločilo("(", ..), argumenti @ .., Ločilo(")", ..) ] => self.asm(argumenti),
            // klic funkcije
            [ ime @ Ime(..), Ločilo("(", ..), argumenti @ .., Ločilo(")", ..) ] => self.funkcijski_klic(ime, argumenti),
            // pretvorba tipa 
            [ izraz @ .., Operator("kot", ..), tip @ Tip(..) ] => self.pretvorba(izraz, tip),
            // zanikanje
            [ Operator("!", ..), ostanek @ .. ] => {
                let drevo = self.drevo(ostanek)?;
                match drevo.tip() {
                    Tip::Bool => Ok(Zanikaj(drevo).rc()),
                    _ => Err(Napake::from_zaporedje(izraz, E9, "Zanikati je mogoče samo Boolove vrednosti"))
                }
            },
            // negacija
            [ Operator("-", ..), ostanek @ .. ] => {
                let drevo = self.drevo(ostanek)?;
                match drevo.tip() {
                    Tip::Celo => Ok(Odštevanje(Tip::Celo, Celo(0).rc(), drevo).rc()),
                    Tip::Real => Ok(Odštevanje(Tip::Real, Celo(0).rc(), drevo).rc()),
                    _ => Err(Napake::from_zaporedje(ostanek, E9, "Izraza ni mogoče negirati"))
                }
            },
            // spremenljivka
            [ ime @ Ime(..) ] => Ok(self.spremenljivke.get(ime.as_str())
                                    .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?.clone()),

            // referenciraj
            [ Operator("@", ..), ime @ Ime(..) ] => {
                let spremenljivka = self.spremenljivke.get(ime.as_str())
                        .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?.clone();

                match spremenljivka.tip() {
                    Tip::Seznam(..) => Ok(RefSeznama(spremenljivka).rc()),
                    _ => Ok(Referenca(spremenljivka).rc())
                }
            }

            // dereferenciraj
            deref @ [ ime @ Ime(..), Operator("@", ..) ] => {
                let referenca = self.spremenljivke.get(ime.as_str())
                    .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?;

                match &**referenca {
                    Spremenljivka { tip, .. } => match &*tip {
                        Tip::Referenca(..) => Ok(Dereferenciraj(referenca.clone()).rc()),
                        _ => Err(Napake::from_zaporedje(deref, E2, "Dereferenciramo lahko samo referenco.")),
                    },
                    _ => Err(Napake::from_zaporedje(deref, E2, "Dereferenciramo lahko samo spremenljivko.")),
                }
            }

            // indeksiraj
            [ ime @ Ime(..), Ločilo("[", ..), indeks @ .., Ločilo("]", ..) ] => {
                let indeks = self.drevo(indeks)?.rc();
                let spremenljivka = self.spremenljivke.get(ime.as_str())
                    .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?;

                match &**spremenljivka {
                    Spremenljivka { tip: Tip::Seznam(..), .. } =>
                        Ok(Indeksiraj{ seznam_ref: RefSeznama(spremenljivka.clone()).rc(), indeks }.rc()),
                    Spremenljivka { tip: Tip::RefSeznama(..), .. } =>
                        Ok(Indeksiraj{ seznam_ref: spremenljivka.clone(), indeks }.rc()),
                    _ => Err(Napake::from_zaporedje(izraz, E2, 
                            &format!("V spremenljivko tipa '{}' ni mogoče indeksirati.", spremenljivka.tip()))),
                }
            }

            [ ime @ Ime(..), Ločilo(".", ..), Ime("dolžina", ..) ] => {
                let spremenljivka = self.spremenljivke.get(ime.as_str())
                    .ok_or(Napake::from_zaporedje(&[*ime], E2, "Neznana spremenljivka"))?;

                match spremenljivka.tip() {
                    Tip::Seznam(..) => Ok(Dolžina(spremenljivka.clone()).rc()),
                    Tip::RefSeznama(..) => Ok(Dolžina(spremenljivka.clone()).rc()),
                    _ => Err(Napake::from_zaporedje(&[*ime], E2, 
                            &format!("Tip '{}' nima dolžine", spremenljivka.tip())))
                }
            },

            [ neznano @ Neznano(..) ] => Err(Napake::from_zaporedje(&[*neznano], E1, "Neznana beseda")),
            [] => Ok(Prazno.rc()),
            _ => Err(Napake::from_zaporedje(izraz, E1,
                    &format!("Neznan izraz: {}", izraz.iter().map(|t| t.as_str()).collect::<Vec<&str>>().join(" "))))
        }
    }

    fn pretvorba(&mut self, izraz: &[Token<'a>], tip_ven_izraz: &Token) -> Result<Rc<Vozlišče>, Napake> {
        let drevo = self.drevo(izraz)?.rc();
        let tip_noter = drevo.tip();
        let tip_ven = Tip::from(&[*tip_ven_izraz])?;

        match (tip_noter.clone(), tip_ven.clone()) {
            (Tip::Real, Tip::Celo) => Ok(RealVCelo(drevo).rc()),
            (Tip::Celo, Tip::Real) => Ok(CeloVReal(drevo).rc()),
            (Tip::Celo, Tip::Znak) => Ok(CeloVZnak(drevo).rc()),
            (Tip::Znak, Tip::Celo) => Ok(ZnakVCelo(drevo).rc()),
            (a, b) if a == b => Ok(drevo),
            _ => Err(Napake::from_zaporedje(&[*tip_ven_izraz], E1,
                    &format!("Tipa {} ni mogoče pretvoriti v {}", tip_noter, tip_ven)))
        }
    }

    fn asm(&self, izraz: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let niz = match izraz {
            [ Literal(L::Niz(niz, ..)) ] => &niz[1..niz.len()-1],
            _ => Err(Napake::from_zaporedje(izraz, E5, "Funkcija 'asm' sprejema samo nize"))?,
        };

        match niz {
            "GETC" => Ok(Preberi.rc()),
            _ => Err(Napake::from_zaporedje(izraz, E1,
                    &format!("Neznan ukaz: {niz}")))
        }
    }
}

#[cfg(test)]
mod testi {
    use std::rc::Rc;

    use crate::parser::tokenizer::Tokenize;

    use super::*;

    #[test]
    fn osnovni() {
        let mut parser = Parser::new();
        assert_eq!(parser.osnovni([ Literal(L::Bool("resnica", 1, 1))].as_slice()).unwrap(), Resnica.rc());
        assert_eq!(parser.osnovni([ Literal(L::Bool("laž", 1, 1))].as_slice()).unwrap(), Laž.rc());
        assert_eq!(parser.osnovni([ Operator("!", 1, 1), Literal(L::Bool("laž", 1, 2))].as_slice()).unwrap(), Zanikaj(Laž.rc()).rc());
        assert_eq!(parser.osnovni([ Ločilo("(", 1, 1), Literal(L::Bool("laž", 1, 2)), Ločilo(")", 1, 5)].as_slice()).unwrap(), Laž.rc());
        assert_eq!(parser.osnovni([ Literal(L::Celo("3", 1, 1))].as_slice()).unwrap(), Celo(3).rc());
        assert_eq!(parser.osnovni([ Literal(L::Real("3.125", 1, 1))].as_slice()).unwrap(), Real(3.125).rc());
        assert_eq!(parser.osnovni([ Literal(L::Celo("1_000", 1, 1))].as_slice()).unwrap(), Celo(1000).rc());
        assert_eq!(parser.osnovni([ Literal(L::Znak("'đ'", 1, 1))].as_slice()).unwrap(), Znak('đ').rc());
        assert_eq!(parser.osnovni([ Literal(L::Niz("\"angleščina\\n\"", 1, 1))].as_slice()).unwrap(), Niz("angleščina\n".to_string()).rc());

        parser.funkcije.insert("fun()".to_string(), Funkcija {
                tip: Tip::Real,
                ime: "fun".to_string(),
                parametri: vec![],
                telo: Zaporedje(vec![
                                Vrni(Prirejanje {
                                    spremenljivka: Spremenljivka { tip: Tip::Real, ime: "vrni".to_string(), naslov: 0, z_odmikom: true }.rc(),
                                    izraz: Real(1.0).rc(),
                                }.rc()).rc()
                ]).rc(),
                prostor: 0,
            }.rc());
        assert_eq!(parser.osnovni([ Ime("fun", 1, 1), Ločilo("(", 1, 4), Ločilo(")", 1, 5)].as_slice()).unwrap(), FunkcijskiKlic { 
            funkcija: parser.funkcije["fun()"].clone(),
            spremenljivke: Zaporedje(vec![]).rc(),
            argumenti: Zaporedje([].to_vec()).rc(),
        }.rc());

        parser.spremenljivke.insert("a".to_string(), Rc::new(Spremenljivka { tip: Tip::Celo, ime: "a".to_string(), naslov: 0, z_odmikom: false }));
        assert_eq!(parser.osnovni([ Ime("a", 1, 1)].as_slice()).unwrap(), parser.spremenljivke["a"].clone());
    }

    #[test]
    fn pretvorba() {
        let mut parser = Parser::new();
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("kot", 1, 3), Tip("real", 1, 7) ].as_slice()).unwrap(),
            CeloVReal(Celo(3).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Real("3.0", 1, 1)), Operator("kot", 1, 3), Tip("celo", 1, 7) ].as_slice()).unwrap(),
            RealVCelo(Real(3.0).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Real("3.0", 1, 1)), Operator("kot", 1, 3), Tip("real", 1, 7) ].as_slice()).unwrap(),
            Real(3.0).rc());
    }

    #[test]
    fn aritmetični() {
        let mut parser = Parser::new();
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("+", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Seštevanje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("-", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Odštevanje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("*", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Množenje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("/", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Deljenje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("%", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Modulo(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo([ Literal(L::Celo("3", 1, 1)), Operator("**", 1, 2), Literal(L::Celo("2", 1, 4)) ].as_slice()).unwrap(),
            Potenca(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());

        assert_eq!(parser.drevo("-(3-4)".tokenize().as_slice()).unwrap(), Odštevanje(Tip::Celo, Celo(0).rc(), Odštevanje(Tip::Celo, Celo(3).rc(), Celo(4).rc()).rc()).rc());
        assert_eq!(parser.drevo("-3".tokenize().as_slice()).unwrap(), Celo(-3).rc());
        assert_eq!(parser.drevo("-3 * 2".tokenize().as_slice()).unwrap(), Množenje(Tip::Celo, Celo(-3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.drevo("3 * -2".tokenize().as_slice()).unwrap(), Množenje(Tip::Celo, Celo(3).rc(), Celo(-2).rc()).rc());
        assert_eq!(parser.drevo("--1".tokenize().as_slice()).unwrap(), Odštevanje(Tip::Celo, Celo(0).rc(), Celo(-1).rc()).rc());
        assert_eq!(parser.drevo("2 + -1".tokenize().as_slice()).unwrap(), Seštevanje(Tip::Celo, Celo(2).rc(), Celo(-1).rc()).rc());
    }

    #[test]
    fn primerjalni() {
        let mut parser = Parser::new();
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("==", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Enako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("!=", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            NiEnako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("<=", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            ManjšeEnako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator(">=", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            VečjeEnako(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator("<", 1, 2), Literal(L::Celo("2", 1, 3)) ].as_slice()).unwrap(),
            Manjše(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
        assert_eq!(parser.primerjalni([ Literal(L::Celo("3", 1, 1)), Operator(">", 1, 2), Literal(L::Celo("2", 1, 4)) ].as_slice()).unwrap(),
            Večje(Tip::Celo, Celo(3).rc(), Celo(2).rc()).rc());
    }
}
