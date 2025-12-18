# Alias 命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Alias 命令模块架构，包括：
- Alias list 功能（列出所有别名）
- Alias add 功能（添加别名，支持直接模式和交互式模式）
- Alias remove 功能（删除别名，支持直接模式和交互式多选模式）

Alias 命令模块提供完整的命令别名管理功能，允许用户为常用命令创建简短别名。别名会在命令解析前自动展开，支持嵌套别名和循环检测。所有命令都提供交互式界面，支持直接模式和交互式模式。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/base/alias/` 模块提供。

**模块统计：**
- 命令数量：3 个（list、add、remove）
- 总代码行数：约 342 行
- 文件数量：4 个
- 主要依赖：`lib/base/alias/`、`lib/base/dialog/`、`lib/base/util/`

---

## 📁 相关文件

### CLI 入口层

```
src/bin/workflow.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：
  - 使用 `clap` 解析命令行参数，将 `workflow alias` 子命令分发到对应的命令处理函数
  - 在命令解析前调用 `AliasManager::expand_args()` 展开别名

### 命令封装层

```
src/commands/alias/
├── mod.rs          # Alias 命令模块声明（12 行）
├── list.rs         # Alias list 命令（60 行）
├── add.rs          # Alias add 命令（152 行）
└── remove.rs       # Alias remove 命令（122 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择、确认等）
- 格式化输出（表格显示）
- 调用核心业务逻辑层 (`lib/base/alias/`) 的功能
- 处理错误情况（循环别名、别名不存在等）

### 核心业务逻辑层

```
src/lib/base/alias/
├── mod.rs          # 模块声明（9 行）
├── manager.rs      # 别名管理器（325 行）
└── config.rs       # 别名配置管理（85 行）
```

**职责**：
- 别名配置的加载和保存
- 别名的展开（支持嵌套别名）
- 别名管理（添加、删除、列表、存在性检查）
- 循环别名检测
- 命令行参数展开

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/base/alias/manager.rs`**：别名管理器
  - `AliasManager::load()` - 加载别名配置
  - `AliasManager::expand()` - 展开别名（支持嵌套）
  - `AliasManager::expand_args()` - 展开命令行参数
  - `AliasManager::add()` - 添加别名
  - `AliasManager::remove()` - 删除别名
  - `AliasManager::list()` - 列出所有别名
  - `AliasManager::exists()` - 检查别名是否存在
  - `AliasManager::check_circular()` - 检查循环别名
- **`lib/base/alias/config.rs`**：别名配置管理
  - `CommandsConfig::load()` - 加载命令配置
  - `CommandsConfig::get_common_commands()` - 获取常用命令列表
- **`lib/base/dialog/`**：用户交互对话框
  - `InputDialog` - 输入对话框（支持验证器）
  - `ConfirmDialog` - 确认对话框
  - `SelectDialog` - 选择对话框
  - `MultiSelectDialog` - 多选对话框
- **`lib/base/util/`**：工具函数
  - `TableBuilder` - 表格构建器
  - `TableStyle` - 表格样式
- **`lib/base/settings/`**：配置管理
  - `Settings::get()` - 获取配置
  - `Paths::workflow_config()` - 获取配置文件路径

