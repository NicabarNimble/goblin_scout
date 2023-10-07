// src/lib.rs

pub mod source {
    pub mod git;
}

pub mod tools {
    pub mod errors;
    pub mod fops;
    pub mod ui;
}

pub mod trans_md {
    pub mod code_md;
}

pub use source::git;
pub use tools::errors::CustomError;
pub use tools::fops;
pub use tools::ui::prompt_for_repo_details;
pub use trans_md::code_md as markdown_processor;
