# UI 框架推荐文档

## 📋 概述

本文档介绍可用于提升 Workflow CLI 用户体验的 Rust 终端 UI 框架，帮助让脚本输出更加美观和交互友好。

---

## 🎨 当前使用的 UI 库

项目目前已经使用了以下库：

- **`colored`** (v2.1) - 终端颜色输出
- **`dialoguer`** (v0.11) - 交互式提示（Input, Select, Confirm）
- **`indicatif`** (v0.17) - 进度条显示

这些库提供了基础的交互功能，但可以进一步优化。

---

## 🚀 推荐的 UI 框架

### 1. Ratatui（强烈推荐）⭐

**GitHub**: https://github.com/ratatui-org/ratatui
**文档**: https://ratatui.rs/

#### 特点

- ✅ **功能强大**：支持复杂的布局、表格、图表、交互式组件
- ✅ **社区活跃**：最流行的 Rust TUI 框架
- ✅ **性能优秀**：高效的渲染引擎
- ✅ **跨平台**：支持 Windows、macOS、Linux
- ✅ **文档完善**：有丰富的示例和文档

#### 适用场景

- 交互式 PR/JIRA ticket 浏览器
- 实时日志查看器
- 交互式命令选择器
- 数据可视化（表格、图表）
- 多面板界面

#### 安装

```toml
[dependencies]
ratatui = "0.27"
crossterm = "0.28"  # 或 termion（Unix only）
```

#### 示例：交互式 PR 浏览器

```rust
use ratatui::prelude::*;
use ratatui::widgets::*;

fn render_pr_list(prs: &[PullRequest]) -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // 标题
                    Constraint::Min(0),     // PR 列表
                    Constraint::Length(3),  // 状态栏
                ])
                .split(f.size());

            // 标题
            let title = Block::default()
                .title("Pull Requests")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(title, chunks[0]);

            // PR 列表
            let items: Vec<ListItem> = prs.iter()
                .map(|pr| ListItem::new(format!("{} - {}", pr.number, pr.title)))
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow));
            f.render_stateful_widget(list, chunks[1], &mut state);

            // 状态栏
            let status = Paragraph::new("Press 'q' to quit, 'Enter' to view details")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(status, chunks[2]);
        })?;

        // 处理键盘输入...
    }
}
```

#### 引入 Ratatui 的改进

相比当前使用的 `dialoguer` + `indicatif` + `colored` 组合，引入 `ratatui` 将带来以下显著改进：

##### 1. 用户体验提升

**当前问题**：
- `dialoguer::Select` 只能显示简单的列表，无法展示丰富信息
- 输出是线性的，无法同时查看多个信息面板
- 进度条和交互提示是分离的，缺乏统一界面

**Ratatui 改进**：
- ✅ **多面板布局**：可以同时显示 PR 列表、详情、状态栏等多个区域
- ✅ **实时更新**：支持实时刷新界面，无需重新渲染整个屏幕
- ✅ **键盘导航**：流畅的键盘操作（↑↓ 导航、Enter 选择、q 退出）
- ✅ **视觉反馈**：高亮选中项、状态指示、进度可视化

**示例对比**：

```rust
// 当前方式（dialoguer）：只能简单选择
let idx = Select::new()
    .with_prompt("Select PR")
    .items(&pr_titles)
    .interact()?;
// 用户只能看到标题，需要选择后才能查看详情

// Ratatui 方式：同时显示列表和详情
// 左侧：PR 列表（可滚动）
// 右侧：选中 PR 的详细信息（标题、作者、状态、描述等）
// 底部：操作提示
```

##### 2. 信息展示能力

**当前限制**：
- `colored` 只能输出简单的彩色文本
- 无法显示表格、图表等结构化数据
- 长列表需要滚动终端，体验不佳

**Ratatui 改进**：
- ✅ **表格组件**：美观的表格展示（如 JIRA ticket 列表、PR 列表）
- ✅ **图表支持**：可以显示简单的图表（如进度统计、时间线）
- ✅ **分页显示**：大列表可以分页，支持搜索和筛选
- ✅ **富文本渲染**：支持 Markdown 渲染、代码高亮等

**实际应用场景**：

1. **PR 列表命令** (`workflow pr list`)
   - 当前：简单的文本列表输出
   - Ratatui：交互式列表，支持：
     - 实时筛选和搜索
     - 显示 PR 状态（颜色标识）
     - 查看详情无需退出列表
     - 支持批量操作

2. **JIRA 搜索命令** (`workflow jira search`)
   - 当前：文本输出，需要多次命令查看详情
   - Ratatui：多面板界面：
     - 左侧：ticket 列表（可搜索）
     - 右侧：选中 ticket 的详细信息
     - 支持快速操作（打开、复制链接等）

3. **日志查看命令** (`workflow log search`)
   - 当前：一次性输出所有日志
   - Ratatui：实时日志查看器：
     - 自动滚动到最新日志
     - 支持搜索高亮
     - 可以暂停/继续滚动
     - 支持日志级别过滤

##### 3. 交互能力增强

**当前交互方式**：
- `dialoguer::Input`：简单的文本输入
- `dialoguer::Select`：单项选择
- `dialoguer::Confirm`：是/否确认

**Ratatui 改进**：
- ✅ **复杂表单**：支持多字段表单，带验证和提示
- ✅ **多选操作**：支持复选框、多选列表
- ✅ **快捷键支持**：自定义快捷键，提高操作效率
- ✅ **命令模式**：类似 vim 的命令模式（如 `:filter`、`:sort`）

##### 4. 性能优化

**当前问题**：
- 每次交互都需要重新渲染整个输出
- 大量数据输出时终端滚动缓慢
- 无法增量更新

**Ratatui 改进**：
- ✅ **增量渲染**：只更新变化的部分，性能更好
- ✅ **虚拟滚动**：大列表只渲染可见部分
- ✅ **异步更新**：支持后台数据加载，不阻塞 UI
- ✅ **双缓冲**：减少闪烁，提供流畅体验

##### 5. 代码组织改进

**当前架构**：
- UI 逻辑分散在各个命令中
- 输出格式不统一
- 难以复用 UI 组件

**Ratatui 改进**：
- ✅ **组件化设计**：可复用的 UI 组件（如 `PRListWidget`、`TicketDetailWidget`）
- ✅ **状态管理**：清晰的 UI 状态管理（如 `AppState`）
- ✅ **统一风格**：统一的主题和样式系统
- ✅ **易于测试**：UI 组件可以独立测试

**代码示例**：

```rust
// 可复用的 PR 列表组件
pub struct PRListWidget {
    prs: Vec<PullRequest>,
    selected: usize,
}

impl Widget for PRListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 渲染逻辑
    }
}

// 在多个命令中复用
PRListWidget::new(prs).render(area, buf);
```

##### 6. 功能扩展性

**当前限制**：
- 难以添加新功能（如实时更新、多窗口等）
- 输出格式固定，难以自定义

**Ratatui 改进**：
- ✅ **插件化架构**：可以轻松添加新的 UI 组件
- ✅ **主题系统**：支持自定义主题（深色/浅色模式）
- ✅ **布局系统**：灵活的布局系统，支持响应式设计
- ✅ **事件系统**：完善的事件处理，支持鼠标、键盘、窗口调整等

##### 7. 具体改进场景对比

| 功能 | 当前实现 | Ratatui 实现 | 改进效果 |
|------|---------|-------------|---------|
| **PR 列表** | 文本输出，需选择后查看详情 | 多面板界面，实时查看详情 | ⭐⭐⭐⭐⭐ |
| **JIRA 搜索** | 简单列表，多次命令操作 | 交互式浏览器，一键操作 | ⭐⭐⭐⭐⭐ |
| **日志查看** | 一次性输出，难以筛选 | 实时查看器，支持搜索和过滤 | ⭐⭐⭐⭐⭐ |
| **进度显示** | 简单进度条 | 多任务进度面板，详细统计 | ⭐⭐⭐⭐ |
| **配置设置** | 多个独立提示 | 统一表单界面，实时预览 | ⭐⭐⭐⭐ |

##### 8. 迁移成本与收益

**迁移成本**：
- ⚠️ **学习曲线**：需要学习 Ratatui API（中等难度）
- ⚠️ **代码改动**：需要重构部分命令的 UI 层
- ⚠️ **依赖增加**：新增 `ratatui` 和 `crossterm` 依赖

**收益**：
- ✅ **用户体验**：显著提升，特别是复杂操作
- ✅ **开发效率**：组件化后，新功能开发更快
- ✅ **维护性**：统一的 UI 系统，更易维护
- ✅ **扩展性**：为未来功能扩展打下基础

**建议**：
- 采用渐进式迁移策略
- 先为高频使用的命令（如 `pr list`、`jira search`）添加 TUI
- 逐步迁移，完全替换现有 UI 库

#### 集成建议

1. **完全替换**：使用 `ratatui` 完全替换 `dialoguer`、`indicatif`、`colored`
2. **渐进式迁移**：先为特定命令（如 `workflow pr list`）添加 TUI 界面
3. **复用现有逻辑**：TUI 只负责展示，业务逻辑保持不变

---

### 2. Inquire（推荐用于增强交互）

**GitHub**: https://github.com/mikaelmello/inquire
**文档**: https://docs.rs/inquire/

#### 特点

- ✅ **现代化 API**：比 `dialoguer` 更易用
- ✅ **功能丰富**：支持模糊搜索、验证、自动补全
- ✅ **类型安全**：更好的类型系统支持
- ✅ **轻量级**：可以作为 `dialoguer` 的直接替代

#### 适用场景

- 替换现有的 `dialoguer` 交互
- 需要模糊搜索的场景（如选择 JIRA ticket）
- 需要输入验证的场景

#### 安装

```toml
[dependencies]
inquire = "0.7"
```

#### 示例：模糊搜索 JIRA tickets

```rust
use inquire::{Select, Text, validator::Validation};

// 模糊搜索选择
let ticket = Select::new(
    "Select JIRA ticket",
    tickets
)
.with_fuzzy_search(true)  // 启用模糊搜索
.with_page_size(10)
.prompt()?;

// 带验证的输入
let email = Text::new("JIRA Email")
    .with_validator(|input: &str| {
        if input.contains('@') {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Invalid email format".into()))
        }
    })
    .prompt()?;
```

#### 迁移建议

可以逐步将 `dialoguer` 替换为 `inquire`，API 类似但功能更强。

---

### 3. 增强现有库（快速改进）

在不引入新框架的情况下，可以通过以下方式优化现有输出：

#### 3.1 优化 `colored` 输出

```rust
use colored::*;

// 添加更多样式
println!("{}", "Success".green().bold().on_black());
println!("{}", "Error".red().underline());
println!("{}", "Info".blue().italic());
```

#### 3.2 增强 `indicatif` 进度条

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(100);
pb.set_style(
    ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-")
);
```

#### 3.3 使用 `console` 替代 `colored`

`console` 提供更多功能（如表格、emoji 支持）：

```toml
[dependencies]
console = "0.15"
```

```rust
use console::{style, Emoji};

