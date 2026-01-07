#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    let stdin = io::stdin();

    loop {
        let mut cmd = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut cmd);
        cmd = cmd.replace("\n", "");

        match cmd.as_str() {
            "exit" => break,
            _ => print!("{}: command not found\n", cmd.replace("\n", ""))
        }
    }
}
