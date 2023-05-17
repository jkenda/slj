use super::*;
use argumenti::*;

impl<'a> Parser<'a> {
    pub fn funkcija(&mut self, ime: &Token, izraz: &[Token]) -> Result<Rc<Vozlišče>, Napake> {
        let (_, _, izraz) = loči_spredaj(izraz, &["("])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '('"))??;
        let (parametri_izraz, _, izraz) = loči_spredaj(izraz, &[")"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan ')'"))??;
        let (tip_izraz, oklepaj, izraz) = loči_spredaj(izraz, &["{"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '{'"))??;

        let tip = match tip_izraz {
            [] => Ok(Tip::Brez),
            [Ločilo("->", ..)] => Err(Napake::from_zaporedje(&[*oklepaj], E5, "Za '->' pričakovan tip")),
            [Ločilo("->", ..), ostanek @ ..] => Tip::from(ostanek),
            _ =>  Err(Napake::from_zaporedje(tip_izraz, E5, "Pričakovan '-> <tip>'")),
        }?;

        let (telo, _, prazno) = loči_zadaj(izraz, &["}"])
            .ok_or(Napake::from_zaporedje(izraz, E5, "Pričakovan '}'"))??;

        if prazno != [] {
            return Err(Napake::from_zaporedje(prazno, E3, "Izraz funkcije se mora zaključiti z '}'"));
        }
        
        let vrni = Spremenljivka { tip: tip.clone(), ime: "0_vrni".to_string(), naslov: 0, z_odmikom: true }.rc();
        let pc   = Spremenljivka { tip: Tip::Celo, ime: "0_PC".to_string(), naslov: vrni.sprememba_stacka() as u32, z_odmikom: true }.rc();

        let mut spr_funkcije = HashMap::from([
            ("0_vrni".to_string(), vrni.clone()),
            ("0_PC".to_string(), pc.clone()),
        ]);

        let mut naslov_nove = (vrni.sprememba_stacka() + pc.sprememba_stacka()) as u32;

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
            let tip = Tip::from(tip)?;

            if spr_funkcije.contains_key(ime.as_str()) {
                return Err(Napake::from_zaporedje(&[*ime], E7, "Imena parametrov morajo biti unikatna"))
            }
            else {
                let spr = Spremenljivka { tip: tip.clone(), ime: ime.to_string(), naslov: naslov_nove, z_odmikom: true }.rc();
                spr_funkcije.insert(ime.to_string(), spr.clone());
                parametri.push(spr);
                naslov_nove += tip.sprememba_stacka() as u32;
            }
        }

        let podpis_funkcije = Self::podpis_funkcije(ime, parametri.iter().map(|p| p.tip()).collect::<Vec<Tip>>().as_slice());
        spr_funkcije.insert("0_OF".to_string(), Spremenljivka { tip: Tip::Celo, ime: "0_OF".to_string(), naslov: naslov_nove, z_odmikom: true }.rc());

        let mut okolje_funkcije = Parser {
            spremenljivke_stack: self.spremenljivke_stack.clone(),
            funkcije_stack: self.funkcije_stack.clone(),
            reference_stack: self.reference_stack.clone(),
            spremenljivke: self.spremenljivke.clone(),
            funkcije: self.funkcije.clone(),
            reference: self.reference.clone(),
            znotraj_funkcije: true,
        };

        okolje_funkcije.spremenljivke_stack.push(spr_funkcije.clone());
        okolje_funkcije.spremenljivke.extend(spr_funkcije);
        okolje_funkcije.funkcije.insert(podpis_funkcije.clone(), Funkcija { 
            tip: tip.clone(),
            ime: podpis_funkcije.clone(),
            parametri: parametri.clone(),
            telo: Prazno.rc(),
            prostor: 0,
            št_klicev: 0,
        }.rc());

        let telo = okolje_funkcije.zaporedje(telo)?;
        let spr_funkcije = okolje_funkcije.spremenljivke_stack.last().unwrap();
        let prostor = spr_funkcije.values().map(|s| s.sprememba_stacka()).sum::<i32>()
            - spr_funkcije["0_vrni"].sprememba_stacka()
            - spr_funkcije["0_PC"].sprememba_stacka()
            - parametri.iter().map(|p| p.sprememba_stacka()).sum::<i32>()
            - spr_funkcije["0_OF"].sprememba_stacka();
        let fun = Funkcija { tip, ime: podpis_funkcije.clone(), parametri, telo, prostor, št_klicev: 0 }.rc();

        self.funkcije_stack.last_mut().unwrap().insert(podpis_funkcije.clone(), fun.clone());
        self.funkcije.insert(podpis_funkcije, fun.clone());
        Ok(fun)
    }

    pub fn funkcijski_klic_zavrzi_izhod(&mut self, ime: &Token, argumenti: &[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let klic = self.funkcijski_klic(ime, argumenti)?;
        let velikost = klic.tip().sprememba_stacka();

        Ok(Zaporedje(vec![
            klic,
            Pop(velikost).rc(),
        ]).rc())
    }

    pub fn funkcijski_klic<'b>(&mut self, ime: &Token, argumenti: &'b[Token<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let Argumenti { tipi, spremenljivke, argumenti } = self.argumenti(argumenti)?;
        let podpis_funkcije = Self::podpis_funkcije(ime, tipi.as_slice());

        let funkcija = self.funkcije.get(&podpis_funkcije)
            .ok_or(Napake::from_zaporedje(&[*ime], E2, &format!("Funkcija '{podpis_funkcije}' ne obstaja")))?
            .clone();

        if let Funkcija { tip, ime, parametri, telo, prostor, št_klicev } = &*funkcija {
            self.funkcije.insert(podpis_funkcije, Funkcija {
                tip: tip.clone(),
                ime: ime.clone(),
                parametri: parametri.clone(),
                telo: telo.clone(),
                prostor: *prostor,
                št_klicev: št_klicev + 1 }.rc());
        }

        Ok(FunkcijskiKlic { funkcija, spremenljivke: Zaporedje(spremenljivke).rc(), argumenti: Zaporedje(argumenti).rc() }.rc())
    }

    pub fn multi_klic<'b>(&mut self, ime: &'b Token<'a>, argumenti_izraz: &'b [Token<'a>]) -> Result<Rc<Vozlišče>, Napake> where 'a: 'b {
        let Argumenti { tipi, spremenljivke, argumenti } = self.argumenti(argumenti_izraz)?;
        let mut funkcijski_klici: Vec<Rc<Vozlišče>> = Vec::new();
        let mut napake = Napake::new();

        for (tip, (spremenljivka, argument)) in iter::zip(tipi, iter::zip(spremenljivke, argumenti)) {
            let podpis_funkcije = Self::podpis_funkcije(ime, &[tip]);
            let funkcija = self.funkcije.get(&podpis_funkcije)
                .ok_or(Napake::from_zaporedje(argumenti_izraz, E2, &format!("Funkcija '{podpis_funkcije}' ne obstaja")))?
                .clone();

            if let Funkcija { tip, .. } = &*funkcija {
                if *tip == Tip::Brez {
                    funkcijski_klici.push(FunkcijskiKlic {
                        funkcija,
                        spremenljivke: Zaporedje(vec![spremenljivka.rc()]).rc(),
                        argumenti: Zaporedje(vec![argument.rc()]).rc(),
                    }.rc());
                }
                else {
                    napake.add_napaka(Napaka::from_zaporedje(&[*ime], E8,
                                      &format!("{podpis_funkcije} -> {tip}: Funkcije, ki jih vključuje multifunkcijski klic, ne smejo ničesar vračati")));
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

    fn podpis_funkcije(ime: &Token, tipi: &[Tip]) -> String {
        format!("{}({})", ime.as_str(), tipi.iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(", "))
    }
}