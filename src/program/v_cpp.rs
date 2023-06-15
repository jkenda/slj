use std::sync::Mutex;

use super::*;
use crate::parser::loci::interpoliraj_niz;

const ADDI: &str = "stack.back().i = stack[stack.size() - 2].i + stack.back().i; stack.pop_back();\n";
const ADDF: &str = "stack.back().f = stack[stack.size() - 2].f + stack.back().f; stack.pop_back();\n";
const SUBI: &str = "stack.back().i = stack[stack.size() - 2].i - stack.back().i; stack.pop_back();\n";
const SUBF: &str = "stack.back().f = stack[stack.size() - 2].f - stack.back().f; stack.pop_back();\n";
const MULI: &str = "stack.back().i = stack[stack.size() - 2].i * stack.back().i; stack.pop_back();\n";
const MULF: &str = "stack.back().f = stack[stack.size() - 2].f * stack.back().f; stack.pop_back();\n";
const DIVI: &str = "stack.back().i = stack[stack.size() - 2].i / stack.back().i; stack.pop_back();\n";
const DIVF: &str = "stack.back().f = stack[stack.size() - 2].f / stack.back().f; stack.pop_back();\n";
const MODI: &str = "stack.back().i = stack[stack.size() - 2].i % stack.back().i; stack.pop_back();\n";
const MODF: &str = "stack.back().f = stack[stack.size() - 2].f.modf(stack.back().f); stack.pop_back();\n";
const POWI: &str = "stack.back().i = stack[stack.size() - 2].i.pow(stack.back().i); stack.pop_back();\n";
const POWF: &str = "stack.back().f = stack[stack.size() - 2].f.pow(stack.back().f); stack.pop_back();\n";

const BOR:  &str = "stack.back().i = stack[stack.size() - 2].i |  stack.back().i; stack.pop_back();\n";
const BXOR: &str = "stack.back().i = stack[stack.size() - 2].i ^  stack.back().i; stack.pop_back();\n";
const BAND: &str = "stack.back().i = stack[stack.size() - 2].i &  stack.back().i; stack.pop_back();\n";
const BSLL: &str = "stack.back().i = stack[stack.size() - 2].i << stack.back().i; stack.pop_back();\n";
const BSLD: &str = "stack.back().i = stack[stack.size() - 2].i >> stack.back().i; stack.pop_back();\n";

const POS:  &str = "stack.back().i = stack.back().f >  0.0f\n";
const ZERO: &str = "stack.back().i = stack.back().f == 0.0f\n";

const ALOC: fn(i32)  -> String = |krat| format!("stack.resize(stack.size() + {krat});\n");
const PUSH: fn(&str) -> String = |podatek| format!("stack.push_back({podatek});\n");
const POP:  &str = "stack.pop_back();\n";

const LOAD: fn(i32) -> String = |naslov| format!("stack.push_back(stack[{naslov}]);\n");
const LDOF: fn(i32) -> String = |naslov| format!("stack.push_back(stack[addroff + {naslov}]);\n");

const STOR: fn(i32) -> String = |naslov| format!("stack[{naslov}] = stack.back(); stack.pop_back();\n");
const STOF: fn(i32) -> String = |naslov| format!("stack[addroff + {naslov}] = stack.back(); stack.pop_back();\n");
const STDY: fn(i32) -> String = |naslov| format!("auto dynaddr = stack.back(); stack.pop_back();\nstack[dynaddr + {naslov}] = stack.back(); stack.pop_back();\n");

const SOFF: &str = "addroff = stack.back().i; stack.pop_back()\n";
const LOFF: &str = "stack.push_back(Podatek { i: *addroff });\n";

const TOP:  fn(i32) -> String = |odmik| format!("addroff = stack.size() + {odmik};\n");

const GETC: &str = "stack.push_back('\0'); std::cin >> stack.back().c;\n";
const PUTC: &str = "std::cout << stack.back().c; stack.pop_back();\n";

