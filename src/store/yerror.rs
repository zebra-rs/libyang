//use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum YangError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("YANG file not found")]
    FileNotFound,

    #[error("YANG file parse error")]
    ParseError,
}
