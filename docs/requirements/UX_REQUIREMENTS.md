# 用户体验优化需求文档

## 📋 概述

本文档详细描述用户体验优化相关的功能需求，包括交互式界面、快捷命令和错误处理与恢复功能。

**文档状态**: 待实现
**创建日期**: 2025-12-09
**优先级**: 高/中/低（见各功能说明）

---

## 🎯 需求列表

### 1. 交互式选择功能

#### 1.1 Fuzzy Finder 选择器

**优先级**: 中
**状态**: ❌ 未实现

**需求描述**：
- 使用 fuzzy finder 选择 tickets/PRs
- 支持多选功能
- 提供交互式选择体验

**功能要求**：
1. **Fuzzy Finder 集成**
   - 使用 `skim` 或 `fzf` 集成
   - 支持模糊搜索
   - 支持键盘导航

2. **多选支持**
   - 支持选择多个项目
   - 显示已选择项目数量
   - 支持取消选择

3. **使用场景**
   - 选择要操作的 PR
   - 选择要查看的 JIRA ticket
   - 选择要合并的分支

**命令接口**：
```bash
workflow pr merge --interactive                     # 交互式选择 PR
workflow jira info --interactive                    # 交互式选择 ticket
```

**实现示例**：

### 方案 1：使用 `inquire` 的模糊搜索（推荐，已集成）✅

**重要说明**：`inquire` 库已经内置了模糊搜索功能，默认启用 `fuzzy` 和 `fuzzy-matcher` 特性。**无需添加额外的依赖**，可以直接使用现有的 `SelectDialog` 和 `MultiSelectDialog`。

#### 1. 使用现有的对话框（已支持模糊搜索）

```rust
// src/commands/pr/merge.rs
use crate::base::dialog::{SelectDialog, MultiSelectDialog};
use crate::github::PullRequest;
use anyhow::Result;

pub fn merge_interactive() -> Result<()> {
    // 获取 PR 列表
    let prs = fetch_pull_requests()?;

    // 格式化选项（显示 PR 标题和编号）
    let options: Vec<String> = prs
        .iter()
        .map(|pr| format!("#{} - {}", pr.number, pr.title))
        .collect();

    // 使用 SelectDialog（已支持模糊搜索）
    // 用户可以直接输入关键词进行模糊匹配
    let selected = SelectDialog::new("Select PR to merge (type to search)", options)
        .prompt()?;

    // 解析选中的 PR 编号
    let pr_number = extract_pr_number(&selected)?;

    // 执行合并
    merge_pr(pr_number)?;

    Ok(())
}
```

```rust
// src/commands/jira/info.rs
use crate::base::dialog::MultiSelectDialog;
use crate::jira::JiraTicket;
use anyhow::Result;

pub fn info_interactive() -> Result<()> {
    // 获取 JIRA tickets 列表
    let tickets = search_jira_tickets()?;

    // 格式化选项
    let options: Vec<String> = tickets
        .iter()
        .map(|t| format!("{} - {} ({})", t.key, t.summary, t.status))
        .collect();

    // 使用 MultiSelectDialog（已支持模糊搜索和多选）
    // 用户可以直接输入关键词进行模糊匹配，支持多选
    let selected = MultiSelectDialog::new("Select tickets to view (type to search)", options)
        .prompt()?;

    // 显示选中的 tickets 信息
    for item in selected {
        let ticket_key = extract_ticket_key(&item)?;
        show_ticket_info(&ticket_key)?;
    }

    Ok(())
}
```

#### 2. 直接使用 `inquire`（如果需要更多自定义）

```rust
use inquire::{Select, MultiSelect};

// 单选 + 模糊搜索（默认启用）
let options = vec!["Option 1", "Option 2", "Option 3"];
let selected = Select::new("Choose (type to search):", options)
    .with_page_size(10)  // 设置每页显示数量
    .prompt()?;

// 多选 + 模糊搜索（默认启用）
let selected = MultiSelect::new("Choose (type to search):", options)
    .with_page_size(10)
    .prompt()?;
```

**优势**：
- ✅ 无需添加新依赖（`inquire` 已集成）
- ✅ 模糊搜索默认启用
- ✅ 与现有代码风格一致
- ✅ 支持键盘导航和搜索
- ✅ 支持多选功能

