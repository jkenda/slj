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
            Prazno => vec![],

            Push(krat) => if *krat > 0 { vec![Osnovni(ALOC(*krat))] } else { iter::repeat(PUSHI(0)).take(*krat as usize).collect() },
            Pop(krat)  => if *krat > 0 { vec![Osnovni(ALOC(-krat))] } else { vec![] },
            Vrh(odmik) => vec![Osnovni(TOP(*odmik as i32))],

            ShraniOdmik => vec![Osnovni(SOFF)],
            NaložiOdmik => vec![Osnovni(LOFF)],

            Znak(znak) => vec![PUSHC(*znak)],
            Niz(niz) => [
                niz
                    .chars().rev()
                    .map(|znak| PUSHC(znak))
                    .collect::<Vec<UkazPodatekRelative>>(),
                vec![PUSHI(niz.chars().count() as i32)],
            ].concat(),
            Celo(število) => vec![PUSHI(*število)],
            Real(število) => vec![PUSHF(*število)],

            Resnica => vec![PUSHI(1)],
            Laž     => vec![PUSHI(0)],

            Spremenljivka{ naslov, z_odmikom, .. } => vec![Osnovni(if *z_odmikom { LDOF(*naslov) } else { LOAD(*naslov) })],
            Referenca(vozlišče) | RefSeznama(vozlišče) => match &**vozlišče {
                Spremenljivka { naslov, z_odmikom, .. } => [
                    [match vozlišče.tip() {
                        Tip::Seznam(..) => PUSHI(*naslov as i32 + 1),
                        _ => PUSHI(*naslov as i32),
                    }].as_slice(),
                    if *z_odmikom {
                        [Osnovni(LOFF),
                        Osnovni(ADDI)].as_slice()
                    }
                    else {
                        [].as_slice()
                    },
                ].concat(),
                _ => unreachable!("Referenciramo lahko samo spremenljivko.")
            },

            Dereferenciraj(vozlišče) => [
                vozlišče.prevedi().as_slice(),
                [Osnovni(LDDY(0))].as_slice(),
            ].concat(),
            Indeksiraj { seznam_ref, indeks } =>
                Dereferenciraj(Seštevanje(Tip::Celo, seznam_ref.clone(), indeks.clone()).rc()).prevedi(),
            Dolžina(vozlišče) => match vozlišče.tip() {
                Tip::Seznam(_, dolžina) => Celo(dolžina).rc().prevedi(),
                Tip::RefSeznama(..) => [
                    vozlišče.prevedi().as_slice(),
                    [Osnovni(LDDY(-1))].as_slice(),
                ].concat(),
                _ => unreachable!("Jemanje dolžine nečesa, kar ni seznam"),
            },

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
            CeloVZnak(vozlišče) => vozlišče.prevedi(),
            ZnakVCelo(vozlišče) => vozlišče.prevedi(),

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

            ProgramskiŠtevec(odmik) => vec![PC(*odmik)],

            Skok(odmik_ime) => vec![JUMPRelative(odmik_ime.clone())],
            DinamičniSkok => vec![Osnovni(JMPD).to_owned()],
            PogojniSkok(pogoj, skok) => [
                pogoj.prevedi().as_slice(),
                [JMPCRelative(*skok)].as_slice(),
            ].concat(),

            PogojniStavek{ pogoj, resnica, laž } => {
                let skok = Skok(OdmikIme::Odmik(resnica.len() as i32 + 1)).rc();
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
                          Skok(OdmikIme::Odmik(-(telo.len() as i32) - pogoj_len as i32 - 1)).rc()
                ]).prevedi()
            },

            Prirejanje{ spremenljivka, izraz } => {
                let (naslov, velikost, z_odmikom) = match &**spremenljivka { 
                    Spremenljivka { naslov, z_odmikom, .. } => (naslov.clone(), izraz.tip().sprememba_stacka(), *z_odmikom),
                    _ => unreachable!("Vedno prirejamo spremenljivki.")
                };

                let shrani = (naslov..naslov+velikost)
                    .map(|naslov| Osnovni(if z_odmikom { STOF(naslov) } else { STOR(naslov) }))
                    .collect::<Vec<UkazPodatekRelative>>();

                [
                    izraz.clone().prevedi().as_slice(),
                    shrani.as_slice()
                ].concat()
            },

            PrirejanjeRef { referenca, indeks, izraz } => {
                let shrani = match indeks {
                    Some(indeks) => [
                        referenca.prevedi().as_slice(),
                        indeks.prevedi().as_slice(),
                        [Osnovni(ADDI),
                        Osnovni(STDY(0))].as_slice(),
                    ].concat(),
                    None => [
                        referenca.prevedi().as_slice(),
                        [Osnovni(STDY(0))].as_slice(),
                    ].concat(),
                };

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
                    .map(|p| p.sprememba_stacka())
                    .sum();

                let pred = Zaporedje(vec![
                    NaložiOdmik.rc(),
                    Vrh((- tip.sprememba_stacka()                    // vrni (+0)
                         - ProgramskiŠtevec(0).sprememba_stacka()    // PC (+1)
                         - parametri_velikost               // [ argumenti ] (+2 ...)
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
                    Skok(OdmikIme::Odmik((1 + pred.len() + telo.len() + za.len()) as i32)).prevedi().as_slice(),
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
                        Push(tip.sprememba_stacka()).rc(),
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

            Natisni(znak) => {
                [
                    znak.prevedi().as_slice(),
                    [Osnovni(PUTC)].as_slice(),
                ].concat()
            },
            Preberi => vec![Osnovni(GETC)],
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
        assert_eq!(Prazno.prevedi(), []);

        assert_eq!(Push(0).prevedi(), []);
        assert_eq!(Push(3).prevedi(), [
                   Osnovni(ALOC(3)),
        ]);

        assert_eq!(Pop(0).prevedi(), []);
        assert_eq!(Pop(3).prevedi(), [
                   Osnovni(ALOC(-3)),
        ]);

        assert_eq!(Vrh(-13).prevedi(), [Osnovni(TOP(-13))]);

        assert_eq!(ShraniOdmik.prevedi(), [Osnovni(SOFF)]);
        assert_eq!(NaložiOdmik.prevedi(), [Osnovni(LOFF)]);

        assert_eq!(Niz("šipa".to_string()).prevedi(), [
                   PUSHC('a'),
                   PUSHC('p'),
                   PUSHC('i'),
                   PUSHC('š'),
                   PUSHI(4),
        ]);
        assert_eq!(Real(-3.14).prevedi(), [PUSHF(-3.14)]);

        assert_eq!(Spremenljivka { tip: Tip::Real, ime: "šmir".to_string(), naslov: 55, z_odmikom: true  }.prevedi(), [Osnovni(LDOF(55))]);
        assert_eq!(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: false }.prevedi(), [Osnovni(LOAD(55))]);
        assert_eq!(
            Referenca(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: true }.rc()).prevedi(),
            [
                PUSHI(55),
                Osnovni(LOFF),
                Osnovni(ADDI),
            ]);
        assert_eq!(
            Referenca(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: false }.rc()).prevedi(),
            [
                PUSHI(55),
            ]);

        assert_eq!(Resnica.prevedi(), [PUSHI(1)]);
        assert_eq!(Laž.prevedi(), [PUSHI(0)]);

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
            resnica: Natisni(Znak('r').rc()).rc(),
            laž: Natisni(Znak('l').rc()).rc(),
        }.prevedi(), [
            PUSHI(1),
            JMPCRelative(4),
            PUSHC('l'),
            Osnovni(PUTC),
            JUMPRelative(OdmikIme::Odmik(3)),
            PUSHC('r'),
            Osnovni(PUTC),
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
            Osnovni(ALOC(2)),
            Osnovni(LDOF(1)),
            Osnovni(STOF(0)),
            Oznaka("vrni".to_string()),
            Osnovni(ALOC(-2)),
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
            JUMPRelative(OdmikIme::Odmik(9)),
            Oznaka("ena".to_string()),
            Osnovni(LOFF),
            Osnovni(TOP(-5)),
            PUSHF(1.0),
            Osnovni(STOF(0)),
            Oznaka("vrni".to_string()),
            Oznaka("konec_funkcije ena".to_string()),
            Osnovni(SOFF),
            Osnovni(ALOC(-2)),
            Osnovni(JMPD),
        ]);

        assert_eq!(FunkcijskiKlic {
            funkcija: funkcija.clone(),
            spremenljivke: Zaporedje(vec![]).rc(),
            argumenti: Zaporedje(vec![Real(1.0).rc(), Real(2.0).rc()]).rc(),
        }.prevedi(), [
            Osnovni(ALOC(1)),
            PC(4),
            PUSHF(1.0),
            PUSHF(2.0),
            JUMPRelative(OdmikIme::Ime("ena".to_string())),
        ]);
    }
}
