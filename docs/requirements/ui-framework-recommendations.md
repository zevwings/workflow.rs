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

#### 集成建议

1. **渐进式集成**：先为特定命令（如 `workflow pr list`）添加 TUI 界面
2. **保留 CLI 模式**：通过 `--tui` 标志启用 TUI，默认保持 CLI 输出
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
- 可以逐步集成，不影响现有功能
- 通过 `--tui` 标志控制，保持向后兼容

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

### 方案 3：混合方案（最佳）

**目标**：结合两种方案的优势

**实施**：
- 使用 `inquire` 增强所有交互式提示
- 使用 `ratatui` 为复杂场景（列表浏览、实时日志）添加 TUI
- 继续使用 `indicatif` 显示进度
- 优化 `colored` 输出样式

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

## ✅ 总结

1. **短期**：使用 `inquire` 增强交互提示，快速提升体验
2. **中期**：为关键命令添加 `ratatui` TUI 界面
3. **长期**：逐步为所有适合的命令添加 TUI 支持

**推荐从 `inquire` 开始**，因为它：
- 改动小，风险低
- 可以立即提升交互体验
- 为后续 TUI 集成打下基础
