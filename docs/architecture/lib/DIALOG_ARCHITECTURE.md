# Dialog 模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Dialog 模块架构，包括：
- 文本输入对话框（InputDialog）
- 单选对话框（SelectDialog）
- 多选对话框（MultiSelectDialog）
- 确认对话框（ConfirmDialog）

该模块提供统一的交互式对话框接口，使用 `inquire` 和 `dialoguer` 作为后端实现。支持链式调用，提供更好的用户体验和代码可读性。

**注意**：本模块是基础设施模块，被整个项目广泛使用。所有需要用户交互的命令都使用这些对话框组件。

**模块统计：**
- 总代码行数：约 600+ 行
- 文件数量：5 个核心文件
- 主要组件：4 个对话框类型（InputDialog, SelectDialog, MultiSelectDialog, ConfirmDialog）
- 依赖库：
  - `inquire`（InputDialog, SelectDialog, MultiSelectDialog）
  - `dialoguer`（ConfirmDialog，支持单键自动完成和 Enter 使用默认值）

---

## 📁 模块结构

### 核心模块文件

```
src/lib/base/dialog/
├── mod.rs          # 模块声明和导出 (96行)
├── input.rs        # 文本输入对话框 (177行)
├── select.rs       # 单选对话框 (103行)
├── multi_select.rs # 多选对话框 (103行)
├── confirm.rs      # 确认对话框 (141行)
└── types.rs        # 类型定义 (6行)
```

### 依赖模块

- **`inquire` crate**：提供交互式终端 UI 功能
- **`anyhow` crate**：错误处理

### 模块集成

Dialog 模块被所有需要用户交互的命令广泛使用：

- **PR 命令**：使用 `InputDialog` 输入 PR 标题、描述等
- **Jira 命令**：使用 `InputDialog` 输入 Jira ID，使用 `SelectDialog` 选择操作
- **Branch 命令**：使用 `MultiSelectDialog` 选择要清理的分支
- **Config 命令**：使用 `SelectDialog` 选择配置项，使用 `InputDialog` 输入配置值
- **GitHub 命令**：使用 `SelectDialog` 选择账号
- **Lifecycle 命令**：使用 `ConfirmDialog` 确认操作

---

## 🏗️ 架构设计

### 设计原则

1. **统一接口**：所有对话框使用 `prompt()` 方法获取用户输入
2. **链式调用**：所有对话框支持链式配置，提供更好的代码可读性
3. **类型安全**：使用泛型支持任意类型（实现 `Display` trait）
4. **错误处理**：用户取消时返回错误，便于调用者处理

### 核心组件

#### 1. InputDialog - 文本输入对话框

提供文本输入功能，支持默认值、验证器和空值处理。

**主要方法**：
- `new(prompt)` - 创建新的输入对话框
- `with_default(default)` - 设置默认值
- `with_validator(validator)` - 设置验证器
- `allow_empty(allow)` - 允许空值
- `prompt()` - 显示对话框并获取用户输入

**特性**：
- 支持默认值
- 支持自定义验证器（返回 `Result<(), String>`）
- 支持空值处理
- 链式调用

**样式示例**：
```
┌─────────────────────────────────────┐
│ Enter your name:                    │
│ > John Doe                          │
│                                     │
│ [Press Enter to confirm]            │
└─────────────────────────────────────┘
```

**使用示例**：
```rust
use workflow::base::dialog::InputDialog;

// 简单输入
let name = InputDialog::new("Enter your name")
    .prompt()?;

// 带默认值
let email = InputDialog::new("Enter email")
    .with_default("user@example.com")
    .prompt()?;

// 带验证器
let age = InputDialog::new("Enter age")
    .with_validator(|input: &str| {
        if input.parse::<u32>().is_ok() {
            Ok(())
        } else {
            Err("Please enter a valid number".to_string())
        }
    })
    .prompt()?;

// 允许空值
let optional = InputDialog::new("Enter value (optional)")
    .allow_empty(true)
    .prompt()?;
```

#### 2. SelectDialog - 单选对话框

提供单选功能，从选项列表中选择一个选项。

**主要方法**：
- `new(prompt, options)` - 创建新的单选对话框
- `with_default(index)` - 设置默认选项索引
- `prompt()` - 显示对话框并获取用户选择

