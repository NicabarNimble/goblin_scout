// markdown_processor.rs

use crate::utilities::is_ignored;
use git2::Repository;
use std::fs;
use std::io::Error as IOError;
use std::path::Path;
use walkdir::WalkDir;

pub fn gather_repo_content(repo: &Repository) -> Result<String, IOError> {
    let mut markdown_content = String::new();
    let repo_path = repo.path().parent().unwrap_or_else(|| Path::new(""));

    for entry in WalkDir::new(&repo_path) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error iterating directory: {}", e);
                continue;
            }
        };

        if is_ignored(&entry) || !entry.path().is_file() {
            continue;
        }

        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        let file_markdown = format!(
            "## File: {}\n\n```\n{}\n```\n",
            entry.path().display(),
            content
        );
        markdown_content.push_str(&file_markdown);
    }

    Ok(markdown_content)
}
