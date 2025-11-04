mod config;
#[allow(clippy::module_inception)]
mod llm;
mod translator;

pub use config::get_llm_provider;
pub use llm::{IssueDesc, LLM};
pub use translator::{should_translate, translate_with_llm};
