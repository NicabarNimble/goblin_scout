use chrono::Utc;
use git2::Repository;
use std::fs;
use std::io::Error as IOError;
use std::path::Path;
use walkdir::WalkDir;

use crate::utilities::{is_ignored, write_to_file};

use super::{
    determine_language_from_extension, format_contributors,
    repo_info::{gather_contributors, get_latest_release_info},
};

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

    let remote = repo.find_remote("origin").unwrap();
    let repo_url = remote.url().unwrap_or("").replace(".git", "");

    let head = repo.head().unwrap();
    let default_branch = head.shorthand().unwrap_or("main");

    let current_datetime = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let contributors = gather_contributors(repo)?;

    // Fetch the latest release info.
    let (latest_release, release_datetime) = get_latest_release_info(repo)?;

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

        let relative_path = entry
            .path()
            .strip_prefix(&repo_path)
            .unwrap_or(entry.path());
        let file_github_url = format!(
            "{}/blob/{}/{}",
            repo_url,
            default_branch,
            relative_path.display()
        );

        let file_name = entry
            .path()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        // Determine the language based on file extension
        let file_extension = entry
            .path()
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let language = determine_language_from_extension(file_extension)
            .unwrap_or_else(|_| String::from(file_extension));

        let contributor_list = format_contributors(&contributors);

        let header = format!(
            "---\n\
            title: {} - {}\n\
            date: {}\n\
            tags:\n\
            - {}\n\
            github: [{}]({})\n\
            contributors: {}\n\
            release: {} - {}\n\
            ---\n\n\
            File\n\
            Path: {}\n\
            Size: {} bytes\n",
            repo_name,
            file_name,
            current_datetime,
            language,
            file_name,
            file_github_url,
            contributor_list,
            latest_release,
            release_datetime,
            relative_path.display(),
            content.len()
        );

        let file_markdown = format!("{}\n```\n{}\n```\n", header, content);

        let output_file_name = format!("{}.md", relative_path.to_string_lossy());
        let output_file_path = output_dir.join(output_file_name);

        write_to_file(&output_file_path, file_markdown)?;
    }

    Ok(())
}
