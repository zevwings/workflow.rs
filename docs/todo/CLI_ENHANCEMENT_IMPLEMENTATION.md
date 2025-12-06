# CLI 增强库实现文档

## 📋 概述

本文档详细说明如何将项目从传统的 CLI 库（`colored`、`dialoguer`）迁移到增强的 CLI 库（`inquire`、`console`、`tabled`），以提供更好的用户体验。

---

## 🎯 目标

### 当前状态
- ✅ `colored = "2.1"` - 基础颜色输出
- ✅ `dialoguer = "0.11"` - 基础交互式提示
- ✅ `indicatif = "0.17"` - 进度条（已在使用，无需迁移）

### 目标状态
- 🎯 `inquire = "0.9"` - 现代化交互式提示（替代 `dialoguer`）
- 🎯 `console = "0.15"` - 增强终端输出（替代 `colored`）
- 🎯 `tabled = "0.14"` - 表格输出（新增功能）

---

## 📊 影响范围分析

### 1. dialoguer → inquire 迁移

**使用 dialoguer 的文件**（共 18 个文件）：

#### Input 使用（10 个文件）
1. `src/commands/jira/info.rs`
2. `src/commands/jira/attachments.rs`
3. `src/commands/jira/clean.rs`
4. `src/commands/log/download.rs`
5. `src/commands/log/find.rs`
6. `src/commands/log/search.rs`
7. `src/commands/github/helpers.rs`
8. `src/commands/pr/create.rs`
9. `src/commands/pr/pick.rs`
10. `src/commands/branch/ignore.rs`

#### Select 使用（8 个文件）
1. `src/commands/config/helpers.rs`
2. `src/commands/config/setup.rs`
3. `src/commands/config/log.rs`
4. `src/commands/llm/setup.rs`
5. `src/commands/github/github.rs`
6. `src/commands/pr/sync.rs`
7. `src/commands/pr/rebase.rs`
8. `src/lib/jira/status.rs`

#### MultiSelect 使用（3 个文件）
1. `src/commands/pr/create.rs`
2. `src/commands/pr/pick.rs`
3. `src/commands/branch/ignore.rs`
4. `src/commands/config/completion.rs`

#### Confirm 使用（2 个文件）
1. `src/lib/base/util/confirm.rs`
2. `src/lib/base/http/retry.rs`

**总计**：约 23 个使用点（部分文件使用多种类型）

---

### 2. colored → console 迁移

**使用 colored 的文件**（1 个核心文件）：

1. `src/lib/base/util/logger.rs` - 所有日志输出函数
   - `Logger::success()` - 绿色 ✓
   - `Logger::error()` - 红色 ✗
   - `Logger::warning()` - 黄色 ⚠
   - `Logger::info()` - 蓝色 ℹ
   - `Logger::debug()` - 灰色 ⚙
   - `Logger::print_separator()` - 分隔线
   - `Logger::print_separator_with_text()` - 带文本的分隔线

**影响范围**：
- 所有使用 `log_*!` 宏的代码（整个项目）
- 约 500+ 行代码间接使用

---

### 3. tabled 新增功能

**潜在使用场景**：
1. PR 列表显示（`src/commands/pr/*.rs`）
2. JIRA issue 列表显示（`src/commands/jira/*.rs`）
3. GitHub 仓库列表（`src/commands/github/*.rs`）
4. 配置信息展示（`src/commands/config/*.rs`）
5. 日志搜索结果（`src/commands/log/*.rs`）

**新增文件**：无需新增，在现有命令中集成

---

## 🔄 迁移策略

### 阶段 1：添加依赖（5 分钟）

#### 1.1 更新 Cargo.toml

```toml
[dependencies]
# 传统 CLI 依赖（保留，逐步迁移）
colored = "2.1"
dialoguer = "0.11"
indicatif = "0.17"

# 增强 CLI 依赖（新增）
inquire = "0.9"
console = "0.15"
tabled = "0.14"
```

#### 1.2 运行 cargo update

```bash
cargo update
```

---

### 阶段 2：创建兼容层（1-2 天）

为了平滑迁移，建议创建兼容层，允许新旧库共存。

#### 2.1 创建 inquire 兼容层

