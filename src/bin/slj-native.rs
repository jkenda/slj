use std::process::Command;

fn main() {
    let command = Command::new("fasm")
        .arg("fasm/ukazi.asm")
        .output();

    match command {
        Ok(output) =>
            println!("{}", String::from_utf8_lossy(&output.stdout)),
        Err(error) => println!("Command failed: {error:?}"),
    }

    let command = Command::new("fasm/ukazi")
        .output();

    match command {
        Ok(output) =>
            print!("{}", String::from_utf8_lossy(&output.stdout)),
        Err(error) => println!("Command failed: {error:?}"),
    }

}
