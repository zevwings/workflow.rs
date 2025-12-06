# CLI 模式恢复分析

## 📋 问题分析

### 当前状态

1. **依赖状态**：
   - `Cargo.toml` 中已注释掉传统 CLI 依赖：
     - `colored = "2.1"` ❌ 已注释
     - `dialoguer = "0.11"` ❌ 已注释
     - `indicatif = "0.17"` ❌ 已注释
   - 已添加 ratatui 依赖：
     - `ratatui = "0.27"` ✅
     - `crossterm = "0.28"` ✅

2. **代码使用情况**：
   - **仍在使用 `colored`**：
     - `src/lib/base/util/logger.rs` - 所有日志输出函数
     - 大量使用 `colored::*` 宏（`.green()`, `.red()`, `.yellow()` 等）

   - **仍在使用 `dialoguer`**：
     - `src/lib/base/util/confirm.rs` - 确认对话框
     - `src/lib/base/http/retry.rs` - 重试确认
     - `src/commands/log/search.rs` - 输入提示
     - `src/commands/log/find.rs` - 输入提示
     - `src/commands/log/download.rs` - 输入提示
     - `src/commands/jira/clean.rs` - 输入提示
     - `src/commands/jira/attachments.rs` - 输入提示
     - `src/commands/jira/info.rs` - 输入提示
     - `src/commands/pr/create.rs` - 输入和多选
     - `src/commands/pr/pick.rs` - 输入和多选
     - `src/commands/config/helpers.rs` - 选择
     - `src/commands/config/setup.rs` - 输入和选择
     - `src/commands/config/log.rs` - 选择
     - `src/commands/config/completion.rs` - 多选
     - `src/commands/branch/ignore.rs` - 输入和多选
     - `src/commands/github/helpers.rs` - 输入
     - `src/commands/github/github.rs` - 选择
     - `src/commands/llm/setup.rs` - 输入和选择
     - `src/lib/jira/status.rs` - 选择
     - `src/commands/pr/sync.rs` - 选择（内联使用）
     - `src/commands/pr/rebase.rs` - 选择（内联使用）

   - **仍在使用 `indicatif`**：
     - `src/commands/lifecycle/update.rs` - 下载进度条

3. **问题**：
   - 依赖被注释但代码仍在使用，会导致编译错误
   - 部分代码已迁移到 ratatui（如 `src/lib/base/ui/dialogs.rs`），但大部分命令仍使用传统库

---

## 🔧 恢复传统 CLI 模式的方案

### 方案 1：完全恢复传统 CLI（推荐用于快速修复）

**步骤**：
1. 恢复 `Cargo.toml` 中的依赖：
   ```toml
   colored = "2.1"
   dialoguer = "0.11"
   indicatif = "0.17"
   ```

2. 保留 ratatui 依赖（可选，用于未来扩展）：
   ```toml
   ratatui = "0.27"  # 可选，用于未来 TUI 功能
   crossterm = "0.28"  # 可选，用于未来 TUI 功能
   ```

3. 确保所有使用传统库的代码正常工作

**优点**：
- ✅ 快速恢复，最小改动
- ✅ 保持现有代码结构
- ✅ 向后兼容

**缺点**：
- ⚠️ 同时维护两套 UI 系统（传统 CLI + ratatui）

---

### 方案 2：增强传统 CLI（推荐用于长期改进）

在恢复传统 CLI 的基础上，引入更现代的库来增强展示效果。

#### 2.1 使用 `inquire` 替代 `dialoguer`

**优势**：
- ✅ 更现代的 API
- ✅ 支持模糊搜索
- ✅ 更好的类型安全
- ✅ 更丰富的验证功能
- ✅ API 与 `dialoguer` 类似，迁移成本低

**安装**：
```toml
[dependencies]
inquire = "0.7"
```

**示例对比**：

```rust
// dialoguer 方式
use dialoguer::Select;
let idx = Select::new()
    .with_prompt("Select option")
    .items(&options)
    .interact()?;

// inquire 方式（增强版）
use inquire::Select;
let option = Select::new("Select option", options)
    .with_fuzzy_search(true)  // 新增：模糊搜索
    .with_page_size(10)        // 新增：分页显示
    .prompt()?;
```

**迁移成本**：低（API 类似）

---

#### 2.2 使用 `console` 增强颜色输出

**优势**：
- ✅ 比 `colored` 功能更丰富
- ✅ 支持表格输出
- ✅ 支持 emoji
- ✅ 更好的跨平台支持

**安装**：
```toml
[dependencies]
console = "0.15"
```

**示例对比**：

```rust
// colored 方式
use colored::*;
println!("{}", "Success".green());

// console 方式（增强版）
use console::{style, Emoji};
println!("{} {}", Emoji("✅", "✓"), style("Success").green());
```

**迁移成本**：中等（需要替换所有 `colored` 调用）

---

#### 2.3 使用 `tabled` 或 `comfy-table` 增强表格显示

**优势**：
- ✅ 美观的表格输出
- ✅ 自动列宽调整
- ✅ 支持对齐、边框等样式

**安装**：
```toml
[dependencies]
tabled = "0.14"
# 或
comfy-table = "7.0"
```

**示例**：

```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct PR {
    number: u32,
    title: String,
    author: String,
    status: String,
}

let prs = vec![
    PR { number: 123, title: "Fix bug".to_string(), author: "Alice".to_string(), status: "Open".to_string() },
    PR { number: 124, title: "Add feature".to_string(), author: "Bob".to_string(), status: "Merged".to_string() },
];

println!("{}", Table::new(prs));
```

**适用场景**：
- `workflow pr list` - PR 列表
- `workflow jira search` - JIRA ticket 列表
- `workflow github list` - GitHub 账号列表

---

#### 2.4 使用 `indicatif` 替代 `spinners`（推荐）⭐

**重要发现**：`indicatif` 已经支持 spinner 功能，**可以完全替代 `spinners`**！

**indicatif 的 spinner 功能**：
- ✅ 支持 spinner 模式（`ProgressBar::new_spinner()`）
- ✅ 可自定义样式和消息
- ✅ 支持与进度条混合使用
- ✅ **项目已在使用**，无需新增依赖

