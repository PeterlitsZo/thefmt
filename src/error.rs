use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Usage,
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse(markdown::message::Message),
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Usage => write!(f, "Usage: thefmt <file.md>"),
            Error::Read { path, source } => {
                write!(f, "failed to read {}: {source}", path.display())
            }
            Error::Parse(source) => write!(f, "failed to parse markdown: {source}"),
            Error::Write { path, source } => {
                write!(f, "failed to write {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {}
