mod shell;
mod capabilities;
mod shell_command;
mod args_parser;

use std::env;
use crate::{shell::{ExecutionResult, Shell}, shell_command::ShellCommand};

fn main() {
    let path = env::var("PATH").unwrap_or("".to_string());
    let mut shell = Shell::new(path);
    shell.run();
}


