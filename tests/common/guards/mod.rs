//! 测试隔离守卫模块
//!
//! 提供用于测试隔离的各种守卫类型，包括环境变量守卫、Git配置守卫和对话框测试守卫。

pub mod dialog_test_guard;
pub mod env_guard;
pub mod git_config_guard;

// 重新导出常用类型
pub use dialog_test_guard::DialogTestGuard;
pub use env_guard::EnvGuard;
pub use git_config_guard::GitConfigGuard;