println!("{} {}", Emoji("✅", "✓"), style("Success").green());
```

---

## 📊 框架对比

| 框架 | 复杂度 | 功能 | 适用场景 | 学习曲线 |
|------|--------|------|----------|----------|
| **Ratatui** | 高 | ⭐⭐⭐⭐⭐ | 复杂交互界面 | 中等 |
| **Inquire** | 低 | ⭐⭐⭐⭐ | 交互式提示 | 低 |
| **增强现有** | 低 | ⭐⭐⭐ | 快速改进 | 低 |

---

## 🎯 推荐方案

### 方案 1：渐进式 TUI 集成（推荐）⭐

**目标**：为关键命令添加 TUI 界面，提升用户体验

**步骤**：

1. **第一阶段**：添加 `ratatui` 依赖
   ```toml
   [dependencies]
   ratatui = "0.27"
   crossterm = "0.28"
   ```

2. **第二阶段**：为 `workflow pr list` 添加 TUI 界面
   - 显示 PR 列表
   - 支持键盘导航
   - 支持查看详情

3. **第三阶段**：为 `workflow jira search` 添加 TUI 界面
   - 交互式 ticket 浏览器
   - 支持筛选和搜索

4. **第四阶段**：为 `workflow log search` 添加实时日志查看器

**优势**：
- 显著提升用户体验
- 可以逐步集成，完全替换现有 UI 库
- 统一的 UI 系统，更易维护

---

### 方案 2：增强交互提示（快速改进）

**目标**：用 `inquire` 替换 `dialoguer`，提升交互体验

**步骤**：

1. 添加 `inquire` 依赖
2. 逐步替换 `dialoguer::Input`、`Select` 等
3. 添加模糊搜索功能

**优势**：
- 改动小，风险低
- 快速见效
- 保持现有架构

---

### 方案 3：完全替换方案（推荐用于统一 UI 系统）

**目标**：完全使用 `ratatui` 替换所有 UI 库

**实施**：
- 移除 `colored`、`dialoguer`、`indicatif` 依赖
- 使用 `ratatui` 实现所有 UI 功能（输入、选择、进度条、日志）
- 统一 UI 组件和样式系统
- 简化依赖管理，单一 UI 框架

**优势**：
- 统一的 UI 系统，代码更一致
- 减少依赖数量
- 更好的用户体验（统一的交互方式）
- 更易维护和扩展

---

## 🔧 实施建议

### 优先级 1：快速改进（1-2 天）

1. ✅ 优化现有 `colored` 输出样式
2. ✅ 增强 `indicatif` 进度条显示
3. ✅ 添加更多 emoji 和图标

### 优先级 2：增强交互（1 周）

1. ✅ 引入 `inquire` 替换部分 `dialoguer`
2. ✅ 为选择操作添加模糊搜索
3. ✅ 添加输入验证和自动补全

### 优先级 3：TUI 集成（2-4 周）

1. ✅ 为 `workflow pr list` 添加 TUI
2. ✅ 为 `workflow jira search` 添加 TUI
3. ✅ 为 `workflow log search` 添加实时查看器

---

## 📝 代码示例

### 示例 1：使用 Inquire 增强选择

```rust
// 替换前（dialoguer）
let selection = Select::new()
    .with_prompt("Select option")
    .items(&options)
    .interact()?;

// 替换后（inquire）
let selection = Select::new("Select option", options)
    .with_fuzzy_search(true)  // 新增：模糊搜索
    .with_page_size(10)       // 新增：分页
    .with_help_message("Use arrow keys and type to search")
    .prompt()?;
```

### 示例 2：使用 Ratatui 创建 PR 列表

```rust
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn show_pr_list(prs: Vec<PullRequest>) -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut selected = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // 创建布局
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // 标题
                    Constraint::Min(0),     // 列表
                    Constraint::Length(1),  // 状态
                ])
                .split(size);

            // 标题
            let title = Paragraph::new("Pull Requests")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // PR 列表
            let items: Vec<ListItem> = prs.iter()
                .enumerate()
                .map(|(i, pr)| {
                    let style = if i == selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!("#{} - {}", pr.number, pr.title)).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow));
            f.render_stateful_widget(list, chunks[1], &mut ListState::default().with_selected(Some(selected)));

            // 状态栏
            let status = Paragraph::new("↑↓ Navigate | Enter: View | q: Quit")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(status, chunks[2]);
        })?;

        // 处理输入...
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => selected = selected.saturating_sub(1),
                KeyCode::Down => selected = (selected + 1).min(prs.len().saturating_sub(1)),
                KeyCode::Char('q') => break,
                KeyCode::Enter => {
                    // 查看详情
                    show_pr_details(&prs[selected])?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
```

---

## 🔗 相关资源

- [Ratatui 官方文档](https://ratatui.rs/)
- [Ratatui 示例](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [Inquire 文档](https://docs.rs/inquire/)
- [Rust TUI 应用示例集合](https://github.com/ratatui-org/ratatui/wiki/Projects-using-ratatui)

---

## 🔄 迁移到 Ratatui 的完整清单

如果决定全部迁移到 `ratatui`，以下是需要修改的所有内容：

### 1. 依赖项修改

**文件**: `Cargo.toml`

**当前依赖**：
```toml
colored = "2.1"
dialoguer = "0.11"
indicatif = "0.17"
```

**修改为**：
```toml
ratatui = "0.27"
crossterm = "0.28"  # 或 termion（仅 Unix）
```

**说明**：
- `colored` → 由 `ratatui` 的样式系统替代
- `dialoguer` → 由 `ratatui` 的交互组件替代
- `indicatif` → 由 `ratatui` 的进度条组件替代

---

### 2. 核心工具模块修改

#### 2.1 日志系统 (`src/lib/base/util/logger.rs`)

**当前实现**：
- 使用 `colored::*` 进行颜色输出
- 使用 `println!` 宏输出日志

**需要修改**：
- ✅ **保留现有 API**：保持 `log_*!` 宏接口不变
- ✅ **内部实现**：改为使用 `ratatui` 的样式系统
- ✅ **统一输出**：所有日志输出使用 `ratatui` 的样式和渲染

**修改内容**：
```rust
// 当前
use colored::*;
println!("{}", "Success".green());

// 修改后
// 使用 ratatui 的样式系统
use ratatui::style::{Color, Style};
let style = Style::default().fg(Color::Green);
// 在 ratatui 终端中渲染，或转换为 ANSI 颜色码输出
```

**影响范围**：
- 所有使用 `log_*!` 宏的代码（整个项目）
- 约 566 行代码需要重构

---

#### 2.2 确认对话框 (`src/lib/base/util/confirm.rs`)

**当前实现**：
```rust
use dialoguer::Confirm;
Confirm::new().with_prompt(prompt).interact()?;
```

**需要修改**：
- ✅ 创建 `ratatui` 确认对话框组件
- ✅ 支持键盘操作（Y/N、Enter/Esc）
- ✅ 保持函数签名不变（API 兼容）

**修改内容**：
```rust
// 完全替换为 ratatui 实现
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    // 使用 ratatui 实现确认对话框
    // 显示对话框，处理键盘输入
    show_confirm_dialog(prompt, default)
}
```

**影响范围**：
- `src/lib/base/util/confirm.rs` (45 行)
- 所有调用 `confirm()` 的地方

---

#### 2.3 进度条 (`src/commands/lifecycle/update.rs`)

**当前实现**：
```rust
use indicatif::{ProgressBar, ProgressStyle};
let pb = ProgressBar::new(size);
pb.set_style(...);
pb.set_position(downloaded_bytes);
```

**需要修改**：
- ✅ 创建 `ratatui` 进度条组件
- ✅ 支持实时更新
- ✅ 支持多任务进度显示

**修改内容**：
```rust
// 创建 ratatui 进度条 widget
struct ProgressWidget {
    current: u64,
    total: u64,
    message: String,
}

impl Widget for ProgressWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 渲染进度条
    }
}
```

**影响范围**：
- `src/commands/lifecycle/update.rs` (约 50 行进度条相关代码)

---

### 3. 命令模块修改

#### 3.1 使用 `dialoguer::Input` 的命令

**需要修改的文件**（共 10 个文件）：

1. **`src/commands/jira/info.rs`**
   - 交互式输入 JIRA ticket ID
   - 修改：使用 `ratatui` 输入框组件

2. **`src/commands/jira/attachments.rs`**
   - 交互式输入 JIRA ticket ID
   - 修改：使用 `ratatui` 输入框组件

3. **`src/commands/jira/clean.rs`**
   - 交互式输入 JIRA ticket ID
   - 修改：使用 `ratatui` 输入框组件

4. **`src/commands/log/download.rs`**
   - 交互式输入 JIRA ticket ID
   - 修改：使用 `ratatui` 输入框组件

5. **`src/commands/log/find.rs`**
   - 交互式输入 JIRA ticket ID 和 request ID
   - 修改：使用 `ratatui` 表单组件

6. **`src/commands/log/search.rs`**
   - 交互式输入 JIRA ticket ID 和搜索词
   - 修改：使用 `ratatui` 表单组件

7. **`src/commands/github/helpers.rs`**
   - 交互式输入 GitHub token
   - 修改：使用 `ratatui` 输入框组件

8. **`src/commands/pr/create.rs`**
   - 交互式输入 JIRA ticket、标题、描述
   - 修改：使用 `ratatui` 表单组件

9. **`src/commands/pr/pick.rs`**
   - 交互式输入分支名
   - 修改：使用 `ratatui` 输入框组件

10. **`src/commands/branch/ignore.rs`**
    - 交互式输入分支名
    - 修改：使用 `ratatui` 输入框组件

**修改模式**：
```rust
// 当前
let input: String = Input::new()
    .with_prompt("Enter Jira ticket ID")
    .interact_text()?;

// 修改后
let input = show_input_dialog("Enter Jira ticket ID", None)?;
```

---

#### 3.2 使用 `dialoguer::Select` 的命令

**需要修改的文件**（共 8 个文件）：

1. **`src/commands/config/helpers.rs`**
   - 语言选择
   - 修改：使用 `ratatui` 列表选择组件

2. **`src/commands/config/setup.rs`**
   - Provider 选择、语言选择
   - 修改：使用 `ratatui` 列表选择组件

3. **`src/commands/config/completion.rs`**
   - Shell 类型选择（MultiSelect）
   - 修改：使用 `ratatui` 多选列表组件

4. **`src/commands/config/log.rs`**
   - 日志级别选择
   - 修改：使用 `ratatui` 列表选择组件

5. **`src/commands/llm/setup.rs`**
   - Provider 选择、语言选择
   - 修改：使用 `ratatui` 列表选择组件

6. **`src/commands/github/github.rs`**
   - GitHub 操作选择
   - 修改：使用 `ratatui` 列表选择组件

7. **`src/commands/pr/sync.rs`**
   - 同步操作选择
   - 修改：使用 `ratatui` 列表选择组件

8. **`src/commands/pr/rebase.rs`**
   - Rebase 操作选择
   - 修改：使用 `ratatui` 列表选择组件

9. **`src/lib/jira/status.rs`**
   - JIRA 状态选择
   - 修改：使用 `ratatui` 列表选择组件

**修改模式**：
```rust
// 当前
let idx = Select::new()
    .with_prompt("Select option")
    .items(&options)
    .interact()?;

// 修改后
let idx = show_select_dialog("Select option", &options, None)?;
```

---

#### 3.3 使用 `dialoguer::MultiSelect` 的命令

**需要修改的文件**（共 3 个文件）：

1. **`src/commands/pr/create.rs`**
   - 变更类型多选
   - 修改：使用 `ratatui` 多选列表组件

2. **`src/commands/pr/pick.rs`**
   - 分支多选
   - 修改：使用 `ratatui` 多选列表组件

3. **`src/commands/branch/ignore.rs`**
   - 分支多选
   - 修改：使用 `ratatui` 多选列表组件

**修改模式**：
```rust
// 当前
let selected = MultiSelect::new()
    .with_prompt("Select items")
    .items(&items)
    .interact()?;

