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

    match option {
        "1" => {
            let markdown_content = markdown_processor::code_md_single_markdown(&repo)?;
            fops::fops_write(&repo_details.markdown_output, markdown_content)?;
            println!("Single markdown file updated.");
        }
        "2" | "3" => {
            let output_directory = determine_output_directory(&repo_details.markdown_output)?;

            if option == "2" {
                markdown_processor::code_md_multi_markdown(&repo, &output_directory)?;
                println!("Individual markdown files generated.");
            } else if option == "3" {
                markdown_processor::code_md_dataset_markdown(&repo, &output_directory)?;
                println!("Dataset markdown generated.");

                println!("Would you like to create a JSON file? (y/n)");
                let mut json_option = String::new();
                io::stdin().read_line(&mut json_option)?;

                if json_option.trim().to_lowercase() == "y" {
                    // Get the repo name from repo_details.markdown_output
                    let repo_name = repo_details
                        .markdown_output
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap();

                    // Create the JSON filename based on the repo name
                    let json_filename = format!("{}.json", repo_name);

                    let json_path = output_directory.join(json_filename);

                    convert_md_to_json(&output_directory, &json_path)?;

                    println!("JSON file created at: {:?}", json_path);
                }
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
