//! Completion 移除命令
//!
//! 移除 Shell Completion 配置和脚本文件。

use crate::bootstrap::get_completion_service;

/// Completion 移除命令
pub struct CompletionRemoveCommand {
    remove_all: bool,
}

impl CompletionRemoveCommand {
    /// 创建新的 CompletionRemoveCommand 实例
    pub fn new(remove_all: bool) -> Self {
        Self { remove_all }
    }

    /// 运行移除命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.remove_all {
            println!("Removing completion config for all shells...\n");
        } else {
            println!("Removing completion config for current shell...\n");
        }

        // 调用 Service 移除配置
        let service = get_completion_service();
        let result = service
            .remove(self.remove_all)
            .map_err(|e| format!("Failed to remove completion config: {}", e))?;

        // 显示移除的配置
        for shell in &result.removed_configs {
            println!("  ✓ Removed {} completion config", shell);
        }

        // 显示删除的脚本文件
        for file in &result.removed_files {
            println!("  ✓ Deleted completion script: {}", file.display());
        }

        // 显示删除的配置文件
        if let Some(ref config_file) = result.removed_config_file {
            println!(
                "  ✓ Deleted completion config file: {}",
                config_file.display()
            );
        }

        // 显示失败的操作
        for (target, error) in &result.failures {
            println!("  ⚠️  Failed {}: {}", target, error);
        }

        // 如果没有任何移除操作
        if result.removed_configs.is_empty()
            && result.removed_files.is_empty()
            && result.removed_config_file.is_none()
        {
            println!("  - No completion config found to remove");
        }

        println!("\n✅ Completion config removed!");

        Ok(())
    }
}