**使用方式**：
- 在 `SelectDialog` 或 `MultiSelectDialog` 中，用户可以直接输入文字进行模糊搜索
- 支持部分匹配、模糊匹配
- 使用方向键导航，Enter 确认

### 方案 2：使用 `skim` 库（可选，高级功能）

如果 `inquire` 的模糊搜索功能无法满足需求（例如需要更复杂的搜索算法、自定义界面等），可以考虑使用 `skim`。

**何时考虑使用 `skim`**：
- 需要更强大的模糊匹配算法
- 需要自定义界面布局
- 需要更复杂的搜索选项（如正则表达式搜索）
- 需要与 `fzf` 完全兼容的体验

**添加依赖**：
```toml
[dependencies]
skim = "0.11"
```

**实现示例**：见下方完整示例部分。

### 方案 3：集成外部 `fzf` 命令（可选）

如果系统已安装 `fzf`，可以通过命令调用：

```rust
use std::process::{Command, Stdio};

pub fn fuzzy_select_fzf(options: Vec<String>) -> Result<String> {
    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let input = options.join("\n");
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    let selected = String::from_utf8(output.stdout)?;
    Ok(selected.trim().to_string())
}
```

**推荐方案**：**优先使用 `inquire`**（方案 1），因为：
1. 无需添加新依赖
2. 已集成到项目中
3. 模糊搜索功能已足够强大
4. 与现有代码风格一致

### 完整使用示例：PR 交互式合并

```rust
// src/commands/pr/merge.rs
use crate::base::dialog::MultiSelectDialog;
use crate::github::{GitHubProvider, PullRequest};
use anyhow::Result;

pub fn merge_interactive() -> Result<()> {
    // 1. 获取当前仓库的 PR 列表
    let provider = GitHubProvider::new()?;
    let prs = provider.list_pull_requests("open")?;

    if prs.is_empty() {
        println!("No open pull requests found");
        return Ok(());
    }

    // 2. 格式化选项，显示关键信息
    let options: Vec<String> = prs
        .iter()
        .map(|pr| {
            format!(
                "#{} | {} | {} | {}",
                pr.number,
                pr.title.chars().take(50).collect::<String>(),
                pr.author,
                pr.state
            )
        })
        .collect();

    // 3. 使用 MultiSelectDialog（已支持模糊搜索）
    // 用户可以直接输入关键词进行模糊匹配，支持多选
    println!("Select PR(s) to merge (type to search, Space to select, Enter to confirm):");
    let selected = MultiSelectDialog::new("PRs:", options)
        .prompt()?;

    // 4. 解析选中的 PR 编号
    let pr_numbers: Vec<u64> = selected
        .iter()
        .filter_map(|s| {
            s.split('|')
                .next()?
                .trim()
                .strip_prefix('#')?
                .parse()
                .ok()
        })
        .collect();

    // 5. 确认并执行合并
    for pr_number in pr_numbers {
        println!("Merging PR #{}...", pr_number);
        provider.merge_pull_request(pr_number, "squash")?;
        println!("✓ PR #{} merged successfully", pr_number);
    }

    Ok(())
}
```

### 完整使用示例：JIRA Ticket 交互式查看

```rust
// src/commands/jira/info.rs
use crate::base::dialog::SelectDialog;
use crate::jira::{JiraClient, JiraTicket};
use anyhow::Result;

pub fn info_interactive() -> Result<()> {
    // 1. 搜索 JIRA tickets
    let client = JiraClient::new()?;
    let tickets = client.search("assignee = currentUser() AND status != Done")?;

    if tickets.is_empty() {
        println!("No tickets found");
        return Ok(());
    }

    // 2. 格式化选项
    let options: Vec<String> = tickets
        .iter()
        .map(|t| {
            format!(
                "{} | {} | {} | {}",
                t.key,
                t.summary.chars().take(40).collect::<String>(),
                t.status,
                t.assignee.as_deref().unwrap_or("Unassigned")
            )
        })
        .collect();

    // 3. 使用 SelectDialog（已支持模糊搜索）
    // 用户可以直接输入关键词进行模糊匹配
    let selected = SelectDialog::new("Select ticket (type to search):", options)
        .prompt()?;

    // 4. 提取 ticket key
    let ticket_key = selected
        .split('|')
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid selection"))?
        .trim()
        .to_string();

    // 5. 显示 ticket 详细信息
    let ticket = client.get_ticket(&ticket_key)?;
    display_ticket_info(&ticket)?;

    Ok(())
}
```

