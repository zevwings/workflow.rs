# Table 模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Table 模块架构，包括：
- **表格构建器**（TableBuilder）：提供链式配置和渲染功能
- **表格样式**（TableStyle）：定义不同的边框和显示风格

该模块提供统一的表格输出接口，使用 `tabled` 库实现。支持自定义样式、边框、对齐、标题等丰富的表格格式化功能，确保整个项目的表格输出格式一致。

**注意**：本模块是基础设施模块，被整个项目广泛使用。所有需要表格格式输出的命令都使用 TableBuilder。

**模块统计：**
- 总代码行数：约 370 行
- 文件数量：1 个核心文件
- 主要组件：
  - TableBuilder（5 个主要方法）
  - TableStyle（5 种样式枚举）
- 依赖库：
  - `tabled` crate：提供表格格式化功能

---

## 📁 Lib 层架构（核心业务逻辑）

### 核心模块文件

```
src/lib/base/table/
└── mod.rs          # 表格输出工具实现 (372行)
```

### 依赖模块

- **`tabled` crate**：提供表格格式化功能
  - `Tabled` trait：用于定义表格行结构
  - `Table`：表格构建和渲染
  - `Settings`：表格样式和配置
- **标准库**：`std::fmt`

### 模块集成

Table 模块被所有需要表格格式输出的命令和模块广泛使用：

- **PR 命令**：使用 `TableBuilder` 和 `PullRequestRow` 显示 PR 列表
- **Jira 命令**：使用 `TableBuilder` 显示 Jira ticket 信息表格
- **Branch 命令**：使用 `TableBuilder` 显示分支列表
- **Stash 命令**：使用 `TableBuilder` 显示 stash 列表
- **Alias 命令**：使用 `TableBuilder` 显示别名列表
- **Config 命令**：使用 `TableBuilder` 显示配置项表格
- **GitHub 命令**：使用 `TableBuilder` 显示账号列表
- **LLM 命令**：使用 `TableBuilder` 显示 LLM 配置表格

---

## 🔄 集成关系

Table 模块是 Workflow CLI 的基础设施模块，为所有需要表格格式输出的命令和模块提供统一的表格接口。该模块通过以下方式与其他模块集成：

1. **命令层集成**：所有命令层模块通过 Table 模块提供的接口进行表格输出
2. **统一格式**：提供统一的表格格式标准，确保所有表格输出格式一致
3. **易于维护**：集中管理表格格式化逻辑，修改时只需更新一处

### 主要集成场景

- **PR 列表显示**：PR 命令使用 `TableBuilder` 和 `PullRequestRow` 显示 PR 列表
- **数据统计**：各种命令使用 `TableBuilder` 显示统计数据表格
- **配置显示**：Config 命令使用 `TableBuilder` 显示配置项表格
- **列表显示**：各种命令使用 `TableBuilder` 显示列表数据

---

## 🏗️ 架构设计

### 设计原则

1. **链式调用**：支持链式配置，提供更好的代码可读性
2. **类型安全**：使用泛型和 `Tabled` trait 保证类型安全
3. **灵活配置**：支持多种样式、对齐方式和宽度控制
4. **自动格式化**：自动处理标题、边框、对齐等格式
5. **易于使用**：简单的 API，易于集成到现有代码中

### 核心组件

#### 1. TableBuilder 结构体 (`mod.rs`)

**职责**：提供表格构建和渲染功能，支持链式配置。

**结构定义**：

```rust
pub struct TableBuilder<T> {
    data: Vec<T>,
    title: Option<String>,
    style: Option<TableStyle>,
    max_width: Option<usize>,
    alignments: Vec<Alignment>,
}
```

**主要方法**：

##### TableBuilder::new

```rust
pub fn new(data: Vec<T>) -> Self
```

**功能**：创建新的表格构建器

**参数**：
- `data` - 要显示的数据，必须实现 `Tabled` trait

**返回**：新的 `TableBuilder` 实例

**要求**：数据类型 `T` 必须实现 `tabled::Tabled` trait

**示例**：
```rust
use tabled::Tabled;
use workflow::base::table::TableBuilder;

#[derive(Tabled)]
struct User {
    name: String,
    age: u32,
}

let users = vec![
    User { name: "Alice".to_string(), age: 30 },
    User { name: "Bob".to_string(), age: 25 },
];

let builder = TableBuilder::new(users);
```

