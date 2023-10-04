// main.rs

mod errors;
mod git_source;
mod markdown_processor;
mod models;
mod utilities;
use std::io;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> Result<(), errors::CustomError> {
    let repo_details = git_source::prompt_for_repo_details()?;
    let repo = git_source::get_or_update_repo(&repo_details)?;

    // Prompt the user for their desired markdown output option
    println!("Please select an option for markdown output:");
    println!("1: Generate a single markdown file.");
    println!("2: Generate individual markdown files.");
    let mut option = String::new();
    io::stdin().read_line(&mut option)?;
    let option = option.trim();

    match option {
        "1" => {
            let markdown_content = markdown_processor::gather_repo_content(&repo)?;
            utilities::write_to_file(&repo_details.markdown_output, markdown_content)?;
            println!("Single markdown file updated.");
        }
        "2" => {
            let output_directory = repo_details.markdown_output.parent().unwrap_or_else(|| {
                println!("Invalid output path provided in repo details. Using current directory as default.");
                std::path::Path::new(".")
            });
            markdown_processor::generate_markdown_files(&repo, &output_directory)?;
            println!("Individual markdown files generated.");
        }
        _ => {
            println!("Invalid option selected.");
            return Err(errors::CustomError::from("Invalid option provided."));
        }
    }

    Ok(())
}
