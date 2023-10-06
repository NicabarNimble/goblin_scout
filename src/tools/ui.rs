// Prompt user for repository details
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
