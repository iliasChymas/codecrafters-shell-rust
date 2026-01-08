use std::collections::{HashMap, HashSet};
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

struct Builtins {
    commands: HashSet<String>
}

impl Builtins {
    fn new() -> Self {
        let cmds = vec![
            "echo".to_string(),
            "exit".to_string(),
            "type".to_string()
        ];
        Self {
            commands: HashSet::from_iter(cmds.into_iter())
        }
    }

    fn is_builtin(&self, cmd: &str) -> bool {
        self.commands.contains(cmd)
    }

    fn echo(&self, cmd: &ShellCommand) -> ExecutionResult {
        println!("{}", cmd.arguments);
        ExecutionResult::CONTIUE
    }

    fn exit(&self, cmd: &ShellCommand) -> ExecutionResult { ExecutionResult::EXIT }

    fn type_(&self, cmd: &ShellCommand) -> ExecutionResult {
        let message = if self.is_builtin(&cmd.arguments) {
            format!("{} is a shell builtin", cmd.arguments)
        } else {
            format!("{}: not found", cmd.arguments)
        };

        println!("{}", message);

        ExecutionResult::CONTIUE
    }
}

struct Shell {
    builtins: Builtins,
    executed: Vec<ShellCommand>
}

#[derive(PartialEq)]
enum ExecutionResult {
    EXIT,
    CONTIUE
}

impl Shell {
    fn execute(&mut self, cmd: ShellCommand) -> Result<ExecutionResult, String> {
        if !self.builtins.is_builtin(&cmd.command) {
            return Err(format!("{}: command not found", cmd.command).to_string());
        }

        let res = match cmd.command.as_str() {
            "echo" => Ok(self.builtins.echo(&cmd)),
            "exit" => Ok(self.builtins.exit(&cmd)),
            "type" => Ok(self.builtins.type_(&cmd)),
            _ => Ok(ExecutionResult::CONTIUE)
        };

        self.executed.push(cmd);

        res
    }
}



fn main() {
    // TODO: Uncomment the code below to pass the first stage
    let mut shell = Shell {
        builtins: Builtins::new(),
        executed: Vec::new()
    };
    let stdin = io::stdin();
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
