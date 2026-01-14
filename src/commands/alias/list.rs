//! 别名列表命令
//!
//! 显示所有已定义的别名，使用表格格式。

use crate::alias::AliasManager;
use crate::interactive::{TableBuilder, TableStyle, Tabled};
use crate::{br, info, success};
use color_eyre::Result;

/// 别名表格行
#[derive(Clone)]
struct AliasRow {
    alias_name: String,
    command: String,
}

impl Tabled for AliasRow {
    fn headers() -> Vec<String> {
        vec!["Alias Name".to_string(), "Command".to_string()]
    }

    fn row(&self) -> Vec<String> {
        vec![self.alias_name.clone(), self.command.clone()]
    }
}

/// 别名列表命令
pub struct AliasListCommand;

impl AliasListCommand {
    /// 列出所有别名
    ///
    /// 使用表格格式显示所有已定义的别名。
    pub fn list() -> Result<()> {
        br!();
        info!("Alias List");

        let aliases = AliasManager::list()?;

        if aliases.is_empty() {
            info!("No aliases defined");
            info!("Run 'workflow alias add' to add an alias.");
            return Ok(());
        }

        // 构建表格数据
        let rows: Vec<AliasRow> = aliases
            .iter()
            .map(|(alias_name, command)| AliasRow {
                alias_name: alias_name.clone(),
                command: command.clone(),
            })
            .collect();

        // 显示表格
        let table = TableBuilder::from_tabled(rows)
            .with_title("Defined Aliases")
            .with_style(TableStyle::Modern)
            .render();

        info!("{}", table);
        success!("Found {} alias/aliases", aliases.len());

        Ok(())
    }
}