**验收标准**：
- [ ] 能够通过 fuzzy finder 搜索和选择 PRs
- [ ] 能够通过 fuzzy finder 搜索和选择 JIRA tickets
- [ ] 支持多选功能
- [ ] 交互体验流畅，响应迅速

---

### 2. 进度显示功能

#### 2.1 长时间操作进度条

**优先级**: 高
**状态**: ✅ 已实现

**需求描述**：
- 为长时间操作显示进度条
- 显示预计完成时间
- 提供操作状态反馈

**功能要求**：
1. **进度条显示**
   - 使用 `indicatif` 或类似库 ✅
   - 显示操作进度百分比 ✅
   - 显示已处理/总数 ✅
   - 显示预计剩余时间（ETA）✅

2. **适用操作**
   - 下载多个附件 ✅（已在 `jira attachments` 和 `log download` 命令中使用）
   - 批量操作
   - 同步多个 PR
   - 导出大量数据

**实现位置**：
- 核心实现：`src/lib/base/indicator/progress.rs`
- 使用示例：
  - `src/commands/jira/attachments.rs` - JIRA 附件下载
  - `src/commands/log/download.rs` - 日志文件下载

**实现特性**：
- ✅ `Progress::new()` - 已知总数的进度条，显示百分比和 ETA
- ✅ `Progress::new_download()` - 下载专用进度条，显示字节数和下载速度
- ✅ `Progress::new_unknown()` - 未知总数的 spinner 模式
- ✅ 支持进度更新、消息更新、完成处理

**验收标准**：
- [x] 长时间操作显示进度条
- [x] 进度条显示准确的进度百分比
- [x] 显示预计完成时间
- [x] 进度条样式美观，信息清晰

---

### 3. 快捷命令功能

#### 3.1 别名系统

**优先级**: 中
**状态**: ❌ 未实现

**需求描述**：
- 支持自定义命令别名
- 简化常用命令输入
- 提高命令输入效率

**功能要求**：
1. **别名配置**
   - 在配置文件中定义别名
   - 支持命令参数传递
   - 支持别名嵌套（别名引用别名）

2. **配置格式**：
```toml
[aliases]
ci = "pr create"
cm = "pr merge"
js = "jira search"
ji = "jira info"
```

3. **使用示例**：
```bash
workflow ci                                        # 等同于 workflow pr create
workflow cm                                        # 等同于 workflow pr merge
workflow js "project = PROJ"                       # 等同于 workflow jira search
```

**实现方案**：

### 1. 配置文件结构扩展

在 `workflow.toml` 中添加别名配置：

```toml
[aliases]
ci = "pr create"
cm = "pr merge"
js = "jira search"
ji = "jira info"
```

### 2. 实现 AliasManager

```rust
// src/lib/base/alias/mod.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::base::settings::paths::Paths;
use crate::jira::config::ConfigManager;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliasConfig {
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

pub struct AliasManager {
    config: AliasConfig,
    config_path: PathBuf,
}

impl AliasManager {
    /// 加载别名配置
    pub fn load() -> Result<Self> {
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<AliasConfig>::new(config_path.clone());
        let config = manager.read().unwrap_or_default();

        Ok(Self {
            config,
            config_path,
        })
    }

    /// 展开别名（支持嵌套）
    pub fn expand_alias(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(command.to_string());
        }

        let alias_name = parts[0];

        // 检查是否是别名
        if let Some(alias_value) = self.config.aliases.get(alias_name) {
            // 递归展开（防止无限循环）
            let mut expanded = alias_value.clone();
            let mut visited = std::collections::HashSet::new();
            visited.insert(alias_name.to_string());

            // 处理嵌套别名
            while let Some(next_alias) = self.find_alias_in_command(&expanded, &mut visited) {
                if let Some(next_value) = self.config.aliases.get(&next_alias) {
                    expanded = expanded.replace(&next_alias, next_value);
                } else {
                    break;
                }
            }

            // 添加剩余参数
            if parts.len() > 1 {
                let args = parts[1..].join(" ");
                expanded = format!("{} {}", expanded, args);
            }

            Ok(expanded)
        } else {
            Ok(command.to_string())
        }
    }

    /// 查找命令中的别名
    fn find_alias_in_command(
        &self,
        command: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> Option<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first) = parts.first() {
            if self.config.aliases.contains_key(*first) && !visited.contains(*first) {
                return Some(first.to_string());
            }
        }
        None
    }

    /// 添加别名
    pub fn add_alias(&mut self, name: &str, value: &str) -> Result<()> {
        self.config.aliases.insert(name.to_string(), value.to_string());
        self.save()
    }

    /// 删除别名
    pub fn remove_alias(&mut self, name: &str) -> Result<()> {
        self.config.aliases.remove(name);
        self.save()
    }

    /// 列出所有别名
    pub fn list_aliases(&self) -> &HashMap<String, String> {
        &self.config.aliases
    }

    /// 保存配置
    fn save(&self) -> Result<()> {
        let manager = ConfigManager::<AliasConfig>::new(self.config_path.clone());
        manager.write(&self.config)
    }
}
```

