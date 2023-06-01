use super::{lekser::Žeton, napaka::{Napake, OznakaNapake}};

pub fn loči_spredaj<'a, 'b>(izraz: &'b[Žeton<'a>], nizi: &'b[&'static str]) ->
    Option<Result<(&'b[Žeton<'a>], &'b Žeton<'a>, &'b[Žeton<'a>]), Napake>>
    where 'a: 'b
{
    let mut navadnih: isize = 0;
    let mut zavitih:  isize = 0;
    let mut oglatih:  isize = 0;

    for (i, tok) in izraz.iter().enumerate() {
        match tok.as_str() {
            ")" => navadnih -= 1,
            "}" => zavitih  -= 1,
            "]" => oglatih  -= 1,
            _   => ()
        }

        if navadnih <= 0 && zavitih <= 0 && oglatih <= 0
            && nizi.iter().any(|s| *s == tok.as_str()) {
                ////println!("{:?} najden", tok);
                return Some(Ok((&izraz[..i], tok, &izraz[i+1..])));
            }

        if navadnih < 0 || zavitih < 0 || oglatih < 0 {
            return Some(Err(Napake::from_zaporedje(&[*tok], OznakaNapake::E1, "Neujemajoč oklepaj")))
        }

        match tok.as_str() {
            "(" => navadnih += 1,
            "{" => zavitih  += 1,
            "[" => oglatih  += 1,
            _   => ()
        }
    }

    if navadnih != 0 || zavitih != 0 || oglatih != 0 {
        return Some(Err(Napake::from_zaporedje(izraz, OznakaNapake::E1, "Oklepaji se ne ujemajo")))
    }

    None
}

pub fn loči_zadaj<'a, 'b>(izraz: &'b[Žeton<'a>], nizi: &[&'static str]) -> Option<Result<(&'b[Žeton<'a>], &'b Žeton<'a>, &'b[Žeton<'a>]), Napake>>
    where 'a: 'b
{
    let mut navadnih: isize = 0;
    let mut zavitih:  isize = 0;
    let mut oglatih:  isize = 0;

    for (i, tok) in izraz.iter().rev().enumerate() {
        // obrni i, drugače ima zadnji element seznama i = 0, predzadnji 1 ...
        let i = izraz.len() - 1 - i;

        match tok.as_str() {
            "(" => navadnih -= 1,
            "{" => zavitih -= 1,
            "[" => oglatih -= 1,
            _ => ()
        }

        if navadnih == 0 && zavitih == 0 && oglatih == 0
            && nizi.iter().any(|s| *s == tok.as_str()) {
                return Some(Ok((&izraz[..i], tok, &izraz[i+1..])));
            }

        if navadnih < 0 || zavitih < 0 || oglatih < 0 {
            return Some(Err(Napake::from_zaporedje(&[*tok], OznakaNapake::E1, "Neujemajoč oklepaj")))
        }

        match tok.as_str() {
            ")" => navadnih += 1,
            "}" => zavitih += 1,
            "]" => oglatih += 1,
            _ => ()
        }
    }

    if navadnih != 0 || zavitih != 0 || oglatih != 0 {
        return Some(Err(Napake::from_zaporedje(izraz, OznakaNapake::E1, "Oklepaji se ne ujemajo")))
    }

    None
}

pub fn razdeli<'a, 'b>(izraz: &'b[Žeton<'a>], nizi: &'b[&'static str]) -> Result<Vec<&'b[Žeton<'a>]>, Napake> where 'a: 'b {
    match loči_spredaj(izraz, nizi) {
        Some(Ok((prvi_stavek, _, ostanek))) => {
            let mut razdeljeno = vec![prvi_stavek];
            razdeljeno.extend(razdeli(ostanek, nizi)?);
            Ok(razdeljeno)
        },
        Some(Err(err)) => Err(err),
        None => Ok(if izraz != [] { vec![izraz] } else { vec![] }),
    }
}

pub fn interpoliraj_niz(niz: &str) -> String {
    niz.to_string()
        .replace(r"\\", "\\")
        .replace(r"\n", "\n")
        .replace(r"\t", "\t")
        .replace(r"\r", "\r")
        .replace(r#"\"""#, "\"")
        .replace(r"\'", "\'")
}

#[cfg(test)]
mod testi {
    use super::*;
    use crate::parser::{Žeton::*, tokenizer::Tokenize};

    #[test]
    fn poišči() {
        assert_eq!(loči_spredaj("{}".tokenize().as_slice(), &["{"]), Some(Ok(([].as_slice(), &Ločilo("{", 1, 1), [Ločilo("}", 1, 2)].as_slice()))));
    }

}
