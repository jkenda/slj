use std::sync::atomic::{AtomicI32, Ordering};

use super::*;

static PROSTOR: AtomicI32 = AtomicI32::new(0);

impl<'a> Parser<'a> {
    pub fn prostor() -> i32 {
        PROSTOR.load(Ordering::Relaxed)
    }

    pub fn okvir(&mut self, izraz: &[Žeton<'a>]) -> Result<Rc<Vozlišče>, Napake> {
        self.v_okvir();
        let zaporedje = self.zaporedje(izraz)?;
        self.iz_okvirja();

        Ok(zaporedje)
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

    pub fn iz_okvirja(&mut self) {
        if !self.znotraj_funkcije {
            let št_spr = self.spremenljivke
                .values()
                .map(|s| s.sprememba_stacka())
                .sum();

            PROSTOR.fetch_max(št_spr, Ordering::Relaxed);
        }

        if !self.znotraj_funkcije {
            for (ime, _) in self.spremenljivke_stack.pop().unwrap() {
                self.spremenljivke.remove(&ime);
            }
            for (ime, _) in self.konstante_stack.pop().unwrap() {
                self.konstante.remove(&ime);
            }
        }
    }

}
