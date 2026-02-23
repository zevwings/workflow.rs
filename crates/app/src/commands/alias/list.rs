//! 别名列表命令
//!
//! 列出所有已定义的别名。

use prompt::{br, info, print};

use crate::bootstrap::get_alias_service;

/// 别名列表命令
pub struct AliasListCommand;

impl AliasListCommand {
    /// 创建新的 AliasListCommand 实例
    pub fn new() -> Self {
        Self
    }

    /// 运行列表命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let service = get_alias_service();
        let result = service.list().map_err(|e| format!("Failed to get alias list: {}", e))?;

        if result.count == 0 {
            print!("No aliases defined.");
            br!();
            info!("Use 'workflow alias add <name> <command>' to add an alias");
            return Ok(());
        }

        print!("Defined aliases ({}):", result.count);
        br!();
        println!("{:-<50}", "");

        // 按名称排序显示
        let mut aliases = result.aliases;
        aliases.sort_by(|a, b| a.name.cmp(&b.name));

        for alias in &aliases {
            println!("  {:<15} -> {}", alias.name, alias.command);
        }

        println!("{:-<50}", "");
        br!();
        info!("Use 'workflow <alias>' to run an alias command");

        Ok(())
    }
}

impl Default for AliasListCommand {
    fn default() -> Self {
        Self::new()
    }
}
