pub mod api;
pub mod errors;
pub mod requests;
pub mod responses;

pub use api::GitHub;
pub use errors::{format_error, GitHubError, GitHubErrorResponse};
pub use responses::GitHubUser;

#[cfg(test)]
mod tests {
    use crate::pr::helpers::extract_pull_request_id_from_url;

    #[test]
    fn test_extract_pull_request_id_from_url() {
        assert_eq!(
            extract_pull_request_id_from_url("https://github.com/owner/repo/pull/123").unwrap(),
            "123"
        );
        assert_eq!(
            extract_pull_request_id_from_url("https://github.com/owner/repo/pull/456/").unwrap(),
            "456"
        );
    }
}
