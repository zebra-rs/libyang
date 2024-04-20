use crate::yang_grammar::YangGrammar;
use crate::yang_parser::parse;
use crate::*;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct YangStore {
    paths: Vec<PathBuf>,
    pub modules: HashMap<String, ModuleNode>,
    pub submodules: HashMap<String, SubmoduleNode>,
    pub import: HashMap<String, ModuleNode>,
}

impl YangStore {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_path(&mut self, paths: &str) {
        for path in paths.split(':') {
            //if let Ok(path) = PathBuf::from(path).expand_home() {
            let path = PathBuf::from(path);
            self.paths.push(path);
            //}
        }
    }

    pub fn read(&mut self, module_name: &str) -> Result<(), YangError> {
        let node = self.load_module(module_name)?;
        if let Node::Module(mut m) = node {
            for import in m.import.iter() {
                self.read(&import.name)?;
            }
            for include in m.include.iter() {
                let sub = self.load_module(&include.name)?;
                if let Node::Submodule(mut sub) = sub {
                    m.grouping.append(&mut sub.grouping);
                }
            }
            self.modules.insert(module_name.to_string(), *m);
        }
        Ok(())
    }

    pub fn identity_resolve(&mut self) {
        for (_, m) in self.modules.iter_mut() {
            m.identity_resolve();
        }
        for (_, m) in self.submodules.iter_mut() {
            m.identity_resolve();
        }
    }

    pub fn read_with_resolve(&mut self, name: &str) -> Result<(), YangError> {
        println!("name {}", name);
        let node = self.load_module(name)?;
        let mut imports = Vec::<String>::new();
        let mut includes = Vec::<String>::new();
        match node {
            Node::Module(m) => {
                for import in m.import.iter() {
                    imports.push(import.name.clone());
                }
                for include in m.include.iter() {
                    includes.push(include.name.clone());
                }
                self.modules.insert(name.to_string(), *m);
            }
            Node::Submodule(m) => {
                for import in m.import.iter() {
                    imports.push(import.name.clone());
                }
                for include in m.include.iter() {
                    includes.push(include.name.clone());
                }
                self.submodules.insert(name.to_string(), *m);
            }
        }
        for import in imports.iter() {
            if !self.modules.contains_key(import) {
                self.read_with_resolve(import)?;
            }
        }
        for include in includes.iter() {
            if !self.submodules.contains_key(include) {
                self.read_with_resolve(include)?;
            }
        }
        Ok(())
    }

    pub fn load_module(&mut self, module_name: &str) -> Result<Node, YangError> {
        let path = self.find_file(module_name)?;
        let input = fs::read_to_string(&path)?;
        let mut yang_grammar = YangGrammar::new();
        match parse(&input, &path, &mut yang_grammar) {
            Ok(_) => yang(yang_grammar),
            Err(err) => {
                println!("{:?}", err);
                Err(YangError::ParseError)
            }
        }
    }

    fn find_file(&self, module_name: &str) -> Result<PathBuf, YangError> {
        for path in &self.paths {
            if path.file_name() == Some(OsStr::new("...")) {
                let mut dir = path.clone();
                if dir.pop() {
                    if let Ok(file_name) = find_in_dir(&dir, module_name, true) {
                        return Ok(file_name);
                    }
                }
            }
            if let Ok(file_name) = find_in_dir(path, module_name, false) {
                return Ok(file_name);
            }
        }
        find_in_dir(&PathBuf::from("."), module_name, false)
    }

    pub fn find_module(&self, name: &str) -> Option<&ModuleNode> {
        self.modules.get(name)
    }

    pub fn find_submodule(&self, name: &str) -> Option<&SubmoduleNode> {
        self.submodules.get(name)
    }
}

fn find_in_dir(dir: &PathBuf, module_name: &str, recursive: bool) -> Result<PathBuf, YangError> {
    let mut file_name = String::from(module_name);
    if !file_name.ends_with(".yang") {
        file_name.push_str(".yang");
    }

    let mut basename = String::from(module_name.trim_end_matches(".yang"));
    basename.push('@');

    let mut revisions = vec![];

    let dirent = fs::read_dir(dir)?;
    for entry in dirent.into_iter().flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Some(os_str) = entry.path().file_name() {
                    if let Some(file_str) = os_str.to_str() {
                        if file_str == file_name {
                            return Ok(entry.path());
                        }
                        // When module_name does not contain '@'.
                        if module_name.find('@').is_none() {
                            // Try revision match such as 'ietf-dhcp@2016-08-25.yang'.
                            if file_str.starts_with(&basename) && file_str.ends_with(".yang") {
                                revisions.push(entry.path());
                            }
                        }
                    }
                }
            } else if file_type.is_dir() && recursive {
                if let Ok(path) = find_in_dir(&entry.path(), module_name, recursive) {
                    return Ok(path);
                }
            }
        }
    }
    if revisions.is_empty() {
        return Err(YangError::FileNotFound);
    }

    // When the specified file is not found by exact match, directories are
    // scanned for "name@revision-date.yang" files, the latest (sorted by
    // YYYY-MM-DD revision-date) of candidates will be selected.
    revisions.sort();

    Ok(revisions.pop().unwrap())
}