**文件**：`src/lib/base/util/prompts.rs`（新建）

```rust
//! 交互式提示兼容层
//!
//! 提供统一的交互式提示接口，支持 dialoguer 和 inquire 两种实现。
//! 当前使用 inquire，未来可以轻松切换。

use anyhow::Result;

/// 输入提示
pub fn input(prompt: &str) -> Result<String> {
    use inquire::Text;
    Text::new(prompt)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Input cancelled: {}", e))
}

/// 输入提示（带默认值）
pub fn input_with_default(prompt: &str, default: &str) -> Result<String> {
    use inquire::Text;
    Text::new(prompt)
        .with_default(default)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Input cancelled: {}", e))
}

/// 输入提示（带验证）
pub fn input_with_validation<F>(prompt: &str, validator: F) -> Result<String>
where
    F: Fn(&str) -> Result<(), String>,
{
    use inquire::Text;
    Text::new(prompt)
        .with_validator(move |input: &str| {
            validator(input).map_err(|e| inquire::error::InquireError::Custom(e.into()))
        })
        .prompt()
        .map_err(|e| anyhow::anyhow!("Input cancelled: {}", e))
}

/// 选择提示
pub fn select<T>(prompt: &str, options: Vec<T>) -> Result<usize>
where
    T: std::fmt::Display,
{
    use inquire::Select;
    Select::new(prompt, options)
        .prompt()
        .map(|item| item.0)
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))
}

/// 选择提示（带默认值）
pub fn select_with_default<T>(prompt: &str, options: Vec<T>, default: usize) -> Result<usize>
where
    T: std::fmt::Display,
{
    use inquire::Select;
    Select::new(prompt, options)
        .with_starting_cursor(default)
        .prompt()
        .map(|item| item.0)
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))
}

/// 多选提示
pub fn multi_select<T>(prompt: &str, options: Vec<T>) -> Result<Vec<usize>>
where
    T: std::fmt::Display,
{
    use inquire::MultiSelect;
    MultiSelect::new(prompt, options)
        .prompt()
        .map(|items| items.iter().map(|item| item.0).collect())
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))
}

/// 多选提示（带默认值）
pub fn multi_select_with_defaults<T>(
    prompt: &str,
    options: Vec<T>,
    defaults: Vec<usize>,
) -> Result<Vec<usize>>
where
    T: std::fmt::Display,
{
    use inquire::MultiSelect;
    let default_items: Vec<_> = defaults
        .iter()
        .filter_map(|&idx| options.get(idx).map(|opt| (idx, opt)))
        .collect();

    MultiSelect::new(prompt, options)
        .with_default(&default_items)
        .prompt()
        .map(|items| items.iter().map(|item| item.0).collect())
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))
}

/// 确认提示
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    use inquire::Confirm;
    Confirm::new(prompt)
        .with_default(default)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Confirmation cancelled: {}", e))
}
```

#### 2.2 创建 console 兼容层

**文件**：`src/lib/base/util/colors.rs`（新建）

```rust
//! 颜色输出兼容层
//!
//! 提供统一的颜色输出接口，支持 colored 和 console 两种实现。
//! 当前使用 console，未来可以轻松切换。

use console::{style, Emoji};

/// 成功消息样式（绿色）
pub fn success(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("✅", "✓"), style(text).green().bold())
}

/// 错误消息样式（红色）
pub fn error(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("❌", "✗"), style(text).red().bold())
}

/// 警告消息样式（黄色）
pub fn warning(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("⚠️", "⚠"), style(text).yellow().bold())
}

/// 信息消息样式（蓝色）
pub fn info(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("ℹ️", "ℹ"), style(text).blue().bold())
}

/// 调试消息样式（灰色）
pub fn debug(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("🔧", "⚙"), style(text).bright().black())
}

/// 分隔线样式（灰色）
pub fn separator(char: char, length: usize) -> String {
    style(char.to_string().repeat(length))
        .bright()
        .black()
        .to_string()
}

/// 带文本的分隔线样式
pub fn separator_with_text(char: char, length: usize, text: impl std::fmt::Display) -> String {
    let text_str = format!("  {} ", text);
    let text_len = text_str.chars().count();

    if text_len >= length {
        return style(text_str).bright().black().to_string();
    }

    let remaining = length - text_len;
    let left_padding = remaining / 2;
    let right_padding = remaining - left_padding;

    let left_sep = char.to_string().repeat(left_padding);
    let right_sep = char.to_string().repeat(right_padding);

    format!(
        "{}{}{}",
        style(left_sep).bright().black(),
        text_str,
        style(right_sep).bright().black()
    )
}
```

