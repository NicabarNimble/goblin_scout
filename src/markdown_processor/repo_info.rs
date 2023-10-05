use git2::{Object, ObjectType, Repository, Revwalk};
use std::{
    collections::HashMap,
    io::{Error as IOError, ErrorKind as IOErrorKind},
};

use super::git_to_io_error;

pub fn get_latest_release_info(repo: &Repository) -> Result<(String, String), IOError> {
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

pub fn gather_contributors(repo: &Repository) -> Result<HashMap<String, usize>, IOError> {
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
