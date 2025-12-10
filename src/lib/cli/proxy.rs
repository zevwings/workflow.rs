//! Proxy management subcommands

use clap::Subcommand;

/// Proxy management subcommands
///
/// Used to manage HTTP/HTTPS proxy environment variable configuration.
#[derive(Subcommand)]
pub enum ProxySubcommand {
    /// Enable proxy (set environment variables)
    ///
    /// Set HTTP_PROXY and HTTPS_PROXY environment variables.
    On,
    /// Disable proxy (clear environment variables)
    ///
    /// Unset HTTP_PROXY and HTTPS_PROXY environment variables.
    Off,
    /// Check proxy status and configuration
    ///
    /// Display current proxy environment variable status and configuration information.
    Check,
}
