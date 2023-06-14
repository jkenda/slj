use super::*;

impl<'a> Parser<'a> {
    pub fn predprocesiraj(izraz: Vec<Žeton<'a>>) -> Vec<Žeton<'a>> {
        let mut predproc = Vec::new();
        let mut znotraj_komentarja = false;

        // predproc = izraz brez komentarjev
        for i in 0..izraz.len() {
            match izraz[i] {
                Ločilo("#", ..)  => znotraj_komentarja = true,
                Ločilo("\n", ..) => { if znotraj_komentarja { znotraj_komentarja = false }; predproc.push(izraz[i]) },
                _ => if !znotraj_komentarja { predproc.push(izraz[i]) },
            }
        }

        let mut i = 0;

        // odstrani razna zaporedja oklepajev, ločil in "\n",
        // da se pravilno prevede
        while i < predproc.len() - 1 {
            i += match predproc[i..] {
                [ Ločilo("\n", ..), Ločilo("{", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("\n", ..), Ločilo("}", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("{", ..),  Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 0 },

                [ Ločilo("\n", ..), Ločilo("(", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("\n", ..), Ločilo(")", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("(", ..),  Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 0 },

                [ Ločilo("\n", ..), Ločilo("=", ..), ..  ] => { predproc.remove(i+0); 0 },
                [ Ločilo("=", ..),  Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 0 },

                [ Ločilo("\n", ..), Rezerviranka("čene", ..) , .. ] => { predproc.remove(i+0); 0 },
                [ Rezerviranka(..), Ločilo("\n", ..), .. ] => { predproc.remove(i+1); 0 },

                [ Ločilo("\n", ..), Ločilo("\n", ..), .. ] => { predproc.remove(i+0); 0 }
                _ => 1,
            };
        }
        predproc
    }
}
