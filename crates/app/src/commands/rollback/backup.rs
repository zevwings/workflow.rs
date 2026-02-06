//! 备份命令
//!
//! 备份当前安装的二进制，用于更新前或回滚场景。

use color_eyre::Result;
use prompt::{br, info, warning};
use toolkit::backup;

use crate::registry::get_path_service;

/// Backup 命令
pub struct BackupCommand;

impl Default for BackupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupCommand {
    /// 创建新的 BackupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行备份
    pub fn run(&self) -> Result<()> {
        info!("Starting backup");
        br!();

        let path_service = get_path_service();
        let install_dir = match path_service.get_binary_install_dir() {
            Ok(d) => d,
            Err(e) => {
                warning!("Failed to get install dir: {}", e);
                return Err(color_eyre::eyre::eyre!("Failed to get install dir"));
            }
        };

        let bin_name = match path_service.get_binary_name() {
            Ok(n) => n,
            Err(e) => {
                warning!("Failed to get binary name: {}", e);
                return Err(color_eyre::eyre::eyre!("Failed to get binary name"));
            }
        };

        match backup(bin_name.as_str(), install_dir) {
            Ok(backup_dir) => {
                info!("Backup created: {}", backup_dir.display());
            }
            Err(e) => {
                warning!("Failed to create backup: {}", e);
                return Err(color_eyre::eyre::eyre!("Failed to create backup"));
            }
        }
        Ok(())
    }
}
