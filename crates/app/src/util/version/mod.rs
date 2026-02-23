mod compare;
mod current;
mod target;

pub use compare::{compare_versions, VersionComparison};
pub use current::get_current_version;
pub use target::get_target_version;