##### TableBuilder::with_title

```rust
pub fn with_title(mut self, title: impl Into<String>) -> Self
```

**功能**：设置表格标题

**参数**：
- `title` - 表格标题

**返回**：返回 `Self`，支持链式调用

**特性**：
- 标题会显示在表格顶部，居中对齐
- 标题行下方会有分隔线
- 自动修复标题行下方的分隔线格式（将 `┼` 替换为 `┬`）

**示例**：
```rust
let builder = TableBuilder::new(data)
    .with_title("Users List");
```

##### TableBuilder::with_style

```rust
pub fn with_style(mut self, style: TableStyle) -> Self
```

**功能**：设置表格样式

**参数**：
- `style` - 表格样式（见 `TableStyle` 枚举）

**返回**：返回 `Self`，支持链式调用

**示例**：
```rust
use workflow::base::table::{TableBuilder, TableStyle};

let builder = TableBuilder::new(data)
    .with_style(TableStyle::Modern);
```

##### TableBuilder::with_max_width

```rust
pub fn with_max_width(mut self, width: usize) -> Self
```

**功能**：设置最大宽度（自动换行）

**参数**：
- `width` - 最大宽度（字符数）

**返回**：返回 `Self`，支持链式调用

**特性**：
- 当内容超过最大宽度时，自动换行
- 适用于终端显示，避免表格过宽
- 使用 `tabled::settings::Width::wrap()` 实现

**示例**：
```rust
let builder = TableBuilder::new(data)
    .with_max_width(80);
```

##### TableBuilder::with_alignment

```rust
pub fn with_alignment(mut self, alignments: Vec<Alignment>) -> Self
```

**功能**：设置列对齐方式

**参数**：
- `alignments` - 每列的对齐方式，按列索引顺序

**返回**：返回 `Self`，支持链式调用

**示例**：
```rust
use tabled::settings::Alignment;
use workflow::base::table::TableBuilder;

let builder = TableBuilder::new(data)
    .with_alignment(vec![Alignment::left(), Alignment::right()]);
```

##### TableBuilder::render

```rust
pub fn render(self) -> String
```

**功能**：构建并渲染表格为字符串

**返回**：格式化后的表格字符串

**特性**：
- 如果数据为空，返回空字符串或标题（如果有）
- 自动修复标题行下方的分隔线格式
- 应用所有配置的样式、对齐、宽度等设置

**实现流程**：
1. 检查数据是否为空
2. 创建 `tabled::Table` 实例
3. 应用样式配置
4. 添加标题（如果有）
5. 应用最大宽度配置
6. 应用列对齐配置
7. 渲染表格并修复标题分隔线格式

**示例**：
```rust
let output = TableBuilder::new(data)
    .with_title("My Table")
    .with_style(TableStyle::Modern)
    .render();
println!("{}", output);
```

#### 2. TableStyle 枚举 (`mod.rs`)

**职责**：定义表格样式配置，提供不同的边框和显示风格。

**枚举定义**：

```rust
pub enum TableStyle {
    Default,  // 默认样式（ASCII）
    Modern,   // 现代样式（带边框，推荐）
    Compact,  // 紧凑样式（无边框）
    Minimal,  // 最小样式（仅分隔符）
    Grid,     // 网格样式（完整网格）
}
```

**样式说明**：

- **Default**：ASCII 字符边框，兼容性好
  - 使用 `Style::ascii()` 实现
  - 适合需要最大兼容性的场景

- **Modern**：现代样式，带圆角边框，视觉效果最佳（推荐）
  - 使用 `Style::modern()` 实现
  - 适合大多数场景，视觉效果最佳

- **Compact**：紧凑样式，无边框，节省空间
  - 使用 `Style::rounded()` 实现
  - 适合需要节省空间的场景

- **Minimal**：最小样式，仅使用分隔符
  - 使用 `Style::blank()` 实现
  - 适合需要最小视觉干扰的场景

- **Grid**：完整网格样式，所有单元格都有边框
  - 使用 `Style::rounded()` 实现
  - 适合需要完整网格的场景