---

### 阶段 3：迁移 logger.rs（1 天）

#### 3.1 更新 logger.rs

**文件**：`src/lib/base/util/logger.rs`

**修改前**：
```rust
use colored::*;

impl Logger {
    pub fn success(message: impl fmt::Display) -> String {
        format!("{} {}", "✓".green(), message)
    }
    // ...
}
```

**修改后**：
```rust
use crate::base::util::colors::{success, error, warning, info, debug, separator, separator_with_text};

impl Logger {
    pub fn success(message: impl fmt::Display) -> String {
        success(message)
    }

    pub fn error(message: impl fmt::Display) -> String {
        error(message)
    }

    pub fn warning(message: impl fmt::Display) -> String {
        warning(message)
    }

    pub fn info(message: impl fmt::Display) -> String {
        info(message)
    }

    pub fn debug(message: impl fmt::Display) -> String {
        debug(message)
    }

    pub fn print_separator(char: Option<char>, length: Option<usize>) {
        let char = char.unwrap_or('-');
        let length = length.unwrap_or(80);
        println!("{}", separator(char, length));
    }

    pub fn print_separator_with_text(char: char, length: usize, text: impl fmt::Display) {
        println!("{}", separator_with_text(char, length, text));
    }
}
```

---

### 阶段 4：迁移 confirm.rs（30 分钟）

#### 4.1 更新 confirm.rs

**文件**：`src/lib/base/util/confirm.rs`

**修改前**：
```rust
use dialoguer::Confirm;

pub fn confirm(prompt: &str, default: bool, message: Option<&str>) -> Result<bool> {
    let confirmed = Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .context("Failed to get user confirmation")?;
    // ...
}
```

**修改后**：
```rust
use crate::base::util::prompts::confirm as prompt_confirm;

pub fn confirm(prompt: &str, default: bool, message: Option<&str>) -> Result<bool> {
    let confirmed = prompt_confirm(prompt, default)
        .context("Failed to get user confirmation")?;

    if !confirmed {
        if let Some(msg) = message {
            anyhow::bail!("{}", msg);
        }
    }

    Ok(confirmed)
}
```

---

### 阶段 5：迁移命令文件（3-5 天）

#### 5.1 迁移 Input 使用

**示例**：`src/commands/jira/info.rs`

**修改前**：
```rust
use dialoguer::Input;

let ticket_id: String = Input::new()
    .with_prompt("Enter JIRA ticket ID")
    .interact_text()?;
```

**修改后**：
```rust
use crate::base::util::prompts::input;

let ticket_id = input("Enter JIRA ticket ID")?;
```

**或使用 inquire 直接调用**：
```rust
use inquire::Text;

let ticket_id = Text::new("Enter JIRA ticket ID")
    .prompt()?;
```

#### 5.2 迁移 Select 使用

**示例**：`src/commands/config/helpers.rs`

**修改前**：
```rust
use dialoguer::Select;

let idx = Select::new()
    .with_prompt("Select language")
    .items(&languages)
    .default(current_idx)
    .interact()?;
```

**修改后**：
```rust
use crate::base::util::prompts::select_with_default;

let idx = select_with_default("Select language", languages, current_idx)?;
```

**或使用 inquire 直接调用**：
```rust
use inquire::Select;

let idx = Select::new("Select language", languages)
    .with_starting_cursor(current_idx)
    .prompt()
    .map(|item| item.0)?;
```

#### 5.3 迁移 MultiSelect 使用

**示例**：`src/commands/pr/create.rs`

**修改前**：
```rust
use dialoguer::MultiSelect;

let selected = MultiSelect::new()
    .with_prompt("Select change types")
    .items(&change_types)
    .interact()?;
```

