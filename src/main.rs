// main.rs

use goblin_scout::source::git;
use goblin_scout::tools::{errors::CustomError, fops, ui::prompt_for_repo_details};
use goblin_scout::trans_md::code_md as markdown_processor;
use goblin_scout::trans_md::md_json::convert_md_to_json;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn determine_output_directory(path: &Path) -> Result<PathBuf, CustomError> {
    path.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| CustomError::StrError("Invalid output path provided.".to_string()))
}

fn run() -> Result<(), CustomError> {
    let repo_details = prompt_for_repo_details()?;
    let repo = git::git_repo_check(&repo_details)?;

    println!("Please select an option for markdown output:");
    println!("1: Generate a single markdown file.");
    println!("2: Generate individual markdown files.");
    println!("3: Generate dataset markdown.");

    let mut option = String::new();
    io::stdin().read_line(&mut option)?;
    let option = option.trim();

    let output_directory = determine_output_directory(&repo_details.markdown_output)?;

    match option {
        "1" => {
            let markdown_content = markdown_processor::code_md_single_markdown(&repo)?;
            fops::fops_write(&repo_details.markdown_output, markdown_content)?;
            println!("Single markdown file updated.");
        }
        "2" => {
            markdown_processor::code_md_multi_markdown(&repo, &output_directory)?;
            println!("Individual markdown files generated.");
        }
        "3" => {
            let repo_name = &repo_details.name;

            let markdown_subdir = output_directory.join("dataset").join(&repo_name);
            markdown_processor::code_md_dataset_markdown(&repo, &markdown_subdir)?;
            println!("Dataset markdown generated.");

            println!("Would you like to create a JSON file? (y/n)");
            let mut json_option = String::new();
            io::stdin().read_line(&mut json_option)?;

            if json_option.trim().eq_ignore_ascii_case("y") {
                let json_path = output_directory.join(format!("{}.json", repo_name));

                convert_md_to_json(&markdown_subdir, &json_path)?;

                println!("JSON file created at: {:?}", json_path);
            }
        }
        _ => {
            println!("Invalid option selected.");
            return Err(CustomError::StrError(
                "Invalid option provided.".to_string(),
            ));
        }
    }

    Ok(())
}