static ŠT_SKOKOV: Mutex<usize> = Mutex::new(0);
static ŠT_FUNKCIJ: Mutex<usize> = Mutex::new(0);

impl Vozlišče {
    pub fn v_cpp(&self) -> String {
        match self {
            Prazno => "".to_string(),

            Push(krat) => if *krat > 1 { ALOC(*krat) } else { PUSH("{ .i = 0 }").repeat(*krat as usize) }
            Pop(krat)  => if *krat > 1 { ALOC(-krat) } else { POP.repeat(*krat as usize) },
            Vrh(odmik) => TOP(*odmik),

            ShraniOdmik => SOFF.to_string(),
            NaložiOdmik => LOFF.to_string(),

            Znak(znak) => PUSH(&format!("{{ '{}' }}", interpoliraj_niz(&znak.to_string()))),
            Niz(niz) => niz
                .chars().rev()
                .map(|znak| Znak(znak).v_cpp())
                .collect::<String>()
                + &PUSH(&format!("{}", niz.chars().count())),
            Celo(število) => PUSH(&format!("{{ {število:?} }}")),
            Real(število) => PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(*število) })),

            Resnica => PUSH("{ 1 }"),
            Laž     => PUSH("{ 0 }"),

            Spremenljivka{ naslov, z_odmikom, .. } => if *z_odmikom { LDOF(*naslov) } else { LOAD(*naslov) },

            Referenca(vozlišče) | RefSeznama(vozlišče) => match &**vozlišče {
                Spremenljivka { naslov, z_odmikom, .. } =>
                    (match vozlišče.tip() {
                        Tip::Seznam(..) => PUSH(&format!("{{ {} }}", naslov + 1)),
                        _ => PUSH(&format!("{{ {naslov} }}")),
                    })
                    + &if *z_odmikom {
                        LOFF.to_string()
                        + ADDI
                    }
                    else { "".to_string() },
                _ => unreachable!("Referenciramo lahko samo spremenljivko.")
            },

            Dereferenciraj(vozlišče) =>
                vozlišče.v_cpp()
                + &PUSH("stack[dynaddr]"),
            Indeksiraj { seznam_ref, indeks } =>
                Dereferenciraj(Seštevanje(Tip::Celo, seznam_ref.clone(), indeks.clone()).rc()).v_cpp(),
            Dolžina(vozlišče) => match vozlišče.tip() {
                Tip::Seznam(_, dolžina) => Celo(dolžina).rc().v_cpp(),
                Tip::RefSeznama(..) =>
                    vozlišče.v_cpp()
                    + &PUSH("stack[dynaddr - 1]"),
                _ => unreachable!("Jemanje dolžine nečesa, kar ni seznam"),
            },

            Seštevanje(Tip::Celo, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + ADDI,
            Seštevanje(Tip::Real, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + ADDF,
            Seštevanje(..) => unreachable!(),
            Odštevanje(Tip::Celo, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + SUBI,
            Odštevanje(Tip::Real, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + SUBF,
            Odštevanje(..) => unreachable!(),
            Množenje(Tip::Celo, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + MULI,
            Množenje(Tip::Real, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + MULF,
            Množenje(..) => unreachable!(),
            Deljenje(Tip::Celo, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + DIVI,
            Deljenje(Tip::Real, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + DIVF,
            Deljenje(..) => unreachable!(),
            Modulo(Tip::Celo, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + MODI,
            Modulo(Tip::Real, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + MODF,
            Modulo(..) => unreachable!(),
            Potenca(Tip::Celo, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + POWI,
            Potenca(Tip::Real, l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + POWF,
            Potenca(..) => unreachable!(),

            CeloVReal(vozlišče) =>
                vozlišče.v_cpp()
                + "stack.back().f = stack.back().i",
            RealVCelo(vozlišče) =>
                vozlišče.v_cpp()
                + "stack.back().i = stack.back().f",
            CeloVZnak(vozlišče) => vozlišče.v_cpp(),
            ZnakVCelo(vozlišče) => vozlišče.v_cpp(),

            Zanikaj(vozlišče) =>
                PUSH("{ 1 }")
                + &vozlišče.v_cpp()
                + SUBI,
            Konjunkcija(l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + MULI,
            Disjunkcija(l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + ADDI
                + POS,
            BitniAli(l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + BOR,
            BitniXor(l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + BXOR,
            BitniIn(l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + BAND,
            BitniPremikLevo(l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + BSLL,
            BitniPremikDesno(l, d) =>
                l.v_cpp()
                + &d.v_cpp()
                + BSLD,

            Enako(tip, l, d) =>
                Odštevanje(tip.clone(), l.clone(), d.clone()).v_cpp()
                + ZERO,
            NiEnako(tip, l, d) => Zanikaj(Enako(tip.clone(), l.clone(), d.clone()).rc()).v_cpp(),

            Večje(tip, l, d) =>
                Odštevanje(tip.clone(), l.clone(), d.clone()).v_cpp()
                + POS,
            Manjše(tip, l, d)      => Večje(tip.clone(), d.clone(), l.clone()).v_cpp(),
            VečjeEnako(tip, l, d)  => Zanikaj(Manjše(tip.clone(), l.clone(), d.clone()).rc()).v_cpp(),
            ManjšeEnako(tip, l, d) => VečjeEnako(tip.clone(), d.clone(), l.clone()).v_cpp(),

            ProgramskiŠtevec(_) => unreachable!(),

            Skok(_) => unreachable!(),
            DinamičniSkok => unreachable!(),
            PogojniSkok(..) => unreachable!(),

            PogojniStavek{ pogoj, resnica, laž } => {
                let (resnica_oznaka, konec_oznaka) = {
                    let mut št_skokov = ŠT_SKOKOV.lock().unwrap();
                    let resnica_oznaka = format!("resnica_{št_skokov}");
                    let konec_oznaka = format!("konec_{št_skokov}");
                    *št_skokov += 1;
                    (resnica_oznaka, konec_oznaka)
                };

                pogoj.v_cpp()
                + &format!("if (stack.back().i != 0) {{ stack.pop_back(); goto {resnica_oznaka}; }} else stack.pop_back();\n")
                + &laž.v_cpp()
                + &format!("goto {konec_oznaka};\n")
                + &format!("{resnica_oznaka}:\n")
                + &resnica.v_cpp()
                + &format!("{konec_oznaka}:\n")
            },

            Zanka { pogoj, telo } => {
                let (zanka_oznaka, konec_oznaka) = {
                    let mut št_skokov = ŠT_SKOKOV.lock().unwrap();
                    let zanka_oznaka = format!("zanka_{št_skokov}");
                    let konec_oznaka = format!("konec_{št_skokov}");
                    *št_skokov += 1;
                    (zanka_oznaka, konec_oznaka)
                };

                pogoj.v_cpp()
                + &format!("{zanka_oznaka}:\n")
                + &format!("if (stack.back().i == 0) {{ stack.pop_back(); goto {konec_oznaka}; }} else stack.pop_back();\n")
                + &telo.v_cpp()
                + &format!("goto {zanka_oznaka};\n")
            },

            Prirejanje{ spremenljivka, izraz } => {
                let (naslov, velikost, z_odmikom) = match &**spremenljivka { 
                    Spremenljivka { naslov, z_odmikom, .. } => (naslov.clone(), izraz.tip().sprememba_stacka(), *z_odmikom),
                    _ => unreachable!("Vedno prirejamo spremenljivki.")
                };

                let shrani = (naslov..naslov+velikost)
                    .map(|naslov| if z_odmikom { STOF(naslov) } else { STOR(naslov) })
                    .collect::<String>();

                izraz.clone().v_cpp()
                + &shrani
            },

            PrirejanjeRef { referenca, indeks, izraz } => {
                let shrani = match indeks {
                    Some(indeks) =>
                        referenca.v_cpp()
                        + &indeks.v_cpp()
                        + ADDI
                        + &STDY(0),
                    None =>
                        referenca.v_cpp()
                        + &STDY(0),
                };

                izraz.v_cpp()
                + &shrani
            },

            Vrni(prirejanje) => {
                let št_funkcij = ŠT_FUNKCIJ.lock().unwrap();
                let vrni = &format!("vrni_{št_funkcij}");

                prirejanje.v_cpp()
                + &format!("goto {vrni};\n")
            },

            Zaporedje(vozlišča) => vozlišča.into_iter().map(|v| v.v_cpp()).collect(),
            Okvir{ zaporedje, št_spr } =>
                ALOC(*št_spr)
                + &zaporedje.v_cpp()
                + &ALOC(-št_spr),

            Funkcija{ tip, ime, parametri, telo, prostor, .. } => {
                /*
                if let None = št_klicev.get(ime) {
                    return Prazno.prevedi(št_klicev)
                }
                */

                let parametri_velikost = parametri.iter()
                    .map(|p| p.sprememba_stacka())
                    .sum::<i32>();

                let pred =
                    "\t".to_string() + LOFF
                    + "\t" + &TOP((- tip.sprememba_stacka()                    // vrni (+0)
                         - ProgramskiŠtevec(0).sprememba_stacka()    // PC (+1)
                         - parametri_velikost               // [ argumenti ] (+2 ...)
                         - NaložiOdmik.sprememba_stacka()            // prejšnji odmik
                        ) as i32);

                let vrni = &{
                    let mut št_funkcij = ŠT_FUNKCIJ.lock().unwrap();
                    let vrni = format!("vrni_{št_funkcij}:\n");
                    *št_funkcij += 1;
                    vrni
                };
                
                let telo = telo.v_cpp()
                    .lines()
                    .map(|l| if !l.ends_with(':') { "\t" } else { "" }.to_string() + l + "\n")
                    .collect::<String>();

                let za =
                    "\t".to_string() + vrni
                    + "\t" + &Pop(*prostor).v_cpp()
                    + "\t" + SOFF
                    + "\t" + &Pop(parametri_velikost).v_cpp();

                let ime = v_veljavno_ime_funkcije(ime);

                format!("void {ime}()\n{{\n")
                    + &pred
                    + &telo
                    + &za
                + "}\n\n"
            },

            FunkcijskiKlic{ funkcija, spremenljivke, argumenti } => {
                let (vrni, klic) = match &**funkcija {
                    Funkcija { tip, ime, .. } => (
                        ALOC(tip.sprememba_stacka()),
                        format!("{}();\n", v_veljavno_ime_funkcije(ime)),
                    ),
                    _ => unreachable!("Funkcijski klic vedno kliče funkcijo"),
                };

                spremenljivke.v_cpp()
                + &vrni
                + &argumenti.v_cpp()
                + &klic
            },

            Natisni(znak) => {
                    znak.v_cpp()
                    + PUTC
            },
            Preberi => GETC.to_string(),
        }
    }
}

fn v_veljavno_ime_funkcije(ime: &str) -> String {
    format!("{}", ime.to_string()
        .replace(|c| match c {
            '('|','|' ' => true,
            _ => false,
        }, "_")
    .replace(')', ""))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn prevedi() {
        assert_eq!(Prazno.v_cpp(), "");

        assert_eq!(Push(0).v_cpp(), "");
        assert_eq!(Push(3).v_cpp(), ALOC(3));

        assert_eq!(Pop(0).v_cpp(), "");
        assert_eq!(Pop(3).v_cpp(), ALOC(-3));

        assert_eq!(Vrh(-13).v_cpp(), TOP(-13));

        assert_eq!(ShraniOdmik.v_cpp(), SOFF);
        assert_eq!(NaložiOdmik.v_cpp(), LOFF);

        assert_eq!(Niz("šipa".to_string()).v_cpp(),
        "stack.push_back({ 'a' });\nstack.push_back({ 'p' });\nstack.push_back({ 'i' });\nstack.push_back({ 'š' });\nstack.push_back(4);\n");

        assert_eq!(Real(-3.14).v_cpp(), PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(-3.14) })));

        assert_eq!(Spremenljivka { tip: Tip::Real, ime: "šmir".to_string(), naslov: 55, z_odmikom: true,  spremenljiva: false }.v_cpp(), LDOF(55));
        assert_eq!(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: false, spremenljiva: false }.v_cpp(), LOAD(55));
        assert_eq!(Referenca(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: true, spremenljiva: false }.rc()).v_cpp(),
            PUSH("{ 55 }")
            + LOFF
            + ADDI
        );
        assert_eq!(
            Referenca(Spremenljivka { tip: Tip::Celo, ime: "šmir".to_string(), naslov: 55, z_odmikom: false, spremenljiva: false }.rc()).v_cpp(),
            PUSH("{ 55 }")
        );

        assert_eq!(Resnica.v_cpp(), PUSH("{ 1 }"));
        assert_eq!(Laž.v_cpp(), PUSH("{ 0 }"));

        assert_eq!(Seštevanje(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).v_cpp(),
            PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(1.0) }))
            + &PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(2.0) }))
            + ADDF
        );
        assert_eq!(Odštevanje(Tip::Celo, Celo(1).rc(), Celo(2).rc()).v_cpp(),
            PUSH("{ 1 }")
            + &PUSH("{ 2 }")
            + SUBI
        );
        assert_eq!(Množenje(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).v_cpp(),
            PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(1.0) }))
            + &PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(2.0) }))
            + MULF
        );
        assert_eq!(Deljenje(Tip::Celo, Celo(1).rc(), Celo(2).rc()).v_cpp(),
            PUSH("{ 1 }")
            + &PUSH("{ 2 }")
            + DIVI
        );
        assert_eq!(Modulo(Tip::Real, Real(1.0).rc(), Real(2.0).rc()).v_cpp(),
            PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(1.0) }))
            + &PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(2.0) }))
            + MODF
        );
        assert_eq!(Potenca(Tip::Celo, Celo(1).rc(), Celo(2).rc()).v_cpp(),
            PUSH("{ 1 }")
            + &PUSH("{ 2 }")
            + POWI
        );

        assert_eq!(Zanikaj(Resnica.rc()).v_cpp(),
            PUSH("{ 1 }")
            + &PUSH("{ 1 }")
            + SUBI
        );
        assert_eq!(Zanikaj(Laž.rc()).v_cpp(),
            PUSH("{ 1 }")
            + &PUSH("{ 0 }")
            + SUBI
        );
        assert_eq!(Konjunkcija(Laž.rc(), Resnica.rc()).v_cpp(),
            PUSH("{ 0 }")
            + &PUSH("{ 1 }")
            + MULI
        );
        assert_eq!(Disjunkcija(Laž.rc(), Resnica.rc()).v_cpp(),
            PUSH("{ 0 }")
            + &PUSH("{ 1 }")
            + ADDI
            + POS
        );

        assert_eq!(Enako(Tip::Real, Real(3.14).rc(), Real(3.14159268).rc()).v_cpp(),
                   PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(3.14) }))
                   + &PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(3.14159268) }))
                   + SUBF
                   + ZERO
        );
        assert_eq!(Večje(Tip::Celo, Celo(13).rc(), Celo(42).rc()).v_cpp(),
                   PUSH("{ 13 }")
                   + &PUSH("{ 42 }")
                   + SUBI
                   + POS
        );

        assert_eq!(PogojniStavek { 
            pogoj: Resnica.rc(),
            resnica: Natisni(Znak('r').rc()).rc(),
            laž: Natisni(Znak('l').rc()).rc(),
        }.v_cpp(),
            PUSH("{ 1 }")
            + "if (stack.back().i != 0) { stack.pop_back(); goto resnica_0; } else stack.pop_back();\n"
            + &PUSH("{ 'l' }")
            + PUTC
            + "goto konec_0;\n"
            + "resnica_0:\n"
            + &PUSH("{ 'r' }")
            + PUTC
            + "konec_0:\n"
        );

        assert_eq!(Zanka {
            pogoj: Laž.rc(), 
            telo: Prirejanje { 
                spremenljivka: Spremenljivka { tip: Tip::Real, ime: "x".to_string(), naslov: 25, z_odmikom: false, spremenljiva: true }.rc(),
                izraz: Real(27.0).rc(),
            }.rc(),
        }.v_cpp(),
            PUSH("{ 0 }")
            + "zanka_1:\n"
            + "if (stack.back().i == 0) { stack.pop_back(); goto konec_1; } else stack.pop_back();\n"
            + &PUSH("{ 1104674816 }")
            + &STOR(25)
            + "goto zanka_1;\n"
        );

        assert_eq!(Prirejanje {
            spremenljivka: Spremenljivka { tip: Tip::Real, ime: "x".to_string(), naslov: 3, z_odmikom: true, spremenljiva: false }.rc(),
            izraz: Real(-3.14).rc(),
        }.v_cpp(),
            PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(-3.14) }))
            + &STOF(3)
        );

        assert_eq!(Vrni(Prirejanje {
            spremenljivka: Spremenljivka { tip: Tip::Real, ime: "vrni".to_string(), naslov: 0, z_odmikom: true, spremenljiva: true }.rc(),
            izraz: Real(2.0).rc()
        }.rc()).v_cpp(),
            PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(2.0) }))
            + &STOF(0)
            + "goto vrni_0;\n"
        );

        assert_eq!(Zaporedje(vec![
                             Real(1.0).rc(),
                             Real(2.0).rc(),
                             Resnica.rc(),
                             Laž.rc(),
        ]).v_cpp(),
            PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(1.0) }))
            + &PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(2.0) }))
            + &PUSH("{ 1 }")
            + &PUSH("{ 0 }")
        );

        assert_eq!(Okvir {
            zaporedje: Zaporedje(vec![
                                 Vrni(Prirejanje {
                                     spremenljivka: Spremenljivka { tip: Tip::Celo, ime: "vrni".to_string(), naslov: 0, z_odmikom: true, spremenljiva: true }.rc(),
                                     izraz: Spremenljivka { tip: Tip::Celo, ime: "x".to_string(), naslov: 1, z_odmikom: true, spremenljiva: false }.rc(),
                                 }.rc()).rc(),
            ]).rc(),
            št_spr: 2
        }.v_cpp(),
            ALOC(2)
            + &LDOF(1)
            + &STOF(0)
            + "goto vrni_0;\n"
            + &ALOC(-2)
        );

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

        assert_eq!(funkcija.clone().v_cpp(),
            "void ena_real()\n{\n".to_string()
            + LOFF
            + &TOP(-5)
            + &PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(1.0) }))
            + &STOF(0)
            + "goto vrni_1;\n"
            + "vrni_0:\n"
            + SOFF
            + &ALOC(-2)
            + "}\n"
        );

        assert_eq!(FunkcijskiKlic {
            funkcija: funkcija.clone(),
            spremenljivke: Zaporedje(vec![]).rc(),
            argumenti: Zaporedje(vec![Real(1.0).rc(), Real(2.0).rc()]).rc(),
        }.v_cpp(),
            ALOC(1)
            + &PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(1.0) }))
            + &PUSH(&format!("{{ {} }}", unsafe { std::mem::transmute::<f32, i32>(2.0) }))
            + "ena_real();\n"
        );
    }
}
