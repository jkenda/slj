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

            Push(krat) => iter::repeat(Osnovni(PUSH(Podatek { i: 0 }))).take(*krat).collect(),
            Pop(krat) => iter::repeat(Osnovni(POP).clone()).take(*krat).collect(),
            Vrh(odmik) => [Osnovni(TOP(*odmik as i32))].to_vec(),

            ShraniOdmik => [Osnovni(SOFF)].to_vec(),
            NaložiOdmik => [Osnovni(LOFF)].to_vec(),

            Niz(niz) => niz.chars().rev()
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

            Zanikaj(vozlišče) => [
                vozlišče.prevedi().as_slice(),
                [Osnovni(NEG)].as_slice(),
            ].concat(),
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

            Zanka{ pogoj, telo } => PogojniStavek{
                    pogoj: pogoj.clone(),
                    resnica: Zaporedje(vec![
                        telo.clone(),
                        Skok(OdmikIme::Odmik(-(telo.len() as isize) - pogoj.len() as isize - 2)).rc()
                    ]).rc(),
                    laž: Prazno.rc(),
                }.prevedi(),

            Prirejanje{ spremenljivka, izraz, z_odmikom } => {
                let naslov = if let Spremenljivka { ime: _, naslov, z_odmikom: _ } = &**spremenljivka { naslov.clone() } else { 0 };
                let shrani = if *z_odmikom { Osnovni(STOF(naslov)) } else { Osnovni(STOR(naslov)) };
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
                let skok = Skok(OdmikIme::Ime(format!(".{}", if let Funkcija { ime, parametri: _, telo: _, prostor: _ } = &**funkcija { ime } else { "" })));
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
                        Niz(_) => iter::repeat(Osnovni(PRTC)).take(izraz.sprememba_stacka() as usize).collect(),
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
