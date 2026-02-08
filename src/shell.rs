use std::{io::stdin, process::Command};

use crate::{ShellCommand, capabilities::Capabilities, args_parser::ArgsParser};
use std::io::{self, Write};

pub struct Shell {
    path: String,
    capabilities: Capabilities,
    executed: Vec<ShellCommand>,
}

#[derive(PartialEq)]
pub enum ExecutionResult {
    EXIT,
    CONTIUE(Option<String>),
}

impl Shell {
    pub fn new(path: String) -> Self {
        Self {
            capabilities: Capabilities::new(&path),
            executed: Vec::new(),
            path: path,
        }
    }

    pub fn parse_cmd(&self, input: &str) -> Result<ShellCommand, String> {
        let text = input.trim();
        let Some((cmd, args)) = text.split_once(" ") else {
            if !self.capabilities.is_exec_or_builtin(text.trim()) {
                return Err(format!("{}: command not found", text));
            }
            return Ok(ShellCommand {
                arguments: vec![],
                command: text.to_owned(),
            });
        };

        if !self.capabilities.is_exec_or_builtin(cmd) {
            println!("{}: command not found", text);
            return Err(format!("{}: command not found", text));
        }
        let mut parser: ArgsParser = ArgsParser::new(args);
        let parsed_args = parser.parse();
        return Ok(ShellCommand {
            arguments: parsed_args,
            command: cmd.to_string(),
        });
    }

    pub fn run(&mut self) {
        let stdin = stdin();
        loop {
            let mut cmd = String::new();
            print!("$ ");
            io::stdout().flush().unwrap();
            stdin.read_line(&mut cmd).expect("Failed to read line");
            let shell_cmd = match self.parse_cmd(&cmd) {
                Ok(parsed_cmd) => parsed_cmd,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            };

            match self.execute(shell_cmd) {
                Ok(res) => {
                    if res == ExecutionResult::EXIT {
                        break;
                    }
                    if let ExecutionResult::CONTIUE(Some(output)) = res {
                        println!("{}", output);
                    }
                }
                Err(msg) => println!("{}", msg),
            }
        }
    }

    fn run_executable(&self, cmd: &ShellCommand) -> Result<ExecutionResult, String> {
        let res = if cmd.arguments.len() > 0 {
            Command::new(&cmd.command)
                .env("PATH", &self.path)
                .args(cmd.arguments.iter().map(|arg| arg.trim()))
                .output()
        } else {
            Command::new(&cmd.command).env("PATH", &self.path).output()
        };

        match res {
            Ok(out) => {
                if let Ok(out) = str::from_utf8(&out.stdout) {
                    print!("{}", out)
                } else if let Ok(err) = str::from_utf8(&out.stderr) {
                    print!("STDER -> {}", err)
                }
                Ok(ExecutionResult::CONTIUE(None))
            }
            Err(error) => {
                println!("Error -> {}", error);
                Err(error.to_string())
            }
        }
    }

    pub fn execute_line(&mut self, line: &str) -> Result<ExecutionResult, String> {
        let cmd = self.parse_cmd(line)?;
        self.execute(cmd)
    }

    pub fn execute(&mut self, cmd: ShellCommand) -> Result<ExecutionResult, String> {
        if !self.capabilities.is_builtin(&cmd.command)
            && !self.capabilities.is_executable(&cmd.command)
        {
            return Err(format!("{}: command not found", cmd.command).to_string());
        }

        let res = match cmd.command.as_str() {
            "echo" => Ok(self.capabilities.echo(&cmd)),
            "exit" => Ok(self.capabilities.exit(&cmd)),
            "type" => Ok(self.capabilities.type_(&cmd)),
            "pwd" => Ok(self.capabilities.pwd(&cmd)),
            "cd" => Ok(self.capabilities.cd(&cmd)),
            _ => self.run_executable(&cmd),
        };

        self.executed.push(cmd);

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_echo_preserves_spacing() {
        let path = env::var("PATH").unwrap();
        let mut shell = Shell::new(path);

        let result = shell.execute_line(r#"echo "foo   bar""#);

        assert!(result.is_ok());
        if let Ok(ExecutionResult::CONTIUE(Some(output))) = result {
            assert_eq!(output, "foo   bar");
        } else {
            panic!("Expected CONTIUE with output");
        }
    }

    #[test]
    fn test_echo_escaped_spaces() {
        let path = env::var("PATH").unwrap();
        let mut shell = Shell::new(path);

        let result = shell.execute_line(r"echo foo\ \ \ bar");

        assert!(result.is_ok());
        if let Ok(ExecutionResult::CONTIUE(Some(output))) = result {
            assert_eq!(output, "foo   bar");
        } else {
            panic!("Expected CONTIUE with output");
        }
        
        // Test escaped space followed by regular spaces
        let result2 = shell.execute_line(r"echo foo\     bar");
        
        assert!(result2.is_ok());
        if let Ok(ExecutionResult::CONTIUE(Some(output))) = result2 {
            assert_eq!(output, "foo bar");
        } else {
            panic!("Expected CONTIUE with output");
        }
        
        // Test escaped newline
        let result3 = shell.execute_line(r"echo test\nexample");
        
        assert!(result3.is_ok());
        if let Ok(ExecutionResult::CONTIUE(Some(output))) = result3 {
            assert_eq!(output, "testnexample");
        } else {
            panic!("Expected CONTIUE with output");
        }
        
        // Test escaped backslash
        let result4 = shell.execute_line(r"echo hello\\world");
        
        assert!(result4.is_ok());
        if let Ok(ExecutionResult::CONTIUE(Some(output))) = result4 {
            assert_eq!(output, r"hello\world");
        } else {
            panic!("Expected CONTIUE with output");
        }
        
        // Test escaped quotes
        let result5 = shell.execute_line(r"echo \'hello\'");
        
        assert!(result5.is_ok());
        if let Ok(ExecutionResult::CONTIUE(Some(output))) = result5 {
            assert_eq!(output, "'hello'");
        } else {
            panic!("Expected CONTIUE with output");
        }
    }
}
