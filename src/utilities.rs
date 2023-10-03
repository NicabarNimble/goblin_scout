// utilities.rs

use std::fs::{self, File};
use std::io::{Error as IOError, Write};
use std::path::PathBuf;
use walkdir::DirEntry;

pub fn is_ignored(entry: &DirEntry) -> bool {
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

pub fn write_to_file(path: &PathBuf, content: String) -> Result<(), IOError> {
    // Ensure the directory for the file exists before writing
    if !path.parent().unwrap().exists() {
        fs::create_dir_all(path.parent().unwrap())?;
    }

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
