use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};

impl Prevedi for Drevo {
    fn prevedi(self) -> Vec<UkazPodatekRelative> {
        let main = self.main.prevedi(&self.št_klicev);
        let funkcije = Zaporedje(self.funkcije.clone()).prevedi(&self.št_klicev);
        [
            Push(self.prostor).prevedi(&self.št_klicev).as_slice(),
            &[JUMPRel("main".to_string())],
            &funkcije,
            &[Oznaka("main".to_string())],
            &main,
            &Pop(self.prostor).prevedi(&self.št_klicev),
        ].concat()
    }

    fn len(&self) -> usize {
        self.main.len(&self.št_klicev)
    }
}

static ŠT_OZNAK: AtomicUsize = AtomicUsize::new(0);

impl Vozlišče {
    fn prevedi(&self, št_klicev: &HashMap<String, usize>) -> Vec<UkazPodatekRelative> {
        match self {
            Prazno => vec![],

            Push(krat) => if *krat > 0 { vec![Osnovni(ALOC(*krat))] } else { vec![] },
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
                Spremenljivka { naslov, z_odmikom, .. } =>
                    match vozlišče.tip() {
                        Tip::Seznam(..) => vec![PUSHREF(*naslov + 1, *z_odmikom)],
                        _               => vec![PUSHREF(*naslov, *z_odmikom)],
                    }
                    
                _ => unreachable!("Referenciramo lahko samo spremenljivko.")
            },

            Dereferenciraj(vozlišče) => [
                vozlišče.prevedi(št_klicev).as_slice(),
                [Osnovni(LDDY(0))].as_slice(),
            ].concat(),
            Indeksiraj { seznam_ref, indeks } => [
                    seznam_ref.prevedi(št_klicev).as_slice(),
                    indeks.prevedi(št_klicev).as_slice(),
                    &[LDINDEXED],
                ].concat(),
            Dolžina(vozlišče) => match vozlišče.tip() {
                Tip::Seznam(_, dolžina) => Celo(dolžina).rc().prevedi(št_klicev),
                Tip::RefSeznama(..) => [
                    vozlišče.prevedi(št_klicev).as_slice(),
                    [Osnovni(LDDY(-1))].as_slice(),
                ].concat(),
                _ => unreachable!("Jemanje dolžine nečesa, kar ni seznam"),
            },

            Add(Tip::Celo, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(ADDI)].as_slice(),
            ].concat(),
            Add(Tip::Real, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(ADDF)].as_slice(),
            ].concat(),
            Add(..) => unreachable!(),
            Sub(Tip::Celo, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(SUBI)].as_slice(),
            ].concat(),
            Sub(Tip::Real, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(SUBF)].as_slice(),
            ].concat(),
            Sub(..) => unreachable!(),
            Mul(Tip::Celo, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(MULI)].as_slice(),
            ].concat(),
            Mul(Tip::Real, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(MULF)].as_slice(),
            ].concat(),
            Mul(..) => unreachable!(),
            Div(Tip::Celo, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(DIVI)].as_slice(),
            ].concat(),
            Div(Tip::Real, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(DIVF)].as_slice(),
            ].concat(),
            Div(..) => unreachable!(),
            Mod(Tip::Celo, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(MODI)].as_slice(),
            ].concat(),
            Mod(Tip::Real, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(MODF)].as_slice(),
            ].concat(),
            Mod(..) => unreachable!(),
            Pow(Tip::Celo, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(POWI)].as_slice(),
            ].concat(),
            Pow(Tip::Real, l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(POWF)].as_slice(),
            ].concat(),
            Pow(..) => unreachable!(),

            CeloVReal(vozlišče) => [
                vozlišče.prevedi(št_klicev).as_slice(),
                [Osnovni(ITOF)].as_slice(),
            ].concat(),
            RealVCelo(vozlišče) => [
                vozlišče.prevedi(št_klicev).as_slice(),
                [Osnovni(FTOI)].as_slice(),
            ].concat(),
            CeloVZnak(vozlišče) => vozlišče.prevedi(št_klicev),
            ZnakVCelo(vozlišče) => vozlišče.prevedi(št_klicev),

            Zanikaj(vozlišče) => [
                [PUSHI(1)].as_slice(),
                vozlišče.prevedi(št_klicev).as_slice(),
                [Osnovni(SUBI)].as_slice(),
            ].concat(),
            Konjunkcija(l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(MULI)].as_slice(),
            ].concat(),
            Disjunkcija(l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(ADDI)].as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),

            BitniAli(l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(BOR)].as_slice(),
            ].concat(),
            BitniXor(l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(BXOR)].as_slice(),
            ].concat(),
            BitniIn(l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(BAND)].as_slice(),
            ].concat(),
            BitniPremikLevo(l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(BSLL)].as_slice(),
            ].concat(),
            BitniPremikDesno(l, d) => [
                l.prevedi(št_klicev).as_slice(),
                d.prevedi(št_klicev).as_slice(),
                [Osnovni(BSLR)].as_slice(),
            ].concat(),

            Enako(tip, l, d) => [
                Sub(tip.clone(), l.clone(), d.clone()).prevedi(št_klicev).as_slice(),
                [Osnovni(ZERO)].as_slice(),
            ].concat(),
            NiEnako(tip, l, d) => Zanikaj(Enako(tip.clone(), l.clone(), d.clone()).rc()).prevedi(št_klicev),

            Večje(tip, l, d) => [
                Sub(tip.clone(), l.clone(), d.clone()).prevedi(št_klicev).as_slice(),
                [Osnovni(POS)].as_slice(),
            ].concat(),
            Manjše(tip, l, d)      => Večje(tip.clone(), d.clone(), l.clone()).prevedi(št_klicev),
            VečjeEnako(tip, l, d)  => Zanikaj(Manjše(tip.clone(), l.clone(), d.clone()).rc()).prevedi(št_klicev),
            ManjšeEnako(tip, l, d) => VečjeEnako(tip.clone(), d.clone(), l.clone()).prevedi(št_klicev),

            ProgramskiŠtevec(odmik) => vec![PC(*odmik)],

            Skok(oznaka) => vec![JUMPRel(oznaka.clone())],
            Klic(oznaka) => vec![CALL(oznaka.clone())],
            DinamičniSkok => vec![Osnovni(JMPD).to_owned()],
            PogojniSkok(pogoj, skok) => [
                pogoj.prevedi(št_klicev).as_slice(),
                [JMPCRel(skok.clone())].as_slice(),
            ].concat(),

            PogojniStavek{ pogoj, resnica, laž } => {
                let oznaka = ŠT_OZNAK.fetch_add(1, Ordering::Relaxed);
                [
                    PogojniSkok(pogoj.clone(), format!("true_{oznaka}")).prevedi(št_klicev).as_slice(),
                    laž.prevedi(št_klicev).as_slice(),
                    &[JUMPRel(format!("end_{oznaka}"))],
                    &[Oznaka(format!("true_{oznaka}"))],
                    resnica.prevedi(št_klicev).as_slice(),
                    &[Oznaka(format!("end_{oznaka}"))],
                ].concat()
            },

            Zanka { pogoj, telo } => {
                let oznaka = ŠT_OZNAK.fetch_add(1, Ordering::Relaxed);
                let pogoj = Zanikaj(pogoj.clone()).rc();
                [
                    [Oznaka(format!("loop_{oznaka}"))].as_slice(),
                    PogojniSkok(pogoj, format!("end_{oznaka}")).prevedi(št_klicev).as_slice(),
                    telo.prevedi(št_klicev).as_slice(),
                    &[JUMPRel(format!("loop_{oznaka}"))],
                    &[Oznaka(format!("end_{oznaka}"))],
                ].concat()
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
                    izraz.clone().prevedi(št_klicev).as_slice(),
                    shrani.as_slice()
                ].concat()
            },

            PrirejanjeRef { referenca, indeks, izraz } => {
                let shrani = match indeks {
                    Some(indeks) => [
                        referenca.prevedi(št_klicev).as_slice(),
                        indeks.prevedi(št_klicev).as_slice(),
                        [STINDEXED].as_slice(),
                    ].concat(),
                    None => [
                        referenca.prevedi(št_klicev).as_slice(),
                        [Osnovni(STDY(0))].as_slice(),
                    ].concat(),
                };

                [
                    izraz.clone().prevedi(št_klicev).as_slice(),
                    shrani.as_slice()
                ].concat()
            },

            Vrni(prirejanje) => [
                prirejanje.prevedi(št_klicev).as_slice(),
                [Oznaka("vrni".to_string())].as_slice(),
            ].concat(),

            Zaporedje(vozlišča) =>
                vozlišča.into_iter()
                .map(|v| v.prevedi(št_klicev))
                .flatten()
                .collect(),

            Okvir{ zaporedje, št_spr } => Zaporedje(vec![
                Push(*št_spr).rc(),
                zaporedje.clone(),
                Pop(*št_spr).rc()
            ]).prevedi(št_klicev),

            Funkcija{ tip, ime, parametri, telo, prostor, .. } => {
                if let None = št_klicev.get(ime) {
                    return Prazno.prevedi(št_klicev)
                }

                let parametri_velikost = parametri.iter()
                    .map(|p| p.sprememba_stacka())
                    .sum::<i32>();

                let pred = Zaporedje(vec![
                    NaložiOdmik.rc(),
                    Vrh((- tip.sprememba_stacka()                    // vrni (+0)
                         - parametri_velikost                        // [ argumenti ] (+1 ...)
                         - ProgramskiŠtevec(0).sprememba_stacka()    // PC
                         - NaložiOdmik.sprememba_stacka()            // prejšnji odmik
                        ) as i32).rc(),
                    Push(*prostor).rc(),
                ]);

                let za = Zaporedje(vec![
                    Pop(*prostor).rc(),           // odstrani spremenljivke funkcije
                    ShraniOdmik.rc(),             // naloži prejšnji odmik stacka
                    DinamičniSkok.rc(),           // skoči iz funkcije na klicatelja
                ]);

                [
                    [Oznaka(format!("fn_{ime}"))].as_slice(),
                    pred.prevedi(št_klicev).as_slice(),
                    telo.prevedi(št_klicev).as_slice(),
                    [Oznaka(format!("fn_end_{ime}"))].as_slice(),
                    za.prevedi(št_klicev).as_slice(),
                ].concat()
            },

            FunkcijskiKlic{ funkcija, spremenljivke, argumenti } => {
                let (vrni, skok) = match &**funkcija {
                    Funkcija { tip, ime, .. } => (
                        Push(tip.sprememba_stacka()).rc(),
                        Klic(format!("fn_{ime}")).rc(),
                    ),
                    _ => unreachable!("Funkcijski klic vedno kliče funkcijo"),
                };

                Zaporedje(vec![
                    spremenljivke.clone(),
                    vrni,              // rezerviraj prostor za rezultat funkcije
                    argumenti.clone(), // naloži argumente
                    ProgramskiŠtevec(2).rc(),
                    skok,              // skoči v funkcijo
                    Pop(argumenti.sprememba_stacka()).rc(),
                ]).prevedi(št_klicev)
            },

            Natisni(znak) => {
                [
                    znak.prevedi(št_klicev).as_slice(),
                    [Osnovni(PUTC)].as_slice(),
                ].concat()
            },
            Preberi => vec![Osnovni(GETC)],
            Splakni => vec![Osnovni(FLUSH)],
        }
    }

    fn len(&self, št_klicev: &HashMap<String, usize>) -> usize {
        match self {
            _ => {
                self.prevedi(št_klicev)
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
        assert_eq!(Prazno.prevedi(&HashMap::new()), []);

        assert_eq!(Push(0).prevedi(&HashMap::new()), []);
        assert_eq!(Push(3).prevedi(&HashMap::new()), [
                   Osnovni(ALOC(3)),
        ]);

        assert_eq!(Pop(0).prevedi(&HashMap::new()), []);
        assert_eq!(Pop(3).prevedi(&HashMap::new()), [
                   Osnovni(ALOC(-3)),
        ]);

        assert_eq!(Vrh(-13).prevedi(&HashMap::new()), [Osnovni(TOP(-13))]);

        assert_eq!(ShraniOdmik.prevedi(&HashMap::new()), [Osnovni(SOFF)]);
        assert_eq!(NaložiOdmik.prevedi(&HashMap::new()), [Osnovni(LOFF)]);

        assert_eq!(Niz("šipa".to_string()).prevedi(&HashMap::new()), [
                   PUSHC('a'),
                   PUSHC('p'),
                   PUSHC('i'),
                   PUSHC('š'),
                   PUSHI(4),
        ]);
        assert_eq!(Real(-3.14).prevedi(&HashMap::new()), [PUSHF(-3.14)]);

        assert_eq!(Spremenljivka { tip: Tip::Real, ime: "šmir".to_string(), naslov: 55, z_odmikom: true,  spremenljiva: false }.prevedi(&HashMap::new()), [Osnovni(LDOF(55))]);
        assert_eq!(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: false, spremenljiva: false }.prevedi(&HashMap::new()), [Osnovni(LOAD(55))]);
        assert_eq!(
            Referenca(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: true, spremenljiva: false }.rc()).prevedi(&HashMap::new()),
            [PUSHREF(55, true)]);
        assert_eq!(
            Referenca(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: false, spremenljiva: false }.rc()).prevedi(&HashMap::new()),
            [PUSHREF(55, false)]);

        assert_eq!(Resnica.prevedi(&HashMap::new()), [PUSHI(1)]);
        assert_eq!(Laž.prevedi(&HashMap::new()), [PUSHI(0)]);

        assert_eq!(Add(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(&HashMap::new()), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(ADDF),
        ]);
        assert_eq!(Sub(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(&HashMap::new()), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(SUBF),
        ]);
        assert_eq!(Mul(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(&HashMap::new()), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(MULF),
        ]);
        assert_eq!(Div(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(&HashMap::new()), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(DIVF),
        ]);
        assert_eq!(Mod(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(&HashMap::new()), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(MODF),
        ]);
        assert_eq!(Pow(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).prevedi(&HashMap::new()), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   Osnovni(POWF),
        ]);

        assert_eq!(Zanikaj(Resnica.rc()).prevedi(&HashMap::new()), [
                   PUSHI(1),
                   PUSHI(1),
                   Osnovni(SUBI),
        ]);
        assert_eq!(Zanikaj(Laž.rc()).prevedi(&HashMap::new()), [
                   PUSHI(1),
                   PUSHI(0),
                   Osnovni(SUBI),
        ]);
        assert_eq!(Konjunkcija(Laž.rc(), Resnica.rc()).prevedi(&HashMap::new()), [
                   PUSHI(0),
                   PUSHI(1),
                   Osnovni(MULI),
        ]);
        assert_eq!(Disjunkcija(Laž.rc(), Resnica.rc()).prevedi(&HashMap::new()), [
                   PUSHI(0),
                   PUSHI(1),
                   Osnovni(ADDI),
                   Osnovni(POS),
        ]);

        assert_eq!(Enako(Tip::Real, Real(3.14).rc(), Real(3.14159268).rc()).prevedi(&HashMap::new()), [
                   PUSHF(3.14),
                   PUSHF(3.14159268),
                   Osnovni(SUBF),
                   Osnovni(ZERO),
        ]);
        assert_eq!(Večje(Tip::Celo, Celo(13).rc(), Celo(42).rc()).prevedi(&HashMap::new()), [
                   PUSHI(13),
                   PUSHI(42),
                   Osnovni(SUBI),
                   Osnovni(POS),

        ]);

        assert_eq!(ProgramskiŠtevec(-7).prevedi(&HashMap::new()), [PC(-7)]);
        assert_eq!(Skok("zanka".to_string()).prevedi(&HashMap::new()), [JUMPRel("zanka".to_string())]);
        assert_eq!(DinamičniSkok.prevedi(&HashMap::new()), [Osnovni(JMPD)]);
        assert_eq!(PogojniSkok(Resnica.rc(), "konec".to_string()).prevedi(&HashMap::new()), [
                   PUSHI(1),
                   JMPCRel("konec".to_string()),
        ]);

        assert_eq!(PogojniStavek { 
            pogoj: Resnica.rc(),
            resnica: Natisni(Znak('r').rc()).rc(),
            laž: Natisni(Znak('l').rc()).rc(),
        }.prevedi(&HashMap::new()), [
            PUSHI(1),
            JMPCRel("true_0".to_string()),
            PUSHC('l'),
            Osnovni(PUTC),
            JUMPRel("end_0".to_string()),
            Oznaka("true_0".to_string()),
            PUSHC('r'),
            Osnovni(PUTC),
            Oznaka("end_0".to_string()),
        ]);

        assert_eq!(Zanka {
            pogoj: Laž.rc(), 
            telo: Prirejanje { 
                spremenljivka: Spremenljivka { tip: Tip::Real, ime: "x".to_string(), naslov: 25, z_odmikom: false, spremenljiva: true }.rc(),
                izraz: Real(27.0).rc(),
            }.rc(),
        }.prevedi(&HashMap::new()), [
            Oznaka("loop_1".to_string()),
            PUSHI(1),
            PUSHI(0),
            Osnovni(SUBI),
            JMPCRel("end_1".to_string()),
            PUSHF(27.0),
            Osnovni(STOR(25)),
            JUMPRel("loop_1".to_string()),
            Oznaka("end_1".to_string()),
        ]);

        assert_eq!(Prirejanje {
            spremenljivka: Spremenljivka { tip: Tip::Real, ime: "x".to_string(), naslov: 3, z_odmikom: true, spremenljiva: false }.rc(),
            izraz: Real(-3.14).rc(),
        }.prevedi(&HashMap::new()), [
            PUSHF(-3.14),
            Osnovni(STOF(3)),
        ]);

        assert_eq!(Vrni(Prirejanje {
            spremenljivka: Spremenljivka { tip: Tip::Real, ime: "vrni".to_string(), naslov: 0, z_odmikom: true, spremenljiva: true }.rc(),
            izraz: Real(2.0).rc()
        }.rc()).prevedi(&HashMap::new()), [
                   PUSHF(2.0),
                   Osnovni(STOF(0)),
                   Oznaka("vrni".to_string()),
        ]);

        assert_eq!(Zaporedje(vec![
                             Real(1.0).rc(),
                             Real(2.0).rc(),
                             Resnica.rc(),
                             Laž.rc(),
        ]).prevedi(&HashMap::new()), [
                   PUSHF(1.0),
                   PUSHF(2.0),
                   PUSHI(1),
                   PUSHI(0),
        ]);

        assert_eq!(Okvir {
            zaporedje: Zaporedje(vec![
                                 Vrni(Prirejanje {
                                     spremenljivka: Spremenljivka { tip: Tip::Celo, ime: "vrni".to_string(), naslov: 0, z_odmikom: true, spremenljiva: true }.rc(),
                                     izraz: Spremenljivka { tip: Tip::Celo, ime: "x".to_string(), naslov: 1, z_odmikom: true, spremenljiva: false }.rc(),
                                 }.rc()).rc(),
            ]).rc(),
            št_spr: 2
        }.prevedi(&HashMap::new()), [
            Osnovni(ALOC(2)),
            Osnovni(LDOF(1)),
            Osnovni(STOF(0)),
            Oznaka("vrni".to_string()),
            Osnovni(ALOC(-2)),
        ]);

        let funkcija = Funkcija {
            tip: Tip::Real,
            ime: "ena(real)".to_string(),
            parametri: vec![
                Spremenljivka { tip: Tip::Celo, ime: "x".to_string(), naslov: 2, z_odmikom: true, spremenljiva: true }.rc(),
                Spremenljivka { tip: Tip::Celo, ime: "y".to_string(), naslov: 3, z_odmikom: true, spremenljiva: false }.rc(),
            ],
            telo: Vrni(Prirejanje {
                spremenljivka: Spremenljivka { tip: Tip::Real, ime: "vrni".to_string(), naslov: 0, z_odmikom: true, spremenljiva: true }.rc(),
                izraz: Real(1.0).rc()
            }.rc()).rc(),
            prostor: 0,
        }.rc();

        let št_klicev = HashMap::from([
            ("ena(real)".to_string(), 1),
        ]);

        assert_eq!(funkcija.clone().prevedi(&št_klicev), [
            Oznaka("fn_ena(real)".to_string()),
            Osnovni(LOFF),
            Osnovni(TOP(-5)),
            PUSHF(1.0),
            Osnovni(STOF(0)),
            Oznaka("vrni".to_string()),
            Oznaka("fn_end_ena(real)".to_string()),
            Osnovni(SOFF),
            Osnovni(JMPD),
        ]);

        assert_eq!(FunkcijskiKlic {
            funkcija: funkcija.clone(),
            spremenljivke: Zaporedje(vec![]).rc(),
            argumenti: Zaporedje(vec![Real(1.0).rc(), Real(2.0).rc()]).rc(),
        }.prevedi(&HashMap::new()), [
            Osnovni(ALOC(1)),
            PUSHF(1.0),
            PUSHF(2.0),
            PC(2),
            CALL("fn_ena(real)".to_string()),
            Osnovni(ALOC(-2))
        ]);
    }
}
