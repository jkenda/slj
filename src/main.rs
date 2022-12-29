mod drevo;
mod prevajanik;
//mod tokenizer;
mod binary;

use drevo::Vozlišče::*;
use prevajanik::*;
use binary::*;

fn main() {
    let a = Spremenljivka { ime: "a".to_owned(), naslov: 0, z_odmikom: false }.rc();
    let b = Spremenljivka { ime: "b".to_owned(), naslov: 1, z_odmikom: false }.rc();
    let program = Okvir { 
        zaporedje: Zaporedje(vec![
                             Prazno.rc(),
                             Prirejanje { spremenljivka: a.clone(), izraz: Število(3.14).rc(), z_odmikom: false }.rc(),
                             Prirejanje { spremenljivka: b.clone(), izraz: Število(2.72).rc(), z_odmikom: false }.rc(),
                             PogojniStavek { 
                                 pogoj: Večje(a.clone(), b.clone()).rc(), 
                                 resnica: Natisni(vec![a.clone(), Niz("\n".to_owned()).rc()]).rc(), 
                                 laž: Natisni(vec![b.clone(), Niz("\n".to_owned()).rc()]).rc()
                             }.rc()
        ]).rc(), 
        št_spr: 2,
    };

    let preveden = program.prevedi();
    let postprocesiran = preveden.postprocesiraj();

    println!("{}", program.drevo(1));
    println!();
    println!("{}", preveden);
    println!("{}", postprocesiran);

    let binary = postprocesiran.to_program();
    binary.run();
}
