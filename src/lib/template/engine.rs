//! Template engine wrapper
//!
//! Provides a unified interface for template rendering using handlebars.

use crate::base::util::date::get_unix_timestamp_nanos;
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use handlebars::Handlebars;
use serde::Serialize;

/// Template engine type
#[derive(Debug, Clone, Copy)]
pub enum TemplateEngineType {
    /// Handlebars template engine
    Handlebars,
}

/// Template engine wrapper
///
/// Provides a unified interface for template rendering.
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        handlebars.register_escape_fn(handlebars::no_escape); // Don't escape HTML

        Self { handlebars }
    }

    /// Register a template
    ///
    /// # Arguments
    ///
    /// * `name` - Template name
    /// * `template` - Template string
    pub fn register_template(&mut self, name: &str, template: &str) -> Result<()> {
        self.handlebars
            .register_template_string(name, template)
            .wrap_err_with(|| format!("Failed to register template: {}", name))?;
        Ok(())
    }

    /// Render a template with variables
    ///
    /// # Arguments
    ///
    /// * `name` - Template name
    /// * `vars` - Template variables (must implement Serialize)
    ///
    /// # Returns
    ///
    /// Rendered template string
    pub fn render<T: Serialize>(&self, name: &str, vars: &T) -> Result<String> {
        self.handlebars
            .render(name, vars)
            .map_err(|e| eyre!("Failed to render template '{}': {}", name, e))
    }

    /// Render a template string directly (without registration)
    ///
    /// # Arguments
    ///
    /// * `template` - Template string
    /// * `vars` - Template variables (must implement Serialize)
    ///
    /// # Returns
    ///
    /// Rendered template string
    pub fn render_string<T: Serialize>(&self, template: &str, vars: &T) -> Result<String> {
        // Register template with a temporary name
        let timestamp = get_unix_timestamp_nanos();
        let temp_name = format!("__temp_{}", timestamp);
        let mut engine = TemplateEngine::new();
        engine.register_template(&temp_name, template)?;
        engine.render(&temp_name, vars)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}
