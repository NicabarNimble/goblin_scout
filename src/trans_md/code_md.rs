// trans_md/code_md.rs

// --------------------------
// IMPORTS
// --------------------------
use crate::git::process_repo_files;
use crate::source::git::{git_contributors, git_latest_release};
use crate::tools::errors::CustomError;
use crate::tools::fops;

use chrono::Utc;
use git2::Repository;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;
use walkdir::DirEntry;

// ---------------------------
// UTILITY GENERAL FUNCTIONS
// ---------------------------

// Return path to the assets/lang_maps.json file.
pub fn get_asset_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src");
    path.push("assets");
    path.push("lang_maps.json");
    path
}

fn generate_uuid() -> String {
    Uuid::new_v4().hyphenated().to_string()
}

// ---------------------------
// UTILITY FORMATING FUNCTIONS
// ---------------------------

// Determine programming language from file extension using asset mappings.
pub fn md_lang_maps(file_extension: &str) -> Result<String, CustomError> {
    let asset_path = get_asset_path();

    // Read the content of the file.
    let content = fs::read_to_string(&asset_path).map_err(|e| {
        CustomError::StrError(format!("Failed to read {}: {}", asset_path.display(), e))
    })?;

    // Deserialize the JSON content into a HashMap.
    let mapping: HashMap<String, Vec<String>> = serde_json::from_str(&content).map_err(|e| {
        CustomError::DetailedJsonParsingError("assets/lang_maps.json".into(), e.to_string())
    })?;

    let formatted_extension = if file_extension.starts_with('.') {
        file_extension.to_string()
    } else {
        format!(".{}", file_extension)
    };

    for (language, extensions) in mapping.iter() {
        if extensions.contains(&formatted_extension) {
            return Ok(language.to_string());
        }
    }

    Ok(formatted_extension)
}

// Format top 5 contributors with more than 1 commit
pub fn md_contrib_five(contributors: &HashMap<String, usize>) -> String {
    let mut contributors_vec: Vec<_> = contributors
        .iter()
        .filter(|&(_, &count)| count > 1)
        .collect();

    contributors_vec.sort_by(|a, b| b.1.cmp(a.1));

    contributors_vec
        .into_iter()
        .take(5)
        .map(|(author, count)| format!("{} ({})", author, count))
        .collect::<Vec<_>>()
        .join(" | ")
}

// Creates the markdown header for a given file, containing metadata such as title, date, tags, etc.
fn create_markdown_header(
    repo_name: &str,
    file_name: &str,
    current_datetime: &str,
    language: &str,
    file_github_url: &str,
    contributor_list: &str,
    latest_release: &str,
    release_datetime: &str,
    relative_path: &Path,
    content_length: usize,
    file_uuid: &str,
) -> String {
    format!(
        "---\n\
        title: {} - {}\n\
        date: {}\n\
        tags:\n\
        - {}\n\
        - [UUID:{}]\n\
        github: [{}]({})\n\
        contributors: {}\n\
        latest_release: {}\n\
        release_date: {}\n\
        path: {:?}\n\
        size: {}\n\
        ---\n",
        repo_name,
        file_name,
        current_datetime,
        language,
        file_uuid,
        file_name,
        file_github_url,
        contributor_list,
        latest_release,
        release_datetime,
        relative_path.display(),
        content_length
    )
}

// Constructs a GitHub URL for the specific file based on its relative path, the repository URL, and branch.
fn construct_github_url(repo_url: &str, default_branch: &str, relative_path: &Path) -> String {
    format!(
        "{}/blob/{}/{}",
        repo_url,
        default_branch,
        relative_path.to_string_lossy()
    )
}

// Generates the appropriate output file name for a markdown file based on its relative path.
fn get_output_file_name(relative_path: &Path) -> String {
    format!("{}.md", relative_path.to_string_lossy())
}
// ----------------------------
// MARKDOWN CREATION FUNCTIONS
// ----------------------------

// Generate a single markdown file from repository content.
pub fn code_md_single_markdown(repo: &Repository) -> Result<String, CustomError> {
    let mut markdown_content = String::new();
    process_repo_files(repo, &mut |entry: &DirEntry| {
        markdown_content.push_str(&format!(
            "## File: {}\n\n```\n{}\n```\n",
            entry.path().display(),
            fs::read_to_string(entry.path())?
        ));
        Ok(())
    })?;
    Ok(markdown_content)
}