// 修改后
let selected = show_multi_select_dialog("Select items", &items)?;
```

---

#### 3.4 使用 `dialoguer::Confirm` 的命令

**需要修改的文件**（共 2 个文件）：

1. **`src/lib/base/http/retry.rs`**
   - 重试确认对话框
   - 修改：使用 `ratatui` 确认对话框组件

2. **`src/lib/base/util/confirm.rs`**
   - 通用确认函数（已在 2.2 中列出）

---

### 4. 需要创建的新组件

为了统一管理 `ratatui` 组件，建议创建以下模块：

#### 4.1 UI 组件模块 (`src/lib/base/ui/`)

```
src/lib/base/ui/
├── mod.rs              # 模块入口
├── components.rs       # 通用组件（输入框、选择列表等）
├── dialogs.rs          # 对话框组件（确认、输入、选择等）
├── progress.rs         # 进度条组件
├── layout.rs           # 布局辅助函数
└── theme.rs            # 主题和样式定义
```

**组件列表**：

1. **`InputDialog`** - 文本输入对话框
2. **`SelectDialog`** - 单选列表对话框
3. **`MultiSelectDialog`** - 多选列表对话框
4. **`ConfirmDialog`** - 确认对话框
5. **`ProgressBar`** - 进度条组件
6. **`FormDialog`** - 多字段表单对话框
7. **`ListWidget`** - 可滚动列表组件
8. **`TableWidget`** - 表格组件

---

### 5. 需要重构的命令（适合 TUI 界面）

以下命令适合完全重构为 TUI 界面：

#### 5.1 高优先级（用户体验提升显著）

1. **`workflow pr list`** (`src/commands/pr/list.rs`)
   - 当前：简单文本输出
   - 重构为：交互式 PR 列表浏览器
   - 功能：列表、详情、筛选、搜索

2. **`workflow jira search`** (如果存在)
   - 重构为：交互式 JIRA ticket 浏览器
   - 功能：列表、详情、筛选、搜索

3. **`workflow log search`** (`src/commands/log/search.rs`)
   - 重构为：实时日志查看器
   - 功能：实时更新、搜索高亮、过滤

#### 5.2 中优先级

4. **`workflow config setup`** (`src/commands/config/setup.rs`)
   - 重构为：统一配置表单界面
   - 功能：多字段表单、实时预览、验证

5. **`workflow llm setup`** (`src/commands/llm/setup.rs`)
   - 重构为：LLM 配置表单界面
   - 功能：多字段表单、实时预览

---

### 6. 非交互式终端处理

#### 6.1 检测非交互式终端

**策略**：检测是否为 TTY，非交互式终端使用简化输出

```rust
use std::io::{self, IsTerminal};

// 检测是否为交互式终端
if !io::stdout().is_terminal() {
    // 非交互式终端（如 CI/CD），使用简化输出
    // 仍然使用 ratatui，但渲染为纯文本格式
    return show_simplified_output();
}
```

#### 6.2 简化输出模式

**处理**：
- 非交互式终端：使用 `ratatui` 渲染为纯文本，不显示交互界面
- 交互式终端：正常显示完整的 TUI 界面
- 所有输出都通过 `ratatui` 统一处理

```rust
// 统一使用 ratatui，根据终端类型调整渲染方式
let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

if io::stdout().is_terminal() {
    // 交互式：显示完整 TUI
    show_full_tui(&mut terminal)?;
} else {
    // 非交互式：渲染为纯文本输出
    show_text_output(&mut terminal)?;
}
```

---

### 7. 修改统计

| 类别 | 文件数量 | 代码行数（估算） | 复杂度 |
|------|---------|----------------|--------|
| **依赖项** | 1 | 3 行 | 低 |
| **核心工具模块** | 3 | ~200 行 | 中 |
| **命令模块（Input）** | 10 | ~100 行 | 低 |
| **命令模块（Select）** | 9 | ~150 行 | 中 |
| **命令模块（MultiSelect）** | 3 | ~50 行 | 中 |
| **命令模块（Confirm）** | 2 | ~30 行 | 低 |
| **新组件创建** | 1 个模块 | ~500 行 | 高 |
| **命令重构（TUI）** | 5 | ~1000 行 | 高 |
| **总计** | **33+** | **~2000+ 行** | **中-高** |

---

### 8. 迁移步骤建议

#### 阶段 1：基础设施（1-2 周）
1. ✅ 添加 `ratatui` 和 `crossterm` 依赖
2. ✅ 移除 `colored`、`dialoguer`、`indicatif` 依赖
3. ✅ 创建 UI 组件模块 (`src/lib/base/ui/`)
4. ✅ 实现基础组件（输入框、选择列表、确认对话框）
5. ✅ 重构 `confirm.rs` 使用新组件
6. ✅ 重构日志系统使用 `ratatui` 样式

#### 阶段 2：简单交互替换（1 周）
1. ✅ 替换所有 `dialoguer::Input` 为 `ratatui` 输入框
2. ✅ 替换所有 `dialoguer::Select` 为 `ratatui` 选择列表
3. ✅ 替换所有 `dialoguer::MultiSelect` 为 `ratatui` 多选列表
4. ✅ 替换所有 `dialoguer::Confirm` 为 `ratatui` 确认对话框

#### 阶段 3：进度条替换（3-5 天）
1. ✅ 创建 `ratatui` 进度条组件
2. ✅ 替换 `indicatif` 进度条

#### 阶段 4：复杂界面重构（2-4 周）
1. ✅ 重构 `workflow pr list` 为 TUI
2. ✅ 重构 `workflow log search` 为实时查看器
3. ✅ 重构配置命令为表单界面

#### 阶段 5：测试和优化（1 周）
1. ✅ 全面测试所有功能
2. ✅ 性能优化
3. ✅ 文档更新

**总时间估算**：6-10 周

---

### 9. 风险评估

#### 高风险项
- ⚠️ **日志系统重构**：影响面广，需要仔细测试
- ⚠️ **非交互式终端处理**：需要确保 CI/CD 环境正常工作
- ⚠️ **完全替换风险**：所有 UI 输出都需要迁移，不能回退

#### 中风险项
- ⚠️ **学习曲线**：团队需要学习 `ratatui` API
- ⚠️ **测试覆盖**：TUI 组件需要新的测试方法
- ⚠️ **迁移不可逆**：一旦替换，无法回退到旧 UI 库

#### 低风险项
- ✅ **依赖管理**：`ratatui` 稳定可靠
- ✅ **性能**：`ratatui` 性能优秀

---

### 10. 迁移注意事项

#### 10.1 完全替换策略

**原则**：
- ✅ 完全移除 `colored`、`dialoguer`、`indicatif` 依赖
- ✅ 所有 UI 输出统一使用 `ratatui`
- ✅ 保持 API 接口不变，只修改内部实现
- ✅ 非交互式终端使用简化输出（仍通过 `ratatui` 渲染）

#### 10.2 测试策略

**重点测试**：
- ✅ 所有交互式命令的功能测试
- ✅ 非交互式终端（CI/CD）的输出测试
- ✅ 不同终端尺寸的适配测试
- ✅ 键盘操作和快捷键测试

#### 10.3 回退方案

如果迁移过程中遇到问题：
- 可以在迁移完成前保留旧代码分支
- 但一旦完成迁移，建议完全移除旧依赖
- 不建议同时维护两套 UI 系统

---

## ✅ 总结

1. **短期**：使用 `inquire` 增强交互提示，快速提升体验
2. **中期**：为关键命令添加 `ratatui` TUI 界面
3. **长期**：逐步为所有适合的命令添加 TUI 支持

**推荐从 `inquire` 开始**，因为它：
- 改动小，风险低
- 可以立即提升交互体验
- 为后续 TUI 集成打下基础

**如果决定全部迁移到 `ratatui`**：
- 预计需要修改 **33+ 个文件**
- 新增代码约 **2000+ 行**
- 总时间约 **6-10 周**
- **完全替换策略**：移除所有旧 UI 库，统一使用 `ratatui`
- 建议采用**渐进式迁移**策略，但最终目标是完全替换

---

## 🔍 Tracing + Ratatui 组合方案

### 概述

**可以使用 `tracing` + `ratatui`！** 这是一个非常强大的组合：

- ✅ **`tracing`**：Rust 生态系统中流行的结构化日志库
- ✅ **`ratatui`**：终端 UI 框架
- ✅ **结合优势**：结构化日志 + 实时日志查看器

### 为什么使用 Tracing + Ratatui？

#### 1. 结构化日志

**当前问题**：
- 自定义日志系统，功能有限
- 难以进行日志过滤和分析
- 不支持 span（追踪代码执行路径）

**Tracing 优势**：
- ✅ **结构化日志**：支持字段、元数据
- ✅ **Span 追踪**：可以追踪函数调用链
- ✅ **事件过滤**：基于字段、级别、模块过滤
- ✅ **生态丰富**：大量工具和集成

#### 2. 实时日志查看器

**实现效果**：
- 实时显示日志流
- 支持日志级别过滤（Error、Warn、Info、Debug）
- 支持搜索和高亮
- 支持按模块过滤
- 可暂停/继续滚动

#### 3. 更好的开发体验

- 开发时：使用 `tracing` 结构化日志
- 运行时：通过 `ratatui` 实时查看日志
- 调试时：可以过滤和搜索日志

---

### 实现方案

#### 方案 1：Tracing + Ratatui 日志查看器（推荐）

**架构**：
```
应用代码
  ↓ (使用 tracing::info! 等)
Tracing Subscriber
  ↓ (收集日志)
Ratatui 日志查看器
  ↓ (显示在终端)
用户界面
```

**依赖**：
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
ratatui = "0.27"
crossterm = "0.28"
```

**实现示例**：

1. **创建 Tracing Subscriber** (`src/lib/base/ui/logging.rs`)：

```rust
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::sync::{Arc, Mutex};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::Registry;

/// 日志条目
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: Level,
    pub message: String,
    pub module: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 日志缓冲区
#[derive(Clone)]
pub struct LogBuffer {
    entries: Arc<Mutex<Vec<LogEntry>>>,
    max_entries: usize,
}

impl LogBuffer {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            max_entries,
        }
    }

    pub fn add_entry(&self, entry: LogEntry) {
        let mut entries = self.entries.lock().unwrap();
        entries.push(entry);

        // 限制缓冲区大小
        if entries.len() > self.max_entries {
            entries.remove(0);
        }
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}

/// Ratatui Tracing Layer
pub struct RatatuiLayer {
    buffer: LogBuffer,
}

impl RatatuiLayer {
    pub fn new(max_entries: usize) -> Self {
        Self {
            buffer: LogBuffer::new(max_entries),
        }
    }

    pub fn buffer(&self) -> LogBuffer {
        self.buffer.clone()
    }
}

impl<S: Subscriber> Layer<S> for RatatuiLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut message = String::new();
        event.record(&mut MessageVisitor(&mut message));

        let entry = LogEntry {
            level: *event.metadata().level(),
            message,
            module: event.metadata().module_path().map(|s| s.to_string()),
            timestamp: chrono::Utc::now(),
        };

        self.buffer.add_entry(entry);
    }
}

struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.0 = format!("{:?}", value);
        }
    }
}
```

2. **创建日志查看器组件** (`src/lib/base/ui/log_viewer.rs`)：

**LogViewer 的作用**：

`LogViewer` 是一个**交互式日志查看器组件**，它的主要作用是：

1. **实时显示日志**：从 `LogBuffer` 中读取日志条目，实时显示在终端界面
2. **交互式操作**：支持键盘操作（滚动、过滤、搜索、暂停等）
3. **日志过滤**：可以按日志级别（Error、Warn、Info、Debug）过滤显示
4. **搜索功能**：支持搜索关键词，快速定位日志
5. **暂停/继续**：可以暂停自动滚动，方便查看历史日志
6. **美观展示**：使用不同颜色显示不同级别的日志，提高可读性

**使用场景**：
- 开发调试时实时查看应用日志
- 运行长时间任务时监控日志输出
- 排查问题时搜索和过滤相关日志
- 查看特定模块或级别的日志

