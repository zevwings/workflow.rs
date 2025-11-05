//! 安装命令入口
//!
//! 这是独立的 `install` 命令入口，用于将 Workflow CLI 安装到系统：
//! - 编译并安装所有二进制文件到 `/usr/local/bin`
//! - 生成并安装 Shell 补全脚本
//! - 设置必要的权限

use anyhow::Result;
use workflow::commands::install::InstallCommand;

/// 主函数
///
/// 执行安装操作。
fn main() -> Result<()> {
    InstallCommand::install()?;
    Ok(())
}
