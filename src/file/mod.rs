//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.json`.
//! It can be useful for testing purposes, or if you want to keep track of sent messages.
//!

use crate::file::error::FileResult;
use crate::{Email, Transport};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

pub mod error;

/// Writes the content and the envelope information to a file
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileTransport {
    path: PathBuf,
}

impl FileTransport {
    /// Creates a new transport to the given directory
    pub fn new<P: AsRef<Path>>(path: P) -> FileTransport {
        FileTransport {
            path: PathBuf::from(path.as_ref()),
        }
    }
}

impl<'a> Transport<'a> for FileTransport {
    type Result = FileResult;

    fn send<E: Into<Email>>(&mut self, email: E) -> FileResult {
        let email = email.into();

        let mut file = self.path.clone();
        file.push(format!("{}.json", email.message_id()));

        let serialized = serde_json::to_vec(&email)?;
        fs::write(file, &serialized)?;
        Ok(())
    }
}
