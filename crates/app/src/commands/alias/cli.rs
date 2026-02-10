//! 别名管理子命令
//!
//! 定义别名管理相关的子命令。

use clap::Subcommand;

/// 别名管理子命令
#[derive(Subcommand, Debug, Clone)]
pub enum AliasCommand {
    /// 列出所有已定义的别名
    ///
    /// 显示当前配置中所有的命令别名。
    List,

    /// 添加新别名
    ///
    /// 为常用命令创建简短的别名。
    /// 如果不提供参数，将进入交互模式。
    Add {
        /// 别名名称
        /// 例如: ci
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// 对应的命令
        /// 例如: "pr create"
        #[arg(value_name = "COMMAND")]
        command: Option<String>,

        /// 强制覆盖已存在的别名
        #[arg(short, long)]
        force: bool,
    },

    /// 移除别名
    ///
    /// 删除指定的命令别名。
    /// 如果不提供参数，将进入交互模式选择要删除的别名。
    Remove {
        /// 要移除的别名名称
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },
}
