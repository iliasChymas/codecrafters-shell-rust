mod executables;
use std::{process::Command, sync::mpsc, thread, time::Instant};
use is_executable::IsExecutable;
use executables::*;
use std::{collections::{HashMap, HashSet}, env, fs::{self, DirEntry, FileType, ReadDir}, io::stdin};
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

struct Capabilities {
    commands: HashSet<String>,
    executables: HashMap<String, DirEntry>
}

impl Capabilities {
    fn find_executables_multi_thread(path: &str) -> HashMap<String, DirEntry> {
        let folders: Vec<String> = env::split_paths(&path)
            .filter(|p| p.is_dir())
            .filter_map(|p| p.to_str().map(|s| s.to_string()))
            .collect();

        let (sender, reciever): (mpsc::Sender<(String, DirEntry)>,
            mpsc::Receiver<(String, DirEntry)>) = mpsc::channel();

        let mut files: HashMap<String, DirEntry> = HashMap::new();

        for chunk in folders.chunks(3) {
            let local_sender = sender.clone();
            let owned_chunk = chunk.to_vec();
            thread::spawn(move || {
                owned_chunk
                    .iter()
                    .filter_map(|p| fs::read_dir(p).ok())
                    .flat_map(|rd| rd.filter_map(Result::ok))
                    .filter(|dirEntry| dirEntry.path().is_executable())
                    .filter_map(|dir_entry| {
                        let name = dir_entry.file_name().into_string().ok()?;
                        Some((name, dir_entry))
                    })
                    .for_each(|entry| local_sender.send(entry).unwrap());
            });
        }

        drop(sender);

        for rec in reciever {
            files.insert(rec.0, rec.1);
        }

        files
    }

    fn new(path: &str) -> Self {
        let cmds = vec![
            "echo".to_string(),
            "exit".to_string(),
            "type".to_string()
        ];

        let files = Capabilities::find_executables_multi_thread(path);
        Self {
            commands: HashSet::from_iter(cmds.into_iter()),
            executables: files
        }
    }

    fn is_executable(&self, input: &str) -> bool {
        self.executables.contains_key(input)
    }

    fn get_location(&self, input: &str) -> Result<String, ()> {
        let entry = self.executables.get(input).ok_or(())?;
        let path = entry.path();

        path.to_str()
            .map(|s| s.to_owned())
            .ok_or(())
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
        } else if let Ok(location) = self.get_location(&cmd.arguments) {
            format!("{} is {}", cmd.arguments, location)
        } else {
            format!("{}: not found", cmd.arguments)
        };

        println!("{}", message);

        ExecutionResult::CONTIUE
    }
}

struct Shell {
    capabilities: Capabilities,
    executed: Vec<ShellCommand>,
    executables: Executables
}

#[derive(PartialEq)]
enum ExecutionResult {
    EXIT,
    CONTIUE
}

impl Shell {

    fn run_executable(&self, cmd: &ShellCommand) -> Result<ExecutionResult, String> {
        let args = cmd.arguments.split(" ")
            .filter(|item| !item.is_empty())
            .collect::<Vec<&str>>();

        let res = if args.len() > 0 {
            Command::new(&cmd.command)
                .args(args)
                .output()
        } else {
            Command::new(&cmd.arguments)
                .output()
        };

        if let Ok(res) = res {
            if let Ok(out) = str::from_utf8(&res.stdout) {
                println!("{}", out)
            } else if let Ok(error) = str::from_utf8(&res.stderr) {
                println!("{}", error)
            }

            return Ok(ExecutionResult::CONTIUE);
        } else {
            return Err("Could not parse command result into string".to_string());
        }
        return Err(format!("{}: command not found", cmd.command).to_string());
    }

    fn execute(&mut self, cmd: ShellCommand) -> Result<ExecutionResult, String> {
        if !self.capabilities.is_builtin(&cmd.command) && !self.capabilities.is_executable(&cmd.command) {
            return Err(format!("{}: command not found", cmd.command).to_string());
        }

        let res = match cmd.command.as_str() {
            "echo" => Ok(self.capabilities.echo(&cmd)),
            "exit" => Ok(self.capabilities.exit(&cmd)),
            "type" => Ok(self.capabilities.type_(&cmd)),
            _ => self.run_executable(&cmd)
        };

        self.executed.push(cmd);

        res
    }
}



fn main() {
    // TODO: Uncomment the code below to pass the first stage
    let path = env::var("PATH").unwrap_or("".to_string());
    let mut shell = Shell {
        capabilities: Capabilities::new(&path),
        executed: Vec::new(),
        executables: Executables::new(&path)
    };
    let stdin = stdin();
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

