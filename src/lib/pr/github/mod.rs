pub mod errors;
pub mod platform;
pub mod requests;
pub mod responses;

pub use errors::{format_error, GitHubError, GitHubErrorResponse};
pub use platform::GitHub;
pub use responses::GitHubUser;