**当前项目使用情况**：
- `src/commands/lifecycle/update.rs` 已使用 `ProgressBar::new_spinner()`

**示例对比**：

```rust
// spinners 方式（需要新增依赖）
use spinners::{Spinner, Spinners};
let mut sp = Spinner::new(Spinners::Dots, "Loading...".into());
// 执行操作...
sp.stop();

// indicatif 方式（已在使用，无需新增依赖）⭐
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

let spinner = ProgressBar::new_spinner();
spinner.set_style(
    ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
);
spinner.enable_steady_tick(Duration::from_millis(100));
spinner.set_message("Loading...".to_string());
// 执行操作...
spinner.finish_with_message("Done!");
```

**优势**：
- ✅ **无需新增依赖**：项目已使用 `indicatif`
- ✅ **功能完整**：支持 spinner 和进度条
- ✅ **统一风格**：与现有进度条保持一致
- ✅ **体积更小**：不增加额外依赖

**适用场景**：
- API 请求等待
- 文件下载（无大小信息时）
- 长时间操作
- 不确定进度的任务

**结论**：**推荐使用 `indicatif` 的 spinner 功能，无需引入 `spinners` 库**。

---

#### 2.5 使用 `indicatif` 增强进度条（已在使用，可优化）

**当前使用**：
- `src/commands/lifecycle/update.rs` - 下载进度

**可优化项**：
- 添加更多进度条样式
- 支持多任务进度
- 添加 ETA 显示

**示例优化**：

```rust
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

let multi = MultiProgress::new();
let pb = multi.add(ProgressBar::new(100));
pb.set_style(
    ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg} ETA: {eta}")
        .unwrap()
        .progress_chars("#>-")
);
```

---

## 📊 增强组件对比