**特性**：
- 支持默认选项
- 支持任意类型（实现 `Display` trait）
- 返回选中项的所有权

**样式示例**：
```
┌─────────────────────────────────────┐
│ Choose an option:                    │
│                                     │
│   > Option 1                        │ ← 当前选中（高亮）
│     Option 2                        │
│     Option 3                        │
│                                     │
│ [↑↓: Move, Enter: Select, Esc: Cancel] │
└─────────────────────────────────────┘
```

**使用示例**：
```rust
use workflow::base::dialog::SelectDialog;

let options = vec!["Option 1", "Option 2", "Option 3"];
let selected = SelectDialog::new("Choose an option", options)
    .with_default(0)
    .prompt()?;
// selected 是 "Option 1" 或 "Option 2" 或 "Option 3"
```

#### 3. MultiSelectDialog - 多选对话框

提供多选功能，从选项列表中选择多个选项。

**主要方法**：
- `new(prompt, options)` - 创建新的多选对话框
- `with_default(indices)` - 设置默认选中的选项索引
- `prompt()` - 显示对话框并获取用户选择（返回 `Vec<T>`）

**特性**：
- 支持多选
- 支持默认选中多个选项
- 返回选中项列表的所有权

**样式示例**：
```
┌─────────────────────────────────────┐
│ Choose options:                      │
│                                     │
│   > [✓] Option 1                    │ ← 已选中
│     [ ] Option 2                    │
│     [✓] Option 3                    │ ← 已选中
│                                     │
│ [↑↓: Move, Space: Toggle, Enter: Confirm, Esc: Cancel] │
└─────────────────────────────────────┘
```

**使用示例**：
```rust
use workflow::base::dialog::MultiSelectDialog;

let options = vec!["Option 1", "Option 2", "Option 3"];
let selected = MultiSelectDialog::new("Choose options", options)
    .prompt()?;
// selected 是 Vec<&str>，包含选中的选项
```

#### 4. ConfirmDialog - 确认对话框

提供确认功能，用于获取用户的 yes/no 选择。

**主要方法**：
- `new(prompt)` - 创建新的确认对话框
- `with_default(default)` - 设置默认选择
- `with_cancel_message(message)` - 设置取消消息（取消时返回错误）
- `prompt()` - 显示对话框并获取用户确认

**特性**：
- 支持默认选择
- 支持取消消息（强制确认）

**样式示例**：

默认值为 true 时：
```
┌─────────────────────────────────────┐
│ Continue? (Y/n)                     │
│ > Yes                               │ ← 默认选中
│   No                                │
│                                     │
│ [Y: Yes, n: No, Enter: Confirm]     │
└─────────────────────────────────────┘
```

默认值为 false 时：
```
┌─────────────────────────────────────┐
│ This operation cannot be undone.    │
│ Continue? (y/N)                     │
│   Yes                               │
│ > No                                │ ← 默认选中
│                                     │
│ [y: Yes, N: No, Enter: Confirm]     │
└─────────────────────────────────────┘
```

**使用示例**：
```rust
use workflow::base::dialog::ConfirmDialog;

// 简单确认
let confirmed = ConfirmDialog::new("Continue?")
    .with_default(true)
    .prompt()?;

// 取消时返回错误
ConfirmDialog::new("This operation cannot be undone. Continue?")
    .with_default(false)
    .with_cancel_message("Operation cancelled.")
    .prompt()?;
```

### 设计模式

#### 链式调用设计

所有对话框支持链式配置，提供更好的代码可读性：

```rust
let result = InputDialog::new("Enter value")
    .with_default("default")
    .with_validator(|s| {
        // 验证逻辑
        Ok(())
    })
    .allow_empty(false)
    .prompt()?;
```

**优势**：
- 代码可读性强
- 配置灵活
- 类型安全

#### 错误处理策略

所有对话框在用户取消时返回错误：

```rust
match dialog.prompt() {
    Ok(value) => {
        // 处理用户输入
    }
    Err(e) => {
        // 处理错误（通常是用户取消）
        log_error!("{}", e);
    }
}
```

**优势**：
- 统一的错误处理方式
- 使用 `?` 操作符简化错误传播
- 使用 `anyhow` 提供详细的错误上下文

