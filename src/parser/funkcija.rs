use super::*;
use argumenti::*;

impl<'a> Parser<'a> {
    pub fn funkcija(&mut self, ime: &Žeton, izraz: &[Žeton]) -> Result<Rc<Vozlišče>, Napake> {
        let (_, _, izraz) = loči_spredaj(izraz, &["("])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '('"))??;
        let (parametri_izraz, _, izraz) = loči_spredaj(izraz, &[")"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan ')'"))??;
        let (tip_izraz, oklepaj, izraz) = loči_spredaj(izraz, &["{"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '{'"))??;

        let tip = match tip_izraz {
            [] => Ok(Tip::Brez),
            [Ločilo("->", ..)] => Err(Napake::from_zaporedje(&[*oklepaj], E5, "Za '->' pričakovan tip")),
            [Ločilo("->", ..), ostanek @ ..] => Tip::from(ostanek, &self.konstante),
            _ =>  Err(Napake::from_zaporedje(tip_izraz, E5, "Pričakovan '-> <tip>'")),
        }?;

        let (telo, _, prazno) = loči_zadaj(izraz, &["}"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '}'"))??;

        if prazno != [] {
            return Err(Napake::from_zaporedje(prazno, E3, "Izraz funkcije se mora zaključiti z '}'"));
        }

        let mut _naslov_nove = 0;
        let mut naslov = |tip: &Tip| {
            let naslov = _naslov_nove;
            _naslov_nove += tip.sprememba_stacka();
            naslov
        };

        let mut spremenljivka = |tip: &Tip, ime: &str| {
            Spremenljivka {
                tip: tip.clone(),
                ime: ime.to_string(),
                naslov: naslov(&tip),
                z_odmikom: true,
                spremenljiva: true
            }.rc()
        };
                
        let mut spr_funkcije = HashMap::from([
            ("0_vrni", spremenljivka(&tip, "0_vrni")),
        ]);

        let mut parametri = Vec::new(); 
        let mut napake = Napake::new();

        for parameter in parametri_izraz.split(|p| if let Ločilo(",", ..) = p { true } else { false }) {
            // imena parametrov, ločena z vejicami
            match parameter {
                [] => break,
                [Ime(..)] => (),
                _ => _ = napake.add_napaka(Napaka::from_zaporedje(parameter, E3, "Neveljavno ime parametra")),
            }

            let (ime, dvopičje, tip) = loči_spredaj(parameter, &[":"])
                .ok_or(Napake::from_zaporedje(parameter, E5, "Pričakovano ':'"))??;

            if tip == [] {
                return Err(Napake::from_zaporedje(&[*dvopičje], E5, "Za ':' pričakovan tip"))
            }

            let ime = &ime[0];
            let tip = Tip::from(tip, &self.konstante)?;

            if spr_funkcije.contains_key(ime.as_str()) {
                return Err(Napake::from_zaporedje(&[*ime], E7, "Imena parametrov morajo biti unikatna"))
            }
            else {
                let spr = spremenljivka(&tip, ime.as_str());
                spr_funkcije.insert(ime.as_str(), spr.clone());
                parametri.push(spr);
            }
        }

        let podpis_funkcije = Self::podpis_funkcije(ime, parametri.iter().map(|p| p.tip()).collect::<Vec<Tip>>().as_slice());
        spr_funkcije.insert("0_PC", spremenljivka(&Tip::Celo, "0_PC"));
        spr_funkcije.insert("0_OF", spremenljivka(&Tip::Celo, "0_OF"));

        let mut okolje_funkcije = self.clone();
        okolje_funkcije.znotraj_funkcije = true;

        okolje_funkcije.spremenljivke_stack.push(spr_funkcije.clone());
        okolje_funkcije.spremenljivke.extend(spr_funkcije);
        okolje_funkcije.funkcije.insert(podpis_funkcije.clone(), Funkcija { 
            tip: tip.clone(),
            ime: podpis_funkcije.clone(),
            parametri: parametri.clone(),
            telo: Prazno.rc(),
            prostor: 0,
        }.rc());

        let telo = okolje_funkcije.zaporedje(telo)?;
        let spr_funkcije = okolje_funkcije.spremenljivke_stack.last().unwrap();
        let prostor = spr_funkcije.values().map(|s| s.sprememba_stacka()).sum::<i32>()
            - spr_funkcije["0_vrni"].sprememba_stacka()
            - spr_funkcije["0_PC"].sprememba_stacka()
            - parametri.iter().map(|p| p.sprememba_stacka()).sum::<i32>()
            - spr_funkcije["0_OF"].sprememba_stacka();
        let fun = Funkcija { tip, ime: podpis_funkcije.clone(), parametri, telo, prostor }.rc();

        for (podpis, št_klicev) in okolje_funkcije.št_klicev {
            match self.št_klicev.get_mut(&podpis) {
                Some(št) => *št += št_klicev,
                None => { self.št_klicev.insert(podpis, št_klicev); }
            }
        }

        self.funkcije.insert(podpis_funkcije, fun.clone());
        self.funkcije_vec.push(fun.clone());
        Ok(fun)
    }

    pub fn funkcijski_klic_zavrzi_izhod(&mut self, ime: &Žeton, argumenti: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let klic = self.funkcijski_klic(ime, argumenti)?;
        let velikost = klic.tip().sprememba_stacka();

        Ok(Zaporedje(vec![
            klic,
            Pop(velikost).rc(),
        ]).rc())
    }

    pub fn funkcijski_klic<'b>(&mut self, ime: &Žeton, argumenti: &'b[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let Argumenti { tipi, spremenljivke, argumenti } = self.argumenti(argumenti)?;
        let podpis_funkcije = Self::podpis_funkcije(ime, tipi.as_slice());

        let funkcija = self.funkcije.get(&podpis_funkcije)
            .ok_or(Napake::from_zaporedje(&[*ime], E2, &format!("Funkcija '{podpis_funkcije}' ne obstaja")))?
            .clone();

        match self.št_klicev.get_mut(&podpis_funkcije) {
            Some(št_klicev) => *št_klicev += 1,
            None => { self.št_klicev.insert(podpis_funkcije, 1); },
        }

        Ok(FunkcijskiKlic { funkcija, spremenljivke: Zaporedje(spremenljivke).rc(), argumenti: Zaporedje(argumenti).rc() }.rc())
    }

    pub fn multi_klic<'b>(&mut self, ime: &'b Žeton<'a>, argumenti_izraz: &'b [Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> where 'a: 'b {
        let Argumenti { tipi, spremenljivke, argumenti } = self.argumenti(argumenti_izraz)?;
        let mut funkcijski_klici: Vec<Rc<Vozlišče>> = Vec::new();
        let mut napake = Napake::new();

        for (tip, (spremenljivka, argument)) in iter::zip(tipi, iter::zip(spremenljivke, argumenti)) {
            let podpis_funkcije = Self::podpis_funkcije(ime, &[tip]);
            let funkcija = self.funkcije.get(&podpis_funkcije);

            match funkcija {
                Some(funkcija) => {
                    funkcijski_klici.push(FunkcijskiKlic {
                        funkcija: funkcija.clone(),
                        spremenljivke: Zaporedje(vec![spremenljivka.rc()]).rc(),
                        argumenti: Zaporedje(vec![argument.rc()]).rc(),
                    }.rc());

                    match self.št_klicev.get_mut(&podpis_funkcije) {
                        Some(št_klicev) => *št_klicev += 1,
                        None => { self.št_klicev.insert(podpis_funkcije, 1); },
                    }
                },
                None => {
                    napake.add_napaka(Napaka::from_zaporedje(&[*ime], E2,
                        &format!("Funkcija '{podpis_funkcije}' ne obstaja")));
                }
            }
        }

        if napake.prazno() {
            Ok(Zaporedje(funkcijski_klici).rc())
        }
        else {
            Err(napake)
        }
    }

    fn podpis_funkcije(ime: &Žeton, tipi: &[Tip]) -> String {
        format!("{}({})", ime.as_str(), tipi.iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(", "))
    }
}
