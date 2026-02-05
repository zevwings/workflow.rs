//! Hook 服务模块
//!
//! 提供 Git hooks 的发现、执行和管理功能。
//! 支持标准 Git hooks 和第三方工具（prek/pre-commit）的兼容性。

mod constants;
mod context;
mod detector;
mod discoverer;
mod script_executor;
mod service;
mod tool_executor;

pub use constants::{git_hooks, pre_commit_hooks};
pub use context::{HookContext, HookResult};
pub use detector::{HookTool, HookToolDetector};
pub use discoverer::HookDiscoverer;
pub use script_executor::ScriptHookExecutor;
pub use service::{HookService, HookServiceImpl};
pub use tool_executor::ToolHookExecutor;
