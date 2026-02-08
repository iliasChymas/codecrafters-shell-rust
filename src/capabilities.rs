use std::env;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, DirEntry},
    path::PathBuf,
    sync::mpsc,
    thread,
};

use is_executable::IsExecutable;

use crate::{ExecutionResult, ShellCommand};

pub struct Capabilities {
    commands: HashSet<String>,
    executables: HashMap<String, DirEntry>,
}

impl Capabilities {
    fn find_executables_multi_thread(path: &str) -> HashMap<String, DirEntry> {
        let folders: Vec<String> = env::split_paths(&path)
            .filter(|p| p.is_dir())
            .filter_map(|p| p.to_str().map(|s| s.to_string()))
            .collect();

        let (sender, reciever): (
            mpsc::Sender<(String, DirEntry)>,
            mpsc::Receiver<(String, DirEntry)>,
        ) = mpsc::channel();

        let mut files: HashMap<String, DirEntry> = HashMap::new();

        for chunk in folders.chunks(3) {
            let local_sender = sender.clone();
            let owned_chunk = chunk.to_vec();
            thread::spawn(move || {
                owned_chunk
                    .iter()
                    .filter_map(|p| fs::read_dir(p).ok())
                    .flat_map(|rd| rd.filter_map(Result::ok))
                    .filter(|dir_entry| dir_entry.path().is_executable())
                    .filter_map(|dir_entry| {
                        let name = dir_entry.file_name().into_string().ok()?;
                        Some((name, dir_entry))
                    })
                    .for_each(|entry| local_sender.send(entry).unwrap());
            });
        }

        drop(sender);

        for (name, pth) in reciever {
            if let Some(loc) = files.get(&name) && loc.path().to_str().unwrap_or_else(|| "").starts_with("/usr/bin") {
                continue;
            }
            files.insert(name, pth);
        }

        files
    }

    pub fn new(path: &str) -> Self {
        let cmds = vec![
            "echo".to_string(),
            "exit".to_string(),
            "type".to_string(),
            "pwd".to_string(),
            "cd".to_string(),
        ];

        let files = Capabilities::find_executables_multi_thread(path);
        Self {
            commands: HashSet::from_iter(cmds.into_iter()),
            executables: files,
        }
    }

    pub fn is_executable(&self, input: &str) -> bool {
        self.executables.contains_key(input)
    }

    pub fn is_exec_or_builtin(&self, input: &str) -> bool {
        self.is_executable(input) || self.is_builtin(input)
    }

    pub fn get_location(&self, input: &str) -> Result<String, ()> {
        let entry = self.executables.get(input).ok_or(())?;
        let path = entry.path();

        path.to_str().map(|s| s.to_owned()).ok_or(())
    }

    pub fn is_builtin(&self, cmd: &str) -> bool {
        self.commands.contains(cmd)
    }

    pub fn echo(&self, cmd: &ShellCommand) -> ExecutionResult {
        let output = if cmd.arguments.len() == 0 {
            String::new()
        } else {
            cmd.arguments.join("")
        };

        ExecutionResult::CONTIUE(Some(output))
    }

    pub fn pwd(&self, _: &ShellCommand) -> ExecutionResult {
        match std::env::current_dir() {
            Ok(wd) => println!("{}", wd.display()),
            Err(e) => println!("[Error] pwd: {}", e),
        }
        ExecutionResult::CONTIUE(None)
    }

    pub fn cd(&mut self, cmd: &ShellCommand) -> ExecutionResult {
        let mut path_str = if cmd.arguments.len() == 0 {
            "~".to_string()
        } else {
            cmd.arguments[0].clone()
        };

        if path_str.starts_with("~") {
            path_str = path_str.replace(
                "~",
                &std::env::var("HOME").expect("[Error] could not read HOME"),
            );
        }

        let mut path = PathBuf::from(&path_str);

        if path.is_relative()
            && let Ok(wd) = std::env::current_dir()
        {
            path = wd.join(&path);
        }

        let normalized_path_opt = std::fs::canonicalize(path);
        if normalized_path_opt.is_err() {
            println!("cd: {}: No such file or directory", path_str);
            return ExecutionResult::CONTIUE(None);
        }

        let normalized_path = normalized_path_opt.unwrap();

        if !normalized_path.exists() || !normalized_path.is_dir() {
            println!("cd: {:?}: No such file or directory", cmd.arguments);
            return ExecutionResult::CONTIUE(None);
        }

        let result = normalized_path.to_str().map(|foo| foo.to_string());

        // Sanity check, should always be ok
        match result {
            Some(p) => std::env::set_current_dir(&p)
                .expect(format!("[Error] Could not cd into -> {}", p.to_string()).as_str()),
            None => println!("cd: {:?}: No such file or directory", cmd.arguments),
        };

        ExecutionResult::CONTIUE(None)
    }

    pub fn exit(&self, _: &ShellCommand) -> ExecutionResult {
        ExecutionResult::EXIT
    }

    pub fn type_(&self, cmd: &ShellCommand) -> ExecutionResult {
        if cmd.arguments.len() == 0 {
            println!(": not found");
            return ExecutionResult::CONTIUE(None);
        }

        let message = if self.is_builtin(&cmd.arguments[0]) {
            format!("{} is a shell builtin", cmd.arguments[0])
        } else if let Ok(location) = self.get_location(&cmd.arguments[0]) {
            format!("{} is {}", cmd.arguments[0], location)
        } else {
            format!("{}: not found", cmd.arguments[0])
        };

        println!("{}", message);

        ExecutionResult::CONTIUE(None)
    }
}
