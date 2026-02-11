//! Install 命令入口
//!
//! 独立的安装程序，用于将 workflow 工具安装到系统路径。
//!
//! ## 功能
//!
//! - 安装二进制文件到系统目录（通常是 /usr/local/bin）
//! - 安装 shell completion 脚本
//! - 支持 Unix 和 Windows 平台
//!
//! ## 使用方式
//!
//! ```bash
//! # 安装全部（二进制文件 + completion）
//! ./install
//!
//! # 仅安装二进制文件
//! ./install --binaries-only
//!
//! # 仅安装 completion 脚本
//! ./install --completions-only
//! ```

use app::commands::install::InstallCommand;
use clap::Parser;

/// Workflow CLI 安装程序
#[derive(Parser, Debug)]
#[command(name = "install")]
#[command(about = "安装 Workflow CLI 到系统路径")]
#[command(version)]
struct Args {
    /// 仅安装二进制文件
    #[arg(long, conflicts_with = "completions_only")]
    binaries_only: bool,

    /// 仅安装 shell completion 脚本
    #[arg(long, conflicts_with = "binaries_only")]
    completions_only: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let cmd = InstallCommand::new(args.binaries_only, args.completions_only);
    cmd.run()
}
