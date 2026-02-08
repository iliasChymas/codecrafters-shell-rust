use std::{io::stdin, process::Command};

use bytes::buf;

use crate::{ShellCommand, capabilities::Capabilities};
use std::io::{self, Write};

pub struct Shell {
    path: String,
    capabilities: Capabilities,
    executed: Vec<ShellCommand>,
}

#[derive(PartialEq)]
pub enum ExecutionResult {
    EXIT,
    CONTIUE,
}

pub struct ArgsParser {
    chars: Vec<char>,
    index: usize,
    reading_string: bool,
    quotes_type: char,
    outside_string_space: bool,
}

impl ArgsParser {
    pub fn new(args: &str) -> Self {
        Self {
            chars: args.chars().collect::<Vec<char>>(),
            index: 0,
            reading_string: false,
            quotes_type: '\'',
            outside_string_space: false,
        }
    }

    pub fn parse(&mut self) -> Vec<String> {
        let mut arguments: Vec<String> = Vec::new();
        let mut buffer = String::new();
        for c in self.chars.iter() {
            match c {
                '\'' | '"' => {
                    if self.reading_string {
                        if &self.quotes_type == c {
                            self.reading_string = false;
                            let arg = if buffer.is_empty() {
                                " ".to_string()
                            } else {
                                buffer.clone()
                            };

                            if arg == " " { continue; }
                            arguments.push(arg.clone());
                            buffer.clear();
                        } else {
                            buffer.push(c.clone());
                        }
                    } else {
                        if buffer.len() > 0 {
                            arguments.push(buffer.clone());
                            buffer.clear();
                        }
                        self.reading_string = true;
                        self.quotes_type = c.clone();
                    }
                }
                ' ' if !self.reading_string => {
                    if buffer.len() != 0 && !buffer.ends_with(' ') {
                        arguments.push(buffer.clone());
                        buffer.clear();
                    }
                    if !buffer.ends_with(' ') {
                        buffer.push(c.clone());
                    }
                }
                '\n' if !self.reading_string => {
                    if buffer.len() != 0 {
                        arguments.push(buffer.clone());
                        buffer.clear();
                    }
                }
                _ => {
                    buffer.push(c.clone());
                    self.outside_string_space = *c == ' ';
                }
            };
        }
        if buffer.len() != 0 {
            arguments.push(buffer.clone());
        }
        arguments
    }
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
                Ok(ExecutionResult::CONTIUE)
            }
            Err(error) => {
                println!("Error -> {}", error);
                Err(error.to_string())
            }
        }
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