---

## 🔄 调用流程与数据流

### 整体架构流程

```
应用层（命令、模块）
  ↓
Dialog API（InputDialog, SelectDialog, MultiSelectDialog, ConfirmDialog）
  ↓
inquire 库（底层终端 UI）
  ↓
用户交互
  ↓
返回结果或错误
```

### 数据流

#### 输入对话框流程

```
InputDialog::new("prompt")
  ↓
with_default("default")  // 可选
  ↓
with_validator(|s| {...})  // 可选
  ↓
allow_empty(true)  // 可选
  ↓
prompt()
  ↓
显示对话框
  ↓
用户输入
  ↓
验证（如果有验证器）
  ↓
返回 Result<String>
```

#### 选择对话框流程

```
SelectDialog::new("prompt", options)
  ↓
with_default(0)  // 可选
  ↓
prompt()
  ↓
显示对话框
  ↓
用户选择
  ↓
返回 Result<T>
```

### 与其他模块的集成

Dialog 模块是基础设施模块，被整个项目广泛使用：

- **CLI 命令层**：所有需要用户交互的命令使用对话框
- **配置管理**：使用对话框获取配置值
- **Git 操作**：使用对话框选择分支、确认操作
- **Jira 操作**：使用对话框输入 Jira ID、选择操作
- **PR 操作**：使用对话框输入 PR 信息

**依赖关系**：

```
dialog (基础设施)
  ↓
所有业务模块（commands, lib/*）
```

---

## 📝 扩展性

### 添加新的对话框类型

1. 在 `dialog/` 目录下创建新的模块文件（如 `date_picker.rs`）
2. 实现对话框结构体和 `prompt()` 方法
3. 在 `mod.rs` 中声明模块并重新导出
4. 在 `src/lib/base/mod.rs` 中添加到全局导出（如果需要）

### 添加新的验证器

验证器是函数类型 `Fn(&str) -> Result<(), String>`，可以轻松添加：

```rust
let validator = |input: &str| -> Result<(), String> {
    if input.len() < 5 {
        Err("Input must be at least 5 characters".to_string())
    } else {
        Ok(())
    }
};

InputDialog::new("Enter value")
    .with_validator(validator)
    .prompt()?;
```

---

## 📚 相关文档

- [总体架构文档](../ARCHITECTURE.md)
- [TOOLS 模块架构文档](./TOOLS_ARCHITECTURE.md)
- [Indicator 模块架构文档](./INDICATOR_ARCHITECTURE.md)

---

## 📋 使用示例

### 文本输入

```rust
use workflow::base::dialog::InputDialog;

let name = InputDialog::new("Enter your name")
    .with_default("John Doe")
    .prompt()?;
```

### 单选

```rust
use workflow::base::dialog::SelectDialog;

let options = vec!["Option 1", "Option 2", "Option 3"];
let selected = SelectDialog::new("Choose an option", options)
    .with_default(0)
    .prompt()?;
```

### 多选

```rust
use workflow::base::dialog::MultiSelectDialog;

let options = vec!["Option 1", "Option 2", "Option 3"];
let selected = MultiSelectDialog::new("Choose options", options)
    .prompt()?;
```

### 确认

```rust
use workflow::base::dialog::ConfirmDialog;

let confirmed = ConfirmDialog::new("Continue?")
    .with_default(true)
    .prompt()?;
```

---

## ✅ 总结

Dialog 模块为整个项目提供统一的交互式对话框接口：

1. **InputDialog**：文本输入，支持默认值、验证器、空值处理
2. **SelectDialog**：单选，支持默认选项
3. **MultiSelectDialog**：多选，支持默认选中多个选项
4. **ConfirmDialog**：确认，支持默认选择和取消消息

**设计优势**：
- ✅ **易用性**：简洁的 API，支持链式调用
- ✅ **一致性**：统一的错误处理方式
- ✅ **类型安全**：使用泛型支持任意类型
- ✅ **灵活性**：支持默认值、验证器、空值处理等多种配置
- ✅ **用户体验**：使用 `inquire` 和 `dialoguer` 提供美观的终端 UI，`ConfirmDialog` 支持单键自动完成

---

**最后更新**: 2025-12-16
