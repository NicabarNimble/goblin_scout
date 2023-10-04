// markdown_processor.rs

use crate::utilities::{is_ignored, write_to_file};
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

pub fn generate_markdown_files(repo: &Repository, base_output_dir: &Path) -> Result<(), IOError> {
    let repo_name = repo
        .workdir()
        .and_then(|path| path.file_name())
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("unknown_repo");
    let output_dir = base_output_dir.join(repo_name);

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
        let header = format!(
            "# File: {}\n\n- Path: {}\n- Size: {} bytes\n\n",
            entry
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            entry.path().display(),
            content.len()
        );

        let file_markdown = format!("{}\n```\n{}\n```\n", header, content);

        let relative_path = entry
            .path()
            .strip_prefix(&repo_path)
            .unwrap_or(entry.path());

        // Adjusted here to keep the original extension and add `.md` after
        let output_file_name = format!("{}.md", relative_path.to_string_lossy());
        let output_file_path = output_dir.join(output_file_name);

        write_to_file(&output_file_path, file_markdown)?;
    }

    Ok(())
}
