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

use anyhow::Result;
use clap::Parser;
use workflow::commands::install::InstallCommand;

/// CLI 主结构体
#[derive(Parser)]
#[command(name = "install")]
#[command(about = "Install Workflow CLI components", long_about = None)]
#[command(version)]
struct Cli {
    /// 只安装二进制文件到 /usr/local/bin
    ///
    /// 将当前目录下的 workflow、pr、qk 二进制文件安装到 /usr/local/bin。
    /// 如果不指定此参数，默认安装全部（二进制文件 + completions）。
    #[arg(long)]
    binaries: bool,

    /// 只安装 shell completion 脚本
    ///
    /// 自动检测 shell 类型（zsh/bash）并安装相应的 completion 脚本。
    /// 如果不指定此参数，默认安装全部（二进制文件 + completions）。
    #[arg(long)]
    completions: bool,
}

/// 主函数
///
/// 解析命令行参数并执行相应的操作。
/// 默认行为（无参数）：安装全部（二进制文件 + completions）
fn main() -> Result<()> {
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
            println!(); // 添加空行分隔
        }
        InstallCommand::install_completions()?;
    }

    Ok(())
}
