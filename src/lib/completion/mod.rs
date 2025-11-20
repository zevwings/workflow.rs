#[allow(clippy::module_inception)]
pub mod completion;
pub mod files;
pub mod generate;

pub use completion::Completion;
pub use files::{
    get_all_completion_files, get_completion_filename, get_completion_files_for_shell,
};
pub use generate::{
    generate_all_completions, generate_pr_completion, generate_qk_completion,
    generate_workflow_completion,
};
