// markdown_processor.rs

use crate::utilities::{is_ignored, write_to_file};
use chrono::Utc;
use git2::{Object, ObjectType, Repository, Revwalk};
use std::collections::HashMap;
use std::fs;
use std::io::{Error as IOError, ErrorKind as IOErrorKind};
use std::path::Path;
use walkdir::WalkDir;

// Adjust the return type to handle both git2::Error and IOError.
fn git_to_io_error(err: git2::Error) -> IOError {
    IOError::new(IOErrorKind::Other, err.to_string())
}

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

fn get_latest_release_info(repo: &Repository) -> Result<(String, String), IOError> {
    let tags = repo.tag_names(None).map_err(git_to_io_error)?;
    let latest_tag = tags
        .iter()
        .last()
        .ok_or_else(|| IOError::new(IOErrorKind::Other, "No tags found"))?
        .ok_or_else(|| IOError::new(IOErrorKind::Other, "Invalid tag found"))?;

    let obj = repo.revparse_single(latest_tag).map_err(git_to_io_error)?;
    let commit = obj.peel_to_commit().map_err(git_to_io_error)?;

    let timestamp = commit.time();
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp.seconds(), 0);
    let formatted_datetime = datetime
        .expect("Expected a DateTime value")
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    Ok((latest_tag.to_string(), formatted_datetime))
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

fn gather_contributors(repo: &Repository) -> Result<HashMap<String, usize>, IOError> {
    let mut revwalk: Revwalk = repo.revwalk().map_err(git_to_io_error)?;
    revwalk.push_head().map_err(git_to_io_error)?;
    let mut contributors: HashMap<String, usize> = HashMap::new();

    for commit_id in revwalk {
        match commit_id {
            Ok(id) => {
                let obj: Object = repo
                    .find_object(id, Some(ObjectType::Commit))
                    .map_err(git_to_io_error)?;
                let commit = obj.into_commit().expect("It's a commit object");
                let author = commit.author().name().unwrap_or("Unknown").to_string();
                *contributors.entry(author).or_insert(0) += 1;
            }
            Err(_) => continue,
        }
    }
    Ok(contributors)
}

fn format_contributors(contributors: &HashMap<String, usize>) -> String {
    let mut contributors_vec: Vec<(&String, &usize)> = contributors.iter().collect();
    contributors_vec.sort_by(|a, b| b.1.cmp(a.1));

    contributors_vec
        .into_iter()
        .filter(|&(_, count)| *count > 1) // filter out those with 1 or fewer commits
        .take(5) // take top 5 contributors
        .map(|(author, count)| format!("{} ({})", author, count))
        .collect::<Vec<String>>()
        .join(" | ") // separate by '|'
}

fn determine_language_from_extension(file_extension: &str) -> Result<String, String> {
    // Load the extension to language mapping from the JSON file
    let file_contents = fs::read_to_string("extension_mapping.json")
        .map_err(|_| "Failed to read the extension_mapping.json".to_string())?;

    // Parse the JSON content
    let mapping: HashMap<String, Vec<String>> = serde_json::from_str(&file_contents)
        .map_err(|_| "Failed to parse the JSON content".to_string())?;

    // Search for the file extension in the mapping
    for (language, extensions) in mapping.iter() {
        if extensions.contains(&file_extension.to_string()) {
            return Ok(language.clone());
        }
    }

    // If the file extension isn't found in the list, return the file extension itself
    Ok(file_extension.to_string())
}
