//! 版本命令
//! 显示 Workflow CLI 的版本信息

use crate::log_success;
use anyhow::Result;

/// 版本命令
pub struct VersionCommand;

impl VersionCommand {
    /// 显示当前版本信息
    ///
    /// 从编译时嵌入的版本号获取（使用 env! 宏）。
    /// 注意：env! 宏在编译时展开，所以这个值在运行时总是可用的。
    pub fn show() -> Result<()> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        log_success!("workflow v{}", VERSION);
        Ok(())
    }
}
