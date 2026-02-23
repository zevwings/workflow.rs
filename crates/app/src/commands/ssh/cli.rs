//! SSH management subcommands
//!
//! SSH 密钥管理子命令结构定义

use std::path::PathBuf;

use clap::Subcommand;

/// SSH 管理子命令
#[derive(Subcommand)]
pub enum SshCommand {
    /// Generate a new SSH key pair
    Generate {
        /// Output path (default: ~/.ssh/id_ed25519)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Key algorithm: ed25519 (default) or rsa
        #[arg(short, long, default_value = "ed25519")]
        algorithm: String,

        /// Key comment
        #[arg(short = 'C', long)]
        comment: Option<String>,

        /// Overwrite existing key file
        #[arg(long)]
        force: bool,

        /// Do not set a passphrase (for non-interactive/CI use)
        #[arg(long)]
        no_passphrase: bool,
    },
    /// Add a key to ssh-agent
    Add {
        /// Path to the private key file
        #[arg(short = 'k', long = "key")]
        key: Option<PathBuf>,

        /// Key lifetime in seconds
        #[arg(short = 't', long)]
        lifetime: Option<u64>,
    },
    /// Remove keys from ssh-agent
    Remove {
        /// Remove key by fingerprint
        #[arg(short, long)]
        fingerprint: Option<String>,

        /// Remove all keys from agent
        #[arg(long)]
        all: bool,
    },
    /// Check SSH configuration
    Check,
    /// Interactive SSH setup
    Setup,
}
