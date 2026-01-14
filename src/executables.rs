use std::{collections::{HashMap, HashSet}, env, ffi::OsString, fs, path::Path};
use fs::DirEntry;

use is_executable::IsExecutable;

pub struct Executables {
    files: HashMap<String, DirEntry>
}

impl Executables {
    pub fn new(path: &str) -> Self {
        let files = env::split_paths(path)
            .filter(|p| p.is_dir())
            .filter_map(|p| fs::read_dir(p).ok())
            .flat_map(|rd| rd.filter_map(Result::ok))
            .filter(|dirEntry| dirEntry.path().is_executable())
            .filter_map(|dir_entry| {
                let name = dir_entry.file_name().into_string().ok()?;
                Some((name, dir_entry))
            })
            .collect();

        Executables { 
            files
        }
    }

    pub fn is_executable(&self, input: &str) -> bool {
        self.files.contains_key(input)
    }

    pub fn get_location(&self, input: &str) -> Result<String, ()> {
        let entry = self.files.get(input).ok_or(())?;
        let path = entry.path();

        path.to_str()
            .map(|s| s.to_owned())
            .ok_or(())
    }
}