**修改后**：
```rust
use crate::base::util::prompts::multi_select;

let selected_indices = multi_select("Select change types", change_types)?;
```

**或使用 inquire 直接调用**：
```rust
use inquire::MultiSelect;

let selected = MultiSelect::new("Select change types", change_types)
    .prompt()
    .map(|items| items.iter().map(|item| item.0).collect::<Vec<_>>())?;
```

---

### 阶段 6：添加表格输出（2-3 天）

#### 6.1 创建表格工具函数

**文件**：`src/lib/base/util/table.rs`（新建）

```rust
//! 表格输出工具
//!
//! 提供统一的表格输出接口，使用 tabled 库。

use tabled::{Table, Tabled};

/// 打印表格
pub fn print_table<T: Tabled>(data: Vec<T>) {
    println!("{}", Table::new(data));
}

/// 打印表格（带标题）
pub fn print_table_with_title<T: Tabled>(title: &str, data: Vec<T>) {
    println!("{}\n", title);
    println!("{}", Table::new(data));
}

/// 从结构体创建表格
///
/// # 示例
/// ```rust
/// #[derive(Tabled)]
/// struct PR {
///     number: u32,
///     title: String,
///     author: String,
///     status: String,
/// }
///
/// let prs = vec![
///     PR { number: 123, title: "Fix bug".to_string(), author: "Alice".to_string(), status: "Open".to_string() },
/// ];
///
/// print_table(prs);
/// ```
pub fn create_table<T: Tabled>(data: Vec<T>) -> String {
    Table::new(data).to_string()
}
```

#### 6.2 在命令中使用表格

**示例**：PR 列表显示

```rust
use crate::base::util::table::print_table;
use tabled::Tabled;

