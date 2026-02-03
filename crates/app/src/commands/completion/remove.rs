//! Completion 移除命令
//!
//! 移除 Shell Completion 配置和脚本文件。

use color_eyre::{eyre::WrapErr, Result};

use crate::registry::get_completion_service;

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
    pub fn run(&self) -> Result<()> {
        if self.remove_all {
            println!("移除所有 shell 的 completion 配置...\n");
        } else {
            println!("移除当前 shell 的 completion 配置...\n");
        }

        // 调用 Service 移除配置
        let service = get_completion_service();
        let result = service.remove(self.remove_all).wrap_err("移除 completion 配置失败")?;

        // 显示移除的配置
        for shell in &result.removed_configs {
            println!("  ✓ 已移除 {} 的 completion 配置", shell);
        }

        // 显示删除的脚本文件
        for file in &result.removed_files {
            println!("  ✓ 已删除 completion 脚本: {}", file.display());
        }

        // 显示删除的配置文件
        if let Some(ref config_file) = result.removed_config_file {
            println!("  ✓ 已删除 completion 配置文件: {}", config_file.display());
        }

        // 显示失败的操作
        for (target, error) in &result.failures {
            println!("  ⚠️  操作失败 {}: {}", target, error);
        }

        // 如果没有任何移除操作
        if result.removed_configs.is_empty()
            && result.removed_files.is_empty()
            && result.removed_config_file.is_none()
        {
            println!("  - 未找到需要移除的 completion 配置");
        }

        println!("\n✅ Completion 配置已移除！");

        Ok(())
    }
}
