// git_source.rs

use crate::models::RepositoryDetails;
use git2::{FetchOptions, Repository};
use std::io::{self, Error as IOError};
use std::path::PathBuf;

pub fn prompt_for_repo_details() -> Result<RepositoryDetails, IOError> {
    let mut input = String::new();
    println!("Please enter the repo URL:");
    io::stdin().read_line(&mut input)?;
    let repo_url = input.trim().to_string();

    let repo_name = repo_url
        .split('/')
        .last()
        .unwrap_or("unknown_repo")
        .to_string();
    let local_repo_path = PathBuf::from("repositories").join(&repo_name);
    let markdown_output = PathBuf::from("markdown").join(format!("{}.md", &repo_name));

    Ok(RepositoryDetails {
        url: repo_url,
        name: repo_name,
        local_path: local_repo_path,
        markdown_output,
    })
}

pub fn get_or_update_repo(details: &RepositoryDetails) -> Result<Repository, IOError> {
    if details.local_path.exists() {
        update_existing_repo(&details.local_path)
    } else {
        clone_new_repo(&details.url, &details.local_path)
    }
}

fn update_existing_repo(local_repo_path: &PathBuf) -> Result<Repository, IOError> {
    println!("Existing repo found. Pulling new data...");
    let repo = Repository::open(&local_repo_path).map_err(|e| {
        IOError::new(
            io::ErrorKind::Other,
            format!("Failed to open existing repo: {}", e),
        )
    })?;

    {
        let mut remote = repo.find_remote("origin").unwrap();
        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);
        remote
            .fetch(&["master"], Some(&mut fetch_options), None)
            .unwrap();
    }

    Ok(repo)
}

fn clone_new_repo(repo_url: &str, local_repo_path: &PathBuf) -> Result<Repository, IOError> {
    println!("No existing repo found. Cloning...");
    Repository::clone(repo_url, local_repo_path)
        .map_err(|e| IOError::new(io::ErrorKind::Other, format!("Failed to clone repo: {}", e)))
}
