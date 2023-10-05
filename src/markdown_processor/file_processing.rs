use chrono::Utc;
use git2::Repository;
use std::fs;
use std::io::{Error as IOError, ErrorKind};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use crate::utilities::{is_ignored, write_to_file};

use super::{
    determine_language_from_extension, format_contributors,
    repo_info::{gather_contributors, get_latest_release_info},
};

/// Process files in a repository and execute a callback for each file.
fn process_repo_files<F>(repo: &Repository, mut callback: F) -> Result<(), IOError>
where
    F: FnMut(&DirEntry) -> Result<(), IOError>,
{
    let repo_path = repo.path().parent().unwrap_or_else(|| Path::new(""));

    for entry in WalkDir::new(&repo_path) {
        let entry = entry.map_err(|e| IOError::new(ErrorKind::Other, e))?;
        if is_ignored(&entry) || !entry.path().is_file() {
            continue;
        }
        callback(&entry)?;
    }

    Ok(())
}

/// Gather the content of the repository in markdown format.
pub fn gather_repo_content(repo: &Repository) -> Result<String, IOError> {
    let mut markdown_content = String::new();

    process_repo_files(repo, &mut |entry: &DirEntry| {
        let content = fs::read_to_string(entry.path())?;
        markdown_content.push_str(&format!(
            "## File: {}\n\n```\n{}\n```\n",
            entry.path().display(),
            content
        ));
        Ok(())
    })?;

    Ok(markdown_content)
}

/// Generate markdown files for the content of the repository.
pub fn generate_markdown_files(repo: &Repository, base_output_dir: &Path) -> Result<(), IOError> {
    let repo_name = repo
        .workdir()
        .and_then(|path| path.file_name())
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("unknown_repo");

    let output_dir = base_output_dir.join(repo_name);
    let repo_path = repo.path().parent().unwrap_or_else(|| Path::new(""));

    // Extract repository remote and URL
    let remote = repo
        .find_remote("origin")
        .map_err(|e| IOError::new(ErrorKind::Other, e))?;
    let repo_url = remote.url().unwrap_or("").replace(".git", "");

    // Extract branch and date details
    let head = repo.head().map_err(|e| IOError::new(ErrorKind::Other, e))?;
    let default_branch = head.shorthand().unwrap_or("main");
    let current_datetime = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Extract contributors and release details
    let contributors = gather_contributors(repo)?;
    let (latest_release, release_datetime) = get_latest_release_info(repo)?;

    process_repo_files(repo, |entry| {
        let content = fs::read_to_string(entry.path())?;
        let relative_path = entry
            .path()
            .strip_prefix(&repo_path)
            .unwrap_or(entry.path());

        // Build URLs, determine language, and get contributor lists
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
        let file_extension = entry
            .path()
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let language = determine_language_from_extension(file_extension)
            .unwrap_or_else(|_| String::from(file_extension));
        let contributor_list = format_contributors(&contributors);

        // Create the markdown header and body
        let header = format!("---\ntitle: {} - {}\ndate: {}\ntags:\n- {}\ngithub: [{}]({})\ncontributors: {}\nrelease: {} - {}\n---\n\nFile\nPath: {}\nSize: {} bytes\n", repo_name, file_name, current_datetime, language, file_name, file_github_url, contributor_list, latest_release, release_datetime, relative_path.display(), content.len());
        let file_markdown = format!("{}\n```\n{}\n```\n", header, content);

        // Define the output file path
        let output_file_name = format!("{}.md", relative_path.to_string_lossy());
        let output_file_path = output_dir.join(output_file_name);

        write_to_file(&output_file_path, file_markdown)
    })?;

    Ok(())
}
