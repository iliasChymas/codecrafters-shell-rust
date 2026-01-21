use std::{io::stdin, process::Command};

use std::io::{self, Write};
use crate::shell_command::parse;
use crate::{ShellCommand, capabilities::Capabilities};

pub struct Shell {
    path: String,
    capabilities: Capabilities,
    executed: Vec<ShellCommand>,
}

#[derive(PartialEq)]
pub enum ExecutionResult {
    EXIT,
    CONTIUE
}

impl Shell {
   pub fn new(path: String) -> Self {

        Self {
            capabilities: Capabilities::new(&path),
            executed: Vec::new(),
            path: path,
        }
    }

    pub fn run(&mut self) {
        let stdin = stdin();
        loop {
            let mut cmd = String::new();
            print!("$ ");
            io::stdout().flush().unwrap();
            stdin.read_line(&mut cmd).expect("Failed to read line");
            let shell_cmd = parse(cmd);
            match self.execute(shell_cmd) {
                Ok(res) => if res == ExecutionResult::EXIT { break; },
                Err(msg) => println!("{}", msg)
            }
        }
    }

    fn run_executable(&self, cmd: &ShellCommand) -> Result<ExecutionResult, String> {
        let args = cmd.arguments.split(" ")
            .filter(|item| !item.is_empty())
            .collect::<Vec<&str>>();

        let res = if args.len() > 0 {
            Command::new(&cmd.command)
                .env("PATH", &self.path)
                .args(args)
                .output()
        } else {
            Command::new(&cmd.command)
                .env("PATH", &self.path)
                .output()
        };

        match res {
            Ok(out) => {
                if let Ok(out) = str::from_utf8(&out.stdout) {
                    print!("{}", out)
                }
                Ok(ExecutionResult::CONTIUE)
            },
            Err(error) => { 
                println!("Error -> {}", error);
                Err(error.to_string())
            }
        }
    }

    pub fn execute(&mut self, cmd: ShellCommand) -> Result<ExecutionResult, String> {
        if !self.capabilities.is_builtin(&cmd.command) && !self.capabilities.is_executable(&cmd.command) {
            return Err(format!("{}: command not found", cmd.command).to_string());
        }

        let res = match cmd.command.as_str() {
            "echo" => Ok(self.capabilities.echo(&cmd)),
            "exit" => Ok(self.capabilities.exit(&cmd)),
            "type" => Ok(self.capabilities.type_(&cmd)),
            "pwd" => Ok(self.capabilities.pwd(&cmd)),
            "cd" => Ok(self.capabilities.cd(&cmd)),
            _ => self.run_executable(&cmd)
        };

        self.executed.push(cmd);

        res
    }
}