```rust
use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::base::ui::logging::{LogBuffer, LogEntry};
use tracing::Level;
use crossterm::event::{Event, KeyCode, KeyEventKind};

/// 日志查看器组件
///
/// 功能：
/// - 实时显示日志流
/// - 支持按级别过滤（Error、Warn、Info、Debug）
/// - 支持搜索关键词
/// - 支持暂停/继续自动滚动
/// - 支持手动滚动查看历史日志
pub struct LogViewer {
    buffer: LogBuffer,
    filter_level: Option<Level>,
    search_query: String,
    scroll_offset: usize,
    paused: bool,
}

impl LogViewer {
    pub fn new(buffer: LogBuffer) -> Self {
        Self {
            buffer,
            filter_level: None,
            search_query: String::new(),
            scroll_offset: 0,
            paused: false,
        }
    }

    pub fn show(&mut self) -> Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        terminal.clear()?;

        loop {
            terminal.draw(|f| {
                self.render(f);
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if self.handle_key(key.code)? {
                            break;
                        }
                    }
                }
            }

            // 自动滚动（如果未暂停）
            if !self.paused {
                self.auto_scroll();
            }
        }

        terminal.clear()?;
        Ok(())
    }

    fn render(&mut self, f: &mut Frame) {
        let area = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 标题和过滤
                Constraint::Min(0),     // 日志列表
                Constraint::Length(3),   // 状态栏
            ])
            .split(area);

        // 标题栏
        let title = Paragraph::new("Log Viewer")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // 日志列表
        let entries = self.get_filtered_entries();
        let items: Vec<ListItem> = entries
            .iter()
            .skip(self.scroll_offset)
            .take(chunks[1].height as usize - 2)
            .map(|entry| {
                let level_color = match entry.level {
                    Level::ERROR => Color::Red,
                    Level::WARN => Color::Yellow,
                    Level::INFO => Color::Cyan,
                    Level::DEBUG => Color::Gray,
                    Level::TRACE => Color::DarkGray,
                };

                let time_str = entry.timestamp.format("%H:%M:%S%.3f").to_string();
                let module_str = entry.module.as_deref().unwrap_or("unknown");
                let text = format!(
                    "[{}] [{}] {}: {}",
                    time_str,
                    entry.level.as_str(),
                    module_str,
                    entry.message
                );

                ListItem::new(text).style(Style::default().fg(level_color))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Logs"));
        f.render_widget(list, chunks[1]);

        // 状态栏
        let status = format!(
            "Level: {:?} | Search: {} | Paused: {} | ↑↓ Scroll | q: Quit | f: Filter | s: Search",
            self.filter_level,
            if self.search_query.is_empty() { "None" } else { &self.search_query },
            self.paused
        );
        let status_widget = Paragraph::new(status)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status_widget, chunks[2]);
    }

    fn handle_key(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('f') => {
                // 切换日志级别过滤
                self.filter_level = match self.filter_level {
                    None => Some(Level::ERROR),
                    Some(Level::ERROR) => Some(Level::WARN),
                    Some(Level::WARN) => Some(Level::INFO),
                    Some(Level::INFO) => Some(Level::DEBUG),
                    Some(Level::DEBUG) => None,
                    _ => None,
                };
            }
            KeyCode::Char('s') => {
                // 搜索模式（简化实现）
                // 实际可以使用 InputDialog
            }
            KeyCode::Char(' ') => {
                self.paused = !self.paused;
            }
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            _ => {}
        }
        Ok(false)
    }

    fn get_filtered_entries(&self) -> Vec<LogEntry> {
        let entries = self.buffer.get_entries();
        entries
            .into_iter()
            .filter(|entry| {
                // 级别过滤
                if let Some(filter_level) = self.filter_level {
                    match entry.level {
                        Level::ERROR => filter_level == Level::ERROR,
                        Level::WARN => filter_level <= Level::WARN,
                        Level::INFO => filter_level <= Level::INFO,
                        Level::DEBUG => filter_level <= Level::DEBUG,
                        Level::TRACE => true,
                    }
                } else {
                    true
                }
            })
            .filter(|entry| {
                // 搜索过滤
                if self.search_query.is_empty() {
                    true
                } else {
                    entry.message.to_lowercase().contains(&self.search_query.to_lowercase())
                }
            })
            .collect()
    }

    fn auto_scroll(&mut self) {
        let entries = self.get_filtered_entries();
        if entries.len() > 0 {
            self.scroll_offset = entries.len().saturating_sub(10);
        }
    }
}
```

3. **使用示例**：

```rust
use tracing::{info, error, warn, debug};
use crate::base::ui::logging::RatatuiLayer;
use crate::base::ui::log_viewer::LogViewer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

fn main() -> Result<()> {
    // 创建 Ratatui Layer
    let ratatui_layer = RatatuiLayer::new(1000);
    let buffer = ratatui_layer.buffer();

    // 初始化 tracing
    let subscriber = Registry::default()
        .with(ratatui_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env());

    tracing::subscriber::set_global_default(subscriber)?;

    // 启动日志查看器（在后台线程）
    let buffer_clone = buffer.clone();
    std::thread::spawn(move || {
        let mut viewer = LogViewer::new(buffer_clone);
        viewer.show().unwrap();
    });

    // 使用 tracing 记录日志
    info!("Application started");
    warn!("This is a warning");
    error!("This is an error");
    debug!("Debug information");

    Ok(())
}
```

---

### 迁移策略

#### 方案 A：完全迁移到 Tracing

**步骤**：
1. 添加 `tracing` 和 `tracing-subscriber` 依赖
2. 创建 `RatatuiLayer` 和 `LogViewer`
3. 将所有 `log_*!` 宏替换为 `tracing::*!` 宏
4. 移除自定义日志系统

**优势**：
- ✅ 使用标准库，生态丰富
- ✅ 支持结构化日志
- ✅ 更好的可扩展性

**劣势**：
- ⚠️ 需要修改所有日志调用
- ⚠️ 学习曲线

#### 方案 B：混合方案（推荐）

**步骤**：
1. 保留现有 `log_*!` 宏（向后兼容）
2. 在宏内部使用 `tracing` 记录日志
3. 创建 `RatatuiLayer` 收集日志
4. 提供日志查看器命令

**实现**：

```rust
// 保留现有 API，内部使用 tracing
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}
```

**优势**：
- ✅ 保持 API 兼容
- ✅ 渐进式迁移
- ✅ 可以同时使用两种方式

---

### 为什么需要 LogViewer？

#### 当前日志系统的问题

**现状**：项目使用 `println!` 直接输出日志到终端

```rust
// 当前方式
log_info!("Fetching ticket...");
log_debug!("Request: GET /rest/api/2/issue");
log_warning!("Rate limit approaching");
log_error!("Connection failed");
```

**存在的问题**：

1. **无法交互**
   - 日志一旦输出就无法操作
   - 无法暂停查看历史日志
   - 无法回滚查看之前的日志

2. **难以过滤**
   - 所有日志混在一起，难以区分
   - 无法只查看 Error 或 Debug 级别的日志
   - 无法按模块过滤（如只看 `jira::api` 的日志）

3. **无法搜索**
   - 如果日志很多，需要手动滚动查找
   - 无法快速定位包含特定关键词的日志
   - 调试时难以找到关键信息

4. **信息过载**
   - 长时间运行的任务会产生大量日志
   - 终端滚动太快，无法看清
   - Debug 日志和 Info 日志混在一起

5. **缺乏上下文**
   - 无法同时看到时间线
   - 无法看到日志之间的关系
   - 难以追踪问题发生的顺序

#### LogViewer 解决的问题

**LogViewer 提供**：

1. **交互式界面**
   - ✅ 可以暂停/继续日志流
   - ✅ 可以手动滚动查看历史
   - ✅ 可以实时查看最新日志

2. **强大的过滤功能**
   - ✅ 按级别过滤（只显示 Error、Warn 等）
   - ✅ 按模块过滤（只显示特定模块的日志）
   - ✅ 动态切换过滤条件

3. **搜索功能**
   - ✅ 输入关键词快速定位日志
   - ✅ 高亮显示匹配的日志
   - ✅ 支持正则表达式搜索（可扩展）

4. **更好的可视化**
   - ✅ 不同级别用不同颜色显示
   - ✅ 显示时间戳、模块、级别
   - ✅ 清晰的界面布局

5. **实时监控**
   - ✅ 长时间运行的任务可以实时查看日志
   - ✅ 不会因为日志太多而丢失信息
   - ✅ 可以暂停查看，不会错过关键信息

#### 实际使用场景对比

**场景 1：长时间运行的任务**

**当前方式**：
```bash
$ workflow pr sync
ℹ Fetching PRs...
⚙ Request: GET /repos/owner/repo/pulls
⚙ Processing 100 PRs...
⚙ Checking PR #1...
⚙ Checking PR #2...
... (大量日志快速滚动)
⚙ Checking PR #100...
✓ Sync completed
```
**问题**：日志滚动太快，无法看清中间发生了什么

**使用 LogViewer**：
```bash
$ workflow pr sync
# 自动打开 LogViewer，可以：
# - 暂停查看中间日志
# - 过滤只显示 Error/Warn
# - 搜索特定 PR 编号
# - 查看时间线
```

**场景 2：调试问题**

**当前方式**：
```bash
$ workflow jira search PROJ-123
ℹ Searching...
⚙ API request: GET /rest/api/2/search
⚙ Processing results...
✗ Error: Connection timeout
```
**问题**：无法查看详细的调试信息，不知道哪一步出错

**使用 LogViewer**：
```bash
$ workflow jira search PROJ-123 --debug
# 打开 LogViewer，可以：
# - 查看所有 Debug 日志
# - 搜索 "timeout" 关键词
# - 查看 API 请求的详细信息
# - 追踪问题发生的完整流程
```

**场景 3：监控多个模块**

**当前方式**：
```bash
$ workflow pr create
ℹ Creating PR...
⚙ jira::api: Fetching ticket...
⚙ pr::create: Generating branch name...
⚙ git::branch: Creating branch...
⚙ jira::api: Updating ticket status...
⚙ pr::github: Creating PR...
```
**问题**：所有模块的日志混在一起，难以区分

**使用 LogViewer**：
```bash
$ workflow pr create
# 打开 LogViewer，可以：
# - 只查看 jira::api 模块的日志
# - 只查看 pr::create 模块的日志
# - 对比不同模块的执行时间
```

---

### LogViewer 的两种用途

LogViewer 可以用于两种不同的场景：

#### 用途 1：查看应用运行时日志（实时日志）

**场景**：查看应用运行过程中通过 `tracing` 记录的实时日志

**数据源**：`RatatuiLayer` 收集的运行时日志

**使用方式**：
```bash
# 运行命令时自动显示日志查看器
workflow pr list

# 或者单独启动日志查看器
workflow log viewer
```

#### 用途 2：查看已下载的日志文件（静态日志）⭐

**场景**：查看从 Jira ticket 下载的日志文件（`workflow log download` 下载的日志）

**数据源**：本地文件系统中的日志文件（如 `flutter-api.log`、`api.log`）

**当前问题**：
- 使用 `workflow log search` 只能搜索关键词，无法浏览完整日志
- 使用文本编辑器打开大文件会很慢
- 无法快速过滤和导航

**LogViewer 解决方案**：
```bash
# 查看已下载的日志文件
workflow log view PROJ-123

# 或者指定日志文件路径
workflow log view --file ~/Downloads/jira/PROJ-123/flutter-api.log
```

