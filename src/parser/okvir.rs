use super::*;

impl<'a> Parser<'a> {
    pub fn okvir(&mut self, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        self.v_okvir();
        let zaporedje = self.zaporedje(izraz)?;
        let št_spr = self.iz_okvirja();

        Ok(Okvir { zaporedje, št_spr }.rc())
    }

    // zaporedje izrazov, ločeno z ";" in "\n"
    pub fn zaporedje(&mut self, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        let zaporedje = razdeli(izraz, &[";", "\n"])?;
        let mut izrazi: Vec<Rc<Vozlišče>> = Vec::new();
        let mut napake = Napake::new();

        for stavek in zaporedje {
            match self.stavek(stavek) {
                Ok(stavek) => izrazi.push(stavek),
                Err(n) => napake.razširi(n),
            }
        }

        if napake.prazno() {
            Ok(Zaporedje(izrazi).rc())
        }
        else {
            Err(napake)
        }
    }

    pub fn v_okvir(&mut self) {
        if !self.znotraj_funkcije {
            self.spremenljivke_stack.push(HashMap::new());
            self.konstante_stack.push(HashMap::new());
        }
    }

    pub fn iz_okvirja(&mut self) -> i32 {
        let št_spr = if !self.znotraj_funkcije {
            self.spremenljivke_stack.last().unwrap()
                .values()
                .map(|s| s.sprememba_stacka())
                .sum()
        }
        else {
            0
        };

        if !self.znotraj_funkcije {
            for (ime, _) in self.spremenljivke_stack.pop().unwrap() {
                self.spremenljivke.remove(&ime);
            }
            for (ime, _) in self.konstante_stack.pop().unwrap() {
                self.konstante.remove(&ime);
            }
        }

        št_spr
    }

}
