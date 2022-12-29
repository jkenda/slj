use std::collections::HashMap;


pub trait Postprocesiraj {
    fn postprocesiraj(&self) -> String;
}

impl Postprocesiraj for String {
    fn postprocesiraj(&self) -> String {
        let mut postproc = String::new();

        let mut vrstice: Vec<String> = self.split('\n').map(|v| v.to_owned()).collect();
        let mut oznake_vrstic :HashMap<String, u32> = HashMap::new();

        // nadomesti "vrni" z JUMP x
        let mut i = 0usize;
        while i < vrstice.len() {
            if vrstice[i] == "return" {
                let mut konec = i + 1;
                while !vrstice[konec].starts_with(".0konec") {
                    konec += 1;
                }
                vrstice[i] = format!("JUMP {}", vrstice[konec]);
            }
            i += 1;
        }

        // preberi oznake vrstic in jih odstrani
        i = 0;
        while i < vrstice.len() {
            if vrstice[i].starts_with('.') {
                oznake_vrstic.insert(vrstice[i].clone(), i as u32);
                vrstice.remove(i);
            }
            else {
                i += 1;
            }
        }

        for (št_vrstice, vrstica) in vrstice.into_iter().enumerate() {
            if vrstica == "" { continue };
            let razdeljen: Vec<&str> = vrstica.split(' ').collect();
            let ukaz = razdeljen[0];

            postproc += &if ukaz == "JUMP" || ukaz == "JMPC" {
                if razdeljen.len() == 1 {
                    ukaz.to_owned() + "\n"
                }
                else {
                    let absolutni_skok = if razdeljen[1].starts_with('.') {
                        let ime = razdeljen[1];
                        oznake_vrstic[ime]
                    }
                    else {
                        let relativni_skok: i32 = razdeljen[1].parse().unwrap();
                        (št_vrstice as i32 + relativni_skok) as u32
                    };
                    format!("{} #{}\n", ukaz, absolutni_skok)
                }
            }
            else if ukaz == "PC" {
                let odmik: i32 = razdeljen[1].parse().unwrap();
                format!("PUSH #{}\n", št_vrstice as i32 + odmik)
            }
            else {
                vrstica + "\n"
            };
        }

        postproc
    }
}

