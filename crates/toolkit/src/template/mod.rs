//! Template module
//!
//! Provides template rendering functionality.

pub(crate) mod engine;
mod error;

pub use engine::{TemplateEngine, TemplateEngineType};
pub use error::TemplateError;
