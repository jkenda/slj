use super::{tokenizer::Token, napaka::{Napake, OznakaNapake}};

pub fn loči_spredaj<'a, 'b>(izraz: &'b[Token<'a>], nizi: &[&'static str]) -> Option<Result<(&'b[Token<'a>], &'b Token<'a>, &'b[Token<'a>]), Napake>>
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
            return Some(Err(Napake::from_zaporedje(&[*tok], OznakaNapake::E1, "Naujemajoč oklepaj")))
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

pub fn loči_zadaj<'a, 'b>(izraz: &'b[Token<'a>], nizi: &[&'static str]) -> Option<Result<(&'b[Token<'a>], &'b Token<'a>, &'b[Token<'a>]), Napake>>
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

pub fn interpoliraj_niz(niz: &str) -> String {
    niz.to_string()
        .replace(r"\\", "\\")
        .replace(r"\n", "\n")
        .replace(r"\t", "\t")
        .replace(r"\r", "\r")
        .replace(r#"\"""#, "\"")
        .replace(r"\'", "\'")
}
