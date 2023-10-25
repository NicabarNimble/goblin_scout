// trans_md/md_json.rs

use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::{self, read_to_string, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct FileMetadata {
    title: String,
    date: String,
    tags: Vec<String>,
    UUID: String,
    github: String,
    contributors: String,
    latest_release: String,
    release_date: String,
    path: String,
    size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Section {
    UUID: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileContent {
    file_metadata: FileMetadata,
    sections: Vec<Section>,
}

fn extract_from_md(file_content: &str) -> FileContent {
    let parts: Vec<&str> = file_content.split("---\n").collect();

    // Parse metadata
    let metadata = parts[1];
    let metadata: FileMetadata =
        serde_yaml::from_str(metadata).expect("Failed to parse YAML metadata");

    // Extract sections using UUID as delimiter
    let content_parts = parts[2..].join("---\n");
    let sections: Vec<&str> = content_parts
        .split("\n[UUID:")
        .filter(|&s| !s.is_empty())
        .collect();

    let mut parsed_sections = Vec::new();

    for section in sections {
        let section_parts: Vec<&str> = section.split("]\n").collect();
        if section_parts.len() != 2 {
            continue;
        }
        let uuid = format!("[UUID:{}", section_parts[0]);
        let content = section_parts[1].to_string();
        parsed_sections.push(Section {
            UUID: uuid,
            content,
        });
    }

    FileContent {
        file_metadata: metadata,
        sections: parsed_sections,
    }
}

fn process_file<P: AsRef<Path>>(path: P) -> io::Result<Option<FileContent>> {
    let content = read_to_string(&path)?;
    if content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(extract_from_md(&content)))
    }
}

fn traverse_directory<P: AsRef<Path>>(path: P) -> io::Result<Vec<FileContent>> {
    let mut result = Vec::new();

    if path.as_ref().is_dir() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                result.extend(traverse_directory(&path)?);
            } else if path.extension() == Some(std::ffi::OsStr::new("md")) {
                if let Some(file_content) = process_file(&path)? {
                    result.push(file_content);
                }
            }
        }
    }
    Ok(result)
}

pub fn convert_md_to_json<P1: AsRef<Path>, P2: AsRef<Path>>(
    src_dir: P1,
    dest_file: P2,
) -> io::Result<()> {
    let files_data = traverse_directory(src_dir)?;
    let json_output = serde_json::to_string_pretty(&files_data)?;
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_file)?
        .write_all(json_output.as_bytes())?;
    Ok(())
}
