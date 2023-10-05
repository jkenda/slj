use super::*;

impl Postprocesiraj for Vec<UkazPodatekRelative> {

    // nadomesti "vrni" z JUMP x
    fn vrni_v_oznake(&self) -> Vec<UkazPodatekRelative> {
        let mut postproc1 = self.clone();
        
        let mut i: usize = 0;
        while i < postproc1.len() {
            if let Oznaka(oznaka) = &postproc1[i] {
                if oznaka == "vrni" {
                    // poišči oznako za konec funkcije
                    // nadomesti oznako "vrni" z relativnim skokom do oznake
                    let mut konec = i + 1;
                    loop {
                        match &postproc1[konec] {
                            Oznaka(oznaka) => if oznaka.starts_with("fn_end") {
                                postproc1[i] = JUMPRel(oznaka.clone());
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

        postproc1
    }

    fn postprocesiraj(&self) -> (Vec<UkazPodatek>, Vec<Tip>) {
        let mut postproc1 = self.clone();
        let mut push_tipi = Vec::new();

        let mut oznake_vrstic: HashMap<String, i32> = HashMap::new();

        // preberi oznake vrstic in jih odstrani
        let mut i: usize = 0;
        while i < postproc1.len() {
            match &postproc1[i] {
                Oznaka(oznaka) => {
                    oznake_vrstic.insert(oznaka.clone(), i as i32);
                    postproc1.remove(i);
                },
                JUMPRel(oznaka) if i + 1 < postproc1.len() && postproc1[i + 1] == Oznaka(oznaka.clone()) => {
                    postproc1.remove(i);
                },
                _ => i += 1,
            }
        }

        // relativni skok -> absolutni skok
        let postproc = postproc1.iter()
            .enumerate()
            .map(|(i, ukaz_podatek)|
                match ukaz_podatek {
                    Osnovni(osnovni_ukaz) => osnovni_ukaz.clone(),
                    PUSHI(celo) => { push_tipi.push(Tip::Celo); PUSH(Podatek { i: *celo }) },
                    PUSHF(real) => { push_tipi.push(Tip::Real); PUSH(Podatek { f: *real }) },
                    PUSHC(znak) => { push_tipi.push(Tip::Znak); PUSH(Podatek { c: *znak }) },
                    JUMPRel(oznaka) => JUMP(oznake_vrstic[oznaka]),
                    JMPCRel(oznaka) => JMPC(oznake_vrstic[oznaka]),
                    CALL(oznaka) => JUMP(oznake_vrstic[oznaka]),
                    PC(odmik) => { push_tipi.push(Tip::Celo); PUSH(Podatek { i: i as i32 + odmik }) },
                    Oznaka(_) => NOOP,

                    _ => unreachable!("ostali ukazi so namenjeni optimizaciji za x86")
                })
        .collect();

        (postproc, push_tipi)
    }
}

