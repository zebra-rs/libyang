use parol_runtime::ParolError;
use std::path::PathBuf;

/// Errors returned while locating, reading and parsing YANG modules.
///
/// Every variant names the file it applies to, and `ParseError` carries
/// the underlying parol diagnostic (which reports the position of the
/// offending token), so a failure can be reported by the caller without
/// the library writing anything to stdout itself. The `Display` text
/// includes the source message, so printing the error alone is enough;
/// `#[source]` is also set for callers that walk the chain.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum YangError {
    /// Reading a YANG file, or scanning a search directory, failed.
    #[error("{}: {source}", path.display())]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// No file matching the module name was found on the search path.
    #[error("YANG module `{name}` not found in the search path")]
    FileNotFound { name: String },

    /// The file was read but did not parse as YANG.
    #[error("{}: {source}", path.display())]
    ParseError {
        path: PathBuf,
        // Boxed because ParolError is comparatively large, and this
        // error rides in every Result the loader returns.
        #[source]
        source: Box<ParolError>,
    },

    /// The file parsed, but contained neither a module nor a submodule.
    #[error("YANG document contains neither a module nor a submodule")]
    EmptyDocument,
}
