//! 基础设施层（Infrastructure）
//!
//! 提供跨领域的基础设施能力，作为底层技术支撑。

pub mod bootstrap;
pub mod http;
pub mod llm;

pub use bootstrap::register_client;