// Generate individual markdown files from repository content maintaining file structure.
pub fn code_md_multi_markdown(
    repo: &Repository,
    base_output_dir: &Path,
) -> Result<(), CustomError> {
    let repo_name = repo
        .workdir()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_repo");

    let output_dir = base_output_dir.join(repo_name);
    let repo_path = repo.path().parent().unwrap_or(Path::new(""));

    fops::fops_mkdir(&output_dir)?;

    let remote = repo.find_remote("origin")?;
    let repo_url = remote.url().unwrap_or("").replace(".git", "");

    let head = repo.head()?;
    let default_branch = head.shorthand().unwrap_or("main");
    let current_datetime = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let contributors = git_contributors(repo)?;
    let (latest_release, release_datetime) = git_latest_release(repo)?;

    process_repo_files(repo, |entry| {
        if fops::fops_skip(entry) {
            return Ok(());
        }

        let content = fs::read_to_string(entry.path())?;
        let relative_path = entry
            .path()
            .strip_prefix(&repo_path)
            .unwrap_or(entry.path());

        let file_github_url = construct_github_url(&repo_url, &default_branch, &relative_path);
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

        let language = md_lang_maps(file_extension).unwrap_or_else(|_| file_extension.to_string());
        let file_uuid = generate_uuid();
        let contributor_list = md_contrib_five(&contributors);
        let header = create_markdown_header(
            &repo_name,
            &file_name,
            &current_datetime,
            &language,
            &file_github_url,
            &contributor_list,
            &latest_release,
            &release_datetime,
            &relative_path,
            content.len(),
            &file_uuid,
        );

        let file_markdown = format!("{}\n```\n{}\n```\n", header, content);
        let output_file_name = get_output_file_name(&relative_path);
        let output_file_path = output_dir.join(&output_file_name);

        fops::fops_write(&output_file_path, file_markdown)?;
        Ok(())
    })?;
    Ok(())
}

// Prepare for datasets individual markdown files from repository content maintaining file structure.
pub fn code_md_dataset_markdown(
    repo: &Repository,
    base_output_dir: &Path,
) -> Result<(), CustomError> {
    let repo_name = repo
        .workdir()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_repo");

    let output_dir = base_output_dir.join(repo_name);
    let repo_path = repo.path().parent().unwrap_or(Path::new(""));

    fops::fops_mkdir(&output_dir)?;

    let remote = repo.find_remote("origin")?;
    let repo_url = remote.url().unwrap_or("").replace(".git", "");

    let head = repo.head()?;
    let default_branch = head.shorthand().unwrap_or("main");
    let current_datetime = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let contributors = git_contributors(repo)?;
    let (latest_release, release_datetime) = git_latest_release(repo)?;

    process_repo_files(repo, |entry| {
        if fops::fops_skip(entry) {
            return Ok(());
        }

        let content = fs::read_to_string(entry.path())?;
        let relative_path = entry
            .path()
            .strip_prefix(&repo_path)
            .unwrap_or(entry.path());

        let file_github_url = construct_github_url(&repo_url, &default_branch, &relative_path);
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

        let language = md_lang_maps(file_extension).unwrap_or_else(|_| file_extension.to_string());

        let file_uuid = generate_uuid();
        let contributor_list = md_contrib_five(&contributors);
        let header = create_markdown_header(
            &repo_name,
            &file_name,
            &current_datetime,
            &language,
            &file_github_url,
            &contributor_list,
            &latest_release,
            &release_datetime,
            &relative_path,
            content.len(),
            &file_uuid,
        );

        let mut split_content = Vec::new();
        let chars = content.chars().collect::<Vec<_>>();
        let mut current_pos = 0;

        while current_pos < chars.len() {
            let mut end_pos = current_pos + 512;
            if end_pos > chars.len() {
                end_pos = chars.len();
            } else {
                // Look for newline if we are at least 500 characters in.
                if end_pos >= current_pos + 500 {
                    let mut found = false;
                    for i in current_pos + 500..=end_pos {
                        if chars[i] == '\n' {
                            end_pos = i + 1; // include the newline character in the current chunk
                            found = true;
                            break;
                        }
                    }

                    // If no newline was found by 512, extend the search up to 750 characters for a newline.
                    if !found && end_pos < chars.len() {
                        for i in end_pos..std::cmp::min(current_pos + 750, chars.len()) {
                            if chars[i] == '\n' {
                                end_pos = i + 1;
                                found = true;
                                break;
                            }
                        }
                    }

                    // If no newline was found by 750, look for whitespace.
                    if !found && end_pos < chars.len() {
                        for i in end_pos..std::cmp::min(current_pos + 750, chars.len()) {
                            if chars[i].is_whitespace() {
                                end_pos = i;
                                break;
                            }
                        }
                    }
                }
            }

            let chunk: String = chars[current_pos..end_pos].into_iter().collect();
            let code_uuid = generate_uuid();
            split_content.push(format!("[UUID:{}]\n```\n{}\n```\n", code_uuid, chunk));

            current_pos = end_pos;
        }

        let file_markdown = format!("{}\n{}", header, split_content.join("\n"));
        let output_file_name = get_output_file_name(&relative_path);
        let output_file_path = output_dir.join(&output_file_name);

        fops::fops_write(&output_file_path, file_markdown)?;
        Ok(())
    })?;
    Ok(())
}

// --------------------------
// TESTS
// --------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Test if the markdown tag is created correctly for given file extension.
    #[test]
    fn test_markdown_tag_creation() {
        // Mock data or setup
        let file_extension = "rs";

        let asset_path = get_asset_path();
        println!("Asset path from test: {}", asset_path.display());

        // Invoke your function
        let language_tag = md_lang_maps(file_extension).unwrap();

        // Assert the expected outcome
        assert_eq!(language_tag, "Rust");
    }

    // ... more tests as needed ...
}
