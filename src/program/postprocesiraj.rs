use super::*;

impl Postprocesiraj for Vec<UkazPodatekRelative> {
    fn postprocesiraj(&self) -> Vec<UkazPodatek> {
        let mut postproc1 = self.clone();
        let mut postproc: Vec<UkazPodatek> = Vec::new();
        let mut oznake_vrstic: HashMap<String, u32> = HashMap::new();

        // nadomesti "vrni" z JUMP x
        let mut i: usize = 0;
        while i < postproc1.len() {
            if let Oznaka(oznaka) = &postproc1[i] {
                if oznaka == "vrni" {
                    // poišči oznako za konec funkcije
                    // nadomesti oznako "vrni" z relativnim skokom
                    let mut konec = i + 1;
                    loop {
                        match &postproc1[konec] {
                            Oznaka(oznaka) => if oznaka.starts_with("konec_funkcije") {
                                postproc1[konec] = JUMPRelative(OdmikIme::Ime(oznaka.split_whitespace().last().unwrap().to_string()));
                                break;
                            },
                            _ => konec += 1,
                        }
                    }
                }
            }
            i += 1;
        }

        // preberi oznake vrstic in jih odstrani
        let mut i: usize = 0;
        while i < postproc1.len() {
            match &postproc1[i] {
                Oznaka(oznaka) => {
                    oznake_vrstic.insert(oznaka.clone(), i as u32);
                    postproc.remove(i);
                }
                _ => i += 1,
            }
        }

        // relativni skok -> absolutni skok
        for (št_vrstice, ukaz_podatek) in postproc1.iter().enumerate() {
            postproc.push(match ukaz_podatek {
                Osnovni(osnovni_ukaz) => osnovni_ukaz.clone(),
                JUMPRelative(odmik_ime) => match odmik_ime {
                    OdmikIme::Odmik(rel_skok) => JUMP((št_vrstice as isize + rel_skok) as u32),
                    OdmikIme::Ime(ime)        => JUMP(oznake_vrstic[ime]),
                },
                JMPCRelative(rel_skok) => JMPC((št_vrstice as i32 + rel_skok) as u32),
                PC(odmik) => PUSH(Podatek { i: št_vrstice as i32 + odmik }),
                Oznaka(_) => NOOP,
            });
        }

        postproc
    }
}