| 组件 | 功能 | 替代目标 | 迁移成本 | 推荐度 | 展示样式 | 文档 | GitHub |
|------|------|----------|----------|--------|----------|------|--------|
| **inquire** | 交互式提示（增强版 dialoguer） | `dialoguer` | 低 | ⭐⭐⭐⭐⭐ | 模糊搜索、键盘导航、彩色高亮 | [docs.rs](https://docs.rs/inquire/) | [github](https://github.com/mikaelmello/inquire) |
| **console** | 颜色输出（增强版 colored） | `colored` | 中 | ⭐⭐⭐⭐ | 丰富颜色、样式、Emoji | [docs.rs](https://docs.rs/console/) | [github](https://github.com/console-rs/console) |
| **tabled** | 表格输出 | 无（新增功能） | 低 | ⭐⭐⭐⭐⭐ | 自动列宽、清晰边框 | [docs.rs](https://docs.rs/tabled/) | [github](https://github.com/zhiburt/tabled) |
| **comfy-table** | 表格输出（替代方案） | 无（新增功能） | 低 | ⭐⭐⭐⭐ | 美观边框、自动换行、ANSI 颜色 | [docs.rs](https://docs.rs/comfy-table/) | [github](https://github.com/Nukesor/comfy-table) |
| **spinners** | 加载动画 | 无（新增功能） | 低 | ⭐⭐ | 60+ 动画样式 | [docs.rs](https://docs.rs/spinners/) | [github](https://github.com/console-rs/spinners) |
| **indicatif** | 进度条+Spinner（已在使用） | - | - | ⭐⭐⭐⭐⭐ | 进度条、Spinner、百分比、ETA、多任务 | [docs.rs](https://docs.rs/indicatif/) | [github](https://github.com/console-rs/indicatif) |

**注意**：`indicatif` 已支持 spinner 功能，**可以替代 `spinners`**，无需新增依赖。

---

## 🎨 展示样式说明

### 1. inquire - 交互式提示

**展示特点**：
- 支持模糊搜索（输入时实时过滤选项）
- 键盘导航（↑↓ 键选择，Enter 确认）
- 彩色高亮显示选中项
- 支持帮助信息和默认值

**示例代码**：
```rust
use inquire::{Select, Text, Confirm};

// 文本输入（带验证）
let name = Text::new("What's your name?")
    .with_validator(|s: &str| {
        if s.len() > 0 {
            Ok(())
        } else {
            Err("Name cannot be empty".into())
        }
    })
    .prompt()?;

// 选择（带模糊搜索）
let option = Select::new("Select an option", vec!["Option 1", "Option 2", "Option 3"])
    .with_fuzzy_search(true)  // 启用模糊搜索
    .with_page_size(10)
    .prompt()?;

// 确认
let confirmed = Confirm::new("Continue?")
    .with_default(true)
    .prompt()?;
```

**展示效果**：
```
? What's your name? █
  > Option 1
    Option 2
    Option 3
  [Use ↑↓ to move, type to filter, Enter to select]
```

---

### 2. console - 终端输出增强

**展示特点**：
- 丰富的颜色支持（16/256/真彩色）
- 文本样式（粗体、斜体、下划线等）
- Emoji 支持
- 表格输出（基础）

**示例代码**：
```rust
use console::{style, Emoji};

// 彩色文本
println!("{}", style("Success").green().bold());
println!("{}", style("Error").red().underline());

// Emoji + 文本
println!("{} {}", Emoji("✅", "✓"), style("Done").green());

// 组合样式
println!("{}", style("Warning").yellow().bold().on_black());
```

**展示效果**：
```
✓ Success (绿色粗体)
✗ Error (红色下划线)
✅ Done (绿色)
⚠ Warning (黄色粗体，黑色背景)
```

---

### 3. tabled - 表格输出

**展示特点**：
- 自动列宽调整
- 清晰的边框和分隔线
- 支持从结构体自动生成表格
- 支持多种数据格式（JSON、CSV、TOML、HTML）

**示例代码**：
```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct PR {
    number: u32,
    title: String,
    author: String,
    status: String,
}

let prs = vec![
    PR { number: 123, title: "Fix bug".to_string(), author: "Alice".to_string(), status: "Open".to_string() },
    PR { number: 124, title: "Add feature".to_string(), author: "Bob".to_string(), status: "Merged".to_string() },
];

println!("{}", Table::new(prs));
```

**展示效果**：
```
+--------+-------------+--------+--------+
| number | title       | author | status |
+--------+-------------+--------+--------+
| 123    | Fix bug     | Alice  | Open   |
+--------+-------------+--------+--------+
| 124    | Add feature | Bob    | Merged |
+--------+-------------+--------+--------+
```

---

### 4. comfy-table - 表格输出（替代方案）

**展示特点**：
- 美观的边框样式
- 自动内容换行
- ANSI 颜色支持
- 自定义边框、对齐、填充

**示例代码**：
```rust
use comfy_table::Table;

let mut table = Table::new();
table
    .set_header(vec!["Header1", "Header2", "Header3"])
    .add_row(vec!["Text 1", "Text 2", "Text 3"])
    .add_row(vec!["Multi\nline\ntext", "Another", "Cell"]);

println!("{table}");
```

**展示效果**：
```
+------------------+----------+----------+
| Header1          | Header2  | Header3  |
+======================================================================+
| Text 1           | Text 2   | Text 3   |
|------------------+----------+----------|
| Multi            | Another  | Cell     |
| line             |          |          |
| text             |          |          |
+------------------+----------+----------+
```

---

### 5. indicatif Spinner - 加载动画（推荐使用）⭐

**展示特点**：
- ✅ 支持 spinner 模式（无需新增依赖）
- ✅ 可自定义样式和消息
- ✅ 支持与进度条混合使用
- ✅ 项目已在使用

**示例代码**：
```rust
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

let spinner = ProgressBar::new_spinner();
spinner.set_style(
    ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
);
spinner.enable_steady_tick(Duration::from_millis(100));
spinner.set_message("Loading...".to_string());
// 执行操作...
spinner.finish_with_message("Done!");
```

**展示效果**：
```
⠋ Loading...  (旋转的点动画)
⠙ Loading...
⠹ Loading...
⠸ Loading...
⠼ Loading...
⠴ Loading...
⠦ Loading...
⠧ Loading...
⠇ Loading...
⠏ Loading...
```

**优势**：
- ✅ **无需新增依赖**：项目已使用 `indicatif`
- ✅ **功能完整**：支持 spinner 和进度条
- ✅ **统一风格**：与现有进度条保持一致

---

### 5.1 spinners - 加载动画（不推荐，indicatif 可替代）

**展示特点**：
- 60+ 种预定义动画样式
- 支持自定义消息
- 自动处理终端兼容性
- 简洁的 API

**为什么不推荐**：
- ❌ 需要新增依赖（~50-100 KB）
- ❌ `indicatif` 已支持 spinner 功能
- ❌ 功能重复，增加维护成本

**仅在以下情况考虑**：
- 需要非常特定的动画样式（indicatif 不支持）
- 需要 60+ 种预定义样式选择

---

### 6. indicatif - 进度条

**展示特点**：
- 多种进度条样式
- 支持多任务进度
- 显示百分比、速度、ETA
- 支持 Spinner 模式

**示例代码**：
```rust
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

let pb = ProgressBar::new(100);
pb.set_style(
    ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg} ETA: {eta}")
        .unwrap()
        .progress_chars("#>-")
);

for i in 0..100 {
    pb.set_message(format!("Processing {}", i));
    pb.inc(1);
    thread::sleep(Duration::from_millis(50));
}
pb.finish_with_message("Done!");
```

**展示效果**：
```
⠋ [00:00:05] [████████████████████████████████░░░░] 80/100 Processing 80 ETA: 00:00:01
```

**多进度条示例**：
```
⠋ [00:00:05] [████████████████░░░░░░░░░░░░░░░░░░░░] 40/100 Downloading file1.zip
⠙ [00:00:03] [████████████████████████████████████] 100/100 Downloading file2.zip
```

---

## 📸 视觉效果对比

### 交互式提示对比

**dialoguer（当前）**：
```
? Select an option:
 > Option 1
   Option 2
   Option 3
```

**inquire（增强）**：
```
? Select an option: █
  > Option 1          [输入时实时过滤]
    Option 2
    Option 3
  [Use ↑↓ to move, type to filter, Enter to select]
```

### 表格输出对比

**纯文本（当前）**：
```
PR #123: Fix bug - Alice - Open
PR #124: Add feature - Bob - Merged
```

**tabled（增强）**：
```
+--------+-------------+--------+--------+
| number | title       | author | status |
+--------+-------------+--------+--------+
| 123    | Fix bug     | Alice  | Open   |
| 124    | Add feature | Bob    | Merged |
+--------+-------------+--------+--------+
```

### 进度条对比

**indicatif（当前使用）**：
```
[████████████████░░░░░░░░] 60% Downloading...
```

**indicatif（优化后）**：
```
⠋ [00:00:05] [████████████████░░░░░░░░] 60/100 Downloading... ETA: 00:00:03
```

---

## 🎯 推荐方案

### 快速恢复方案（立即执行）

1. **恢复依赖**：
   ```toml
   colored = "2.1"
   dialoguer = "0.11"
   indicatif = "0.17"
   ```

2. **移除 TUI 相关依赖**（如果不需要 TUI）：
   ```toml
   # 移除或注释以下依赖
   # ratatui = "0.27"
   # crossterm = "0.28"
   ```

3. **移除 TUI 相关代码**（如果不需要 TUI）：
   - `src/lib/base/ui/` 目录（整个目录）
   - `src/lib/logging/` 目录（如果与 ratatui 集成）

4. **验证编译**：
   ```bash
   cargo build
   ```

5. **测试功能**：
   ```bash
   cargo test
   ```

---

### 增强方案（分阶段实施）

#### 阶段 1：快速增强（1-2 天）
1. ✅ 恢复传统 CLI 依赖
2. ✅ 引入 `inquire` 替代部分 `dialoguer`（关键命令）
3. ✅ 引入 `tabled` 用于列表展示

#### 阶段 2：全面增强（1 周）
1. ✅ 全面迁移到 `inquire`
2. ✅ 引入 `console` 替代 `colored`
3. ✅ 优化 `indicatif` 进度条和 spinner 样式
4. ✅ 使用 `indicatif` 的 spinner 功能（无需新增依赖）

#### 阶段 3：功能增强（2 周）
1. ✅ 为所有列表命令添加表格输出
2. ✅ 统一 UI 风格和主题
3. ✅ 添加更多交互式功能

---

## 📝 需要修改的文件清单

### 必须修改（恢复依赖）

1. **`Cargo.toml`**：
   - 取消注释 `colored`, `dialoguer`, `indicatif`

### 可选修改（增强功能）

1. **引入 `inquire`**：
   - 逐步替换 `dialoguer::Input` → `inquire::Text`
   - 逐步替换 `dialoguer::Select` → `inquire::Select`
   - 逐步替换 `dialoguer::MultiSelect` → `inquire::MultiSelect`
   - 逐步替换 `dialoguer::Confirm` → `inquire::Confirm`

2. **引入 `console`**：
   - 替换 `src/lib/base/util/logger.rs` 中的 `colored` 使用

3. **引入 `tabled` 或 `comfy-table`**：
   - `src/commands/pr/list.rs` - PR 列表
   - `src/commands/github/github.rs::list()` - GitHub 账号列表
   - `src/commands/branch/ignore.rs::list()` - 分支忽略列表
   - `src/commands/jira/info.rs` - JIRA ticket 附件列表
   - `src/commands/log/search.rs` - 日志搜索结果列表
   - 未来：`workflow jira list` - JIRA ticket 列表（待实现）

---

## 📋 tabled / comfy-table 快速对比

| 对比项 | tabled | comfy-table | 推荐 |
|--------|--------|-------------|------|
| **体积增加** | ~70-130 KB | ~250-410 KB | ✅ tabled |
| **依赖数量** | 2-3 个 | 6 个 | ✅ tabled |
| **no_std 支持** | ✅ | ❌ | ✅ tabled |
| **性能** | 优秀（编译时优化） | 良好 | ✅ tabled |
| **代码简洁度** | 优秀（derive 宏） | 良好 | ✅ tabled |
| **样式丰富度** | 基础 | 丰富 | comfy-table |
| **自动换行** | 基础 | 优秀 | comfy-table |
| **颜色支持** | 需配合其他库 | 内置 | comfy-table |
| **学习成本** | 低 | 中等 | ✅ tabled |
| **项目匹配度** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ✅ tabled |

**结论**：**推荐使用 `tabled`**，体积更小、性能更好、代码更简洁，完全满足项目需求。

---

## 📋 tabled / comfy-table 使用场景汇总

| 命令 | 文件 | 当前实现 | 表格列 | 优先级 | 推荐库 |
|------|------|----------|--------|--------|--------|
| `workflow pr list` | `src/commands/pr/list.rs` | 简单文本输出 | #, Title, Author, Status, Created | ⭐⭐⭐⭐⭐ | `tabled` |
| `workflow github list` | `src/commands/github/github.rs` | 逐行详细信息 | Name, Email, Branch Prefix, Status | ⭐⭐⭐⭐ | `tabled` |
| `workflow branch ignore list` | `src/commands/branch/ignore.rs` | 编号列表 | #, Branch Name | ⭐⭐⭐ | `tabled` |
| `workflow jira info` | `src/commands/jira/info.rs` | 附件列表 | Filename, Size, Type | ⭐⭐⭐ | `tabled` |
| `workflow log search` | `src/commands/log/search.rs` | 搜索结果列表 | File, ID, URL | ⭐⭐⭐ | `tabled` |
| `workflow jira list` | 待实现 | - | Key, Summary, Status, Assignee, Updated | ⭐⭐⭐⭐⭐ | `comfy-table` |

---

## 📋 tabled / comfy-table 使用场景详解

### 1. PR 列表 (`workflow pr list`)

**当前实现** (`src/commands/pr/list.rs`)：
```rust
// 当前：简单文本输出
log_info!("{}", output);  // 直接输出 provider 返回的字符串
```

**使用表格后的效果**：
```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct PRRow {
    #[tabled(rename = "#")]
    number: u32,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Author")]
    author: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Created")]
    created: String,
}

// 转换 PR 数据为表格
let rows: Vec<PRRow> = prs.iter().map(|pr| {
    PRRow {
        number: pr.number,
        title: pr.title.clone(),
        author: pr.author.clone(),
        status: pr.status.clone(),
        created: pr.created_at.format("%Y-%m-%d").to_string(),
    }
}).collect();

println!("{}", Table::new(rows));
```

**展示效果**：
```
+--------+------------------+--------+--------+------------+
| #      | Title            | Author | Status | Created    |
+--------+------------------+--------+--------+------------+
| 123    | Fix bug          | Alice  | Open   | 2024-01-15 |
| 124    | Add feature      | Bob    | Merged | 2024-01-14 |
| 125    | Update docs      | Carol  | Closed | 2024-01-13 |
+--------+------------------+--------+--------+------------+
```

---

### 2. GitHub 账号列表 (`workflow github list`)

**当前实现** (`src/commands/github/github.rs::list()`)：
```rust
// 当前：逐行显示每个账号的详细信息
for (index, account) in github.accounts.iter().enumerate() {
    log_break!('-', 40, &format!("Account {}: {}{}", index + 1, account.name, current_marker));
    log_message!("  Name: {}", account.name);
    log_message!("  Email: {}", account.email);
    log_message!("  API Token: {}", mask_sensitive_value(&account.api_token));
    // ...
}
```

**使用表格后的效果**：
```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct GitHubAccountRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Email")]
    email: String,
    #[tabled(rename = "Branch Prefix")]
    branch_prefix: String,
    #[tabled(rename = "Status")]
    status: String,  // "current" 或 ""
}

let rows: Vec<GitHubAccountRow> = github.accounts.iter().map(|account| {
    let is_current = /* 判断逻辑 */;
    GitHubAccountRow {
        name: account.name.clone(),
        email: account.email.clone(),
        branch_prefix: account.branch_prefix.clone().unwrap_or_default(),
        status: if is_current { "✓ current".to_string() } else { "".to_string() },
    }
}).collect();

println!("{}", Table::new(rows));
```

**展示效果**：
```
+----------+------------------+---------------+-----------+
| Name     | Email            | Branch Prefix | Status   |
+----------+------------------+---------------+-----------+
| work     | work@example.com | work/         | ✓ current|
| personal | personal@ex.com  |               |          |
+----------+------------------+---------------+-----------+
```

---

### 3. 分支忽略列表 (`workflow branch ignore list`)

**当前实现** (`src/commands/branch/ignore.rs::list()`)：
```rust
// 当前：简单编号列表
for (index, branch) in ignore_branches.iter().enumerate() {
    log_info!("  {}. {}", index + 1, branch);
}
```

**使用表格后的效果**：
```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct BranchRow {
    #[tabled(rename = "#")]
    index: usize,
    #[tabled(rename = "Branch Name")]
    name: String,
}

let rows: Vec<BranchRow> = ignore_branches.iter()
    .enumerate()
    .map(|(i, branch)| BranchRow {
        index: i + 1,
        name: branch.clone(),
    })
    .collect();

println!("{}", Table::new(rows));
```

**展示效果**：
```
+-----+------------------+
| #   | Branch Name      |
+-----+------------------+
| 1   | main             |
| 2   | develop          |
| 3   | release/v1.0.0   |
+-----+------------------+
```

---

### 4. JIRA Ticket 附件列表 (`workflow jira info`)

**当前实现** (`src/commands/jira/info.rs`)：
```rust
// 当前：逐行显示附件信息
if let Some(attachments) = &issue.fields.attachment {
    if !attachments.is_empty() {
        for attachment in attachments {
            log_message!("  - {} ({})", attachment.filename, format_size(attachment.size));
        }
    }
}
```

**使用表格后的效果**：
```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct AttachmentRow {
    #[tabled(rename = "Filename")]
    filename: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Type")]
    content_type: String,
}

let rows: Vec<AttachmentRow> = attachments.iter().map(|att| {
    AttachmentRow {
        filename: att.filename.clone(),
        size: format_size(att.size),
        content_type: att.mime_type.clone().unwrap_or_default(),
    }
}).collect();

println!("Attachments:");
println!("{}", Table::new(rows));
```

**展示效果**：
```
Attachments:
+------------------+--------+------------------+
| Filename         | Size   | Type             |
+------------------+--------+------------------+
| api.log          | 2.5 MB | text/plain      |
| flutter-api.log  | 1.2 MB | text/plain      |
+------------------+--------+------------------+
```

---

### 5. 日志搜索结果 (`workflow log search`)

**当前实现** (`src/commands/log/search.rs`)：
```rust
// 当前：逐行显示搜索结果
for entry in api_results {
    if let Some(id) = entry.id {
        if let Some(url) = entry.url {
            log_message!("URL: {}, ID: {}", url, id);
        }
    }
}
```

**使用表格后的效果**：
```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct SearchResultRow {
    #[tabled(rename = "File")]
    file: String,
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "URL")]
    url: String,
}

let mut rows = Vec::new();
for entry in api_results {
    rows.push(SearchResultRow {
        file: "api.log".to_string(),
        id: entry.id.unwrap_or_default(),
        url: entry.url.unwrap_or_default(),
    });
}
// 同样处理 flutter_api_results

println!("{}", Table::new(rows));
```

**展示效果**：
```
+-------------+--------+----------------------------------+
| File        | ID     | URL                             |
+-------------+--------+----------------------------------+
| api.log     | req-1  | https://api.example.com/endpoint|
| api.log     | req-2  | https://api.example.com/endpoint|
| flutter-api | req-3  | https://api.example.com/endpoint|
+-------------+--------+----------------------------------+
```

---

### 6. 未来场景：JIRA Ticket 列表 (`workflow jira list`)

**计划实现**（参考 `docs/todo/JIRA_TODO.md`）：
```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct JiraTicketRow {
    #[tabled(rename = "Key")]
    key: String,
    #[tabled(rename = "Summary")]
    summary: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Assignee")]
    assignee: String,
    #[tabled(rename = "Updated")]
    updated: String,
}

// 从 JIRA API 获取 tickets
let tickets = jira.search_tickets(project, filters)?;
let rows: Vec<JiraTicketRow> = tickets.iter().map(|ticket| {
    JiraTicketRow {
        key: ticket.key.clone(),
        summary: ticket.summary.clone(),
        status: ticket.status.clone(),
        assignee: ticket.assignee.clone().unwrap_or_default(),
        updated: ticket.updated.format("%Y-%m-%d").to_string(),
    }
}).collect();

println!("{}", Table::new(rows));
```

**展示效果**：
```
+---------+------------------+-------------+----------+------------+
| Key     | Summary          | Status      | Assignee | Updated    |
+---------+------------------+-------------+----------+------------+
| PROJ-123| Fix bug          | In Progress | Alice    | 2024-01-15 |
| PROJ-124| Add feature      | Open        | Bob      | 2024-01-14 |
| PROJ-125| Update docs      | Done        | Carol    | 2024-01-13 |
+---------+------------------+-------------+----------+------------+
```

---

## 🔄 indicatif vs spinners 对比分析

### 功能对比

| 功能 | indicatif | spinners | 说明 |
|------|-----------|----------|------|
| **Spinner 支持** | ✅ 支持 | ✅ 支持 | 两者都支持 |
| **进度条支持** | ✅ 支持 | ❌ 不支持 | indicatif 更全面 |
| **动画样式数量** | 基础样式 | 60+ 种 | spinners 样式更多 |
| **自定义样式** | ✅ 支持 | ✅ 支持 | 两者都支持 |
| **消息更新** | ✅ 支持 | ✅ 支持 | 两者都支持 |
| **多任务支持** | ✅ MultiProgress | ❌ 不支持 | indicatif 更强大 |
| **项目依赖** | ✅ 已在使用 | ❌ 需新增 | **关键差异** |
| **体积影响** | 0 KB（已存在） | ~50-100 KB | **关键差异** |

---

### 代码对比

#### 使用 indicatif（推荐）⭐

```rust
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// 创建 spinner
let spinner = ProgressBar::new_spinner();
spinner.set_style(
    ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
);
spinner.enable_steady_tick(Duration::from_millis(100));
spinner.set_message("Loading...".to_string());

// 执行操作...
// ...

// 完成
spinner.finish_with_message("Done!");
```

**优点**：
- ✅ 无需新增依赖
- ✅ 与现有进度条代码风格一致
- ✅ 可以轻松切换到进度条模式

---

#### 使用 spinners（不推荐）

```rust
use spinners::{Spinner, Spinners};

// 创建 spinner
let mut sp = Spinner::new(Spinners::Dots, "Loading...".into());

// 执行操作...
// ...

// 完成
sp.stop();
```

**缺点**：
- ❌ 需要新增依赖（~50-100 KB）
- ❌ 功能与 indicatif 重复
- ❌ 增加维护成本

---

### 实际使用场景

#### 场景 1：API 请求等待

**indicatif 方式**：
```rust
let spinner = ProgressBar::new_spinner();
spinner.set_style(ProgressStyle::default_spinner()
    .template("{spinner:.green} {msg}")
    .unwrap());
spinner.enable_steady_tick(Duration::from_millis(100));
spinner.set_message("Fetching data...".to_string());

let response = http_client.get(url).send()?;

spinner.finish_with_message("Data fetched!");
```

**spinners 方式**：
```rust
let sp = Spinner::new(Spinners::Dots, "Fetching data...".into());
let response = http_client.get(url).send()?;
sp.stop();
```

**结论**：indicatif 功能足够，无需新增依赖。

---

#### 场景 2：不确定进度的任务

**indicatif 方式**：
```rust
let spinner = ProgressBar::new_spinner();
spinner.set_style(ProgressStyle::default_spinner()
    .template("{spinner:.green} [{elapsed_precise}] {msg}")
    .unwrap());
spinner.enable_steady_tick(Duration::from_millis(100));
spinner.set_message("Processing...".to_string());

// 执行任务...
process_data()?;

spinner.finish_with_message("Processing complete!");
```

**优势**：可以显示已用时间，spinners 需要额外代码。

---

#### 场景 3：需要切换到进度条

**indicatif 方式**：
```rust
// 开始时使用 spinner
let pb = ProgressBar::new_spinner();
pb.set_message("Starting...".to_string());

// 获取到大小后切换到进度条
if let Some(size) = total_size {
    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::default_bar()...);
    // 使用进度条
} else {
    // 继续使用 spinner
}
```

**优势**：可以在 spinner 和进度条之间无缝切换。

---

### 体积影响分析

#### 如果使用 spinners

**新增依赖**：
- `spinners` (~50-100 KB)
- **总计新增**：~50-100 KB

#### 如果使用 indicatif（推荐）

**新增依赖**：
- 无（项目已使用）
- **总计新增**：0 KB

**结论**：使用 indicatif 可以节省 ~50-100 KB 体积。

---

### 最终建议

**强烈推荐使用 `indicatif` 的 spinner 功能** ⭐⭐⭐⭐⭐

**理由**：
1. ✅ **无需新增依赖**：项目已在使用 `indicatif`
2. ✅ **功能完整**：支持 spinner 和进度条
3. ✅ **体积优势**：节省 ~50-100 KB
4. ✅ **统一风格**：与现有代码保持一致
5. ✅ **灵活切换**：可以在 spinner 和进度条之间切换
6. ✅ **功能足够**：满足所有项目需求

**不推荐使用 `spinners`**：
- ❌ 需要新增依赖
- ❌ 功能与 indicatif 重复
- ❌ 增加维护成本
- ❌ 除非需要非常特定的动画样式

**项目当前使用情况**：
- `src/commands/lifecycle/update.rs` 已使用 `ProgressBar::new_spinner()`
- 证明 indicatif 的 spinner 功能已足够使用

---

## 🎯 选择建议：tabled vs comfy-table

### 📦 包体积分析

#### tabled 依赖分析

**直接依赖**（tabled 0.14+）：
- `papergrid` - 表格网格计算（轻量级）
- `tabled-derive` - 派生宏支持（编译时）
- 可选依赖：
  - `csv` - CSV 格式支持（可选）
  - `json` - JSON 格式支持（可选）
  - `html` - HTML 格式支持（可选）

**特点**：
- ✅ 支持 `no_std` 模式（可显著减小体积）
- ✅ `CompactTable` 针对最小内存和 CPU 使用优化
- ✅ 核心功能轻量级，可选功能按需启用
- ✅ 无运行时依赖（大部分功能在编译时完成）

**估算体积影响**：
- 基础表格功能：~50-100 KB（未压缩二进制）
- 包含派生宏：~100-150 KB
- 启用所有格式支持：~200-300 KB

---

#### comfy-table 依赖分析

**直接依赖**（comfy-table 7.0+）：
- `ansi_str` - ANSI 字符串处理
- `crossterm` - 跨平台终端操作（**已在项目中使用**）
- `itertools` - 迭代器工具
- `pad` - 字符串填充
- `serde` - 序列化支持（**已在项目中使用**）
- `thiserror` - 错误类型定义

**特点**：
- ⚠️ 需要标准库（不支持 `no_std`）
- ⚠️ 依赖 `crossterm`（但项目已使用，无额外成本）
- ✅ 依赖 `serde`（但项目已使用，无额外成本）
- ⚠️ 包含更多运行时功能（动态内容处理、自动换行）

**估算体积影响**：
- 基础表格功能：~150-250 KB（未压缩二进制）
- 包含所有功能：~250-400 KB
- **注意**：`crossterm` 和 `serde` 已在项目中，实际增加量可能更小

---

### 📊 体积对比总结

| 指标 | tabled | comfy-table | 说明 |
|------|--------|-------------|------|
| **基础体积** | ~50-100 KB | ~150-250 KB | 未压缩二进制 |
| **完整功能** | ~200-300 KB | ~250-400 KB | 包含所有可选功能 |
| **依赖数量** | 少（2-3 个核心） | 多（6 个直接依赖） | 但部分已存在 |
| **no_std 支持** | ✅ 支持 | ❌ 不支持 | tabled 可更小 |
| **运行时开销** | 低 | 中等 | comfy-table 有更多动态处理 |
| **编译时间** | 快 | 中等 | tabled 更简单 |

---

### 💡 对项目的影响分析

#### 当前项目依赖情况

**已存在的依赖**（可复用）：
- ✅ `crossterm` - comfy-table 需要，但项目已使用
- ✅ `serde` - comfy-table 需要，但项目已使用

**新增依赖**：

**如果选择 tabled**：
- `tabled` (~50-100 KB)
- `papergrid` (~20-30 KB)
- `tabled-derive` (编译时，不影响运行时体积)
- **总计新增**：~70-130 KB

**如果选择 comfy-table**：
- `comfy-table` (~150-250 KB)
- `ansi_str` (~30-50 KB)
- `itertools` (~40-60 KB)
- `pad` (~20-30 KB)
- `thiserror` (~10-20 KB)
- **总计新增**：~250-410 KB
- **但**：`crossterm` 和 `serde` 已存在，实际可能更少

---

### 🎯 推荐建议

#### 推荐使用 `tabled` ⭐⭐⭐⭐⭐

**理由**：

1. **体积更小**：
   - 基础功能仅增加 ~70-130 KB
   - 支持 `no_std` 模式，可进一步优化
   - 无运行时依赖

2. **性能更好**：
   - `CompactTable` 针对性能优化
   - 编译时生成，运行时开销小

3. **代码更简洁**：
   - 使用 `#[derive(Tabled)]` 自动生成
   - API 更简单，学习成本低

4. **功能足够**：
   - 满足项目所有表格展示需求
   - 支持多种输出格式（可选）

5. **项目匹配度高**：
   - 所有使用场景都是简单列表展示
   - 不需要复杂的样式和动态处理

**适用场景**：
- ✅ PR 列表
- ✅ GitHub 账号列表
- ✅ 分支忽略列表
- ✅ JIRA 附件列表
- ✅ 日志搜索结果

---

#### 考虑使用 `comfy-table` 的场景

**仅在以下情况考虑**：

1. **需要状态颜色标识**：
   - 例如：JIRA ticket 状态需要不同颜色
   - 但可以通过 `tabled` + `colored`/`console` 实现

2. **需要复杂的动态样式**：
   - 例如：根据数据动态调整边框样式
   - 项目当前不需要此功能

3. **需要自动换行处理**：
   - 例如：长文本自动换行
   - 项目当前数据较简单，不需要

---

### 📝 最终建议

**推荐方案：统一使用 `tabled`**

**原因**：
1. ✅ **体积优势明显**：比 comfy-table 小 50-70%
2. ✅ **性能更好**：编译时优化，运行时开销小
3. ✅ **代码更简洁**：`#[derive(Tabled)]` 一行搞定
4. ✅ **功能足够**：满足所有项目需求
5. ✅ **维护成本低**：单一库，统一风格

**实施步骤**：
1. 添加 `tabled = "0.14"` 到 `Cargo.toml`
2. 逐步替换列表输出为表格格式
3. 统一使用 `tabled` 保持一致性

**如果未来需要复杂样式**：
- 可以同时引入 `comfy-table`（按需使用）
- 或使用 `tabled` + `colored`/`console` 实现颜色

---

### 📊 体积影响估算（最终二进制）

**当前项目二进制大小**（估算）：
- Release 模式：~5-10 MB（未压缩）

**添加 tabled 后**：
- 增加：~70-130 KB（约 1-2%）
- 最终大小：~5.1-10.1 MB

**添加 comfy-table 后**：
- 增加：~250-410 KB（约 3-5%）
- 最终大小：~5.3-10.4 MB

**结论**：两种方案对最终二进制大小的影响都很小，但 `tabled` 更优。

---

## 🔍 代码使用统计

### colored 使用情况
- **核心文件**：`src/lib/base/util/logger.rs`（所有日志函数）
- **影响范围**：整个项目的日志输出

### dialoguer 使用情况
- **Input**：12+ 个文件
- **Select**：9+ 个文件
- **MultiSelect**：4+ 个文件
- **Confirm**：2 个文件

### indicatif 使用情况
- **ProgressBar**：1 个文件（`src/commands/lifecycle/update.rs`）

---

## 🔍 crossterm 和 ratatui 移除分析

### 快速决策表

| 场景 | ratatui | crossterm | 代码目录 | 体积影响 | 推荐度 |
|------|---------|-----------|----------|----------|--------|
| **完全移除 TUI** | ❌ 移除 | ❌ 移除 | 移除 `src/lib/base/ui/` | 节省 ~700-1100 KB | ⭐⭐⭐⭐⭐ |
| **保留但禁用** | ⚠️ 注释 | ⚠️ 注释 | 保留但不用 | 无影响（不编译） | ⭐⭐⭐⭐ |
| **继续使用 TUI** | ✅ 保留 | ✅ 保留 | 保留并使用 | 增加 ~700-1100 KB | ⭐⭐⭐ |

**结论**：如果恢复传统 CLI 模式，**应该移除 `crossterm` 和 `ratatui`**。

---

### 当前使用情况

**crossterm 使用位置**：
- `src/lib/base/ui/dialogs.rs` - ratatui 对话框组件
- `src/lib/base/ui/progress.rs` - ratatui 进度条组件
- `src/lib/base/ui/` 目录下的所有文件（都是 ratatui 相关）

**ratatui 使用位置**：
- `src/lib/base/ui/` 目录（整个目录）
- `src/lib/logging/` 目录（如果与 ratatui 集成）

**关键发现**：
- ✅ `crossterm` **只**在 ratatui 相关代码中使用
- ✅ 没有其他地方直接使用 `crossterm`
- ✅ `ratatui` 依赖 `crossterm` 作为后端

---

### 移除决策

#### 如果恢复传统 CLI 模式（不使用 TUI）

**应该移除**：
1. ✅ `ratatui = "0.27"` - TUI 框架
2. ✅ `crossterm = "0.28"` - ratatui 的后端，不需要 TUI 时不需要
3. ✅ `src/lib/base/ui/` 目录 - 所有 ratatui 相关代码
4. ✅ `src/lib/logging/` 目录 - 如果与 ratatui 集成

**移除步骤**：
```toml
# Cargo.toml
# 移除或注释
# ratatui = "0.27"
# crossterm = "0.28"
```

```bash
# 移除 TUI 相关代码目录
rm -rf src/lib/base/ui/
# 如果 logging 与 ratatui 集成，也需要处理
```

**体积影响**：
- 移除 `ratatui`：节省 ~500-800 KB
- 移除 `crossterm`：节省 ~200-300 KB
- **总计节省**：~700-1100 KB

---

#### 如果保留 TUI 功能（未来可能使用）

**保留但注释**：
```toml
# UI Framework - 可选 TUI 功能
# ratatui = "0.27"  # 可选，用于未来 TUI 功能
# crossterm = "0.28"  # 可选，ratatui 的后端
```

**代码处理**：
- 保留 `src/lib/base/ui/` 目录
- 但确保传统 CLI 代码不依赖这些模块
- 使用条件编译或特性标志控制

---

### 推荐方案

**方案 1：完全移除 TUI（推荐用于快速恢复）** ⭐⭐⭐⭐⭐

**优点**：
- ✅ 体积更小（节省 ~700-1100 KB）
- ✅ 依赖更少，编译更快
- ✅ 代码更简洁，维护更容易

**缺点**：
- ⚠️ 未来如果需要 TUI，需要重新添加

**适用场景**：
- 确定不需要 TUI 功能
- 优先考虑体积和性能
- 快速恢复传统 CLI

---

**方案 2：保留但禁用（推荐用于灵活方案）** ⭐⭐⭐⭐

**优点**：
- ✅ 保留未来扩展的可能性
- ✅ 可以逐步迁移

**缺点**：
- ⚠️ 即使不使用也会增加编译时间
- ⚠️ 代码更复杂

**适用场景**：
- 未来可能需要 TUI
- 希望保持灵活性

---

### 最终建议

**如果确定不需要 TUI**：
1. ✅ 移除 `ratatui` 和 `crossterm` 依赖
2. ✅ 移除 `src/lib/base/ui/` 目录
3. ✅ 恢复传统 CLI 依赖（`colored`, `dialoguer`, `indicatif`）

**如果可能使用 TUI**：
1. ✅ 保留但注释 `ratatui` 和 `crossterm`
2. ✅ 保留 `src/lib/base/ui/` 目录
3. ✅ 使用特性标志控制编译

---

## ⚠️ 注意事项

1. **向后兼容**：
   - 恢复传统 CLI 后，确保所有现有功能正常工作
   - 如果引入新库，确保 API 兼容或提供迁移路径

2. **测试覆盖**：
   - 恢复后需要全面测试所有交互式命令
   - 特别关注用户输入、选择、确认等功能

3. **文档更新**：
   - 如果引入新库，更新相关文档
   - 更新开发指南

4. **性能考虑**：
   - 新库可能增加编译时间和二进制大小
   - 评估对用户体验的影响

5. **TUI 移除**：
   - 如果移除 TUI，确保所有相关代码都被移除
   - 检查是否有其他地方引用了 TUI 模块

---

## 📚 参考资源

### 库文档和官方地址

#### 1. inquire - 现代化交互式提示
- **文档**: https://docs.rs/inquire/
- **GitHub**: https://github.com/mikaelmello/inquire
- **crates.io**: https://crates.io/crates/inquire
- **最新版本**: 0.9.1
- **特点**: 支持模糊搜索、验证、自动补全、编辑器模式、日期选择等

#### 2. console - 终端输出增强
- **文档**: https://docs.rs/console/
- **GitHub**: https://github.com/console-rs/console
- **crates.io**: https://crates.io/crates/console
- **特点**: 比 `colored` 功能更丰富，支持表格、emoji、样式等

#### 3. tabled - 表格输出
- **文档**: https://docs.rs/tabled/
- **GitHub**: https://github.com/zhiburt/tabled
- **crates.io**: https://crates.io/crates/tabled
- **特点**: 从 Rust 结构体和枚举轻松创建表格，支持多种数据格式（JSON、CSV、TOML、HTML）

#### 4. comfy-table - 表格输出（替代方案）
- **文档**: https://docs.rs/comfy-table/
- **GitHub**: https://github.com/Nukesor/comfy-table
- **crates.io**: https://crates.io/crates/comfy-table
- **特点**: 美观的终端表格，自动内容换行，支持 ANSI 样式、自定义边框等

#### 5. spinners - 加载动画
- **文档**: https://docs.rs/spinners/
- **GitHub**: https://github.com/console-rs/spinners
- **crates.io**: https://crates.io/crates/spinners
- **最新版本**: 4.1.1
- **特点**: 提供 60+ 种优雅的终端加载动画

#### 6. indicatif - 进度条（已在使用）
- **文档**: https://docs.rs/indicatif/
- **GitHub**: https://github.com/console-rs/indicatif
- **crates.io**: https://crates.io/crates/indicatif
- **特点**: 进度条、加载动画、基本颜色支持，支持与 `log` 和 `tracing` 集成

### 相关文档
- `docs/todo/ui-framework-recommendations.md` - UI 框架推荐
- `.git-separation-plan.md` - Git 分支分离计划

---

## ✅ 下一步行动

### 📋 执行恢复方案

**详细恢复方案**：请参考 [`CLI_RESTORE_PLAN.md`](./CLI_RESTORE_PLAN.md)

该方案包含：
- ✅ 分步骤的详细操作指南
- ✅ 代码修复示例
- ✅ 验证清单
- ✅ 问题排查指南
- ✅ 回滚方案

### 快速开始

1. **立即执行**（按照恢复方案）：
   - [ ] 创建备份分支
   - [ ] 恢复 `Cargo.toml` 中的传统 CLI 依赖
   - [ ] 移除 TUI 相关代码和依赖
   - [ ] 验证编译通过
   - [ ] 运行测试确保功能正常

2. **短期计划**（1-2 天）：
   - [ ] 评估是否引入 `inquire`
   - [ ] 评估是否引入 `tabled`
   - [ ] 制定增强计划

3. **长期计划**（1-2 周）：
   - [ ] 逐步迁移到增强库
   - [ ] 统一 UI 风格
   - [ ] 完善文档