### 3. 在主入口集成别名展开

```rust
// src/bin/workflow.rs
use workflow::base::alias::AliasManager;

fn main() -> Result<()> {
    // ... 现有初始化代码 ...

    let mut cli = Cli::parse();

    // 展开别名（如果是第一个参数）
    if let Some(first_arg) = std::env::args().nth(1) {
        let alias_manager = AliasManager::load().unwrap_or_default();
        if let Ok(expanded) = alias_manager.expand_alias(&first_arg) {
            if expanded != first_arg {
                // 别名已展开，重新解析命令
                let expanded_args: Vec<String> = expanded
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                // 重新构建命令行参数
                let mut new_args = vec!["workflow".to_string()];
                new_args.extend(expanded_args);
                new_args.extend(std::env::args().skip(2));

                // 重新解析
                cli = Cli::parse_from(new_args);
            }
        }
    }

    // ... 继续处理命令 ...
}
```

### 4. 添加别名管理命令

```rust
// src/lib/cli/commands.rs
#[derive(Subcommand)]
pub enum Commands {
    // ... 现有命令 ...

    /// Manage command aliases
    Alias {
        #[command(subcommand)]
        subcommand: AliasSubcommand,
    },
}

#[derive(Subcommand)]
pub enum AliasSubcommand {
    /// List all aliases
    List,
    /// Add a new alias
    Add {
        name: String,
        value: String,
    },
    /// Remove an alias
    Remove {
        name: String,
    },
}
```

```rust
// src/commands/alias/mod.rs
use crate::base::alias::AliasManager;
use anyhow::Result;

pub fn list() -> Result<()> {
    let manager = AliasManager::load()?;
    let aliases = manager.list_aliases();

    if aliases.is_empty() {
        println!("No aliases defined");
        return Ok(());
    }

    println!("Defined aliases:");
    for (name, value) in aliases {
        println!("  {} = {}", name, value);
    }
    Ok(())
}

pub fn add(name: String, value: String) -> Result<()> {
    let mut manager = AliasManager::load()?;
    manager.add_alias(&name, &value)?;
    println!("Alias '{}' added: {}", name, value);
    Ok(())
}

pub fn remove(name: String) -> Result<()> {
    let mut manager = AliasManager::load()?;
    manager.remove_alias(&name)?;
    println!("Alias '{}' removed", name);
    Ok(())
}
```

**验收标准**：
- [ ] 能够在配置文件中定义别名
- [ ] 别名能够正确展开为完整命令
- [ ] 支持命令参数传递
- [ ] 支持别名嵌套

---

#### 3.2 命令历史

**优先级**: 中
**状态**: ❌ 未实现

**需求描述**：
- 记录常用命令历史
- 支持快速重放历史命令
- 提高重复操作效率

**功能要求**：
1. **历史记录**
   - 保存命令历史到文件
   - 记录命令参数
   - 记录执行时间

2. **历史文件位置**：
   - `~/.workflow/history` 或类似位置

3. **命令接口**：
```bash
workflow history                                   # 查看命令历史
workflow history --replay 5                        # 重放第 5 条命令
workflow history --search "pr create"              # 搜索历史命令
```

**实现方案**：