**选择建议**：
- **一般情况**：使用 `Modern` 样式，视觉效果最佳
- **需要兼容性**：使用 `Default` 样式，兼容性最好
- **需要节省空间**：使用 `Compact` 样式，节省显示空间
- **需要最小干扰**：使用 `Minimal` 样式，视觉干扰最小
- **需要完整网格**：使用 `Grid` 样式，所有单元格都有边框

### 设计模式

#### 1. 建造者模式（Builder Pattern）

TableBuilder 使用建造者模式，支持链式配置：

```rust
let output = TableBuilder::new(data)
    .with_title("My Table")
    .with_style(TableStyle::Modern)
    .with_max_width(80)
    .with_alignment(vec![Alignment::left(), Alignment::right()])
    .render();
```

**优势**：
- 链式调用，代码可读性强
- 可选配置，灵活性强
- 类型安全，编译时检查

#### 2. 策略模式（Strategy Pattern）

TableStyle 使用策略模式，支持不同的样式策略：

```rust
match style {
    TableStyle::Modern => table.with(Style::modern()),
    TableStyle::Default => table.with(Style::ascii()),
    // ...
}
```

**优势**：
- 易于扩展新的样式
- 样式切换简单
- 代码组织清晰

### 错误处理

Table 模块的错误处理策略：

- **空数据处理**：如果数据为空，返回空字符串或标题（如果有）
- **类型检查**：编译时通过 `Tabled` trait 保证类型安全
- **配置验证**：运行时验证配置参数（如宽度、对齐数量等）

---

## 📋 使用示例

### 基本使用

```rust
use tabled::Tabled;
use workflow::base::table::{TableBuilder, TableStyle};
use workflow::log_message;

#[derive(Tabled)]
struct User {
    name: String,
    age: u32,
    email: String,
}

let users = vec![
    User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    },
    User {
        name: "Bob".to_string(),
        age: 25,
        email: "bob@example.com".to_string(),
    },
];

let output = TableBuilder::new(users)
    .with_title("Users List")
    .with_style(TableStyle::Modern)
    .render();

log_message!("{}", output);
```

### 链式配置

```rust
use tabled::settings::Alignment;
use workflow::base::table::{TableBuilder, TableStyle};

let output = TableBuilder::new(data)
    .with_title("My Table")
    .with_style(TableStyle::Modern)
    .with_max_width(80)
    .with_alignment(vec![Alignment::left(), Alignment::right()])
    .render();
```

### 使用 Display trait

TableBuilder 实现了 `Display` trait，可以直接使用：

```rust
use workflow::base::table::TableBuilder;

let builder = TableBuilder::new(users)
    .with_title("Users")
    .with_style(TableStyle::Modern);

// 直接使用 Display trait
println!("{}", builder);
```

### 自定义列名

使用 `Tabled` trait 的 `rename` 属性自定义列名：

```rust
use tabled::Tabled;

#[derive(Tabled)]
struct PullRequestRow {
    #[tabled(rename = "#")]
    pub number: String,

    #[tabled(rename = "State")]
    pub state: String,

    #[tabled(rename = "Branch")]
    pub branch: String,

    #[tabled(rename = "Title")]
    pub title: String,
}
```

### 不同样式示例

```rust
// Modern 样式（推荐）
let output = TableBuilder::new(data)
    .with_style(TableStyle::Modern)
    .render();

// Default 样式（兼容性）
let output = TableBuilder::new(data)
    .with_style(TableStyle::Default)
    .render();

// Compact 样式（节省空间）
let output = TableBuilder::new(data)
    .with_style(TableStyle::Compact)
    .render();
```

### 实际使用场景

#### PR 列表显示

```rust
use tabled::Tabled;
use workflow::base::table::{TableBuilder, TableStyle};
use workflow::log_message;

#[derive(Tabled)]
struct PullRequestRow {
    #[tabled(rename = "#")]
    number: String,
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "Branch")]
    branch: String,
    #[tabled(rename = "Title")]
    title: String,
}

let prs: Vec<PullRequestRow> = fetch_prs()?;

let output = TableBuilder::new(prs)
    .with_title("Pull Requests")
    .with_style(TableStyle::Modern)
    .with_max_width(120)
    .render();

log_message!("{}", output);
```

