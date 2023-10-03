// models.rs

use std::path::PathBuf;

pub struct RepositoryDetails {
    pub url: String,
    pub name: String,
    pub local_path: PathBuf,
    pub markdown_output: PathBuf,
}
