//! Completion 管理子命令
//!
//! 定义 Shell Completion 管理相关的子命令。

use clap::Subcommand;

/// Shell Completion 管理子命令
#[derive(Subcommand, Debug, Clone)]
pub enum CompletionCommand {
    /// 生成 completion 脚本并配置 shell
    ///
    /// 自动检测当前 shell 类型，生成对应的 completion 脚本，
    /// 并将其配置到 shell 配置文件中。
    Generate {
        /// 指定 shell 类型（zsh, bash, fish, powershell, elvish）
        /// 如果不指定，将自动检测当前 shell
        #[arg(short, long)]
        shell: Option<String>,

        /// 指定输出目录
        /// 如果不指定，将使用默认目录 ~/.workflow/completions/
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 检查 completion 配置状态
    ///
    /// 显示各个 shell 的 completion 配置状态。
    Check,

    /// 移除 completion 配置
    ///
    /// 从 shell 配置文件中移除 completion 配置，
    /// 并删除生成的 completion 脚本文件。
    Remove {
        /// 移除所有 shell 的配置
        #[arg(long)]
        all: bool,
    },
}
