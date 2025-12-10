//! LLM configuration management subcommands

use clap::Subcommand;

/// LLM configuration management subcommands
///
/// Used to manage LLM provider, API keys, models, and language settings.
#[derive(Subcommand)]
pub enum LLMSubcommand {
    /// Show current LLM configuration
    ///
    /// Display current LLM provider, API key (masked), model, and language settings.
    Show,
    /// Setup LLM configuration
    ///
    /// Interactively configure LLM provider, proxy URL, API key, model, and language settings.
    Setup,
}
