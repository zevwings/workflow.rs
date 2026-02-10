//! Rollback management subcommands
//!
//! 回滚管理子命令（仅 develop feature 下可用）

use clap::Subcommand;

/// 回滚管理子命令
#[derive(Subcommand)]
pub enum RollbackCommand {
    /// 备份当前安装的二进制
    Backup,
    /// 从备份恢复
    Restore {
        #[arg(short, long)]
        backup_dir: String,
    },
}