**功能**：
- ✅ 分页显示大日志文件（不一次性加载全部）
- ✅ 搜索关键词并高亮
- ✅ 按日志级别过滤（如果日志有级别标记）
- ✅ 按模块过滤（如只看特定 API 的日志）
- ✅ 显示行号和上下文
- ✅ 支持多个日志文件切换查看

**实现示例**：

```rust
// 扩展 LogViewer 支持文件模式
pub struct FileLogViewer {
    file_path: PathBuf,
    lines: Vec<String>,
    current_line: usize,
    search_query: String,
}

impl FileLogViewer {
    pub fn from_file(path: PathBuf) -> Result<Self> {
        // 读取文件（可以分块读取，避免大文件内存问题）
        let content = std::fs::read_to_string(&path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        Ok(Self {
            file_path: path,
            lines,
            current_line: 0,
            search_query: String::new(),
        })
    }

    pub fn show(&mut self) -> Result<()> {
        // 使用 ratatui 显示日志文件内容
        // 支持搜索、过滤、滚动等
    }
}
```

**使用场景对比**：

| 场景 | 当前方式 | LogViewer 方式 |
|------|---------|---------------|
| **搜索关键词** | `workflow log search PROJ-123 "error"` | `workflow log view PROJ-123` + 按 `s` 搜索 |
| **查看完整日志** | 用编辑器打开（大文件很慢） | 分页加载，快速浏览 |
| **过滤日志** | 无法过滤 | 按级别、模块过滤 |
| **查看上下文** | 手动滚动 | 自动显示相关上下文 |

---

### 新增 `log view` 命令设计

#### 命令结构

**新增命令**：
```bash
workflow log view [JIRA_ID] [OPTIONS]
```

**现有命令保持不变**：
- `workflow log download [JIRA_ID]` - 下载日志（功能不变）
- `workflow log find [JIRA_ID] [REQUEST_ID]` - 查找请求 ID（功能不变）
- `workflow log search [JIRA_ID] [KEYWORD]` - 搜索关键词（功能不变）

#### 命令职责划分

| 命令 | 用途 | 输出方式 | 适用场景 |
|------|------|---------|---------|
| **`log download`** | 下载日志文件 | CLI 文本输出 | 首次下载或更新日志 |
| **`log find`** | 查找特定请求 ID | CLI 文本输出 | 快速查找单个请求的 URL |
| **`log search`** | 搜索关键词 | CLI 文本输出 | 快速搜索，获取匹配列表 |
| **`log view`** ⭐ | 交互式查看日志 | TUI 界面 | 浏览完整日志、详细分析 |

#### 使用场景对比

**场景 1：快速查找请求 ID**
```bash
# 使用 find 命令（快速、精确）
workflow log find PROJ-123 abc123
# 输出：URL: https://..., ID: abc123
```

**场景 2：搜索包含关键词的请求**
```bash
# 使用 search 命令（快速、批量）
workflow log search PROJ-123 "error"
# 输出：所有包含 "error" 的请求列表
```

**场景 3：详细查看和分析日志**
```bash
# 使用 view 命令（交互式、详细）
workflow log view PROJ-123
# 打开 LogViewer，可以：
# - 浏览完整日志内容
# - 搜索关键词并高亮
# - 查看请求的完整上下文
# - 过滤和导航
```

#### 命令参数设计

```rust
/// 查看日志文件（交互式）
pub struct ViewCommand;

impl ViewCommand {
    /// 查看日志文件
    ///
    /// # 参数
    ///
    /// * `jira_id` - JIRA ticket ID（可选，如果未提供则交互式输入）
    /// * `file` - 直接指定日志文件路径（可选，优先级高于 jira_id）
    /// * `file_type` - 指定查看的文件类型（可选：flutter-api.log 或 api.log）
    pub fn view(
        jira_id: Option<String>,
        file: Option<PathBuf>,
        file_type: Option<String>,
    ) -> Result<()> {
        // 1. 确定日志文件路径
        let log_file = if let Some(file_path) = file {
            // 如果指定了文件路径，直接使用
            file_path
        } else {
            // 否则根据 jira_id 查找日志文件
            let jira_id = jira_id
                .or_else(|| {
                    // 交互式输入 JIRA ID
                    InputDialog::new("Enter Jira ticket ID (e.g., PROJ-123)")
                        .show()
                        .ok()
                })
                .context("JIRA ID is required")?;

            let logs = JiraLogs::new()?;

            // 根据 file_type 选择文件
            match file_type.as_deref() {
                Some("api") => logs.get_api_log_file_path(&jira_id)?,
                Some("flutter-api") | None => logs.ensure_log_file_exists(&jira_id)?,
                _ => anyhow::bail!("Invalid file type. Use 'api' or 'flutter-api'"),
            }
        };

        // 2. 创建 FileLogViewer 并显示
        let mut viewer = FileLogViewer::from_file(log_file)?;
        viewer.show()?;

        Ok(())
    }
}
```

#### CLI 参数定义

```rust
/// Log 子命令枚举
#[derive(Subcommand)]
pub enum LogSubcommand {
    /// Download log files from Jira ticket
    Download {
        /// JIRA ticket ID (e.g., PROJ-123)
        jira_id: Option<String>,
    },
    /// Find request ID in log files
    Find {
        /// JIRA ticket ID (e.g., PROJ-123)
        jira_id: Option<String>,
        /// Request ID to find
        request_id: Option<String>,
    },
    /// Search keyword in log files
    Search {
        /// JIRA ticket ID (e.g., PROJ-123)
        jira_id: Option<String>,
        /// Search keyword
        keyword: Option<String>,
    },
    /// View log files interactively (NEW)
    View {
        /// JIRA ticket ID (e.g., PROJ-123)
        #[arg(short, long)]
        jira_id: Option<String>,
        /// Directly specify log file path
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// File type to view: 'api' or 'flutter-api' (default: flutter-api)
        #[arg(short = 't', long = "type")]
        file_type: Option<String>,
    },
}
```

#### 实现文件结构

```
src/commands/log/
├── mod.rs          # Log 命令模块声明
├── download.rs     # 下载日志命令（保持不变）
├── find.rs         # 查找请求 ID 命令（保持不变）
├── search.rs       # 搜索关键词命令（保持不变）
└── view.rs         # 查看日志命令（新增）⭐
```

#### 命令集成

在 `src/bin/workflow.rs` 中添加：

```rust
// Log 命令
Some(Commands::Log { subcommand }) => match subcommand {
    LogSubcommand::Download { jira_id } => {
        log::DownloadCommand::download(jira_id)?;
    }
    LogSubcommand::Find { jira_id, request_id } => {
        log::FindCommand::find(jira_id, request_id)?;
    }
    LogSubcommand::Search { jira_id, keyword } => {
        log::SearchCommand::search(jira_id, keyword)?;
    }
    LogSubcommand::View { jira_id, file, file_type } => {
        log::ViewCommand::view(jira_id, file, file_type)?;
    }
},
```

#### 使用示例

```bash
# 1. 查看指定 JIRA ticket 的日志（默认 flutter-api.log）
workflow log view PROJ-123

# 2. 查看 api.log 文件
workflow log view PROJ-123 --type api

# 3. 直接指定文件路径
workflow log view --file ~/Downloads/jira/PROJ-123/flutter-api.log

# 4. 交互式输入 JIRA ID
workflow log view

# 5. 现有命令保持不变
workflow log download PROJ-123
workflow log find PROJ-123 abc123
workflow log search PROJ-123 "error"
```

#### 命令互补关系

**`log search` vs `log view`**：

- **`log search`**：快速搜索，获取匹配列表
  - 适合：快速查找包含关键词的所有请求
  - 输出：简洁的列表（URL + ID）
  - 场景：需要批量查找时

- **`log view`**：交互式查看，详细分析
  - 适合：需要查看完整上下文、详细分析
  - 输出：交互式界面，可搜索、过滤、导航
  - 场景：深入分析问题时

**工作流程示例**：

```bash
# 步骤 1：下载日志
workflow log download PROJ-123

# 步骤 2：快速搜索包含 "error" 的请求
workflow log search PROJ-123 "error"
# 输出：找到 5 个匹配的请求

# 步骤 3：查看其中一个请求的详细信息
workflow log find PROJ-123 abc123
# 输出：URL: https://...

# 步骤 4：在 LogViewer 中查看完整日志上下文
workflow log view PROJ-123
# 在界面中搜索 "abc123"，查看完整的请求/响应
```

#### 向后兼容性

- ✅ **完全向后兼容**：所有现有命令保持不变
- ✅ **新增功能**：`log view` 是新增命令，不影响现有功能
- ✅ **可选使用**：用户可以选择使用 CLI 命令或 TUI 界面
- ✅ **渐进式采用**：可以逐步从 CLI 命令迁移到 TUI 界面

---

### 使用场景

#### 1. 实时日志查看

**LogViewer 的核心作用**：提供一个**交互式终端界面**，实时显示和管理应用产生的日志。

```bash
# 启动应用，自动显示日志查看器
workflow pr list

# 或者单独启动日志查看器
workflow log viewer
```

**界面效果**：
```
┌─────────────────────────────────────────┐
│          Log Viewer                      │
├─────────────────────────────────────────┤
│ [12:34:56.789] [INFO] jira::api: Fetching ticket... │
│ [12:34:56.790] [DEBUG] jira::api: Request: GET /rest/api/2/issue │
│ [12:34:56.850] [WARN] jira::api: Rate limit approaching │
│ [12:34:56.900] [ERROR] jira::api: Connection failed │
│ [12:34:57.000] [INFO] jira::api: Retrying... │
│ ...                                     │
├─────────────────────────────────────────┤
│ Level: None | Search: None | Paused: false │
│ ↑↓ Scroll | q: Quit | f: Filter | s: Search │
└─────────────────────────────────────────┘
```

#### 2. 日志搜索和过滤

**LogViewer 提供的功能**：

- **按级别过滤**（Error、Warn、Info、Debug）
  - 按 `f` 键切换过滤级别
  - 只显示指定级别及以上的日志

- **按模块过滤**
  - 可以过滤特定模块的日志（如 `jira::api`、`pr::create`）

- **搜索关键词**
  - 按 `s` 键进入搜索模式
  - 输入关键词，只显示包含该关键词的日志

- **导出日志**
  - 可以将当前显示的日志导出到文件

#### 3. 开发调试

**实时日志调试**：
```rust
use tracing::{span, Level};

fn complex_function() {
    let span = span!(Level::DEBUG, "complex_function");
    let _guard = span.enter();

    // 所有在这个函数内的日志都会带有 span 信息
    tracing::debug!("Inside complex function");
}
```

**查看已下载的日志文件**：
```bash
# 下载日志
workflow log download PROJ-123

# 使用 LogViewer 查看（替代文本编辑器）
workflow log view PROJ-123

# 在 LogViewer 中：
# - 按 's' 搜索关键词（如 "error"、"timeout"）
# - 按 'f' 过滤日志级别
# - 使用 ↑↓ 滚动查看
# - 查看完整的请求/响应上下文
```

---

### 依赖对比

**当前方案**：
```toml
colored = "2.1"  # 仅颜色输出
```

**Tracing + Ratatui 方案**：
```toml
tracing = "0.1"
tracing-subscriber = "0.3"
ratatui = "0.27"
crossterm = "0.28"
```

**优势**：
- ✅ 功能更强大（结构化日志 + UI）
- ✅ 生态更丰富
- ✅ 更易扩展

---

### 总结

**推荐使用 `tracing + ratatui`**，因为：

1. ✅ **更好的日志系统**：结构化日志，支持 span、过滤等
2. ✅ **实时日志查看**：通过 ratatui 实现美观的日志查看器
3. ✅ **开发体验**：调试时可以看到实时日志流
4. ✅ **生态丰富**：tracing 是 Rust 标准日志方案
5. ✅ **向后兼容**：可以保留现有 `log_*!` 宏，内部使用 tracing

