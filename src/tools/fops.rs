// tools/fops.rs

use crate::tools::errors::CustomError;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub fn fops_skip(file_path: &Path) -> Result<bool, CustomError> {
    let mut buffer = [0; 1024]; // Read first 1024 bytes
    let mut file = File::open(file_path)?;
    file.read(&mut buffer)?;
    Ok(buffer.contains(&0x00))
}

pub fn fops_write(path: &PathBuf, content: String) -> Result<(), CustomError> {
    // Ensure the directory for the file exists before writing
    fops_mkdir(&path)?;

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn fops_mkdir(path: &PathBuf) -> Result<(), CustomError> {
    let parent_directory = path.parent().ok_or_else(|| {
        CustomError::StrError("Failed to get parent directory of path".to_string())
    })?;
    if !parent_directory.exists() {
        fs::create_dir_all(parent_directory)?;
    }
    Ok(())
}
