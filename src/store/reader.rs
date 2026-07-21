use crate::yang_grammar::YangGrammar;
use crate::yang_parser::parse;
use crate::*;
use std::cell::{Ref, RefCell};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct YangStore {
    paths: Vec<PathBuf>,
    // Crate-private: reach these through `find_module` /
    // `find_submodule`. Keeping the storage an implementation detail
    // means its type can change without breaking callers — which is
    // exactly what the switch below could not do while it was public.
    //
    // These are `BTreeMap`, not `HashMap`, so iterating them is
    // deterministic. `to_entry` walks `modules` to apply each loaded
    // module's augments, and augmented nodes are appended to their
    // target's `dir` in application order — so with a randomly ordered
    // map the shape of the resulting tree changed on every run of the
    // same program over the same files. Key order (module name) is
    // arbitrary but stable, which is what consumers need to produce
    // reproducible output.
    pub(crate) modules: BTreeMap<String, ModuleNode>,
    pub(crate) submodules: BTreeMap<String, SubmoduleNode>,
    // Problems found while building entry trees. `to_entry` takes the
    // store by shared reference and the whole augment path already has
    // it in scope, so a `RefCell` collects diagnostics here without
    // threading a channel through 20 functions.
    diagnostics: RefCell<Vec<Diagnostic>>,
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

    pub fn identity_resolve(&mut self) {
        for m in self.modules.values_mut() {
            identity_resolve(m);
        }
        for m in self.submodules.values_mut() {
            identity_resolve(m);
        }
    }

    pub fn read_with_resolve(&mut self, name: &str) -> Result<(), YangError> {
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
        let input = fs::read_to_string(&path).map_err(|source| YangError::IoError {
            path: path.clone(),
            source,
        })?;
        let mut yang_grammar = YangGrammar::new();
        match parse(&input, &path, &mut yang_grammar) {
            Ok(_) => yang(yang_grammar),
            // Hand the diagnostic back to the caller rather than
            // printing it: a library has no business writing to stdout,
            // and the position information is what makes the failure
            // actionable.
            Err(source) => Err(YangError::ParseError {
                path,
                source: Box::new(source),
            }),
        }
    }

    fn find_file(&self, module_name: &str) -> Result<PathBuf, YangError> {
        for path in &self.paths {
            if path.file_name() == Some(OsStr::new("...")) {
                let mut dir = path.clone();
                if dir.pop()
                    && let Ok(file_name) = find_in_dir(&dir, module_name, true)
                {
                    return Ok(file_name);
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

    /// Problems found while building entry trees with
    /// [`to_entry`](crate::to_entry), in the order they were found.
    ///
    /// These are warnings: a tree is still produced, with the offending
    /// augment skipped. They accumulate across calls, so a caller
    /// checking one module at a time should use
    /// [`take_diagnostics`](Self::take_diagnostics) instead.
    ///
    /// The returned guard borrows the store; hold it only as long as
    /// needed, since building another tree while it is alive will
    /// panic.
    pub fn diagnostics(&self) -> Ref<'_, Vec<Diagnostic>> {
        self.diagnostics.borrow()
    }

    /// Take the collected diagnostics, leaving the store empty.
    pub fn take_diagnostics(&self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics.borrow_mut())
    }

    /// Record a problem found while building an entry tree.
    pub(crate) fn diag(&self, diagnostic: Diagnostic) {
        self.diagnostics.borrow_mut().push(diagnostic);
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

    let dirent = fs::read_dir(dir).map_err(|source| YangError::IoError {
        path: dir.clone(),
        source,
    })?;
    for entry in dirent.into_iter().flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Some(os_str) = entry.path().file_name()
                    && let Some(file_str) = os_str.to_str()
                {
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
            } else if file_type.is_dir()
                && recursive
                && let Ok(path) = find_in_dir(&entry.path(), module_name, recursive)
            {
                return Ok(path);
            }
        }
    }
    if revisions.is_empty() {
        return Err(YangError::FileNotFound {
            name: module_name.to_string(),
        });
    }

    // When the specified file is not found by exact match, directories are
    // scanned for "name@revision-date.yang" files, the latest (sorted by
    // YYYY-MM-DD revision-date) of candidates will be selected.
    revisions.sort();

    Ok(revisions.pop().unwrap())
}
