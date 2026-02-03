//! 别名移除命令
//!
//! 移除已定义的别名。

use color_eyre::{eyre::WrapErr, Result};
use prompt::{br, info, success, SelectBuilder};

use crate::registry::get_alias_service;

/// 别名移除命令
pub struct AliasRemoveCommand {
    /// 要移除的别名名称（可选，为空时进入交互模式）
    name: Option<String>,
}

impl AliasRemoveCommand {
    /// 创建新的 AliasRemoveCommand 实例
    pub fn new(name: Option<String>) -> Self {
        Self { name }
    }

    /// 运行移除命令
    pub fn run(&self) -> Result<()> {
        let service = get_alias_service();

        // 获取别名名称
        let name = if let Some(ref n) = self.name {
            n.clone()
        } else {
            // 交互模式：显示别名列表让用户选择
            self.interactive_select(&service)?
        };

        // 移除别名
        let result = service.remove(&name).wrap_err("移除别名失败")?;

        // 显示结果
        br!();
        success!("别名 '{}' 已移除", result.name);
        info!("  {} -> {}", result.name, result.command);

        Ok(())
    }

    /// 交互式选择要移除的别名
    fn interactive_select(
        &self,
        service: &std::sync::Arc<dyn domain::alias::AliasService>,
    ) -> Result<String> {
        // 获取别名列表
        let list_result = service.list().wrap_err("获取别名列表失败")?;

        if list_result.count == 0 {
            color_eyre::eyre::bail!("当前没有定义任何别名");
        }

        info!("选择要移除的别名（交互模式）");
        br!();

        // 构建选项列表
        let mut aliases = list_result.aliases;
        aliases.sort_by(|a, b| a.name.cmp(&b.name));

        // 获取别名名称列表
        let names: Vec<String> = aliases.iter().map(|a| a.name.clone()).collect();

        // 构建显示选项（名称 -> 命令）
        let display_options: Vec<String> =
            aliases.iter().map(|a| format!("{} -> {}", a.name, a.command)).collect();

        // 显示选择器，使用索引方式
        let selected_display = SelectBuilder::new("请选择要移除的别名", display_options)
            .prompt()
            .wrap_err("选择别名失败")?;

        // 从选中的显示字符串中提取别名名称
        let selected_name = selected_display
            .split(" -> ")
            .next()
            .map(|s| s.to_string())
            .unwrap_or_else(|| names[0].clone());

        Ok(selected_name)
    }
}
