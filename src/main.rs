use std::collections::HashSet;
#[allow(unused_imports)]
use std::io::{self, Write};

struct ShellCommand {
    arguments: String,
    command: String
}

impl ShellCommand {
    fn parse(mut cmd: String) -> ShellCommand {
        cmd = cmd.replace("\n", "");
        if let Some(split) = cmd.split_once(" ") {
                return Self {
                    command: split.0.to_string(),
                    arguments:  split.1.to_string()
                };
        }

        return Self {
            command: cmd,
            arguments: "".to_string()
        };
    }
}

struct Shell {
    executed: Vec<ShellCommand>
}

#[derive(PartialEq)]
enum ExecutionResult {
    EXIT,
    CONTIUE
}

impl Shell {

    fn execute(&mut self, cmd: ShellCommand) -> Result<ExecutionResult, String> {
        match cmd.command.as_str() {
            "echo" => {
                print!("{}\n", cmd.arguments);
                self.executed.push(cmd);
                return Ok(ExecutionResult::CONTIUE);
            }
            "exit" => { 
                self.executed.push(cmd);
                Ok(ExecutionResult::EXIT)
            }
            _ => Err(format!("{}: command not found", cmd.command).into())
        }
    }
}


fn main() {
    // TODO: Uncomment the code below to pass the first stage
    let stdin = io::stdin();
    let mut shell = Shell { executed: vec![] };
    loop {
        let mut cmd = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut cmd);
        let shell_cmd = ShellCommand::parse(cmd);
        match shell.execute(shell_cmd) {
            Ok(res) => if res == ExecutionResult::EXIT { break; },
            Err(msg) => println!("{}", msg)
        }
    }
}
