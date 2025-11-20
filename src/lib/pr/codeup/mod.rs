pub mod errors;
pub mod platform;
pub mod requests;
pub mod responses;

pub use errors::{format_error, CodeupErrorResponse};
pub use platform::Codeup;
pub use responses::CodeupUser;
