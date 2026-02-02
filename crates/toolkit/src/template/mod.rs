//! Template module
//!
//! Provides template rendering functionality.

pub mod engine;
mod error;

pub use engine::{TemplateEngine, TemplateEngineType};
pub use error::TemplateError;
