use std::rc::Rc;

#[allow(dead_code)]
#[derive(Clone)]
pub enum OdmikIme {
    Odmik(isize),
    Ime(String),
}

impl ToString for OdmikIme {
    fn to_string(&self) -> String {
        match self {
            OdmikIme::Odmik(odmik) => odmik.to_string(),
            OdmikIme::Ime(ime) => ime.clone(),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Vozlišče {
    Prazno,

    Push(usize),
    Pop(usize),
    Vrh(isize),

    ShraniOdmik,
    NaložiOdmik,

    Niz(String),
    Število(f32),
    Spremenljivka{ ime: String, naslov: i64, z_odmikom: bool },

    Resnica,
    Laž,

    Seštevanje(Rc<Vozlišče>, Rc<Vozlišče>),
    Odštevanje(Rc<Vozlišče>, Rc<Vozlišče>),
    Množenje(Rc<Vozlišče>, Rc<Vozlišče>),
    Deljenje(Rc<Vozlišče>, Rc<Vozlišče>),
    Modulo(Rc<Vozlišče>, Rc<Vozlišče>),
    Potenca(Rc<Vozlišče>, Rc<Vozlišče>),

    Zanikaj(Rc<Vozlišče>),
    Konjunkcija(Rc<Vozlišče>, Rc<Vozlišče>),
    Disjunkcija(Rc<Vozlišče>, Rc<Vozlišče>),
    Enako(Rc<Vozlišče>, Rc<Vozlišče>),
    Večje(Rc<Vozlišče>, Rc<Vozlišče>),
    VečjeEnako(Rc<Vozlišče>, Rc<Vozlišče>),
    Manjše(Rc<Vozlišče>, Rc<Vozlišče>),
    ManjšeEnako(Rc<Vozlišče>, Rc<Vozlišče>),

    ProgramskiŠtevec(isize),
    Skok(OdmikIme),
    DinamičniSkok,
    PogojniSkok(Rc<Vozlišče>, isize),

    PogojniStavek{ pogoj: Rc<Vozlišče>, resnica: Rc<Vozlišče>, laž: Rc<Vozlišče> },
    Zanka{ pogoj: Rc<Vozlišče>, telo: Rc<Vozlišče> },

    Prirejanje{ spremenljivka: Rc<Vozlišče>, izraz: Rc<Vozlišče>, z_odmikom: bool },

    Vrni(Rc<Vozlišče>),
    Zaporedje(Vec<Rc<Vozlišče>>),
    Okvir{ zaporedje: Rc<Vozlišče>, št_spr: usize },

    Funkcija{ ime: String, parametri: Vec<Vozlišče>, telo: Rc<Vozlišče>, prostor: usize },
    FunkcijskiKlic{ funkcija: Rc<Vozlišče>, argumenti: Rc<Vozlišče> },

    Natisni(Vec<Rc<Vozlišče>>),
}

use Vozlišče::*;

impl ToString for Vozlišče {
    fn to_string(&self) -> String {
        match self {
            Prazno => "()".to_owned(),

            Niz(niz) => "\"".to_owned() 
                    + &niz
                    .replace("\\", "\\\\")
                    .replace("\n", "\\n")
                    .replace("\t", "\\t")
                    .replace("\r", "\\r")
                    .replace("\"", "\\\"")
                    .replace("\'", "\\'")
                    + "\"",

            Število(število) => število.to_string(),
            Spremenljivka{ ime, naslov, z_odmikom } => format!("{} ({}{})", ime,
                if *z_odmikom { "+" } else { "@" }, naslov),

            Resnica => "resnica".to_owned(),
            Laž     => "laž".to_owned(),

            Seštevanje(..) => "+".to_owned(),
            Odštevanje(..) => "-".to_owned(),
            Množenje(..)   => "*".to_owned(),
            Deljenje(..)   => "/".to_owned(),
            Modulo(..)     => "mod".to_owned(),
            Potenca(..)    => "^".to_owned(),

            Zanikaj(..)     => "ne".to_owned(),
            Konjunkcija(..) => "in".to_owned(),
            Disjunkcija(..) => "ali".to_owned(),
            Enako(..)       => "==".to_owned(),
            Večje(..)       => ">".to_owned(),
            VečjeEnako(..)  => ">=".to_owned(),
            Manjše(..)      => "<".to_owned(),
            ManjšeEnako(..) => "<=".to_owned(),

            PogojniStavek{..} => "če".to_owned(),
            Zanka{..}         => "dokler".to_owned(),

            Prirejanje{ spremenljivka, .. } => spremenljivka.to_string() + " = ",
            Vrni(_) => "vrni".to_owned(),

            Funkcija{ ime, parametri, .. } => {
                let parametri = parametri.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(", ");
                format!("{}({})", ime, parametri)
            },
            FunkcijskiKlic{ funkcija, .. } => format!("{}(", if let Funkcija { ime, parametri: _, telo: _, prostor: _ } = &**funkcija { ime } else { "" }),
            _ => "".to_owned(),
        }
    }
}

impl Vozlišče {

    pub fn drevo(&self, globina: usize) -> String {
        match self {
            Prazno => "\t".repeat(globina) + "()\n",

            Niz(_) | Število(_) | Spremenljivka {..} | Resnica | Laž => 
                "\t".repeat(globina) + &self.to_string() + "\n",

            Potenca(l, d) | Množenje(l, d) | Deljenje(l, d) | Modulo(l, d) | Seštevanje(l, d) | Odštevanje(l, d)
                | Konjunkcija(l, d) | Disjunkcija(l, d) 
                | Enako(l, d) | Večje(l, d) | VečjeEnako(l, d) | Manjše(l, d) | ManjšeEnako(l, d) =>
                "\t".repeat(globina) + &self.to_string() + "\n"
                + &l.drevo(globina + 1) 
                + &d.drevo(globina + 1),

            Zanikaj(vozlišče) =>
                "\t".repeat(globina) + &self.to_string() + "\n"  
                + &vozlišče.drevo(globina + 1),

            PogojniStavek { pogoj, resnica, laž } =>
                "\t".repeat(globina) + "če (\n" 
                + &pogoj.drevo(globina + 1) 
                + &"\t".repeat(globina) + ")\n"
                + &resnica.drevo(globina + 1)
                + &match &**laž {
                    Prazno => "".to_owned(),
                    _ => "\t".repeat(globina) + &"čene\n".to_owned() 
                        + &laž.drevo(globina + 1),
                },

            Zanka { pogoj, telo } => 
                "\t".repeat(globina) + "dokler(\n"
                + &pogoj.drevo(globina + 1)
                + &"\t".repeat(globina) + ") {\n"
                + &telo.drevo(globina + 1)
                + &"\t".repeat(globina) + "}\n",

            Prirejanje{ spremenljivka: _, izraz, z_odmikom: _ } => 
                "\t".repeat(globina) + &self.to_string() + "\n" 
                + &izraz.drevo(globina + 1),

            Vrni(prirejanje) => 
                "\t".repeat(globina) + "vrni (\n"
                + &prirejanje.drevo(globina + 1)
                + &"\t".repeat(globina) + ")\n",

            Zaporedje(vozlišča) => vozlišča.into_iter().map(|v| v.drevo(globina + 1)).collect::<Vec<String>>().join(&("\t".repeat(globina) + ",\n")),

            Okvir{ zaporedje, .. } => 
                "\t".repeat(globina) + "{\n" 
                + &zaporedje.drevo(globina + 1)
                + &"\t".repeat(globina) + "}\n",

            Funkcija { ime: _, parametri: _, telo, prostor: _ } =>
                "\t".repeat(globina) + &self.to_string() + " {\n"
                + &telo.drevo(globina + 1)
                + &"\t".repeat(globina) + "}\n",

            FunkcijskiKlic { funkcija: _, argumenti } =>
                "\t".repeat(globina) + &self.to_string() + "(\n"
                + &argumenti.drevo(globina + 1)
                + ")\n",

            Natisni(zaporedje) => 
                "\t".repeat(globina) + "natisni(\n" 
                + &zaporedje
                    .into_iter()
                    .map(|v| v.drevo(globina + 1))
                    .collect::<Vec<String>>()
                    .join(&("\t".repeat(globina) + ",\n"))
                + &"\t".repeat(globina) + ")\n",

            _ => "".to_owned()
        }
    }

    pub fn prevedi(&self) -> String {
        match self {
            Prazno => String::new(),

            Push(krat) => "PUSH #0\n".repeat(*krat),
            Pop(krat) => "POP\n".repeat(*krat),
            Vrh(odmik) => format!("TOP {}\n", odmik),

            ShraniOdmik => "SOFF\n".to_owned(),
            NaložiOdmik => "LOFF\n".to_owned(),

            Niz(niz) => niz.chars().rev()
                .map(|c| format!("PUSH '{}'\n", c.to_string()
                                 .replace("\\", "\\\\")
                                 .replace("\n", "\\n")
                                 .replace("\t", "\\t")
                                 .replace("\r", "\\r")
                                 .replace("\"", "\\\"")
                                 .replace("\'", "\\'")
                                ))
                .collect::<Vec<String>>()
                .join(""),

            Število(število) => format!("PUSH #{}\n", število),
            Spremenljivka{ ime: _, naslov, z_odmikom } => if *z_odmikom { format!("LDOF +{}\n", naslov) } else { format!("LOAD @{}\n", naslov) },

            Resnica => "PUSH #1\n".to_owned(),
            Laž     => "PUSH #0\n".to_owned(),

            Seštevanje(l, d) => 
                d.prevedi() 
                + &l.prevedi() 
                + "ADD\n",

            Odštevanje(l, d) => 
                d.prevedi() 
                + &l.prevedi() 
                + "SUB\n",

            Množenje(l, d) =>
                d.prevedi() 
                + &l.prevedi() 
                + "MUL\n",

            Deljenje(l, d) =>
                d.prevedi() 
                + &l.prevedi() 
                + "DIV\n",

            Modulo(l, d) =>
                d.prevedi() 
                + &l.prevedi() 
                + "MOD\n",

            Potenca(l, d) =>
                d.prevedi() 
                + &l.prevedi() 
                + "POW\n",

            Zanikaj(vozlišče) => Odštevanje(Število(1.0).rc(), vozlišče.clone()).prevedi(),
            Konjunkcija(l, d) => Množenje(l.clone(), d.clone()).prevedi(),
            Disjunkcija(l, d) =>
                Seštevanje(l.clone(), d.clone()).prevedi() 
                + "POS\n",

            Enako(l, d) =>
                Odštevanje(l.clone(), d.clone()).prevedi() 
                + "ZERO\n",

            Večje(l, d) =>
                Odštevanje(l.clone(), d.clone()).prevedi() 
                + "POS\n",

            VečjeEnako(l, d) =>
                Zaporedje(vec![
                    Odštevanje(l.clone(), d.clone()).rc(), 
                    Spremenljivka { ime: String::new(), naslov: -1, z_odmikom: true }.rc()
                ]).prevedi()
                + "POS\n"
                + "ZERO\n",

            Manjše(l, d)      => Večje(d.clone(), l.clone()).prevedi(),
            ManjšeEnako(l, d) => VečjeEnako(d.clone(), l.clone()).prevedi(),

            ProgramskiŠtevec(odmik) => format!("PC {}\n", odmik),
            Skok(odmik_ime) => format!("JUMP {}{}\n", 
                                       if let OdmikIme::Odmik(odmik) = odmik_ime { if *odmik >= 0 { "+" } else { "" } } else { "" },
                                       &odmik_ime.to_string()),

            DinamičniSkok => "JMPD\n".to_owned(),
            PogojniSkok(pogoj, skok) =>
                pogoj.prevedi()
                + &format!("JMPC {}{}\n", if *skok >= 0 { "+" } else { "" }, skok),

            PogojniStavek{ pogoj, resnica, laž } => {
                let skok = Skok(OdmikIme::Odmik((resnica.len() + 1) as isize)).rc();
                Zaporedje(vec![
                          PogojniSkok(pogoj.clone(), (laž.len() + skok.len() + 1) as isize).rc(),
                          laž.clone(),
                          skok,
                          resnica.clone(),
                ]).prevedi()
            },

            Zanka{ pogoj, telo } => PogojniStavek{
                    pogoj: pogoj.clone(),
                    resnica: Zaporedje(vec![
                        telo.clone(),
                        Rc::new(Skok(OdmikIme::Odmik(-(telo.len() as isize) - pogoj.len() as isize - 2)))
                    ]).rc(),
                    laž: Prazno.rc(),
                }.prevedi(),

            Prirejanje{ spremenljivka, izraz, z_odmikom } => {
                let naslov = if let Spremenljivka { ime: _, naslov, z_odmikom: _ } = &**spremenljivka { naslov.clone() } else { 0i64 };
                let shrani = if *z_odmikom { format!("STOF +{}\n", naslov) } else { format!("STOR @{}\n", naslov) };
                izraz.clone().prevedi()
                + &shrani
            },

            Vrni(prirejanje) => prirejanje.prevedi(),
            Zaporedje(vozlišča) => vozlišča.into_iter().map(|v| v.prevedi()).collect::<Vec<String>>().join(""),
            Okvir{ zaporedje, št_spr } => Zaporedje(vec![
                Push(*št_spr).rc(),
                zaporedje.clone(),
                Pop(*št_spr).rc()
            ]).prevedi(),

            Funkcija{ ime, parametri, telo, prostor } => {
                let pred = Zaporedje(vec![
                    NaložiOdmik.rc(),
                       Vrh(
                           -NaložiOdmik.sprememba_stacka()
                           - parametri.len() as isize
                           - ProgramskiŠtevec(0).sprememba_stacka()
                           - Push(1).sprememba_stacka()).rc(),
                       Push(*prostor).rc(),
                ]);

                let za = Zaporedje(vec![
                    Pop(*prostor).rc(),
                    ShraniOdmik.rc(),
                    Pop(parametri.len()).rc(),
                    DinamičniSkok.rc()
                ]);

                Skok(OdmikIme::Odmik((1 + pred.len() + telo.len() + za.len()) as isize)).prevedi()
                + &format!(".{}\n", ime)
                + &pred.prevedi()
                + &telo.prevedi()
                + &format!(".0konec{}\n", ime)
                + &za.prevedi()
            },

            FunkcijskiKlic{ funkcija, argumenti } => {
                let vrni = Push(1);
                let skok = Skok(OdmikIme::Ime(format!(".{}", if let Funkcija { ime, parametri: _, telo: _, prostor: _ } = &**funkcija { ime } else { "" })));
                let pc   = ProgramskiŠtevec((1 + argumenti.len() + skok.len()) as isize);

                Zaporedje(vec![
                    vrni.rc(),
                    pc.rc(),
                    argumenti.rc(),
                    skok.rc(),
                ]).prevedi()
            },

            Natisni(izrazi) => izrazi.into_iter()
                .map(|izraz| {
                    izraz.prevedi()
                    + &match &**izraz {
                        Niz(_) => "PRTC\n".repeat(izraz.sprememba_stacka() as usize),
                        Število(_) | Spremenljivka{..} => "PRTN\n".to_owned(),
                        _ => panic!("Neveljaven print: {}", izraz.to_string()),
                    }
                }).collect::<Vec<String>>().join(""),
        }

    }

    fn len(&self) -> usize {
        match self {
            _ => {
                let ukazi = self.prevedi();
                if !ukazi.contains('\n') {
                    0
                }
                else {
                    ukazi.trim()
                        .split('\n')
                        .collect::<Vec<&str>>()
                        .into_iter()
                        .filter(|u| !u.starts_with('.'))
                        .count()
                }
            },
        }
    }

    pub fn rc(&self) -> Rc<Self> {
        Rc::new(self.clone())
    }

    fn sprememba_stacka(&self) -> isize {
        match self {
            Število(_) => 1,
            Niz(niz)   => niz.len() as isize,
            _ => 0
        }
    }

}

