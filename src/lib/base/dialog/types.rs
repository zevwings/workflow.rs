use anyhow::Result;
use std::sync::Arc;

/// Type alias for validator functions to reduce type complexity
pub(crate) type ValidatorFn = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;
