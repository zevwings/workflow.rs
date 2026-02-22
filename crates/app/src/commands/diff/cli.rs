use clap::{Args, Subcommand};

/// Compare trees, commits, branches, files and more
#[derive(Subcommand)]
pub enum DiffCommand {
    /// Show changes between commits, commit and working tree, etc
    Commit(CommitDiffArgs),
    /// Show changes between two branches
    Merge(MergeArgs),
    /// Show changes to files in a commit
    Files(CommitFilesArgs),
}

#[derive(Args)]
pub struct MergeArgs {
    pub source_branch: String,
    pub target_branch: String,
}

#[derive(Args)]
pub struct CommitDiffArgs {
    pub ref_or_sha: String,
}

#[derive(Args)]
pub struct CommitFilesArgs {
    pub ref_or_sha: String,
}