**实施建议**：
- 采用**混合方案**：保留现有 API，内部使用 tracing
- 创建 `RatatuiLayer` 和 `LogViewer` 组件
- 为需要实时日志的命令添加日志查看器
- 逐步迁移，不影响现有功能

---

## 🚀 UI Framework 重构步骤指南

### 概述

本文档提供从 `colored` + `dialoguer` + `indicatif` 迁移到 `ratatui` + `tracing` 的完整重构步骤。

**重构策略**：渐进式迁移，完全替换，保持 API 兼容

**预计时间**：6-10 周

**影响范围**：33+ 个文件，约 2000+ 行代码

### 📊 当前进度

**已完成阶段**：
- ✅ **阶段 0**：准备工作（1-2 天）
- ✅ **阶段 1**：基础设施搭建（1-2 周）
- ✅ **阶段 2**：日志系统重构（1 周）
- ✅ **阶段 3**：替换简单交互（1 周）

**进行中**：
- ⏳ **阶段 4**：替换进度条（3-5 天）

**待完成**：
- ⏸️ **阶段 5**：实现 LogViewer（1 周）
- ⏸️ **阶段 6**：测试和验证（1 周）
- ⏸️ **阶段 7**：清理和优化（3-5 天）
- ⏸️ **阶段 8**：发布准备（2-3 天）

**完成度**：约 40%（4/8 阶段完成）

**统计数据**：
- 已修改文件：26 个 Rust 文件
- 代码变更：+810 行，-4186 行（净减少 3376 行）
- 所有 `dialoguer` 引用已移除
- 日志模块已重构并移动到 `src/lib/logging/`

---

### 阶段 0：准备工作（1-2 天）✅ 已完成

#### 0.1 创建重构分支

```bash
git checkout -b refactor/ui-framework-ratatui
```

#### 0.2 备份当前状态

```bash
# 确认当前版本 tag 存在（1.4.8 已存在，可直接使用）
git tag | grep "1.4.8"

# 如果需要，可以切换到 1.4.8 tag 确认状态
# git checkout 1.4.8
# git checkout -b refactor/ui-framework-ratatui

# 导出当前依赖列表（用于对比）
cargo tree > dependencies-before.txt

# 导出当前 git 状态
git log --oneline -10 > git-log-before.txt
git status > git-status-before.txt
```

**注意**：如果当前 HEAD 就是 1.4.8，或者 1.4.8 tag 已经标记了重构前的状态，则不需要创建新的备份 tag，可以直接使用现有的 1.4.8 tag 作为备份点。

**状态**：✅ 已完成 - 已创建重构分支 `refactor/ui-framework-ratatui`，已备份依赖列表

#### 0.3 文档准备

- [x] 阅读 `ratatui` 官方文档：https://ratatui.rs/
- [x] 阅读 `tracing` 官方文档：https://docs.rs/tracing/
- [x] 查看示例项目：https://github.com/ratatui-org/ratatui/tree/main/examples

---

### 阶段 1：基础设施搭建（1-2 周）✅ 已完成

#### 1.1 更新依赖项

**文件**：`Cargo.toml`

```toml
[dependencies]
# 移除旧依赖
# colored = "2.1"        # 移除
# dialoguer = "0.11"      # 移除
# indicatif = "0.17"      # 移除

# 添加新依赖
ratatui = "0.27"
crossterm = "0.28"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**检查清单**：
- [x] 更新 `Cargo.toml`
- [ ] 运行 `cargo check` 确认依赖正确（待网络恢复后验证）
- [ ] 运行 `cargo build` 确认编译通过（待网络恢复后验证）

#### 1.2 创建 UI 组件模块结构

**创建目录结构**：

```bash
mkdir -p src/lib/base/ui
touch src/lib/base/ui/mod.rs
touch src/lib/base/ui/components.rs
touch src/lib/base/ui/dialogs.rs
touch src/lib/base/ui/progress.rs
touch src/lib/base/ui/layout.rs
touch src/lib/base/ui/theme.rs
touch src/lib/base/ui/logging.rs
touch src/lib/base/ui/log_viewer.rs
touch src/lib/base/ui/file_log_viewer.rs
```

**文件**：`src/lib/base/ui/mod.rs`

```rust
//! UI 组件模块
//!
//! 提供基于 ratatui 的 UI 组件，包括对话框、进度条、日志查看器等。

pub mod components;
pub mod dialogs;
pub mod progress;
pub mod layout;
pub mod theme;
pub mod logging;
pub mod log_viewer;
pub mod file_log_viewer;

// 重新导出常用组件
pub use dialogs::{InputDialog, SelectDialog, MultiSelectDialog, ConfirmDialog};
pub use progress::ProgressBar;
pub use log_viewer::LogViewer;
pub use file_log_viewer::FileLogViewer;
```

**检查清单**：
- [x] 创建所有文件
- [x] 在 `src/lib/base/mod.rs` 中添加 `pub mod ui;`
- [ ] 运行 `cargo check` 确认模块结构正确（待网络恢复后验证）

#### 1.3 实现主题和样式

**文件**：`src/lib/base/ui/theme.rs`

```rust
use ratatui::style::{Color, Style, Modifier};

/// 应用主题
pub struct Theme;

impl Theme {
    /// 成功消息样式（绿色）
    pub fn success() -> Style {
        Style::default().fg(Color::Green)
    }

    /// 错误消息样式（红色）
    pub fn error() -> Style {
        Style::default().fg(Color::Red)
    }

    /// 警告消息样式（黄色）
    pub fn warning() -> Style {
        Style::default().fg(Color::Yellow)
    }

    /// 信息消息样式（青色）
    pub fn info() -> Style {
        Style::default().fg(Color::Cyan)
    }

    /// 调试消息样式（灰色）
    pub fn debug() -> Style {
        Style::default().fg(Color::Gray)
    }

    /// 标题样式
    pub fn title() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    /// 高亮样式
    pub fn highlight() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }
}
```

**检查清单**：
- [x] 实现主题模块
- [ ] 运行 `cargo check` 确认编译通过（待网络恢复后验证）

#### 1.4 实现基础对话框组件

**文件**：`src/lib/base/ui/dialogs.rs`

按照文档中的示例实现：
- [x] `InputDialog` - 文本输入对话框（已扩展支持验证、默认值、允许空值）
- [x] `SelectDialog` - 单选列表对话框
- [x] `MultiSelectDialog` - 多选列表对话框
- [x] `ConfirmDialog` - 确认对话框

**检查清单**：
- [x] 实现所有对话框组件
- [ ] 编写单元测试（待后续补充）
- [ ] 运行 `cargo test` 确认测试通过（待网络恢复后验证）

#### 1.5 实现进度条组件

**文件**：`src/lib/base/ui/progress.rs`

按照文档中的示例实现 `ProgressBar` 组件。

**检查清单**：
- [x] 实现进度条组件
- [ ] 测试进度更新功能（待网络恢复后验证）
- [ ] 运行 `cargo test` 确认测试通过（待网络恢复后验证）

---

### 阶段 2：日志系统重构（1 周）✅ 已完成

#### 2.1 创建 Tracing Layer

**文件**：`src/lib/base/ui/logging.rs`

按照文档中的示例实现 `RatatuiLayer` 和 `LogBuffer`。

**检查清单**：
- [x] 实现 `RatatuiLayer`（已移动到 `src/lib/logging/tracing.rs`）
- [x] 实现 `LogBuffer`（已移动到 `src/lib/logging/tracing.rs`）
- [ ] 测试日志收集功能（待网络恢复后验证）
- [ ] 运行 `cargo test` 确认测试通过（待网络恢复后验证）

#### 2.2 重构 Logger 模块

**文件**：`src/lib/base/util/logger.rs`

**修改策略**：
1. 保留所有 `log_*!` 宏的 API
2. 内部实现改为使用 `tracing`
3. 将 `colored` 样式转换为 `ratatui` 样式（用于 CLI 输出）

**修改示例**：

```rust
// 修改前
use colored::*;
pub fn print_success(message: impl fmt::Display) {
    println!("{}", message.to_string().green());
}

// 修改后
use tracing::info;
use crate::base::ui::theme::Theme;

pub fn print_success(message: impl fmt::Display) {
    // 使用 tracing 记录
    info!("{}", message);

    // CLI 输出（转换为 ANSI 颜色码）
    let styled = format_with_style(
        message.to_string(),
        Theme::success()
    );
    println!("{}", styled);
}
```

**检查清单**：
- [x] 重构所有 `Logger` 方法（已移动到 `src/lib/logging/logger.rs`）
- [x] 保持所有宏的 API 不变
- [ ] 测试所有日志级别输出（待网络恢复后验证）
- [ ] 运行 `cargo test` 确认测试通过（待网络恢复后验证）
- [ ] 运行现有命令确认日志输出正常（待网络恢复后验证）

#### 2.3 初始化 Tracing

**文件**：`src/bin/workflow.rs`

在 `main()` 函数中初始化 tracing：

```rust
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use crate::base::ui::logging::RatatuiLayer;

fn main() -> Result<()> {
    // 初始化 tracing（可选，如果启用 LogViewer）
    let ratatui_layer = RatatuiLayer::new(1000);
    let subscriber = Registry::default()
        .with(ratatui_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env());

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    // ... 原有代码
}
```

**检查清单**：
- [x] 添加 tracing 初始化代码
- [ ] 测试日志记录功能（待网络恢复后验证）
- [ ] 确认不影响现有功能（待网络恢复后验证）

---

### 阶段 3：替换简单交互（1 周）✅ 已完成

#### 3.1 替换 `dialoguer::Input`

**需要修改的文件**（10 个）：
- `src/commands/jira/info.rs`
- `src/commands/jira/attachments.rs`
- `src/commands/jira/clean.rs`
- `src/commands/log/download.rs`
- `src/commands/log/find.rs`
- `src/commands/log/search.rs`
- `src/commands/github/helpers.rs`
- `src/commands/pr/create.rs`
- `src/commands/pr/pick.rs`
- `src/commands/branch/ignore.rs`

**修改模式**：

```rust
// 修改前
use dialoguer::Input;
let input: String = Input::new()
    .with_prompt("Enter Jira ticket ID")
    .interact()?;

// 修改后
use crate::base::ui::dialogs::InputDialog;
let input = InputDialog::new("Enter Jira ticket ID")
    .show()?;
```

**检查清单**：
- [x] 逐个文件替换 `dialoguer::Input`（已完成 12+ 个文件）
- [ ] 测试每个命令的交互功能（待网络恢复后验证）
- [ ] 确认功能正常（待网络恢复后验证）

#### 3.2 替换 `dialoguer::Select`

**需要修改的文件**（9 个）：
- `src/commands/config/helpers.rs`
- `src/commands/config/setup.rs`
- `src/commands/config/completion.rs`
- `src/commands/config/log.rs`
- `src/commands/llm/setup.rs`
- `src/commands/github/github.rs`
- `src/commands/pr/sync.rs`
- `src/commands/pr/rebase.rs`
- `src/lib/jira/status.rs`

**修改模式**：

```rust
// 修改前
use dialoguer::Select;
let idx = Select::new()
    .with_prompt("Select option")
    .items(&options)
    .interact()?;

// 修改后
use crate::base::ui::dialogs::SelectDialog;
let idx = SelectDialog::new("Select option", &options)
    .with_default(current_idx)
    .show()?;
