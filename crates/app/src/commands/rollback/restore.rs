//! 恢复命令
//!
//! 从备份恢复已安装的二进制。

use std::path::PathBuf;

use prompt::{br, info, warning};
use toolkit::rollback;

use crate::registry::get_path_service;

/// Restore 命令
pub struct RestoreCommand {
    path: String,
}

impl RestoreCommand {
    /// 创建新的 RestoreCommand
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// 运行 Restore 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Restore command");
        br!();

        let path_service = get_path_service();
        let install_dir = match path_service.get_binary_install_dir() {
            Ok(d) => d,
            Err(e) => {
                warning!("Failed to get install dir: {}", e);
                return Err("Failed to get install dir".into());
            }
        };

        let bin_name = match path_service.get_binary_name() {
            Ok(n) => n,
            Err(e) => {
                warning!("Failed to get binary name: {}", e);
                return Err("Failed to get binary name".into());
            }
        };

        let backup_dir = PathBuf::from(self.path.clone());

        match rollback(bin_name.as_str(), backup_dir, install_dir) {
            Ok(()) => {
                info!("Restore completed");
            }
            Err(e) => {
                warning!("Failed to restore: {}", e);
                return Err("Failed to restore".into());
            }
        }
        Ok(())
    }
}