详细架构文档：参见 [Settings 模块架构文档](../lib/SETTINGS_ARCHITECTURE.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/bin/workflow.rs (workflow 主命令，参数解析)
  ↓
AliasManager::expand_args() (别名展开)
  ↓
commands/alias/*.rs (命令封装层，处理交互)
  ↓
lib/base/alias/manager.rs (业务逻辑层，别名管理)
  ↓
lib/base/settings/ (配置读写)
  ↓
配置文件 (workflow.toml)
```

### 别名展开流程

```
用户输入: workflow ci
  ↓
AliasManager::expand_args() (检查第一个参数是否为别名)
  ↓
AliasManager::expand() (递归展开别名，支持嵌套)
  ↓
展开为: workflow pr create
  ↓
继续正常的命令解析流程
```

### 各命令详细流程

#### 1. `alias list` 命令流程

```
用户输入: workflow alias list
  ↓
AliasListCommand::list()
  ↓
AliasManager::list() (加载所有别名)
  ↓
构建表格数据 (AliasRow)
  ↓
TableBuilder::render() (渲染表格)
  ↓
输出表格
```

**关键代码**：
```rust
// src/commands/alias/list.rs
pub fn list() -> Result<()> {
    let aliases = AliasManager::list()?;
    // 构建表格并显示
    let table = TableBuilder::new(rows)
        .with_title("Defined Aliases")
        .with_style(TableStyle::Modern)
        .render();
    log_message!("{}", table);
    Ok(())
}
```

#### 2. `alias add` 命令流程

**直接模式**：
```
用户输入: workflow alias add ci "pr create"
  ↓
AliasAddCommand::add(Some("ci"), Some("pr create"))
  ↓
检查循环别名 (AliasManager::check_circular())
  ↓
检查别名是否已存在 (AliasManager::exists())
  ↓
如果存在，询问是否覆盖
  ↓
AliasManager::add() (保存别名到配置文件)
  ↓
询问是否更新补全脚本
```

**交互式模式**：
```
用户输入: workflow alias add
  ↓
AliasAddCommand::add(None, None)
  ↓
步骤1: InputDialog 输入别名名称（带验证器）
  ↓
检查别名是否已存在，如果存在询问是否覆盖
  ↓
步骤2: SelectDialog 选择命令输入方式
  - "Select from common commands" -> 从常用命令列表选择
  - "Enter manually" -> 手动输入命令
  ↓
步骤3: 获取命令（从列表选择或手动输入）
  ↓
检查循环别名
  ↓
AliasManager::add() (保存别名)
  ↓
询问是否更新补全脚本
```

**关键代码**：
```rust
// src/commands/alias/add.rs
pub fn add(name: Option<String>, command: Option<String>) -> Result<()> {
    let (alias_name, alias_command, is_direct_mode) = if let (Some(name), Some(cmd)) = (name, command) {
        // 直接模式
        (name, cmd, true)
    } else {
        // 交互式模式：通过对话框输入
        // ...
    };

    // 检查循环别名
    if AliasManager::check_circular(&alias_name, &alias_command)? {
        return Err(...);
    }

    // 保存别名
    AliasManager::add(&alias_name, &alias_command)?;
    Ok(())
}
```

#### 3. `alias remove` 命令流程

**直接模式**：
```
用户输入: workflow alias remove ci
  ↓
AliasRemoveCommand::remove(Some("ci"))
  ↓
检查别名是否存在
  ↓
确认删除 (ConfirmDialog)
  ↓
AliasManager::remove() (从配置文件删除)
  ↓
询问是否更新补全脚本
```

**交互式模式**：
```
用户输入: workflow alias remove
  ↓
AliasRemoveCommand::remove(None)
  ↓
MultiSelectDialog 多选要删除的别名
  ↓
确认删除 (ConfirmDialog)
  ↓
批量删除 (AliasManager::remove())
  ↓
询问是否更新补全脚本
```

**关键代码**：
```rust
// src/commands/alias/remove.rs
pub fn remove(name: Option<String>) -> Result<()> {
    let names_to_remove = if let Some(name) = name {
        // 直接模式
        vec![name]
    } else {
        // 交互式模式：多选
        let selected = MultiSelectDialog::new(...).prompt()?;
        // 提取别名名称
        selected.iter().filter_map(...).collect()
    };

    // 确认删除
    let confirmed = ConfirmDialog::new(...).prompt()?;

    // 批量删除
    for name in &names_to_remove {
        AliasManager::remove(name)?;
    }
    Ok(())
}
```

---

## 🎯 核心功能

### 1. 别名展开

别名展开是 Alias 模块的核心功能，在命令解析前自动执行。

**展开流程**：
1. 检查第一个参数是否是别名
2. 如果是别名，递归展开（支持嵌套别名）
3. 检测循环引用（防止无限递归）
4. 限制最大展开深度（10 层）

**示例**：
```bash
# 定义别名
workflow alias add ci "pr create"
workflow alias add prc "pr create"

# 使用别名
workflow ci              # 展开为: workflow pr create
workflow prc             # 展开为: workflow pr create
```

**嵌套别名支持**：
```bash
# 定义嵌套别名
workflow alias add a "b"
workflow alias add b "c"
workflow alias add c "pr create"

# 使用嵌套别名
workflow a               # 展开为: workflow pr create (a -> b -> c -> pr create)
```

**循环检测**：
```bash
# 定义循环别名（会被检测并拒绝）
workflow alias add a "b"  # 如果 b 指向 a，会被拒绝
```

### 2. 别名管理

**添加别名**：
- 支持直接模式和交互式模式
- 检查循环别名
- 检查别名是否已存在（交互式询问是否覆盖）
- 支持从常用命令列表选择或手动输入

**删除别名**：
- 支持直接模式和交互式多选模式
- 批量删除支持
- 删除后询问是否更新补全脚本

**列出别名**：
- 使用表格格式显示所有别名
- 显示别名名称和对应的命令

### 3. 配置管理

别名配置存储在 `workflow.toml` 配置文件中：

```toml
[aliases]
ci = "pr create"
st = "pr status"
```

**配置读写**：
- 使用 `Settings` 模块统一管理配置
- 配置文件路径：`~/.workflow/config/workflow.toml` (macOS/Linux) 或 `%APPDATA%\workflow\config\workflow.toml` (Windows)
- 文件权限：Unix 系统设置为 `0o600`（仅所有者可读写）

---

## 🔍 设计要点

### 1. 别名展开时机

别名展开在命令解析**之前**执行，这样可以：
- 支持别名作为第一个参数（如 `workflow ci`）
- 保持命令解析逻辑的简洁性
- 支持嵌套别名和复杂场景

**实现位置**：`src/bin/workflow.rs:66`
```rust
// 别名展开：在解析前展开别名
let args: Vec<String> = std::env::args().collect();
let expanded_args = AliasManager::expand_args(args)?;

// 使用展开后的参数重新解析
let cli = Cli::parse_from(expanded_args);
```

### 2. 循环检测

为了防止循环别名导致无限递归，实现了循环检测机制：

**检测方法**：
1. 使用 `HashSet` 跟踪已访问的别名
2. 限制最大展开深度（10 层）
3. 在添加别名时检查是否会导致循环

**实现位置**：`src/lib/base/alias/manager.rs:54-98`

### 3. 交互式体验

所有命令都支持交互式模式，提供友好的用户体验：

- **输入验证**：别名名称不能为空、不能包含空格
- **确认对话框**：覆盖已存在的别名、删除别名时都需要确认
- **多选支持**：删除命令支持多选多个别名
- **常用命令列表**：添加别名时可以从常用命令列表选择

### 4. 补全脚本更新

添加或删除别名后，会询问用户是否更新补全脚本，确保补全功能正常工作。

---

## 📊 数据流

### 别名配置数据流

```
workflow.toml (配置文件)
  ↓
Settings::get() (加载配置)
  ↓
settings.aliases (HashMap<String, String>)
  ↓
AliasManager::load() (返回别名映射)
  ↓
命令层使用
```

### 别名展开数据流

```
命令行参数: ["workflow", "ci", "PROJ-123"]
  ↓
AliasManager::expand_args()
  ↓
检查 "ci" 是否是别名
  ↓
AliasManager::expand("ci")
  ↓
递归展开（如果包含嵌套别名）
  ↓
展开结果: ["workflow", "pr", "create", "PROJ-123"]
  ↓
继续命令解析
```

---

## 🧪 测试

### 单元测试

别名管理器的核心功能在 `src/lib/base/alias/manager.rs` 中有测试占位符，实际测试在集成测试中完成。

### 集成测试

补全完整性测试（`tests/completion/completeness.rs`）确保 alias 命令包含在补全脚本中。

---

## 🔗 相关文档

- [Settings 模块架构文档](../lib/SETTINGS_ARCHITECTURE.md) - 配置管理
- [Dialog 模块架构文档](../lib/DIALOG_ARCHITECTURE.md) - 用户交互
- [Completion 模块架构文档](../lib/COMPLETION_ARCHITECTURE.md) - 补全脚本生成

---

## 📝 使用示例

### 基本使用

```bash
# 列出所有别名
workflow alias list

# 添加别名（直接模式）
workflow alias add ci "pr create"
workflow alias add st "pr status"

# 添加别名（交互式模式）
workflow alias add

# 删除别名（直接模式）
workflow alias remove ci

# 删除别名（交互式模式）
workflow alias remove

# 使用别名
workflow ci PROJ-123        # 等同于: workflow pr create PROJ-123
workflow st                 # 等同于: workflow pr status
```

### 嵌套别名

```bash
# 定义嵌套别名
workflow alias add a "b"
workflow alias add b "c"
workflow alias add c "pr create"

# 使用嵌套别名
workflow a                  # 展开为: workflow pr create
```

### 循环检测

```bash
# 尝试创建循环别名（会被拒绝）
workflow alias add a "b"
workflow alias add b "a"    # 错误: Circular alias detected
```

---

## 🎨 代码组织

### 命令层职责

- **参数解析**：解析命令行参数
- **用户交互**：处理输入、选择、确认等交互
- **输出格式化**：表格显示、日志输出
- **错误处理**：友好的错误提示

### 业务逻辑层职责

- **别名管理**：添加、删除、列表、存在性检查
- **别名展开**：递归展开、循环检测
- **配置管理**：读写配置文件
- **常用命令**：提供常用命令列表

### 设计原则

1. **单一职责**：每个模块只负责一个功能
2. **依赖倒置**：命令层依赖业务逻辑层的抽象
3. **错误处理**：使用 `anyhow::Result` 统一错误处理
4. **用户体验**：提供交互式和直接模式两种使用方式

---

**文档版本**：1.0
**最后更新**: 2025-12-13