```

**检查清单**：
- [x] 逐个文件替换 `dialoguer::Select`（已完成 9 个文件）
- [ ] 测试每个命令的选择功能（待网络恢复后验证）
- [ ] 确认功能正常（待网络恢复后验证）

#### 3.3 替换 `dialoguer::MultiSelect`

**需要修改的文件**（3 个）：
- `src/commands/pr/create.rs`
- `src/commands/pr/pick.rs`
- `src/commands/branch/ignore.rs`

**检查清单**：
- [x] 逐个文件替换 `dialoguer::MultiSelect`（已完成 4 个文件）
- [ ] 测试每个命令的多选功能（待网络恢复后验证）
- [ ] 确认功能正常（待网络恢复后验证）

#### 3.4 替换 `dialoguer::Confirm`

**需要修改的文件**（2 个）：
- `src/lib/base/util/confirm.rs`
- `src/lib/base/http/retry.rs`

**检查清单**：
- [x] 替换 `confirm.rs` 中的实现
- [x] 替换 `retry.rs` 中的确认对话框
- [ ] 测试确认功能（待网络恢复后验证）
- [ ] 确认功能正常（待网络恢复后验证）

---

### 阶段 4：替换进度条（3-5 天）

#### 4.1 替换 `indicatif::ProgressBar`

**需要修改的文件**：
- `src/commands/lifecycle/update.rs`

**修改模式**：

```rust
// 修改前
use indicatif::{ProgressBar, ProgressStyle};
let pb = ProgressBar::new(size);
pb.set_style(...);
pb.set_position(downloaded_bytes);

// 修改后
use crate::base::ui::progress::ProgressBar;
let mut progress = ProgressBar::new(size, "Downloading...");
progress.update(downloaded_bytes)?;
```

**检查清单**：
- [x] 替换进度条实现
- [ ] 测试下载进度显示
- [ ] 确认进度更新正常

---

### 阶段 5：实现 LogViewer（1 周）

#### 5.1 实现实时日志查看器

**文件**：`src/lib/base/ui/log_viewer.rs`

按照文档中的示例实现 `LogViewer` 组件。

**检查清单**：
- [ ] 实现 `LogViewer` 组件
- [ ] 测试实时日志显示
- [ ] 测试过滤和搜索功能
- [ ] 运行 `cargo test` 确认测试通过

#### 5.2 实现文件日志查看器

**文件**：`src/lib/base/ui/file_log_viewer.rs`

实现 `FileLogViewer` 组件，用于查看已下载的日志文件。

**检查清单**：
- [ ] 实现 `FileLogViewer` 组件
- [ ] 实现分页加载（大文件）
- [ ] 测试搜索和过滤功能
- [ ] 运行 `cargo test` 确认测试通过

#### 5.3 实现 `log view` 命令

**文件**：`src/commands/log/view.rs`

按照文档中的设计实现 `ViewCommand`。

**检查清单**：
- [ ] 创建 `view.rs` 文件
- [ ] 实现 `ViewCommand`
- [ ] 在 `mod.rs` 中导出
- [ ] 在 CLI 中添加命令
- [ ] 测试命令功能

---

### 阶段 6：测试和验证（1 周）

#### 6.1 功能测试

**测试清单**：
- [ ] 测试所有交互式命令（Input、Select、Confirm）
- [ ] 测试进度条显示
- [ ] 测试日志输出（所有级别）
- [ ] 测试 LogViewer（实时日志）
- [ ] 测试 FileLogViewer（文件日志）
- [ ] 测试 `log view` 命令

#### 6.2 兼容性测试

**测试清单**：
- [ ] 测试非交互式终端（CI/CD 环境）
- [ ] 测试不同终端尺寸
- [ ] 测试不同操作系统（macOS、Linux、Windows）
- [ ] 测试大文件处理（日志文件）

#### 6.3 性能测试

**测试清单**：
- [ ] 测试大日志文件的加载性能
- [ ] 测试实时日志的更新性能
- [ ] 测试 UI 渲染性能

#### 6.4 回归测试

**测试清单**：
- [ ] 运行所有现有测试
- [ ] 测试所有命令的基本功能
- [ ] 确认没有功能回退

---

### 阶段 7：清理和优化（3-5 天）

#### 7.1 移除旧依赖

**文件**：`Cargo.toml`

```toml
# 确认以下依赖已完全移除
# colored = "2.1"        # 已移除
# dialoguer = "0.11"      # 已移除
# indicatif = "0.17"      # 已移除
```

**检查清单**：
- [ ] 确认所有旧依赖已移除
- [ ] 运行 `cargo build` 确认编译通过
- [ ] 运行 `cargo test` 确认测试通过

#### 7.2 代码清理

**检查清单**：
- [ ] 移除所有 `use colored::*` 导入
- [ ] 移除所有 `use dialoguer::*` 导入
- [ ] 移除所有 `use indicatif::*` 导入
- [ ] 运行 `cargo clippy` 检查代码质量
- [ ] 运行 `cargo fmt` 格式化代码

#### 7.3 文档更新

**检查清单**：
- [ ] 更新 README.md（如果有 UI 相关说明）
- [ ] 更新架构文档
- [ ] 更新使用示例
- [ ] 添加新功能的文档

---

### 阶段 8：发布准备（2-3 天）

#### 8.1 最终检查

**检查清单**：
- [ ] 所有功能测试通过
- [ ] 所有单元测试通过
- [ ] 代码审查完成
- [ ] 文档更新完成
- [ ] 性能测试通过

#### 8.2 版本更新

**文件**：`Cargo.toml`

```toml
[package]
version = "1.5.0"  # 更新版本号
```

#### 8.3 提交和合并

```bash
# 提交所有更改
git add .
git commit -m "refactor: migrate to ratatui + tracing

- Replace colored with ratatui styles
- Replace dialoguer with ratatui dialogs
- Replace indicatif with ratatui progress bar
- Add tracing for structured logging
- Add LogViewer for real-time log viewing
- Add FileLogViewer for viewing downloaded logs
- Add log view command

BREAKING CHANGE: UI framework completely replaced"

# 推送到远程
git push origin refactor/ui-framework-ratatui

# 创建 Pull Request
```

---

### 重构检查清单总结

#### 依赖管理
- [x] 添加 `ratatui` 和 `crossterm`
- [x] 添加 `tracing` 和 `tracing-subscriber`
- [ ] 移除 `colored`、`dialoguer`、`indicatif`（已注释，待阶段 7 完全移除）
- [ ] 更新 `Cargo.lock`（待网络恢复后运行 cargo build）

#### 组件实现
- [x] 创建 UI 模块结构
- [x] 实现主题和样式
- [x] 实现所有对话框组件（InputDialog、SelectDialog、MultiSelectDialog、ConfirmDialog）
- [x] 实现进度条组件
- [ ] 实现 LogViewer（阶段 5）
- [ ] 实现 FileLogViewer（阶段 5）

#### 代码迁移
- [x] 重构日志系统（logger.rs，已移动到 `src/lib/logging/logger.rs`）
- [x] 替换所有 `dialoguer::Input`（已完成 12+ 个文件）
- [x] 替换所有 `dialoguer::Select`（已完成 9 个文件）
- [x] 替换所有 `dialoguer::MultiSelect`（已完成 4 个文件）
- [x] 替换所有 `dialoguer::Confirm`（已完成 2 个文件）
- [ ] 替换 `indicatif::ProgressBar`（1 个文件，阶段 4）

#### 新功能
- [ ] 实现 `log view` 命令
- [ ] 集成 LogViewer
- [ ] 集成 FileLogViewer

#### 测试和验证
- [ ] 所有功能测试通过
- [ ] 所有单元测试通过
- [ ] 兼容性测试通过
- [ ] 性能测试通过
- [ ] 回归测试通过

#### 清理和优化
- [ ] 移除所有旧依赖引用
- [ ] 代码格式化
- [ ] 代码质量检查
- [ ] 文档更新

---

### 常见问题处理

#### Q1: 编译错误 - 找不到模块

**解决方案**：
```bash
# 清理并重新构建
cargo clean
cargo build
```

#### Q2: 测试失败 - 非交互式终端

**解决方案**：
在测试中检测是否为 TTY，非交互式终端使用简化输出。

#### Q3: 性能问题 - 大文件加载慢

**解决方案**：
实现分页加载，只加载可见部分。

---

### 回退方案

如果重构过程中遇到严重问题：

1. **保留重构分支**：不删除重构分支
2. **创建修复分支**：从重构分支创建修复分支
3. **逐步修复**：逐个修复问题
4. **如果无法修复**：可以回退到备份标签

```bash
# 回退到重构前状态
git checkout backup-before-ui-refactor
```

---

### 时间线估算

| 阶段 | 任务 | 时间 |
|------|------|------|
| 阶段 0 | 准备工作 | 1-2 天 |
| 阶段 1 | 基础设施搭建 | 1-2 周 |
| 阶段 2 | 日志系统重构 | 1 周 |
| 阶段 3 | 替换简单交互 | 1 周 |
| 阶段 4 | 替换进度条 | 3-5 天 |
| 阶段 5 | 实现 LogViewer | 1 周 |
| 阶段 6 | 测试和验证 | 1 周 |
| 阶段 7 | 清理和优化 | 3-5 天 |
| 阶段 8 | 发布准备 | 2-3 天 |
| **总计** | | **6-10 周** |

---

### 成功标准

重构成功的标准：

1. ✅ 所有现有功能正常工作
2. ✅ 所有测试通过
3. ✅ 代码质量检查通过
4. ✅ 性能不低于原有实现
5. ✅ 用户体验提升
6. ✅ 文档完整
7. ✅ 无功能回退

---

### 下一步

完成重构后，可以考虑：

1. 为更多命令添加 TUI 界面（如 `pr list`、`jira search`）
2. 优化 LogViewer 功能（添加更多过滤选项）
3. 添加主题支持（深色/浅色模式）
4. 性能优化（虚拟滚动、异步加载）

---

## 📝 迁移后代码示例

以下是迁移到 `ratatui` 后，代码会变成什么样：

### 示例 1：输入对话框（Input）

**迁移前** (`src/commands/jira/info.rs`)：
```rust
use dialoguer::Input;

let jira_id = if let Some(id) = jira_id {
    id
} else {
    Input::<String>::new()
        .with_prompt("Enter Jira ticket ID (e.g., PROJ-123)")
        .interact()
        .context("Failed to read Jira ticket ID")?
};
```

**迁移后**：
```rust
use crate::base::ui::dialogs::InputDialog;

let jira_id = if let Some(id) = jira_id {
    id
} else {
    InputDialog::new("Enter Jira ticket ID (e.g., PROJ-123)")
        .with_placeholder("PROJ-123")
        .show()?
};
```

**UI 组件实现** (`src/lib/base/ui/dialogs.rs`)：
```rust
use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::{Event, KeyCode, KeyEventKind};

pub struct InputDialog {
    prompt: String,
    placeholder: Option<String>,
    input: String,
}

impl InputDialog {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            placeholder: None,
            input: String::new(),
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn show(&mut self) -> Result<String> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        terminal.clear()?;

        loop {
            terminal.draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ])
                    .split(area);

                // 提示信息
                let prompt = Paragraph::new(self.prompt.as_str())
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(prompt, chunks[0]);

                // 输入框
                let input_text = if self.input.is_empty() {
                    self.placeholder.as_deref().unwrap_or("").to_string()
                } else {
                    self.input.clone()
                };
                let input = Paragraph::new(input_text.as_str())
                    .style(if self.input.is_empty() {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    })
                    .block(Block::default().borders(Borders::ALL).title("Input"));
                f.render_widget(input, chunks[1]);
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter => {
                                if !self.input.is_empty() {
                                    break;
                                }
                            }
                            KeyCode::Char(c) => {
                                self.input.push(c);
                            }
                            KeyCode::Backspace => {
                                self.input.pop();
                            }
                            KeyCode::Esc => {
                                anyhow::bail!("User cancelled input");
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        terminal.clear()?;
        Ok(self.input.clone())
    }
}
```

---

### 示例 2：选择对话框（Select）

**迁移前** (`src/commands/config/helpers.rs`)：
```rust
use dialoguer::Select;