#### 配置显示

```rust
use tabled::Tabled;
use workflow::base::table::{TableBuilder, TableStyle};

#[derive(Tabled)]
struct ConfigRow {
    #[tabled(rename = "Key")]
    key: String,
    #[tabled(rename = "Value")]
    value: String,
}

let config: Vec<ConfigRow> = load_config()?;

let output = TableBuilder::new(config)
    .with_title("Configuration")
    .with_style(TableStyle::Modern)
    .render();

println!("{}", output);
```

---

## 📝 扩展性

### 添加新的表格样式

1. **在 TableStyle 枚举中添加新样式**：
```rust
pub enum TableStyle {
    // ... 现有样式
    Custom,  // 新样式
}
```

2. **在 apply_to_table 方法中添加样式实现**：
```rust
impl TableStyle {
    fn apply_to_table(&self, table: &mut Table) {
        match self {
            // ... 现有样式
            TableStyle::Custom => {
                table.with(Style::custom());
            }
        }
    }
}
```

### 添加新的配置选项

1. **在 TableBuilder 结构体中添加新字段**：
```rust
pub struct TableBuilder<T> {
    // ... 现有字段
    custom_option: Option<CustomType>,
}
```

2. **添加配置方法**：
```rust
impl<T: Tabled> TableBuilder<T> {
    pub fn with_custom_option(mut self, option: CustomType) -> Self {
        self.custom_option = Some(option);
        self
    }
}
```

3. **在 render 方法中应用配置**：
```rust
pub fn render(self) -> String {
    // ... 现有实现
    if let Some(option) = self.custom_option {
        // 应用自定义配置
    }
    // ...
}
```

---

## 🎯 最佳实践

### 1. 样式选择

- **优先使用 `Modern` 样式**：视觉效果最佳，适合大多数场景
- **需要兼容性时使用 `Default` 样式**：确保在所有终端中正常显示
- **需要节省空间时使用 `Compact` 样式**：减少显示空间占用

### 2. 宽度控制

- **终端显示**：使用 `with_max_width(80)` 避免表格过宽
- **宽屏显示**：可以使用更大的宽度值（如 120）
- **自动换行**：当内容超过最大宽度时，自动换行

### 3. 列对齐

- **数字列**：使用右对齐（`Alignment::right()`）
- **文本列**：使用左对齐（`Alignment::left()`）
- **混合列**：根据列内容类型选择合适的对齐方式

### 4. 标题使用

- **添加有意义的标题**：提升表格可读性
- **标题居中**：自动居中对齐，视觉效果更好
- **标题简洁**：保持标题简洁明了

### 5. 空数据处理

- **检查空数据**：在渲染前检查数据是否为空
- **提供提示**：如果数据为空，提供有意义的提示信息
- **避免空表格**：避免显示空的表格

### 6. 性能考虑

- **大数据量**：对于大数据量，考虑分页或限制显示数量
- **渲染优化**：TableBuilder 在渲染时进行优化，避免不必要的计算
- **内存使用**：注意大数据量时的内存使用

---

## 📚 相关文档

- [主架构文档](./architecture.md) - 项目总体架构
- [工具函数模块架构文档](./tools.md) - Table 模块的简要说明
- [Format 模块架构文档](./format.md) - 格式化相关模块
- [Dialog 模块架构文档](./dialog.md) - 用户交互相关模块

---

## ✅ 总结

Table 模块采用清晰的建造者模式设计：

1. **链式调用**：支持链式配置，提供更好的代码可读性
2. **类型安全**：使用泛型和 `Tabled` trait 保证类型安全
3. **灵活配置**：支持多种样式、对齐方式和宽度控制
4. **自动格式化**：自动处理标题、边框、对齐等格式
5. **易于使用**：简单的 API，易于集成到现有代码中

**设计优势**：
- ✅ **统一性**：所有表格输出使用统一的格式和样式
- ✅ **可维护性**：集中管理表格格式化逻辑，易于维护
- ✅ **可扩展性**：易于添加新的样式和配置选项
- ✅ **性能**：高效的渲染实现，支持大数据量
- ✅ **易用性**：简单的 API，易于学习和使用

---

**最后更新**: 2025-12-23

