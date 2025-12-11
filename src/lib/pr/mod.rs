pub mod body_parser;
// pub mod codeup;  // Codeup support has been removed
pub mod github;
pub mod helpers;
pub mod llm;
pub mod platform;
pub mod table;

pub use body_parser::{
    extract_description_from_body, extract_info_from_source_pr, extract_jira_ticket_from_body,
    parse_change_types_from_body, ExtractedPrInfo, SourcePrInfo,
};
// pub use codeup::errors::CodeupErrorResponse;  // Codeup support has been removed
// pub use codeup::{Codeup, CodeupUser};  // Codeup support has been removed
pub use github::errors::{GitHubError, GitHubErrorResponse};
pub use github::{GitHub, GitHubUser};
pub use helpers::{
    detect_repo_type, extract_pull_request_id_from_url, generate_commit_title,
    generate_pull_request_body, get_current_branch_pr_id,
};
pub use llm::{PullRequestContent, PullRequestLLM, PullRequestSummary};
pub use platform::{
    create_provider, get_all_change_types, get_change_type_by_index, get_change_type_by_name,
    map_branch_type_to_change_type_index, map_branch_type_to_change_types, ChangeType,
    PlatformProvider, PullRequestStatus, CHANGE_TYPES, TYPES_OF_CHANGES,
};
pub use table::PullRequestRow;
