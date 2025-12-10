#[allow(clippy::module_inception)]
pub mod completion;
pub mod generate;
pub mod helpers;

pub use completion::{Completion, CompletionConfigResult, CompletionRemovalResult};
pub use generate::CompletionGenerator;
pub use helpers::{
    get_all_completion_files, get_completion_filename, get_completion_files_for_shell,
};
