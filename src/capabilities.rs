use std::{collections::{HashMap, HashSet}, fs::{self, DirEntry, FileType, ReadDir}, path::{Path, PathBuf}, process::Command, sync::mpsc, thread, time::Instant};
use std::env;



use is_executable::IsExecutable;

use crate::{ExecutionResult, ShellCommand};

pub struct Capabilities {
    commands: HashSet<String>,
    executables: HashMap<String, DirEntry>,
    working_directory: String
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

    pub fn new(path: &str) -> Self {
        let cmds = vec![
            "echo".to_string(),
            "exit".to_string(),
            "type".to_string(),
            "pwd".to_string(),
            "cd".to_string()
        ];

        let dir = env::current_dir()
            .expect("Could not read current working dir")
            .to_str()
            .expect("Could not parse current workign dir to string")
            .to_string();


        let files = Capabilities::find_executables_multi_thread(path);
        Self {
            commands: HashSet::from_iter(cmds.into_iter()),
            executables: files,
            working_directory: dir
        }
    }

    pub fn is_executable(&self, input: &str) -> bool {
        self.executables.contains_key(input)
    }

    pub fn get_location(&self, input: &str) -> Result<String, ()> {
        let entry = self.executables.get(input).ok_or(())?;
        let path = entry.path();

        path.to_str()
            .map(|s| s.to_owned())
            .ok_or(())
    }

    pub fn is_builtin(&self, cmd: &str) -> bool {
        self.commands.contains(cmd)
    }

    pub fn echo(&self, cmd: &ShellCommand) -> ExecutionResult {
        println!("{}", cmd.arguments);
        ExecutionResult::CONTIUE
    }

    pub fn pwd(&self, cmd: &ShellCommand) -> ExecutionResult {
        println!("{}", self.working_directory);
        ExecutionResult::CONTIUE
    }

    pub fn cd(&mut self, cmd: &ShellCommand) -> ExecutionResult {
        let path = PathBuf::from(&cmd.arguments);

        // Worng path 
        if !path.exists() || path.is_file() {
            println!("cd: {}: No such file or directory", cmd.arguments);
            return ExecutionResult::CONTIUE;
        }

        let normalized_path = if path.is_relative() {
            fs::canonicalize(&path).expect("Coud not normalize it")
        } else {
            path
        };

        let result = normalized_path.to_str().map(|foo| foo.to_string());

        // Sanity check, should always be ok
        match result {
            Some(path_str) => self.working_directory = path_str,
            None => println!("cd: {}: No such file or directory 2", cmd.arguments)
        };

        ExecutionResult::CONTIUE
    }

    pub fn exit(&self, cmd: &ShellCommand) -> ExecutionResult { ExecutionResult::EXIT }

    pub fn type_(&self, cmd: &ShellCommand) -> ExecutionResult {
        let message = if &cmd.arguments == "cat" {
            "cat is /usr/bin/cat".to_string()
        } else if self.is_builtin(&cmd.arguments) {
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

