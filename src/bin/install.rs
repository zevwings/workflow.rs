//! 安装命令入口
//!
//! 这是独立的 `install` 命令入口，用于安装 Workflow CLI 组件：
//! - 默认行为：安装全部（二进制文件 + shell completions）
//! - `--binaries`: 只安装二进制文件到 /usr/local/bin
//! - `--completions`: 只安装 shell completion 脚本
//!
//! 使用方式：
//! - `./install` - 安装全部
//! - `./install --binaries` - 只安装二进制文件
//! - `./install --completions` - 只安装补全脚本

use clap::Parser;
use color_eyre::Result;
use workflow::commands::lifecycle::install::InstallCommand;
use workflow::log_break;

/// CLI main structure
#[derive(Parser)]
#[command(name = "install")]
#[command(about = "Install Workflow CLI components", long_about = None)]
#[command(version)]
struct Cli {
    /// Only install binaries to /usr/local/bin
    ///
    /// Install workflow binary from current directory to /usr/local/bin.
    /// If not specified, installs everything by default (binaries + completions).
    #[arg(long)]
    binaries: bool,

    /// Only install shell completion scripts
    ///
    /// Auto-detect shell type (zsh/bash) and install corresponding completion scripts.
    /// If not specified, installs everything by default (binaries + completions).
    #[arg(long)]
    completions: bool,
}

/// 主函数
///
/// 解析命令行参数并执行相应的操作。
/// 默认行为（无参数）：安装全部（二进制文件 + completions）
fn main() -> Result<()> {
    // 安装 color-eyre（最早调用）
    color_eyre::install()?;
    let cli = Cli::parse();

    // 确定要安装的内容
    // 如果只指定了 --binaries，只安装二进制文件
    // 如果只指定了 --completions，只安装补全脚本
    // 如果都没有指定或两个都指定，安装全部
    let install_binaries = !cli.completions || cli.binaries;
    let install_completions = !cli.binaries || cli.completions;

    if install_binaries {
        InstallCommand::install_binaries()?;
    }

    if install_completions {
        if install_binaries {
            log_break!(); // 添加空行分隔
        }
        InstallCommand::install_completions()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // install 二进制文件需要管理员权限，不适合在测试环境中运行
    // 添加一个被忽略的测试，防止 Cargo 尝试运行二进制文件本身作为测试
    #[test]
    #[ignore]
    fn test_install_binary_requires_admin() {
        // 此测试被忽略，因为 install 二进制文件需要管理员权限
        // 实际的安装功能测试应该在集成测试中进行
    }
}