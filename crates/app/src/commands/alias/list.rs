//! 别名列表命令
//!
//! 列出所有已定义的别名。

use color_eyre::{eyre::WrapErr, Result};
use prompt::{br, info, print};

use crate::registry::get_alias_service;

/// 别名列表命令
pub struct AliasListCommand;

impl AliasListCommand {
    /// 创建新的 AliasListCommand 实例
    pub fn new() -> Self {
        Self
    }

    /// 运行列表命令
    pub fn run(&self) -> Result<()> {
        let service = get_alias_service();
        let result = service.list().wrap_err("获取别名列表失败")?;

        if result.count == 0 {
            print!("当前没有定义任何别名。");
            br!();
            info!("使用 'workflow alias add <name> <command>' 添加别名");
            return Ok(());
        }

        print!("已定义的别名 ({} 个):", result.count);
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
        info!("使用 'workflow <alias>' 执行别名命令");

        Ok(())
    }
}

impl Default for AliasListCommand {
    fn default() -> Self {
        Self::new()
    }
}