### 1. 历史记录数据结构

```rust
// src/lib/base/history/mod.rs
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::base::settings::paths::Paths;
use crate::jira::config::ConfigManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    pub entries: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub args: Vec<String>,
    pub timestamp: String,
    pub exit_code: Option<i32>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

pub struct HistoryManager {
    history_path: PathBuf,
    max_entries: usize,
}

impl HistoryManager {
    /// 创建历史管理器
    pub fn new() -> Result<Self> {
        let history_path = Paths::workflow_dir()?.join("history.toml");
        Ok(Self {
            history_path,
            max_entries: 1000, // 最多保存 1000 条记录
        })
    }

    /// 记录命令
    pub fn record_command(&self, command: &str, args: &[String], exit_code: Option<i32>) -> Result<()> {
        let mut history = self.load()?;

        let entry = HistoryEntry {
            command: command.to_string(),
            args: args.to_vec(),
            timestamp: Utc::now().to_rfc3339(),
            exit_code,
        };

        history.entries.push(entry);

        // 限制历史记录数量
        if history.entries.len() > self.max_entries {
            history.entries.drain(0..history.entries.len() - self.max_entries);
        }

        self.save(&history)
    }

    /// 加载历史记录
    pub fn load(&self) -> Result<CommandHistory> {
        if !self.history_path.exists() {
            return Ok(CommandHistory::default());
        }

        let manager = ConfigManager::<CommandHistory>::new(self.history_path.clone());
        manager.read().context("Failed to load command history")
    }

    /// 保存历史记录
    fn save(&self, history: &CommandHistory) -> Result<()> {
        let manager = ConfigManager::<CommandHistory>::new(self.history_path.clone());
        manager.write(history)
    }

    /// 列出历史记录
    pub fn list(&self, limit: Option<usize>) -> Result<Vec<HistoryEntry>> {
        let history = self.load()?;
        let entries = history.entries;

        let limit = limit.unwrap_or(entries.len());
        Ok(entries.into_iter().rev().take(limit).collect())
    }

    /// 搜索历史记录
    pub fn search(&self, query: &str) -> Result<Vec<HistoryEntry>> {
        let history = self.load()?;
        let query_lower = query.to_lowercase();

        let results: Vec<HistoryEntry> = history
            .entries
            .into_iter()
            .rev()
            .filter(|entry| {
                entry.command.to_lowercase().contains(&query_lower)
                    || entry.args.iter().any(|arg| arg.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    /// 获取指定索引的历史记录
    pub fn get(&self, index: usize) -> Result<HistoryEntry> {
        let history = self.load()?;
        let entries: Vec<_> = history.entries.into_iter().rev().collect();

        entries
            .get(index)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("History entry at index {} not found", index))
    }

    /// 清空历史记录
    pub fn clear(&self) -> Result<()> {
        let history = CommandHistory::default();
        self.save(&history)
    }
}
```

### 2. 在主入口记录命令

```rust
// src/bin/workflow.rs
use workflow::base::history::HistoryManager;

fn main() -> Result<()> {
    // ... 现有初始化代码 ...

    let cli = Cli::parse();
    let args: Vec<String> = std::env::args().skip(1).collect();

    // 执行命令
    let result = match &cli.command {
        // ... 现有命令处理 ...
    };

    // 记录命令历史
    let history_manager = HistoryManager::new()?;
    let exit_code = if result.is_ok() { Some(0) } else { Some(1) };
    let _ = history_manager.record_command("workflow", &args, exit_code);

    result
}
```

### 3. 添加历史命令

```rust
// src/lib/cli/commands.rs
#[derive(Subcommand)]
pub enum Commands {
    // ... 现有命令 ...

    /// View and manage command history
    History {
        /// Replay a command from history
        #[arg(long, short)]
        replay: Option<usize>,

        /// Search history
        #[arg(long, short)]
        search: Option<String>,

        /// Limit number of entries to show
        #[arg(long, short, default_value = "20")]
        limit: usize,

        /// Clear all history
        #[arg(long)]
        clear: bool,
    },
}
```

