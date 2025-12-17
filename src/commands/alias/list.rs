//! 别名列表命令
//!
//! 显示所有已定义的别名，使用表格格式。

use crate::base::alias::AliasManager;
use crate::base::table::{TableBuilder, TableStyle};
use crate::{log_break, log_info, log_message, log_success};
use color_eyre::Result;
use tabled::Tabled;

/// 别名表格行
#[derive(Tabled, Clone)]
struct AliasRow {
    #[tabled(rename = "Alias Name")]
    alias_name: String,
    #[tabled(rename = "Command")]
    command: String,
}

/// 别名列表命令
pub struct AliasListCommand;

impl AliasListCommand {
    /// 列出所有别名
    ///
    /// 使用表格格式显示所有已定义的别名。
    pub fn list() -> Result<()> {
        log_break!();
        log_message!("Alias List");

        let aliases = AliasManager::list()?;

        if aliases.is_empty() {
            log_info!("No aliases defined");
            log_message!("Run 'workflow alias add' to add an alias.");
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
        let table = TableBuilder::new(rows)
            .with_title("Defined Aliases")
            .with_style(TableStyle::Modern)
            .render();

        log_message!("{}", table);
        log_success!("Found {} alias/aliases", aliases.len());

        Ok(())
    }
}
