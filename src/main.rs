#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    let mut cmd = String::new();
    let stdin = io::stdin();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut cmd);
        print!("{}: command not found\n", cmd.replace("\n", ""));
    }
}