```rust
// src/commands/history/mod.rs
use crate::base::history::HistoryManager;
use anyhow::Result;
use std::process::Command;

pub fn handle_history(replay: Option<usize>, search: Option<String>, limit: usize, clear: bool) -> Result<()> {
    let manager = HistoryManager::new()?;

    if clear {
        manager.clear()?;
        println!("Command history cleared");
        return Ok(());
    }

    if let Some(index) = replay {
        // 重放命令
        let entry = manager.get(index)?;
        println!("Replaying: workflow {}", entry.args.join(" "));

        // 重新执行命令
        let mut cmd = Command::new("workflow");
        cmd.args(&entry.args);
        cmd.status()?;

        return Ok(());
    }

    if let Some(query) = search {
        // 搜索历史
        let results = manager.search(&query)?;
        if results.is_empty() {
            println!("No matching history entries found");
            return Ok(());
        }

        println!("Found {} matching entries:", results.len());
        for (i, entry) in results.iter().enumerate() {
            println!("  [{}] {} {}", i, entry.command, entry.args.join(" "));
            println!("      Time: {}", entry.timestamp);
        }
    } else {
        // 列出历史
        let entries = manager.list(Some(limit))?;
        if entries.is_empty() {
            println!("No command history");
            return Ok(());
        }

        println!("Command history (last {} entries):", limit);
        for (i, entry) in entries.iter().enumerate() {
            println!("  [{}] {} {}", i, entry.command, entry.args.join(" "));
            println!("      Time: {}", entry.timestamp);
        }
    }

    Ok(())
}
```

**验收标准**：

---

#### 3.3 智能补全

**优先级**: 低
**状态**: ❌ 未实现

**需求描述**：
- 增强 shell completion 功能
- 支持动态补全（从 API 获取数据）
- 提供基于上下文的补全建议

**功能要求**：
1. **补全类型**
   - 命令补全
   - 参数补全
   - 值补全（JIRA ticket keys、PR numbers、分支名等）

2. **动态补全**
   - 从 API 获取数据用于补全
   - 支持 JIRA ticket keys 补全
   - 支持 PR numbers 补全
   - 支持分支名补全

3. **Shell 支持**
   - bash completion
   - zsh completion
   - fish completion

4. **使用示例**：
```bash
# 自动补全 JIRA ticket keys
workflow jira info PROJ-<TAB>

# 自动补全 PR numbers
workflow pr merge <TAB>
```

**实现方案**：

### 1. 基础补全（已实现）

项目已使用 `clap_complete` 实现基础补全，支持：
- 命令补全
- 参数补全
- 子命令补全

### 2. 动态补全实现

需要实现自定义补全函数，从 API 获取数据：

```rust
// src/lib/completion/dynamic.rs
use anyhow::Result;
use std::process::Command;

/// 动态补全提供者
pub struct DynamicCompletion;

impl DynamicCompletion {
    /// 获取 JIRA ticket keys
    pub fn jira_ticket_keys(project: Option<&str>) -> Result<Vec<String>> {
        // 调用 JIRA API 获取 tickets
        // 这里需要根据实际的 JIRA 客户端实现
        let client = crate::jira::JiraClient::new()?;
        let jql = if let Some(proj) = project {
            format!("project = {}", proj)
        } else {
            "assignee = currentUser()".to_string()
        };

        let tickets = client.search(&jql)?;
        Ok(tickets.iter().map(|t| t.key.clone()).collect())
    }

    /// 获取 PR numbers
    pub fn pr_numbers() -> Result<Vec<String>> {
        let provider = crate::github::GitHubProvider::new()?;
        let prs = provider.list_pull_requests("open")?;
        Ok(prs.iter().map(|pr| pr.number.to_string()).collect())
    }

    /// 获取分支名
    pub fn branch_names() -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(&["branch", "--format=%(refname:short)"])
            .output()?;

        let branches: Vec<String> = String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(branches)
    }
}
```

### 3. 创建补全脚本包装器

```bash
# ~/.workflow/completions/_workflow_dynamic
# 动态补全脚本（zsh 示例）

_workflow_dynamic_jira_tickets() {
    local -a tickets
    tickets=($(workflow _completion jira-tickets))
    _describe 'tickets' tickets
}

_workflow_dynamic_pr_numbers() {
    local -a prs
    prs=($(workflow _completion pr-numbers))
    _describe 'PRs' prs
}

_workflow_dynamic_branches() {
    local -a branches
    branches=($(workflow _completion branches))
    _describe 'branches' branches
}

# 在现有的补全函数中集成
_workflow_jira_info() {
    _arguments \
        '1:ticket:_workflow_dynamic_jira_tickets' \
        # ... 其他参数
}
```

