use std::collections::HashMap;
use std::fs;
use std::io::{Error as IOError, ErrorKind as IOErrorKind};

pub fn git_to_io_error(err: git2::Error) -> IOError {
    IOError::new(IOErrorKind::Other, err.to_string())
}

pub fn format_contributors(contributors: &HashMap<String, usize>) -> String {
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

pub fn determine_language_from_extension(file_extension: &str) -> Result<String, String> {
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
