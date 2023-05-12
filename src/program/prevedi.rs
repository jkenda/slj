use super::*;

use std::iter;

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

            Push(krat) => iter::repeat(PUSHI(0)).take(*krat).collect(),
            Pop(krat) => iter::repeat(Osnovni(POP).clone()).take(*krat).collect(),
            Vrh(odmik) => [Osnovni(TOP(*odmik as i32))].to_vec(),

            ShraniOdmik => [Osnovni(SOFF)].to_vec(),
            NaložiOdmik => [Osnovni(LOFF)].to_vec(),

            Znak(znak) => [PUSHC(*znak)].to_vec(),
            Niz(niz) => niz
                .chars().rev()
                .map(|znak| PUSHC(znak))
                .collect::<Vec<UkazPodatekRelative>>(),
            Celo(število) => [PUSHI(*število)].to_vec(),
            Real(število) => [PUSHF(*število)].to_vec(),

            Resnica => [PUSHI(1)].to_vec(),
            Laž     => [PUSHI(0)].to_vec(),

            Spremenljivka{ naslov, z_odmikom, .. } => [Osnovni(if *z_odmikom { LDOF(*naslov) } else { LOAD(*naslov) })].to_vec(),
            Referenca(vozlišče) => match &**vozlišče {
                Spremenljivka { naslov, .. } => [
                    PUSHI(*naslov as i32),
                    Osnovni(LOFF),
                    Osnovni(ADDI),
                ].to_vec(),
                _ => unreachable!("Referenciramo lahko samo spremenljivko.")
            },
            Dereferenciraj(vozlišče) => [
                vozlišče.prevedi().as_slice(),
                [Osnovni(LDDY(0))].as_slice(),
            ].concat(),

            Seštevanje(Tip::Celo, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(ADDI)].as_slice(),
            ].concat(),
            Seštevanje(Tip::Real, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(ADDF)].as_slice(),
            ].concat(),
            Seštevanje(..) => unreachable!(),
            Odštevanje(Tip::Celo, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(SUBI)].as_slice(),
            ].concat(),
            Odštevanje(Tip::Real, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(SUBF)].as_slice(),
            ].concat(),
            Odštevanje(..) => unreachable!(),
            Množenje(Tip::Celo, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(MULI)].as_slice(),
            ].concat(),
            Množenje(Tip::Real, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(MULF)].as_slice(),
            ].concat(),
            Množenje(..) => unreachable!(),
            Deljenje(Tip::Celo, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(DIVI)].as_slice(),
            ].concat(),
            Deljenje(Tip::Real, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(DIVF)].as_slice(),
            ].concat(),
            Deljenje(..) => unreachable!(),
            Modulo(Tip::Celo, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(MODI)].as_slice(),
            ].concat(),
            Modulo(Tip::Real, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(MODF)].as_slice(),
            ].concat(),
            Modulo(..) => unreachable!(),
            Potenca(Tip::Celo, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(POWI)].as_slice(),
            ].concat(),
            Potenca(Tip::Real, l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(POWF)].as_slice(),
            ].concat(),
            Potenca(..) => unreachable!(),

            CeloVReal(vozlišče) => [
                vozlišče.prevedi().as_slice(),
                [Osnovni(ITOF)].as_slice(),
            ].concat(),
            RealVCelo(vozlišče) => [
                vozlišče.prevedi().as_slice(),
                [Osnovni(FTOI)].as_slice(),
            ].concat(),

            Zanikaj(vozlišče) => [
                [PUSHI(1)].as_slice(),
                vozlišče.prevedi().as_slice(),
                [Osnovni(SUBI)].as_slice(),
            ].concat(),
            Konjunkcija(l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(MULI)].as_slice(),
            ].concat(),
            Disjunkcija(l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(ADDI)].as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),

            BitniAli(l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(BOR)].as_slice(),
            ].concat(),
            BitniXor(l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(BXOR)].as_slice(),
            ].concat(),
            BitniIn(l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(BAND)].as_slice(),
            ].concat(),
            BitniPremikLevo(l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(BSLL)].as_slice(),
            ].concat(),
            BitniPremikDesno(l, d) => [
                l.prevedi().as_slice(),
                d.prevedi().as_slice(),
                [Osnovni(BSLD)].as_slice(),
            ].concat(),

            Enako(tip, l, d) => [
                Odštevanje(tip.clone(), l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(ZERO)].as_slice(),
            ].concat(),
            NiEnako(tip, l, d) => Zanikaj(Enako(tip.clone(), l.clone(), d.clone()).rc()).prevedi(),

            Večje(tip, l, d) => [
                Odštevanje(tip.clone(), l.clone(), d.clone()).prevedi().as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),
            Manjše(tip, l, d)      => Večje(tip.clone(), d.clone(), l.clone()).prevedi(),
            VečjeEnako(tip, l, d)  => Zanikaj(Manjše(tip.clone(), l.clone(), d.clone()).rc()).prevedi(),
            ManjšeEnako(tip, l, d) => VečjeEnako(tip.clone(), d.clone(), l.clone()).prevedi(),

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
                let (naslov, velikost, z_odmikom) = match &**spremenljivka { 
                    Spremenljivka { tip, naslov, z_odmikom, .. } => (naslov.clone(), tip.sprememba_stacka(), *z_odmikom),
                    _ => unreachable!("Vedno prirejamo spremenljivki.")
                };

                let shrani = (naslov..naslov+velikost as u32)
                    .map(|naslov| Osnovni(if z_odmikom { STOF(naslov) } else { STOR(naslov) }))
                    .collect::<Vec<UkazPodatekRelative>>();

                [
                    izraz.clone().prevedi().as_slice(),
                    shrani.as_slice()
                ].concat()
            },

            PrirejanjeRef{ referenca, izraz } => {
                let (začetek, velikost, z_odmikom) = match &**referenca { 
                    Spremenljivka { tip, naslov, z_odmikom, .. } => (naslov.clone(), tip.sprememba_stacka(), *z_odmikom),
                    _ => unreachable!("Vedno prirejamo spremenljivki, na katero kaže referenca.")
                };

                let shrani = (začetek..začetek+velikost as u32)
                    .rev()
                    .map(|naslov| [
                        Osnovni(if z_odmikom { LDOF(naslov) } else { LOAD(naslov) }),
                        Osnovni(STDY(naslov - začetek)),
                    ])
                    .flatten()
                    .collect::<Vec<UkazPodatekRelative>>();

                [
                    izraz.clone().prevedi().as_slice(),
                    shrani.as_slice()
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

            Funkcija{ tip, ime, parametri, telo, prostor, .. } => {
                let parametri_velikost = parametri.iter()
                    .map(|p| p.sprememba_stacka() as usize)
                    .sum::<usize>();

                let pred = Zaporedje(vec![
                    NaložiOdmik.rc(),
                    Vrh((- tip.sprememba_stacka()                    // vrni (+0)
                         - ProgramskiŠtevec(0).sprememba_stacka()    // PC (+1)
                         - parametri_velikost as isize               // [ argumenti ] (+2 ...)
                         - NaložiOdmik.sprememba_stacka()            // prejšnji odmik
                        ) as i32).rc(),
                    Push(*prostor).rc(),
                ]);

                let za = Zaporedje(vec![
                    Pop(*prostor).rc(),           // odstrani spremenljivke funkcije
                    ShraniOdmik.rc(),             // naloži prejšnji odmik stacka
                    Pop(parametri_velikost).rc(), // odstrani parametre
                    DinamičniSkok.rc(),           // skoči iz funkcije na klicatelja
                ]);

                [
                    Skok(OdmikIme::Odmik((1 + pred.len() + telo.len() + za.len()) as isize)).prevedi().as_slice(),
                    [Oznaka(ime.clone())].as_slice(),
                    pred.prevedi().as_slice(),
                    telo.prevedi().as_slice(),
                    [Oznaka(format!("konec_funkcije {}", ime))].as_slice(),
                    za.prevedi().as_slice(),
                ].concat()
            },

            FunkcijskiKlic{ funkcija, spremenljivke, argumenti } => {
                let (vrni, skok) = match &**funkcija {
                    Funkcija { tip, ime, .. } => (
                        Push(tip.sprememba_stacka() as usize).rc(),
                        Skok(OdmikIme::Ime(ime.clone())).rc()),
                    _ => unreachable!("Funkcijski klic vedno kliče funkcijo"),
                };
                let pc = ProgramskiŠtevec((1 + argumenti.len() + skok.len()) as i32).rc();

                Zaporedje(vec![
                    spremenljivke.clone(),
                    vrni,              // rezerviraj prostor za rezultat funkcije
                    pc,                // naloži PC (kam se vrniti iz funkcije)
                    argumenti.clone(), // naloži argumente
                    skok,              // skoči v funkcijo
                ]).prevedi()
            },

            Natisni(izrazi) => {
                izrazi.into_iter()
                .map(|izraz| [
                    izraz.prevedi().as_slice(),
                    match &**izraz {
                        Niz(_) => iter::repeat(Osnovni(PRTC))
                            .take(izraz.sprememba_stacka() as usize)
                            .collect(),
                        _ => match izraz.tip() {
                            Tip::Real => [Osnovni(PRTF)].to_vec(),
                            _         => [Osnovni(PRTI)].to_vec(),
                        }
                    }.as_slice()
                ].concat()).flatten().collect()
            },
        }
    }

    fn len(&self) -> usize {
        match self {
            _ => {
                self.prevedi()
                    .iter()
                    .filter(|u| if let Oznaka(oznaka) = u { oznaka == "vrni" } else { true })
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
                   PUSHI(0),
                   PUSHI(0),
                   PUSHI(0),
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
                   PUSHC('a'),
                   PUSHC('p'),
                   PUSHC('i'),
                   PUSHC('š'),
        ]);
        assert_eq!(Real(-3.14).prevedi(), [PUSHF(-3.14)]);

        assert_eq!(Spremenljivka { tip: Tip::Real, ime: "šmir".to_string(), naslov: 55, z_odmikom: true  }.prevedi(), [Osnovni(LDOF(55))].to_vec());
        assert_eq!(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: false }.prevedi(), [Osnovni(LOAD(55))].to_vec());
        assert_eq!(
            Referenca(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: false }.rc()).prevedi(),
            [
                PUSHI(55),
                Osnovni(LOFF),
                Osnovni(ADDI),
            ].to_vec());

        assert_eq!(Resnica.prevedi(), [PUSHI(1)].to_vec());
        assert_eq!(Laž.prevedi(), [PUSHI(0)].to_vec());

        assert_eq!(Seštevanje(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(ADDF),
        ]);
        assert_eq!(Odštevanje(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(SUBF),
        ]);
        assert_eq!(Množenje(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(MULF),
        ]);
        assert_eq!(Deljenje(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(DIVF),
        ]);
        assert_eq!(Modulo(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(MODF),
        ]);
        assert_eq!(Potenca(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(POWF),
        ]);

        assert_eq!(Zanikaj(Resnica.rc()).prevedi(), [
                   PUSHI(1),
                   PUSHI(1),
                   Osnovni(SUBI),
        ]);
        assert_eq!(Zanikaj(Laž.rc()).prevedi(), [
                   PUSHI(1),
                   PUSHI(0),
                   Osnovni(SUBI),
        ]);
        assert_eq!(Konjunkcija(Laž.rc(), Resnica.rc()).prevedi(), [
                   PUSHI(0),
                   PUSHI(1),
                   Osnovni(MULI),
        ]);
        assert_eq!(Disjunkcija(Laž.rc(), Resnica.rc()).prevedi(), [
                   PUSHI(0),
                   PUSHI(1),
                   Osnovni(ADDI),
                   Osnovni(POS),
        ]);

        assert_eq!(Enako(Tip::Real, Real(3.14).rc(), Real(3.14159268).rc()).prevedi(), [
                   PUSHF(3.14),
                   PUSHF(3.14159268),
                   Osnovni(SUBF),
                   Osnovni(ZERO),
        ]);
        assert_eq!(Večje(Tip::Celo, Celo(13).rc(), Celo(42).rc()).prevedi(), [
                   PUSHI(13),
                   PUSHI(42),
                   Osnovni(SUBI),
                   Osnovni(POS),

        ]);

        assert_eq!(ProgramskiŠtevec(-7).prevedi(), [PC(-7)]);
        assert_eq!(Skok(OdmikIme::Odmik(69)).prevedi(), [JUMPRelative(OdmikIme::Odmik(69))]);
        assert_eq!(DinamičniSkok.prevedi(), [Osnovni(JMPD)]);
        assert_eq!(PogojniSkok(Resnica.rc(), -33).prevedi(), [
                   PUSHI(1),
                   JMPCRelative(-33),
        ]);

        assert_eq!(PogojniStavek { 
            pogoj: Resnica.rc(),
            resnica: Natisni([Niz("res".to_owned()).rc()].to_vec()).rc(),
            laž: Natisni([Niz("laž".to_owned()).rc()].to_vec()).rc(),
        }.prevedi(), [
            PUSHI(1),
            JMPCRelative(8),
            PUSHC('ž'),
            PUSHC('a'),
            PUSHC('l'),
            Osnovni(PRTC),
            Osnovni(PRTC),
            Osnovni(PRTC),
            JUMPRelative(OdmikIme::Odmik(7)),
            PUSHC('s'),
            PUSHC('e'),
            PUSHC('r'),
            Osnovni(PRTC),
            Osnovni(PRTC),
            Osnovni(PRTC),
        ]);

        assert_eq!(Zanka {
            pogoj: Laž.rc(), 
            telo: Prirejanje { 
                spremenljivka: Spremenljivka { tip: Tip::Real, ime: "x".to_string(), naslov: 25, z_odmikom: false }.rc(),
                izraz: Real(27.0).rc(),
            }.rc(),
        }.prevedi(), [
            PUSHI(1),
            PUSHI(0),
            Osnovni(SUBI),
            JMPCRelative(4),
            PUSHF(27.0),
            Osnovni(STOR(25)),
            JUMPRelative(OdmikIme::Odmik(-6)),
        ]);

        assert_eq!(Prirejanje {
            spremenljivka: Spremenljivka { tip: Tip::Real, ime: "x".to_string(), naslov: 3, z_odmikom: true }.rc(),
            izraz: Real(-3.14).rc(),
        }.prevedi(), [
            PUSHF(-3.14),
            Osnovni(STOF(3)),
        ]);

        assert_eq!(Vrni(Prirejanje {
            spremenljivka: Spremenljivka { tip: Tip::Real, ime: "vrni".to_string(), naslov: 0, z_odmikom: true }.rc(),
            izraz: Real(2.0).rc()
        }.rc()).prevedi(), [
                   PUSHF(2.0),
                   Osnovni(STOF(0)),
                   Oznaka("vrni".to_string()),
        ]);

        assert_eq!(Zaporedje(vec![
                             Real(1.0).rc(),
                             Real(2.0).rc(),
                             Resnica.rc(),
                             Laž.rc(),
        ]).prevedi(), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   PUSHI(1),
                   PUSHI(0),
        ]);

        assert_eq!(Okvir {
            zaporedje: Zaporedje(vec![
                                 Vrni(Prirejanje {
                                     spremenljivka: Spremenljivka { tip: Tip::Celo, ime: "vrni".to_string(), naslov: 0, z_odmikom: true }.rc(),
                                     izraz: Spremenljivka { tip: Tip::Celo, ime: "x".to_string(), naslov: 1, z_odmikom: true }.rc(),
                                 }.rc()).rc(),
            ]).rc(),
            št_spr: 2
        }.prevedi(), [
            PUSHI(0),
            PUSHI(0),
            Osnovni(LDOF(1)),
            Osnovni(STOF(0)),
            Oznaka("vrni".to_string()),
            Osnovni(POP),
            Osnovni(POP),
        ]);

        let funkcija = Funkcija {
            tip: Tip::Real,
            ime: "ena".to_string(),
            parametri: vec![
                Spremenljivka { tip: Tip::Celo, ime: "x".to_string(), naslov: 2, z_odmikom: true }.rc(),
                Spremenljivka { tip: Tip::Celo, ime: "y".to_string(), naslov: 3, z_odmikom: true }.rc(),
            ],
            telo: Vrni(Prirejanje {
                spremenljivka: Spremenljivka { tip: Tip::Real, ime: "vrni".to_string(), naslov: 0, z_odmikom: true }.rc(),
                izraz: Real(1.0).rc()
            }.rc()).rc(),
            prostor: 0,
            št_klicev: 1,
        }.rc();

        assert_eq!(funkcija.clone().prevedi(), [
            JUMPRelative(OdmikIme::Odmik(10)),
            Oznaka("ena".to_string()),
            Osnovni(LOFF),
            Osnovni(TOP(-5)),
            PUSHF(1.0),
            Osnovni(STOF(0)),
            Oznaka("vrni".to_string()),
            Oznaka("konec_funkcije ena".to_string()),
            Osnovni(SOFF),
            Osnovni(POP),
            Osnovni(POP),
            Osnovni(JMPD),
        ]);

        assert_eq!(FunkcijskiKlic {
            funkcija: funkcija.clone(),
            spremenljivke: Zaporedje(vec![]).rc(),
            argumenti: Zaporedje(vec![Real(1.0).rc(), Real(2.0).rc()]).rc(),
        }.prevedi(), [
            PUSHI(0),
            PC(4),
            PUSHF(1.0),
            PUSHF(2.0),
            JUMPRelative(OdmikIme::Ime("ena".to_string())),
        ]);

        assert_eq!(Natisni([Real(13.0).rc()].to_vec()).prevedi(), [
                   PUSHF(13.0),
                   Osnovni(PRTF),
        ]);
        assert_eq!(Natisni([Celo(13).rc()].to_vec()).prevedi(), [
                   PUSHI(13),
                   Osnovni(PRTI),
        ]);
        assert_eq!(Natisni([Niz("đins\n".to_string()).rc()].to_vec()).prevedi(), [
                   PUSHC('\n'),
                   PUSHC('s' ),
                   PUSHC('n' ),
                   PUSHC('i' ),
                   PUSHC('đ' ),
                   Osnovni(PRTC),
                   Osnovni(PRTC),
                   Osnovni(PRTC),
                   Osnovni(PRTC),
                   Osnovni(PRTC),
        ]);
    }
}