### 4. 添加内部补全命令

```rust
// src/lib/cli/commands.rs
#[derive(Subcommand)]
pub enum Commands {
    // ... 现有命令 ...

    /// Internal completion command (for dynamic completion)
    #[command(hide = true)]
    Completion {
        #[arg(name = "type")]
        completion_type: String,
        #[arg(name = "context")]
        context: Option<String>,
    },
}
```

```rust
// src/commands/completion/dynamic.rs
use crate::completion::dynamic::DynamicCompletion;
use anyhow::Result;

pub fn handle_completion(completion_type: &str, context: Option<&str>) -> Result<()> {
    match completion_type {
        "jira-tickets" => {
            let tickets = DynamicCompletion::jira_ticket_keys(context)?;
            for ticket in tickets {
                println!("{}", ticket);
            }
        }
        "pr-numbers" => {
            let prs = DynamicCompletion::pr_numbers()?;
            for pr in prs {
                println!("{}", pr);
            }
        }
        "branches" => {
            let branches = DynamicCompletion::branch_names()?;
            for branch in branches {
                println!("{}", branch);
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown completion type: {}", completion_type));
        }
    }
    Ok(())
}
```

### 5. 更新补全脚本生成

修改 `src/lib/completion/generate.rs`，在生成的补全脚本中添加动态补全调用：

```rust
// 在生成补全脚本时，添加动态补全函数
fn generate_dynamic_completion_functions(shell: &ClapShell) -> String {
    match shell {
        ClapShell::Zsh => r#"
_workflow_dynamic_jira_tickets() {
    local -a tickets
    tickets=($(workflow _completion jira-tickets 2>/dev/null))
    _describe 'tickets' tickets
}
"#.to_string(),
        ClapShell::Bash => r#"
_workflow_dynamic_jira_tickets() {
    local tickets
    tickets=($(workflow _completion jira-tickets 2>/dev/null))
    COMPREPLY=($(compgen -W "${tickets[*]}" -- "$cur"))
}
"#.to_string(),
        _ => String::new(),
    }
}
```

**注意事项**：
- 动态补全可能较慢（需要调用 API），考虑添加缓存
- 需要处理网络错误，避免补全失败影响用户体验
- 可以添加超时机制，避免长时间等待

**验收标准**：
- [ ] 支持 bash/zsh/fish 补全
- [ ] 能够动态补全 JIRA ticket keys
- [ ] 能够动态补全 PR numbers
- [ ] 补全响应迅速

---

### 4. 错误处理与恢复

#### 4.1 配置重试策略

**优先级**: 中
**状态**: ❌ 未实现（基础重试已实现）

**需求描述**：
- 支持配置重试策略
- 支持不同错误类型的重试策略
- 提供灵活的重试配置

**功能要求**：
1. **当前状态**：HTTP 客户端已实现基础重试机制 ✅

2. **扩展需求**：
   - 支持配置重试策略
   - 支持不同错误类型的重试策略
   - 支持自定义重试参数

3. **配置示例**：
```toml
[retry]
enabled = true
max_retries = 3
initial_delay = "1s"
max_delay = "10s"
backoff_multiplier = 2.0

[retry.strategies]
network_error = { max_retries = 5, initial_delay = "500ms" }
rate_limit = { max_retries = 3, initial_delay = "5s" }
server_error = { max_retries = 2, initial_delay = "2s" }
```

**验收标准**：
- [ ] 能够在配置文件中设置重试策略
- [ ] 支持不同错误类型的重试策略
- [ ] 重试策略能够正确应用

---

#### 4.2 操作撤销

**优先级**: 低
**状态**: ❌ 未实现

**需求描述**：
- 记录操作历史
- 支持撤销最近的操作
- 提供操作回滚能力

**功能要求**：
1. **操作记录**
   - 记录所有可撤销操作
   - 记录操作参数和结果
   - 使用操作日志持久化

2. **可撤销操作**：
   - JIRA 状态转换
   - JIRA 分配
   - PR 合并（如果 API 支持）
   - 分支删除（如果 API 支持）

