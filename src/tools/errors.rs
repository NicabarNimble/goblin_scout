// tools/errors.rs

use git2::Error as GitError;
use std::fmt;
use std::io::{self, Error as IOError, ErrorKind as IOErrorKind};

#[derive(Debug)]
pub enum CustomError {
    IOError(io::Error),
    StrError(String),
}

impl From<io::Error> for CustomError {
    fn from(error: io::Error) -> Self {
        CustomError::IOError(error)
    }
}

impl From<&str> for CustomError {
    fn from(error: &str) -> Self {
        CustomError::StrError(error.to_string())
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::IOError(e) => write!(f, "IOError: {}", e),
            CustomError::StrError(s) => write!(f, "StrError: {}", s),
        }
    }
}

// Convert git2::Error (or your custom GitError) to standard IOError
pub fn git_to_io_error(err: GitError) -> IOError {
    IOError::new(IOErrorKind::Other, err.to_string())
}
