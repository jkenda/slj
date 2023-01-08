use super::*;

use std::iter;

const NIČ: Podatek = Podatek { i: 0 };

impl Prevedi for Drevo {
    fn prevedi(&self) -> Vec<UkazPodatekRelative> {
        self.root.prevedi()
    }

    fn len(&self) -> usize {
        self.root.len()
    }
}

impl Prevedi for Vozlišče {
    fn prevedi(&self) -> Vec<UkazPodatekRelative> {
        match self {
            Prazno => [].to_vec(),

            Push(krat) => iter::repeat(Osnovni(PUSH(NIČ))).take(*krat).collect(),
            Pop(krat) => iter::repeat(Osnovni(POP).clone()).take(*krat).collect(),
            Vrh(odmik) => [Osnovni(TOP(*odmik as i32))].to_vec(),

            ShraniOdmik => [Osnovni(SOFF)].to_vec(),
            NaložiOdmik => [Osnovni(LOFF)].to_vec(),

            Niz(niz) => niz
                .chars().rev()
                .map(|znak| Osnovni(PUSH(Podatek { c: znak })))
                .collect::<Vec<UkazPodatekRelative>>(),
            Število(število) => [Osnovni(PUSH(Podatek { f: *število }))].to_vec(),
            Spremenljivka{ ime: _, naslov, z_odmikom } => [if *z_odmikom { Osnovni(LDOF(*naslov)) } else { Osnovni(LOAD(*naslov)) }].to_vec(),

            Resnica => [Osnovni(PUSH(RESNICA))].to_vec(),
            Laž     => [Osnovni(PUSH(LAŽ))].to_vec(),

            Seštevanje(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(ADD)].as_slice(),
            ].concat(),
            Odštevanje(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(SUB)].as_slice(),
            ].concat(),
            Množenje(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(MUL)].as_slice(),
            ].concat(),
            Deljenje(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(DIV)].as_slice(),
            ].concat(),
            Modulo(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(MOD)].as_slice(),
            ].concat(),
            Potenca(l, d) => [
                d.prevedi().as_slice(),
                l.prevedi().as_slice(),
                [Osnovni(POW)].as_slice(),
            ].concat(),

            Zanikaj(vozlišče) => Odštevanje(Število(1.0).rc(), vozlišče.clone()).prevedi(),
            Konjunkcija(l, d) => Množenje(l.clone(), d.clone()).prevedi(),
            Disjunkcija(l, d) => [
                Seštevanje(l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),

            Enako(l, d) => [
                Odštevanje(l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(ZERO)].as_slice(),
            ].concat(),
            NiEnako(l, d) => Zanikaj(Enako(l.clone(), d.clone()).rc()).prevedi(),

            Večje(l, d) => [
                Odštevanje(l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),
            Manjše(l, d)      => Večje(d.clone(), l.clone()).prevedi(),
            VečjeEnako(l, d)  => Zanikaj(Manjše(l.clone(), d.clone()).rc()).prevedi(),
            ManjšeEnako(l, d) => VečjeEnako(d.clone(), l.clone()).prevedi(),

            ProgramskiŠtevec(odmik) => [PC(*odmik)].to_vec(),

            Skok(odmik_ime) => [JUMPRelative(odmik_ime.clone())].to_vec(),
            DinamičniSkok => [Osnovni(JMPD).to_owned()].to_vec(),
            PogojniSkok(pogoj, skok) => [
                pogoj.prevedi().as_slice(),
                [JMPCRelative(*skok)].as_slice(),
            ].concat(),

            PogojniStavek{ pogoj, resnica, laž } => {
                let skok = Skok(OdmikIme::Odmik((resnica.len() + 1) as isize)).rc();
                Zaporedje(vec![
                          PogojniSkok(pogoj.clone(), (laž.len() + skok.len() + 1) as i32).rc(),
                          laž.clone(),
                          skok,
                          resnica.clone(),
                ]).prevedi()
            },

            Zanka { pogoj, telo } => {
                let pogoj = Zanikaj(pogoj.clone()).rc();
                let pogoj_len = pogoj.len();
                Zaporedje(vec![
                          PogojniSkok(
                              pogoj,
                              (telo.len() + 2) as i32).rc(),
                          telo.clone(),
                          Skok(OdmikIme::Odmik(-(telo.len() as isize) - pogoj_len as isize - 1)).rc()
                ]).prevedi()
            },

            Prirejanje{ spremenljivka, izraz } => {
                let (naslov, z_odmikom) = if let Spremenljivka { ime: _, naslov, z_odmikom } = &**spremenljivka { (naslov.clone(), *z_odmikom) } else { (0, false) };
                let shrani = if z_odmikom { Osnovni(STOF(naslov)) } else { Osnovni(STOR(naslov)) };
                [
                    izraz.clone().prevedi().as_slice(),
                    [shrani].as_slice()
                ].concat()
            },

            Vrni(prirejanje) => [
                prirejanje.prevedi().as_slice(),
                [Oznaka("vrni".to_string())].as_slice(),
            ].concat(),

            Zaporedje(vozlišča) => vozlišča.into_iter().map(|v| v.prevedi()).flatten().collect(),
            Okvir{ zaporedje, št_spr } => Zaporedje(vec![
                Push(*št_spr).rc(),
                zaporedje.clone(),
                Pop(*št_spr).rc()
            ]).prevedi(),

            Funkcija{ ime, parametri, telo, prostor } => {
                let pred = Zaporedje(vec![
                    NaložiOdmik.rc(),
                       Vrh((
                           -NaložiOdmik.sprememba_stacka()
                           - parametri.len() as isize
                           - ProgramskiŠtevec(0).sprememba_stacka()
                           - Push(1).sprememba_stacka()) as i32).rc(),
                       Push(*prostor).rc(),
                ]);

                let za = Zaporedje(vec![
                    Pop(*prostor).rc(),
                    ShraniOdmik.rc(),
                    Pop(parametri.len()).rc(),
                    DinamičniSkok.rc()
                ]);

                [
                    Skok(OdmikIme::Odmik((1 + pred.len() + telo.len() + za.len()) as isize)).prevedi().as_slice(),
                    [Oznaka(ime.clone())].as_slice(),
                    pred.prevedi().as_slice(),
                    telo.prevedi().as_slice(),
                    [Oznaka(format!("konec_funkcije {}", ime))].as_slice(),
                    za.prevedi().as_slice()
                ].concat()
            },

            FunkcijskiKlic{ funkcija, argumenti } => {
                let vrni = Push(1);
                let skok = Skok(OdmikIme::Ime(if let Funkcija { ime, parametri: _, telo: _, prostor: _ } = &**funkcija { ime.clone() } else { "".to_string() }));
                let pc   = ProgramskiŠtevec((1 + argumenti.len() + skok.len()) as i32);

                Zaporedje(vec![
                    vrni.rc(),
                    pc.rc(),
                    argumenti.rc(),
                    skok.rc(),
                ]).prevedi()
            },

            Natisni(izrazi) => izrazi.into_iter()
                .map(|izraz| [
                    izraz.prevedi().as_slice(),
                    match &**izraz {
                        Niz(_) => iter::repeat(Osnovni(PRTC))
                            .take(izraz.sprememba_stacka() as usize)
                            .collect(),
                        _ => [Osnovni(PRTN)].to_vec(),
                    }.as_slice()
                ].concat()).flatten().collect(),
        }
    }

    fn len(&self) -> usize {
        match self {
            _ => {
                self.prevedi()
                    .iter()
                    .filter(|u| if let Oznaka(..) = u { false } else { true })
                    .count()
            },
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn prevedi() {
        assert_eq!(Prazno.prevedi(), [].to_vec());

        assert_eq!(Push(0).prevedi(), [].to_vec());
        assert_eq!(Push(3).prevedi(), [
                   Osnovni(PUSH(NIČ)),
                   Osnovni(PUSH(NIČ)),
                   Osnovni(PUSH(NIČ)),
        ].to_vec());

        assert_eq!(Pop(0).prevedi(), [].to_vec());
        assert_eq!(Pop(3).prevedi(), [
                   Osnovni(POP),
                   Osnovni(POP),
                   Osnovni(POP),
        ].to_vec());

        assert_eq!(Vrh(-13).prevedi(), [Osnovni(TOP(-13))].to_vec());

        assert_eq!(ShraniOdmik.prevedi(), [Osnovni(SOFF)].to_vec());
        assert_eq!(NaložiOdmik.prevedi(), [Osnovni(LOFF)].to_vec());

        assert_eq!(Niz("šipa".to_string()).prevedi(), [
                   Osnovni(PUSH(Podatek { c: 'a' })),
                   Osnovni(PUSH(Podatek { c: 'p' })),
                   Osnovni(PUSH(Podatek { c: 'i' })),
                   Osnovni(PUSH(Podatek { c: 'š' })),
        ]);
        assert_eq!(Število(-3.14).prevedi(), [Osnovni(PUSH(Podatek { f: -3.14 }))]);
        assert_eq!(Spremenljivka { ime: "šmir".to_string(), naslov: 55, z_odmikom: true  }.prevedi(), [Osnovni(LDOF(55))].to_vec());
        assert_eq!(Spremenljivka { ime: "šmir".to_string(), naslov: 55, z_odmikom: false }.prevedi(), [Osnovni(LOAD(55))].to_vec());

        assert_eq!(Resnica.prevedi(), [Osnovni(PUSH(RESNICA))].to_vec());
        assert_eq!(Laž.prevedi(), [Osnovni(PUSH(LAŽ))].to_vec());

        assert_eq!(Seštevanje(Število(1.0).rc(), Število(2.0).rc()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 2.0 })),
                   Osnovni(PUSH(Podatek { f: 1.0 })),
                   Osnovni(ADD),
        ]);
        assert_eq!(Odštevanje(Število(1.0).rc(), Število(2.0).rc()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 2.0 })),
                   Osnovni(PUSH(Podatek { f: 1.0 })),
                   Osnovni(SUB),
        ]);
        assert_eq!(Množenje(Število(1.0).rc(), Število(2.0).rc()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 2.0 })),
                   Osnovni(PUSH(Podatek { f: 1.0 })),
                   Osnovni(MUL),
        ]);
        assert_eq!(Deljenje(Število(1.0).rc(), Število(2.0).rc()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 2.0 })),
                   Osnovni(PUSH(Podatek { f: 1.0 })),
                   Osnovni(DIV),
        ]);
        assert_eq!(Modulo(Število(1.0).rc(), Število(2.0).rc()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 2.0 })),
                   Osnovni(PUSH(Podatek { f: 1.0 })),
                   Osnovni(MOD),
        ]);
        assert_eq!(Potenca(Število(1.0).rc(), Število(2.0).rc()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 2.0 })),
                   Osnovni(PUSH(Podatek { f: 1.0 })),
                   Osnovni(POW),
        ]);

        assert_eq!(Zanikaj(Resnica.rc()).prevedi(), [
                   Osnovni(PUSH(RESNICA)),
                   Osnovni(PUSH(Podatek { f: 1.0 })),
                   Osnovni(SUB),
        ]);
        assert_eq!(Zanikaj(Laž.rc()).prevedi(), [
                   Osnovni(PUSH(LAŽ)),
                   Osnovni(PUSH(Podatek { f: 1.0 })),
                   Osnovni(SUB),
        ]);
        assert_eq!(Konjunkcija(Laž.rc(), Resnica.rc()).prevedi(), [
                   Osnovni(PUSH(RESNICA)),
                   Osnovni(PUSH(LAŽ)),
                   Osnovni(MUL),
        ]);
        assert_eq!(Disjunkcija(Laž.rc(), Resnica.rc()).prevedi(), [
                   Osnovni(PUSH(RESNICA)),
                   Osnovni(PUSH(LAŽ)),
                   Osnovni(ADD),
                   Osnovni(POS),
        ]);

        assert_eq!(Enako(Število(3.14).rc(), Število(3.14159268).rc()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 3.14159268 })),
                   Osnovni(PUSH(Podatek { f: 3.14 })),
                   Osnovni(SUB),
                   Osnovni(ZERO),
        ]);
        assert_eq!(Večje(Število(13.0).rc(), Število(42.0).rc()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 42.0 })),
                   Osnovni(PUSH(Podatek { f: 13.0 })),
                   Osnovni(SUB),
                   Osnovni(POS),

        ]);

        assert_eq!(ProgramskiŠtevec(-7).prevedi(), [PC(-7)]);
        assert_eq!(Skok(OdmikIme::Odmik(69)).prevedi(), [JUMPRelative(OdmikIme::Odmik(69))]);
        assert_eq!(DinamičniSkok.prevedi(), [Osnovni(JMPD)]);
        assert_eq!(PogojniSkok(Resnica.rc(), -33).prevedi(), [
                   Osnovni(PUSH(RESNICA)),
                   JMPCRelative(-33),
        ]);

        assert_eq!(PogojniStavek { 
            pogoj: Resnica.rc(),
            resnica: Natisni([Niz("res".to_owned()).rc()].to_vec()).rc(),
            laž: Natisni([Niz("laž".to_owned()).rc()].to_vec()).rc(),
        }.prevedi(), [
            Osnovni(PUSH(RESNICA)),
            JMPCRelative(8),
            Osnovni(PUSH(Podatek { c: 'ž' })),
            Osnovni(PUSH(Podatek { c: 'a' })),
            Osnovni(PUSH(Podatek { c: 'l' })),
            Osnovni(PRTC),
            Osnovni(PRTC),
            Osnovni(PRTC),
            JUMPRelative(OdmikIme::Odmik(7)),
            Osnovni(PUSH(Podatek { c: 's' })),
            Osnovni(PUSH(Podatek { c: 'e' })),
            Osnovni(PUSH(Podatek { c: 'r' })),
            Osnovni(PRTC),
            Osnovni(PRTC),
            Osnovni(PRTC),
        ]);

        assert_eq!(Zanka {
            pogoj: Laž.rc(), 
            telo: Prirejanje { 
                spremenljivka: Spremenljivka { ime: "x".to_string(), naslov: 25, z_odmikom: false }.rc(),
                izraz: Število(27.0).rc(),
            }.rc(),
        }.prevedi(), [
            Osnovni(PUSH(LAŽ)),
            Osnovni(PUSH(Podatek { f: 1.0 })),
            Osnovni(SUB),
            JMPCRelative(4),
            Osnovni(PUSH(Podatek { f: 27.0 })),
            Osnovni(STOR(25)),
            JUMPRelative(OdmikIme::Odmik(-6)),
        ]);

        assert_eq!(Prirejanje {
            spremenljivka: Spremenljivka { ime: "x".to_string(), naslov: 3, z_odmikom: true }.rc(),
            izraz: Število(-3.14).rc(),
        }.prevedi(), [
            Osnovni(PUSH(Podatek { f: -3.14 })),
            Osnovni(STOF(3)),
        ]);

        assert_eq!(Vrni(Prirejanje {
            spremenljivka: Spremenljivka { ime: "vrni".to_string(), naslov: 0, z_odmikom: true }.rc(),
            izraz: Število(2.0).rc()
        }.rc()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 2.0 })),
                   Osnovni(STOF(0)),
                   Oznaka("vrni".to_string()),
        ]);

        assert_eq!(Zaporedje(vec![
                             Število(1.0).rc(),
                             Število(2.0).rc(),
                             Resnica.rc(),
                             Laž.rc(),
        ]).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 1.0 })),
                   Osnovni(PUSH(Podatek { f: 2.0 })),
                   Osnovni(PUSH(RESNICA)),
                   Osnovni(PUSH(LAŽ)),
        ]);

        assert_eq!(Okvir {
            zaporedje: Zaporedje(vec![
                                 Vrni(Prirejanje {
                                     spremenljivka: Spremenljivka { ime: "vrni".to_string(), naslov: 0, z_odmikom: true }.rc(),
                                     izraz: Spremenljivka { ime: "x".to_string(), naslov: 1, z_odmikom: true }.rc(),
                                 }.rc()).rc(),
            ]).rc(),
            št_spr: 2
        }.prevedi(), [
            Osnovni(PUSH(NIČ)),
            Osnovni(PUSH(NIČ)),
            Osnovni(LDOF(1)),
            Osnovni(STOF(0)),
            Oznaka("vrni".to_string()),
            Osnovni(POP),
            Osnovni(POP),
        ]);

        let funkcija = Funkcija {
            ime: "ena".to_string(),
            parametri: vec![],
            telo: Vrni(Prirejanje {
                spremenljivka: Spremenljivka { ime: "vrni".to_string(), naslov: 0, z_odmikom: true }.rc(),
                izraz: Število(1.0).rc()
            }.rc()).rc(),
            prostor: 0,
        }.rc();

        assert_eq!(funkcija.clone().prevedi(), [
            JUMPRelative(OdmikIme::Odmik(7)),
            Oznaka("ena".to_string()),
            Osnovni(LOFF),
            Osnovni(TOP(-3)),
            Osnovni(PUSH(Podatek { f: 1.0 })),
            Osnovni(STOF(0)),
            Oznaka("vrni".to_string()),
            Oznaka("konec_funkcije ena".to_string()),
            Osnovni(SOFF),
            Osnovni(JMPD),
        ]);

        assert_eq!(FunkcijskiKlic {
            funkcija: funkcija.clone(),
            argumenti: Prazno.rc(),
        }.prevedi(), [
            Osnovni(PUSH(NIČ)),
            PC(2),
            JUMPRelative(OdmikIme::Ime("ena".to_string())),
        ]);

        assert_eq!(Natisni([Število(13.0).rc()].to_vec()).prevedi(), [
                   Osnovni(PUSH(Podatek { f: 13.0 })),
                   Osnovni(PRTN),
        ]);
        assert_eq!(Natisni([Niz("đins\n".to_string()).rc()].to_vec()).prevedi(), [
                   Osnovni(PUSH(Podatek { c: '\n' })),
                   Osnovni(PUSH(Podatek { c: 's' })),
                   Osnovni(PUSH(Podatek { c: 'n' })),
                   Osnovni(PUSH(Podatek { c: 'i' })),
                   Osnovni(PUSH(Podatek { c: 'đ' })),
                   Osnovni(PRTC),
                   Osnovni(PRTC),
                   Osnovni(PRTC),
                   Osnovni(PRTC),
                   Osnovni(PRTC),
        ]);
    }
}