3. **命令接口**：
```bash
workflow undo                                      # 撤销最近的操作
workflow undo --list                                # 列出可撤销的操作
workflow undo --count 3                             # 撤销最近 3 个操作
```

4. **实现建议**：
   - 使用操作日志记录所有可撤销操作
   - 支持操作回滚（如果 API 支持）

**验收标准**：
- [ ] 能够记录可撤销操作
- [ ] 能够列出可撤销操作
- [ ] 能够撤销最近的操作
- [ ] 撤销操作能够正确回滚

---

#### 4.3 详细错误信息

**优先级**: 高
**状态**: ❌ 未实现

**需求描述**：
- 提供友好的错误提示
- 提供解决建议
- 提供错误代码和解决方案链接

**功能要求**：
1. **错误信息格式**：
```
Error: Failed to create PR

Reason: Branch 'feature/new-feature' not found

Possible solutions:
  1. Create the branch first: workflow branch create feature/new-feature
  2. Check branch name: workflow branch list
  3. See documentation: https://docs.example.com/pr-create

Error code: PR_CREATE_BRANCH_NOT_FOUND
```

2. **实现建议**：
   - 使用 `anyhow` 的上下文信息
   - 提供错误代码和解决方案链接
   - 为常见错误提供解决建议

3. **实现示例**：
```rust
use anyhow::{Context, Result};

pub fn create_pr_with_error_context(params: CreatePrParams) -> Result<PullRequest> {
    create_pr(&params)
        .context("Failed to create PR")
        .with_context(|| format!("Branch: {}", params.branch))
}
```

**验收标准**：
- [ ] 错误信息友好易懂
- [ ] 提供具体的解决建议
- [ ] 包含错误代码
- [ ] 错误信息格式统一

---

## 📊 优先级总结

### 高优先级
1. ~~**进度显示** - 长时间操作的进度条~~ ✅ 已实现
2. **详细错误信息** - 友好的错误提示和解决建议

### 中优先级
1. **交互式选择** - Fuzzy finder 选择 tickets/PRs
2. **别名系统** - 自定义命令别名
3. **命令历史** - 记录常用命令
4. **配置重试策略** - 配置重试策略

### 低优先级
1. **智能补全** - 增强 shell completion 和动态补全
2. **操作撤销** - 记录操作历史和撤销功能

---

## 🚀 实施计划

### 第一阶段：基础用户体验优化
- [x] 进度显示功能 ✅
- [ ] 详细错误信息功能

### 第二阶段：交互式功能
- [ ] 交互式选择（fuzzy finder）
- [ ] 别名系统
- [ ] 命令历史

### 第三阶段：高级功能
- [ ] 智能补全
- [ ] 操作撤销
- [ ] 配置重试策略扩展

---

## 🛠️ 技术栈

1. **Fuzzy Finder**：使用 `inquire`（已集成，默认支持模糊搜索）✅
2. **进度条**：使用 `indicatif` 显示进度 ✅
3. **Shell Completion**：使用 `clap_complete` 生成补全脚本
4. **错误处理**：使用 `anyhow` 提供详细错误信息
5. **操作日志**：记录操作历史以支持撤销

---

## ✅ 验收检查清单

### 交互式选择
- [ ] Fuzzy finder 集成完成
- [ ] 多选功能实现
- [ ] 交互体验流畅

### 进度显示 ✅
- [x] 进度条显示准确
- [x] 预计时间计算正确
- [x] 样式美观

### 别名系统
- [ ] 别名配置功能
- [ ] 别名展开正确
- [ ] 支持参数传递

### 命令历史
- [ ] 历史记录功能
- [ ] 重放功能
- [ ] 搜索功能

### 智能补全
- [ ] Shell 补全脚本生成
- [ ] 动态补全实现
- [ ] 补全响应迅速

### 错误处理
- [ ] 重试策略配置
- [ ] 详细错误信息
- [ ] 操作撤销功能

---

## 📚 相关文档

- [UX TODO 文档](../todo/UX_TODO.md)
- [JIRA 模块待办事项](../todo/JIRA_TODO.md)
- [Git 工作流待办事项](../todo/GIT_TODO.md)
- [工作流自动化待办事项](../todo/WORKFLOW_TODO.md)

---

**最后更新**: 2025-12-09
