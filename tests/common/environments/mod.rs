//! 统一测试环境模块
//!
//! 提供基于 TestIsolation 的统一测试环境，包括 CLI、Git 和 Repo 测试环境。

pub mod cli_test_env;
pub mod git_test_env;

// 重新导出常用类型
pub use cli_test_env::CliTestEnv;
pub use git_test_env::GitTestEnv;

