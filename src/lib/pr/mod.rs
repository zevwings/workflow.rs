pub mod body_parser;
pub mod codeup;
pub mod github;
pub mod helpers;
pub mod llm;
pub mod platform;
pub mod table;

pub use body_parser::{
    extract_description_from_body, extract_info_from_source_pr, extract_jira_ticket_from_body,
    parse_change_types_from_body, ExtractedPrInfo, SourcePrInfo,
};
pub use codeup::errors::CodeupErrorResponse;
pub use codeup::{Codeup, CodeupUser};
pub use github::errors::{GitHubError, GitHubErrorResponse};
pub use github::{GitHub, GitHubUser};
pub use helpers::{
    detect_repo_type, extract_pull_request_id_from_url, generate_branch_name,
    generate_commit_title, generate_pull_request_body, get_current_branch_pr_id,
    transform_to_branch_name,
};
pub use llm::{PullRequestContent, PullRequestLLM, PullRequestSummary};
pub use platform::{create_provider, PlatformProvider, PullRequestStatus, TYPES_OF_CHANGES};
pub use table::PullRequestRow;
