use crate::modules::*;
use crate::parser::*;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{BufReader, Error, ErrorKind};
use std::path::PathBuf;

pub struct Yang {
    paths: Vec<PathBuf>,
    pub modules: HashMap<String, Module>,
}

impl Yang {
    pub fn new() -> Self {
        Yang {
            paths: vec![],
            modules: HashMap::new(),
        }
    }

    // Add colon ':' separated path to YANG file load paths.
    pub fn add_path(&mut self, paths: &str) {
        for path in paths.split(":") {
            self.paths.push(PathBuf::from(path));
        }
    }

    pub fn paths(&self) -> &Vec<PathBuf> {
        &self.paths
    }

    pub fn scan_dir(&self, dir: &str, name: &str, recursive: bool) -> Result<PathBuf, Error> {
        let mut candidate = vec![];

        let mut file_name = String::from(name);
        if !file_name.ends_with(".yang") {
            file_name.push_str(".yang");
        }

        let mut basename = String::from(name.trim_end_matches(".yang"));
        basename.push_str("@");

        let dirent = fs::read_dir(dir)?;
        for entry in dirent {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        // File.
                        if let Some(os_str) = entry.path().file_name() {
                            if let Some(file_str) = os_str.to_str() {
                                if file_str == file_name {
                                    return Ok(entry.path());
                                }
                                if let None = name.find('@') {
                                    // Try revision match such as 'ietf-dhcp@2016-08-25.yang'.
                                    if file_str.starts_with(&basename)
                                        && file_str.ends_with(".yang")
                                    {
                                        candidate.push(entry.path());
                                    }
                                }
                            }
                        }
                    } else if file_type.is_dir() && recursive {
                        // Directory.
                        if let Some(dir_str) = entry.path().to_str() {
                            if let Ok(pathbuf) = self.scan_dir(dir_str, name, recursive) {
                                return Ok(pathbuf);
                            }
                        }
                    }
                }
            }
        }
        if candidate.len() == 0 {
            return Err(Error::new(
                ErrorKind::Other,
                "can't find candidate YANG file",
            ));
        }

        // When the specified file is not found by exact match, directories are
        // scanned for "name@revision-date.yang" files, the latest (sorted by
        // YYYY-MM-DD revision-date) of candidates will be selected.
        candidate.sort();

        Ok(candidate.pop().unwrap())
    }

    pub fn find_file(&mut self, file_name: &str) -> Result<File, Error> {
        let mut file_path = PathBuf::from(file_name);

        // When file does not have path, scan current dir.
        if let None = file_name.find('/') {
            if let Ok(fp) = self.scan_dir(".", file_name, false) {
                file_path = fp;
            }
        }

        match File::open(&file_path) {
            Ok(file) => {
                // When file can be opened and has a path, add the path to paths.
                if file_path.pop() {
                    self.paths.push(file_path);
                }
                return Ok(file);
            }
            Err(_) => {
                if let Some(_) = file_name.find('/') {
                    return Err(Error::new(ErrorKind::Other, "can't find file"));
                }
            }
        }

        for path in self.paths() {
            if path.file_name() == Some(OsStr::new("...")) {
                let mut dir = path.clone();
                if dir.pop() {
                    if let Some(dir_str) = dir.to_str() {
                        if let Ok(fp) = self.scan_dir(dir_str, file_name, true) {
                            file_path = fp;
                        }
                    }
                }
            } else {
                let mut dir = path.clone();
                dir.push(file_name);
                file_path = dir;
            }
        }
        return File::open(&file_path);
    }

    pub fn read_file(&self, file: File) -> Result<String, Error> {
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn read(&mut self, _ms: &Modules, name: &str) -> Result<String, Error> {
        // Find and open file.
        let file = self.find_file(name)?;

        // Read file contents.
        let data = self.read_file(file)?;

        // Parse file.
        // let ast = parse_data(data)?;

        Ok(data)
    }

    pub fn read_and_parse(&mut self, name: &str) -> Result<(), Error> {
        // Find and open file.
        let file = self.find_file(name)?;

        // Read file contents.
        let data = self.read_file(file)?;

        // Parse file.
        let ast = yang_parse(&data);
        match ast {
            Ok((_, module)) => {
                self.modules.insert(module.prefix.to_owned(), module);
                Ok(())
            }
            Err(e) => {
                println!("module parse: {:?}", e);
                Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_path() {
        let mut yang = Yang::new();
        yang.add_path("/etc/openconfigd/yang:/opt/zebra/yang");
        yang.add_path("/var/yang");

        let paths = vec![
            PathBuf::from("/etc/openconfigd/yang"),
            PathBuf::from("/opt/zebra/yang"),
            PathBuf::from("/var/yang"),
        ];

        assert_eq!(yang.paths(), &paths);
    }

    // #[test]
    // fn module_read() {
    //     let mut yang = Yang::new();
    //     yang.add_path("/etc/openconfigd/yang:/opt/zebra/yang:./tests/...");

    //     let ms = Modules::new();
    //     yang.read(&ms, "coreswitch").unwrap();
    // }
}
