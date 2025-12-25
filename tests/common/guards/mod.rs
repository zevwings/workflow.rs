//! 测试隔离守卫模块
//!
//! 提供用于测试隔离的各种守卫类型，包括环境变量守卫和Git配置守卫。

pub mod env_guard;
pub mod git_config_guard;

// 重新导出常用类型
pub use env_guard::EnvGuard;
pub use git_config_guard::GitConfigGuard;

