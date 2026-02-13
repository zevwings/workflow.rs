//! 基础设施层（Infrastructure）
//!
//! 提供跨领域的基础设施能力，作为底层技术支撑。

pub mod bootstrap;
pub mod github;
pub mod http;
pub mod jira;
pub mod llm;

pub use bootstrap::register_client;
