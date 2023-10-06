// source/git.rs
use git2::{AutotagOption, FetchOptions, Object, ObjectType, Repository, Revwalk};
use std::{
    collections::HashMap,
    io::{self, Error as IOError, ErrorKind as IOErrorKind},
    path::PathBuf,
};
use walkdir::{DirEntry, WalkDir};

pub struct RepositoryDetails {
    pub url: String,
    pub name: String,
    pub local_path: PathBuf,
    pub markdown_output: PathBuf,
}

pub fn git_repo_check(details: &RepositoryDetails) -> Result<Repository, IOError> {
    if details.local_path.exists() {
        git_repo_update(&details.local_path)
    } else {
        git_repo_clone(&details.url, &details.local_path)
    }
}

fn git_repo_update(local_repo_path: &PathBuf) -> Result<Repository, IOError> {
    println!("Existing repo found. Pulling new data...");
    let repo = Repository::open(&local_repo_path).map_err(|e| {
        IOError::new(
            io::ErrorKind::Other,
            format!("Failed to open existing repo: {}", e),
        )
    })?;

    let mut remote = repo.find_remote("origin")?;
    let mut fetch_options = FetchOptions::new();
    fetch_options.download_tags(AutotagOption::All);
    remote.fetch(&["master"], Some(&mut fetch_options), None)?;

    Ok(repo)
}

fn git_repo_clone(repo_url: &str, local_repo_path: &PathBuf) -> Result<Repository, IOError> {
    println!("No existing repo found. Cloning...");
    Repository::clone(repo_url, local_repo_path)
        .map_err(|e| IOError::new(io::ErrorKind::Other, format!("Failed to clone repo: {}", e)))
}

pub fn git_latest_release(repo: &Repository) -> Result<(String, String), IOError> {
    let tags = repo.tag_names(None)?;
    let latest_tag = tags
        .iter()
        .last()
        .ok_or_else(|| IOError::new(IOErrorKind::Other, "No tags found"))?
        .ok_or_else(|| IOError::new(IOErrorKind::Other, "Invalid tag found"))?;

    let obj = repo.revparse_single(latest_tag)?;
    let commit = obj.peel_to_commit()?;

    let timestamp = commit.time();
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp.seconds(), 0)
        .expect("Expected a DateTime value");
    let formatted_datetime = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

    Ok((latest_tag.to_string(), formatted_datetime))
}

pub fn git_contributors(repo: &Repository) -> Result<HashMap<String, usize>, IOError> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let mut contributors: HashMap<String, usize> = HashMap::new();

    for commit_id in revwalk {
        if let Ok(id) = commit_id {
            let obj = repo.find_object(id, Some(ObjectType::Commit))?;
            let commit = obj.into_commit().expect("It's a commit object");
            let author = commit.author().name().unwrap_or("Unknown").to_string();
            *contributors.entry(author).or_insert(0) += 1;
        }
    }

    Ok(contributors)
}

// Process files in a repository and execute a callback for each file.

pub fn process_repo_files<F>(repo: &Repository, mut callback: F) -> Result<(), IOError>
where
    F: FnMut(&DirEntry) -> Result<(), IOError>,
{
    let repo_path = repo.path().parent().unwrap_or_else(|| Path::new(""));

    for entry in WalkDir::new(&repo_path) {
        let entry = entry.map_err(|e| IOError::new(ErrorKind::Other, e))?;
        if fops_skip(&entry) || !entry.path().is_file() {
            continue;
        }
        callback(&entry)?;
    }

    Ok(())
}
