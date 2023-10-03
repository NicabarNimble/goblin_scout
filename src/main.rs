// main.rs

mod errors;
mod git_source;
mod markdown_processor;
mod models;
mod utilities;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> Result<(), errors::CustomError> {
    let repo_details = git_source::prompt_for_repo_details()?;
    let repo = git_source::get_or_update_repo(&repo_details)?;

    let markdown_content = markdown_processor::gather_repo_content(&repo)?;
    utilities::write_to_file(&repo_details.markdown_output, markdown_content)?;

    println!("Markdown file updated.");
    Ok(())
}
