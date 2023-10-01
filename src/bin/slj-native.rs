

fn main() {
    let command =
        Command::new("fasm")
        .arg("fasm/ukazi.asm")
        .output();

    match command {
        Some(output) =>
            println!("{}", String::from_utf8_lossy(&output.stdout)),
        None => println!("Command failed"),
    }
}