let selected_idx = Select::new()
    .with_prompt(&prompt)
    .items(&language_display_names)
    .default(current_idx)
    .interact()
    .context("Failed to select language")?;
```

**迁移后**：
```rust
use crate::base::ui::dialogs::SelectDialog;

let selected_idx = SelectDialog::new(&prompt, &language_display_names)
    .with_default(current_idx)
    .show()?;
```

**UI 组件实现** (`src/lib/base/ui/dialogs.rs`)：
```rust
pub struct SelectDialog<'a> {
    prompt: String,
    items: Vec<String>,
    selected: usize,
    default: Option<usize>,
}

impl<'a> SelectDialog<'a> {
    pub fn new(prompt: impl Into<String>, items: &[impl ToString]) -> Self {
        Self {
            prompt: prompt.into(),
            items: items.iter().map(|i| i.to_string()).collect(),
            selected: 0,
            default: None,
        }
    }

    pub fn with_default(mut self, default: usize) -> Self {
        self.default = Some(default);
        self.selected = default.min(self.items.len().saturating_sub(1));
        self
    }

    pub fn show(&mut self) -> Result<usize> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        terminal.clear()?;

        loop {
            terminal.draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ])
                    .split(area);

                // 提示信息
                let prompt = Paragraph::new(self.prompt.as_str())
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(prompt, chunks[0]);

                // 列表
                let items: Vec<ListItem> = self.items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let style = if i == self.selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        ListItem::new(item.as_str()).style(style)
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(Style::default().fg(Color::Yellow));
                f.render_stateful_widget(
                    list,
                    chunks[1],
                    &mut ListState::default().with_selected(Some(self.selected))
                );

                // 状态栏
                let status = Paragraph::new("↑↓ Navigate | Enter: Select | Esc: Cancel")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(status, chunks[2]);
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Up => {
                                self.selected = self.selected.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                self.selected = (self.selected + 1)
                                    .min(self.items.len().saturating_sub(1));
                            }
                            KeyCode::Enter => {
                                break;
                            }
                            KeyCode::Esc => {
                                anyhow::bail!("User cancelled selection");
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        terminal.clear()?;
        Ok(self.selected)
    }
}
```

---

### 示例 3：进度条（Progress Bar）

**迁移前** (`src/commands/lifecycle/update.rs`)：
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(size);
pb.set_style(
    ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"),
);

// 更新进度
pb.set_position(downloaded_bytes);
pb.finish_with_message("Download complete");
```

**迁移后**：
```rust
use crate::base::ui::progress::ProgressBar;

let mut progress = ProgressBar::new(size, "Downloading update package");
progress.set_message("Downloading...");

// 在下载循环中更新
loop {
    // ... 下载逻辑 ...
    downloaded_bytes += bytes_read as u64;
    progress.update(downloaded_bytes)?;
}

progress.finish("Download complete")?;
```

**UI 组件实现** (`src/lib/base/ui/progress.rs`)：
```rust
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Instant;

pub struct ProgressBar {
    current: u64,
    total: u64,
    message: String,
    start_time: Instant,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl ProgressBar {
    pub fn new(total: u64, title: impl Into<String>) -> Result<Self> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        terminal.clear()?;

        Ok(Self {
            current: 0,
            total,
            message: title.into(),
            start_time: Instant::now(),
            terminal,
        })
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    pub fn update(&mut self, current: u64) -> Result<()> {
        self.current = current;
        self.render()?;
        Ok(())
    }

    pub fn finish(&mut self, message: impl Into<String>) -> Result<()> {
        self.message = message.into();
        self.current = self.total;
        self.render()?;
        std::thread::sleep(std::time::Duration::from_millis(500));
        self.terminal.clear()?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let area = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(5),
                    Constraint::Min(0),
                ])
                .split(area);

            // 标题
            let title = Paragraph::new(self.message.as_str())
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // 进度条
            let progress = (self.current as f64 / self.total as f64).min(1.0);
            let elapsed = self.start_time.elapsed();
            let bytes_per_sec = if elapsed.as_secs() > 0 {
                self.current / elapsed.as_secs()
            } else {
                0
            };
            let eta = if bytes_per_sec > 0 {
                (self.total - self.current) / bytes_per_sec
            } else {
                0
            };

            let progress_text = format!(
                "{} / {} ({:.1}%) | {}/s | ETA: {}s",
                format_bytes(self.current),
                format_bytes(self.total),
                progress * 100.0,
                format_bytes(bytes_per_sec),
                eta
            );

            let progress_widget = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Progress"))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent((progress * 100.0) as u16)
                .label(progress_text);
            f.render_widget(progress_widget, chunks[1]);
        })?;
        Ok(())
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
```

---

### 示例 4：日志系统（Logger）

**迁移前** (`src/lib/base/util/logger.rs`)：
```rust
use colored::*;

pub fn print_success(message: impl fmt::Display) {
    println!("{}", message.to_string().green());
}

pub fn print_error(message: impl fmt::Display) {
    println!("{}", message.to_string().red());
}
```

**迁移后**：
```rust
use ratatui::style::{Color, Style, Modifier};

pub fn print_success(message: impl fmt::Display) {
    let styled = format_with_style(
        message.to_string(),
        Style::default().fg(Color::Green)
    );
    println!("{}", styled);
}

pub fn print_error(message: impl fmt::Display) {
    let styled = format_with_style(
        message.to_string(),
        Style::default().fg(Color::Red)
    );
    println!("{}", styled);
}

// 将 ratatui Style 转换为 ANSI 颜色码
fn format_with_style(text: String, style: Style) -> String {
    use std::fmt::Write;
    let mut result = String::new();

    if let Some(fg) = style.fg {
        write!(&mut result, "\x1b[{}m", color_to_ansi(fg)).unwrap();
    }

    if style.add_modifier.contains(Modifier::BOLD) {
        write!(&mut result, "\x1b[1m").unwrap();
    }

    write!(&mut result, "{}", text).unwrap();
    write!(&mut result, "\x1b[0m").unwrap(); // Reset

    result
}

fn color_to_ansi(color: Color) -> u8 {
    match color {
        Color::Black => 30,
        Color::Red => 31,
        Color::Green => 32,
        Color::Yellow => 33,
        Color::Blue => 34,
        Color::Magenta => 35,
        Color::Cyan => 36,
        Color::White => 37,
        Color::Gray => 90,
        Color::DarkGray => 90,
        _ => 37, // Default to white
    }
}
```

---

### 示例 5：确认对话框（Confirm）

**迁移前** (`src/lib/base/util/confirm.rs`)：
```rust
use dialoguer::Confirm;

pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    let confirmed = Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .context("Failed to get user confirmation")?;
    Ok(confirmed)
}
```

**迁移后**：
```rust
use crate::base::ui::dialogs::ConfirmDialog;

pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    ConfirmDialog::new(prompt)
        .with_default(default)
        .show()
}
```

**UI 组件实现** (`src/lib/base/ui/dialogs.rs`)：
```rust
pub struct ConfirmDialog {
    prompt: String,
    default: bool,
}

impl ConfirmDialog {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            default: true,
        }
    }

    pub fn with_default(mut self, default: bool) -> Self {
        self.default = default;
        self
    }

    pub fn show(&mut self) -> Result<bool> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        terminal.clear()?;

        let mut selected = if self.default { 0 } else { 1 };

        loop {
            terminal.draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ])
                    .split(area);

                // 提示信息
                let prompt = Paragraph::new(self.prompt.as_str())
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(prompt, chunks[0]);

                // 按钮
                let buttons = vec!["Yes", "No"];
                let button_text: Vec<Span> = buttons
                    .iter()
                    .enumerate()
                    .map(|(i, text)| {
                        let style = if i == selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        Span::styled(
                            if i == 0 { format!("[{}]", text) } else { format!(" {} ", text) },
                            style
                        )
                    })
                    .collect();

                let buttons_widget = Paragraph::new(Line::from(button_text))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(buttons_widget, chunks[1]);
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Left | KeyCode::Char('n') => {
                                selected = 1;
                            }
                            KeyCode::Right | KeyCode::Char('y') => {
                                selected = 0;
                            }
                            KeyCode::Enter => {
                                terminal.clear()?;
                                return Ok(selected == 0);
                            }
                            KeyCode::Esc => {
                                terminal.clear()?;
                                return Ok(false);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
```

---

### 示例 6：完整的命令迁移（JIRA Info）

**迁移前** (`src/commands/jira/info.rs`)：
```rust
use dialoguer::Input;
use crate::{log_break, log_message};

pub fn show(jira_id: Option<String>) -> Result<()> {
    let jira_id = if let Some(id) = jira_id {
        id
    } else {
        Input::<String>::new()
            .with_prompt("Enter Jira ticket ID")
            .interact()?
    };

    let issue = Jira::get_ticket_info(&jira_id)?;

    log_break!('=', 40, "Ticket Information");
    log_message!("Key: {}", issue.key);
    log_message!("Summary: {}", issue.fields.summary);
    // ...
}
```

**迁移后**：
```rust
use crate::base::ui::dialogs::InputDialog;
use crate::base::ui::widgets::InfoWidget;

pub fn show(jira_id: Option<String>) -> Result<()> {
    let jira_id = if let Some(id) = jira_id {
        id
    } else {
        InputDialog::new("Enter Jira ticket ID (e.g., PROJ-123)")
            .show()?
    };

    let issue = Jira::get_ticket_info(&jira_id)?;

    // 使用 ratatui 显示信息界面
    InfoWidget::new(&issue).show()?;
    Ok(())
}
```

**信息显示组件** (`src/lib/base/ui/widgets.rs`)：
```rust
use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct InfoWidget {
    issue: JiraIssue,
}

impl InfoWidget {
    pub fn new(issue: &JiraIssue) -> Self {
        Self { issue: issue.clone() }
    }

    pub fn show(&mut self) -> Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        terminal.clear()?;

        loop {
            terminal.draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // 标题
                        Constraint::Min(0),     // 内容
                        Constraint::Length(1), // 状态栏
                    ])
                    .split(area);

                // 标题
                let title = Paragraph::new("Ticket Information")
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(title, chunks[0]);

                // 信息表格
                let rows = vec![
                    Row::new(vec!["Key", &self.issue.key]),
                    Row::new(vec!["Summary", &self.issue.fields.summary]),
                    Row::new(vec!["Status", &self.issue.fields.status.name]),
                ];

                let table = Table::new(rows)
                    .block(Block::default().borders(Borders::ALL))
                    .widths(&[Constraint::Length(15), Constraint::Min(0)])
                    .column_spacing(2);
                f.render_widget(table, chunks[1]);

                // 状态栏
                let status = Paragraph::new("Press 'q' to quit")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(status, chunks[2]);
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }

        terminal.clear()?;
        Ok(())
    }
}
```

---

### 总结

迁移后的代码特点：

1. **统一的 UI 系统**：所有 UI 组件都使用 `ratatui`
2. **更好的用户体验**：交互式界面，支持键盘导航
3. **组件化设计**：可复用的 UI 组件，易于维护
4. **一致的风格**：统一的主题和样式
5. **更丰富的展示**：支持表格、布局、实时更新等

**主要变化**：
- `dialoguer::Input` → `InputDialog`
- `dialoguer::Select` → `SelectDialog`
- `dialoguer::Confirm` → `ConfirmDialog`
- `indicatif::ProgressBar` → `ProgressBar` (自定义组件)
- `colored::*` → `ratatui::style::*`
