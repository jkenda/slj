use super::*;

impl Postprocesiraj for Vec<UkazPodatekRelative> {
    fn postprocesiraj(&self) -> (Vec<UkazPodatek>, Vec<Tip>) {
        let mut postproc1 = self.clone();
        let mut postproc: Vec<UkazPodatek> = Vec::new();
        let mut push_tipi = Vec::new();

        // nadomesti "vrni" z JUMP x
        let mut i: usize = 0;
        while i < postproc1.len() {
            if let Oznaka(oznaka) = &postproc1[i] {
                if oznaka == "vrni" {
                    // poišči oznako za konec funkcije
                    // nadomesti oznako "vrni" z relativnim skokom do oznake
                    let mut konec = i + 1;
                    loop {
                        match &postproc1[konec] {
                            Oznaka(oznaka) => if oznaka.starts_with("konec_funkcije") {
                                postproc1[i] = JUMPRelative(OdmikIme::Ime(oznaka.clone()));
                                break;
                            }
                            else {
                                konec += 1;
                            },
                            _ => konec += 1,
                        }
                    }
                }
            }
            i += 1;
        }

        let mut oznake_vrstic: HashMap<String, i32> = HashMap::new();

        // preberi oznake vrstic in jih odstrani
        let mut i: usize = 0;
        while i < postproc1.len() {
            match &postproc1[i] {
                Oznaka(oznaka) => {
                    oznake_vrstic.insert(oznaka.clone(), i as i32);
                    postproc1.remove(i);
                },
                JUMPRelative(OdmikIme::Odmik(1)) => {
                    postproc1.remove(i);
                },
                _ => i += 1,
            }
        }

        // relativni skok -> absolutni skok
        for (št_vrstice, ukaz_podatek) in postproc1.iter().enumerate() {
            postproc.push(match ukaz_podatek {
                Osnovni(osnovni_ukaz) => osnovni_ukaz.clone(),
                PUSHI(celo) => { push_tipi.push(Tip::Celo); PUSH(Podatek { i: *celo }) },
                PUSHF(real) => { push_tipi.push(Tip::Real); PUSH(Podatek { f: *real }) },
                PUSHC(znak) => { push_tipi.push(Tip::Znak); PUSH(Podatek { c: *znak }) },
                JUMPRelative(odmik_ime) => match odmik_ime {
                    OdmikIme::Odmik(rel_skok) => JUMP(št_vrstice as i32 + rel_skok),
                    OdmikIme::Ime(ime)        => JUMP(oznake_vrstic[ime]),
                },
                JMPCRelative(rel_skok) => JMPC(št_vrstice as i32 + rel_skok),
                PC(odmik) => { push_tipi.push(Tip::Celo); PUSH(Podatek { i: št_vrstice as i32 + odmik }) },
                Oznaka(_) => NOOP,
            });
        }

        (postproc, push_tipi)
    }
}