#[derive(Tabled)]
struct PRRow {
    #[tabled(rename = "PR #")]
    number: u32,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Author")]
    author: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub fn list_prs(prs: Vec<PullRequest>) -> Result<()> {
    let rows: Vec<PRRow> = prs
        .into_iter()
        .map(|pr| PRRow {
            number: pr.number,
            title: pr.title,
            author: pr.author,
            status: pr.status.to_string(),
        })
        .collect();

    print_table(rows);
    Ok(())
}
```

---

## 📝 详细迁移步骤

### 步骤 1：准备阶段（1 小时）

1. **创建新分支**
   ```bash
   git checkout -b feature/cli-enhancement
   ```

2. **更新 Cargo.toml**
   - 添加 `inquire = "0.9"`
   - 添加 `console = "0.15"`
   - 添加 `tabled = "0.14"`

3. **运行 cargo update**
   ```bash
   cargo update
   ```

4. **验证编译**
   ```bash
   cargo check
   ```

---

### 步骤 2：创建兼容层（2-3 小时）

1. **创建 `src/lib/base/util/prompts.rs`**
   - 实现所有交互式提示函数
   - 使用 inquire 作为后端

2. **创建 `src/lib/base/util/colors.rs`**
   - 实现所有颜色输出函数
   - 使用 console 作为后端

3. **创建 `src/lib/base/util/table.rs`**
   - 实现表格输出函数
   - 使用 tabled 作为后端

4. **更新 `src/lib/base/util/mod.rs`**
   ```rust
   pub mod prompts;
   pub mod colors;
   pub mod table;
   ```

5. **验证编译**
   ```bash
   cargo check
   ```

---

### 步骤 3：迁移核心模块（1 天）

1. **迁移 `logger.rs`**
   - 替换 `colored::*` 为 `colors::*`
   - 保持 API 不变
   - 测试所有日志函数

2. **迁移 `confirm.rs`**
   - 替换 `dialoguer::Confirm` 为 `prompts::confirm`
   - 保持 API 不变
   - 测试确认功能

3. **验证编译和测试**
   ```bash
   cargo build
   cargo test
   ```

---

### 步骤 4：迁移命令文件（3-5 天）

按优先级迁移：

#### 优先级 1：高频使用（1-2 天）
1. `src/commands/config/setup.rs`
2. `src/commands/github/github.rs`
3. `src/commands/pr/create.rs`

#### 优先级 2：中频使用（1-2 天）
4. `src/commands/jira/info.rs`
5. `src/commands/log/*.rs`
6. `src/commands/config/*.rs`

#### 优先级 3：低频使用（1 天）
7. 其他命令文件

**迁移模式**：
- 每个文件独立迁移
- 迁移后立即测试
- 提交独立的 commit

---

### 步骤 5：添加表格功能（2-3 天）

1. **识别需要表格输出的场景**
   - PR 列表
   - JIRA issue 列表
   - GitHub 仓库列表
   - 配置信息展示

2. **创建表格结构体**
   - 为每个场景创建 `Tabled` 结构体
   - 定义列标题和格式

3. **集成表格输出**
   - 替换现有的文本输出
   - 使用 `print_table()` 函数

4. **测试和验证**
   - 测试各种数据场景
   - 验证表格格式正确

---

### 步骤 6：清理和优化（1 天）

1. **移除旧依赖**
   - 从 `Cargo.toml` 移除 `colored` 和 `dialoguer`
   - 运行 `cargo update`

2. **清理未使用的导入**
   - 搜索所有 `use colored::*`
   - 搜索所有 `use dialoguer::*`
   - 移除未使用的导入

3. **更新文档**
   - 更新 README
   - 更新架构文档

4. **最终测试**
   ```bash
   cargo test
   cargo build --release
   ```

---

## 🧪 测试策略

### 单元测试

为每个兼容层函数添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_color() {
        let result = success("Test");
        assert!(result.contains("Test"));
        assert!(result.contains("✓") || result.contains("✅"));
    }

    #[test]
    fn test_input_prompt() {
        // 测试输入提示（需要模拟用户输入）
    }
}
```

### 集成测试

测试完整的命令流程：

```rust
#[test]
fn test_config_setup_flow() {
    // 测试配置设置流程
    // 包括输入、选择、确认等交互
}
```

### 手动测试清单

- [ ] 所有交互式命令正常工作
- [ ] 所有日志输出颜色正确
- [ ] 表格输出格式正确
- [ ] 错误处理正常
- [ ] 非交互式模式（CI）正常工作

---

## 📊 迁移进度跟踪

### 文件迁移状态

#### dialoguer → inquire

- [ ] `src/lib/base/util/confirm.rs`
- [ ] `src/lib/base/http/retry.rs`
- [ ] `src/commands/jira/info.rs`
- [ ] `src/commands/jira/attachments.rs`
- [ ] `src/commands/jira/clean.rs`
- [ ] `src/commands/log/download.rs`
- [ ] `src/commands/log/find.rs`
- [ ] `src/commands/log/search.rs`
- [ ] `src/commands/github/helpers.rs`
- [ ] `src/commands/github/github.rs`
- [ ] `src/commands/pr/create.rs`
- [ ] `src/commands/pr/pick.rs`
- [ ] `src/commands/pr/sync.rs`
- [ ] `src/commands/pr/rebase.rs`
- [ ] `src/commands/branch/ignore.rs`
- [ ] `src/commands/config/helpers.rs`
- [ ] `src/commands/config/setup.rs`
- [ ] `src/commands/config/log.rs`
- [ ] `src/commands/config/completion.rs`
- [ ] `src/commands/llm/setup.rs`
- [ ] `src/lib/jira/status.rs`

#### colored → console

- [ ] `src/lib/base/util/logger.rs`

#### tabled 新增

- [ ] PR 列表显示
- [ ] JIRA issue 列表显示
- [ ] GitHub 仓库列表
- [ ] 配置信息展示
- [ ] 日志搜索结果

---

## ⚠️ 注意事项

### 1. API 兼容性

- 保持所有公共 API 不变
- 只改变内部实现
- 确保向后兼容

### 2. 错误处理

- `inquire` 的错误类型与 `dialoguer` 不同
- 需要统一错误处理
- 确保用户取消操作时正确处理

### 3. 非交互式模式

- 确保在 CI 环境中正常工作
- 处理非 TTY 环境
- 提供合理的默认值

### 4. 性能考虑

- `console` 比 `colored` 功能更丰富，但体积稍大
- `inquire` 比 `dialoguer` 功能更丰富，但可能有性能差异
- 测试实际性能影响

### 5. 依赖体积

**新增依赖体积估算**：
- `inquire`: ~200-300 KB
- `console`: ~100-150 KB
- `tabled`: ~150-200 KB
- **总计新增**: ~450-650 KB

**移除依赖体积**：
- `colored`: ~50-100 KB
- `dialoguer`: ~100-150 KB
- **总计移除**: ~150-250 KB

**净增加**: ~300-400 KB

---

## 🎯 成功标准

### 功能完整性
- ✅ 所有交互式命令正常工作
- ✅ 所有日志输出正常
- ✅ 表格输出格式正确

### 代码质量
- ✅ 无编译警告
- ✅ 所有测试通过
- ✅ 代码覆盖率不降低

### 用户体验
- ✅ 交互更流畅（inquire 的模糊搜索）
- ✅ 输出更美观（console 的样式）
- ✅ 信息更清晰（tabled 的表格）

---

## 📚 参考资源

### 官方文档
- [inquire 文档](https://docs.rs/inquire/)
- [console 文档](https://docs.rs/console/)
- [tabled 文档](https://docs.rs/tabled/)

### 示例代码
- [inquire 示例](https://github.com/mikaelmello/inquire/tree/main/examples)
- [console 示例](https://github.com/console-rs/console/tree/main/examples)
- [tabled 示例](https://github.com/zhiburt/tabled/tree/main/examples)

---

## 🔄 回滚计划

如果迁移过程中遇到问题，可以按以下步骤回滚：

1. **保留旧依赖**
   - 在迁移完成前，不要移除 `colored` 和 `dialoguer`
   - 允许新旧库共存

2. **使用特性标志**
   - 可以使用 Cargo features 控制使用哪个库
   - 默认使用新库，但可以切换到旧库

3. **Git 回滚**
   - 如果问题严重，可以回滚到迁移前的 commit
   - 保留迁移分支以便后续分析

---

## 📅 时间估算

| 阶段 | 时间估算 | 优先级 |
|------|----------|--------|
| 阶段 1：添加依赖 | 5 分钟 | 高 |
| 阶段 2：创建兼容层 | 1-2 天 | 高 |
| 阶段 3：迁移 logger.rs | 1 天 | 高 |
| 阶段 4：迁移 confirm.rs | 30 分钟 | 高 |
| 阶段 5：迁移命令文件 | 3-5 天 | 中 |
| 阶段 6：添加表格功能 | 2-3 天 | 低 |
| 阶段 7：清理和优化 | 1 天 | 中 |

**总计**：约 8-12 个工作日

---

## ✅ 检查清单

### 准备阶段
- [ ] 创建新分支
- [ ] 更新 Cargo.toml
- [ ] 运行 cargo update
- [ ] 验证编译

### 兼容层
- [ ] 创建 prompts.rs
- [ ] 创建 colors.rs
- [ ] 创建 table.rs
- [ ] 更新 mod.rs
- [ ] 添加单元测试

### 核心模块迁移
- [ ] 迁移 logger.rs
- [ ] 迁移 confirm.rs
- [ ] 运行测试

### 命令文件迁移
- [ ] 迁移所有 Input 使用
- [ ] 迁移所有 Select 使用
- [ ] 迁移所有 MultiSelect 使用
- [ ] 迁移所有 Confirm 使用

### 表格功能
- [ ] 识别使用场景
- [ ] 创建表格结构体
- [ ] 集成表格输出
- [ ] 测试表格格式

### 清理阶段
- [ ] 移除旧依赖
- [ ] 清理未使用导入
- [ ] 更新文档
- [ ] 最终测试

---

## 🎉 完成后的收益

### 用户体验提升
- ✅ 更流畅的交互（模糊搜索、键盘导航）
- ✅ 更美观的输出（丰富的样式、Emoji）
- ✅ 更清晰的信息展示（表格格式）

### 开发体验提升
- ✅ 更现代的 API
- ✅ 更好的错误处理
- ✅ 更丰富的功能

### 维护性提升
- ✅ 更活跃的社区
- ✅ 更好的文档
- ✅ 更频繁的更新

---

**文档版本**：1.0
**创建日期**：2024-12-06
**最后更新**：2024-12-06
