#[allow(clippy::module_inception)]
pub mod rollback;

pub use rollback::{BackupInfo, BackupResult, RollbackManager, RollbackResult};
