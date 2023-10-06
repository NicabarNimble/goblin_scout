// tools/fops.rs

use crate::tools::errors::CustomError;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use walkdir::DirEntry;

pub fn fops_skip(entry: &DirEntry) -> bool {
    let path = entry.path();
    let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

    file_name.starts_with('.')
        || path.to_string_lossy().contains("/.git/")
        || [
            "png", "jpg", "jpeg", "gif", "ico", "bin", "exe", "dll", "so", "dylib",
        ]
        .iter()
        .any(|&ext| path.extension().map_or(false, |p_ext| p_ext == ext))
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
