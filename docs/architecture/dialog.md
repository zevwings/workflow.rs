# Dialog 模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Dialog 模块架构，包括：
- 基础对话框组件：
  - 文本输入对话框（InputDialog）
  - 单选对话框（SelectDialog）
  - 多选对话框（MultiSelectDialog）
  - 确认对话框（ConfirmDialog）
- 表单构建器（FormBuilder）：
  - 支持 Group/Step/Field 三层结构
  - 支持条件逻辑和可选组
  - 提供统一的表单构建和交互接口

该模块提供统一的交互式对话框接口，使用 `inquire` 和 `dialoguer` 作为后端实现。支持链式调用，提供更好的用户体验和代码可读性。

**注意**：本模块是基础设施模块，被整个项目广泛使用。所有需要用户交互的命令都使用这些对话框组件。

**模块统计：**
- 总代码行数：约 2000+ 行
- 文件数量：11 个核心文件（5 个基础对话框 + 6 个 Form 子模块文件）
- 主要组件：
  - 4 个基础对话框类型（InputDialog, SelectDialog, MultiSelectDialog, ConfirmDialog）
  - 1 个表单构建器（FormBuilder）及其支持组件
- 依赖库：
  - `inquire`（InputDialog, SelectDialog, MultiSelectDialog）
  - `dialoguer`（ConfirmDialog，支持单键自动完成和 Enter 使用默认值；Password 输入）

---

## 📁 Lib 层架构（核心业务逻辑）

### 核心模块文件

```
src/lib/base/dialog/
├── mod.rs          # 模块声明和导出 (121行)
├── input.rs        # 文本输入对话框 (180行)
├── select.rs       # 单选对话框 (159行)
├── multi-_select.rs # 多选对话框 (106行)
├── confirm.rs      # 确认对话框 (139行)
├── types.rs        # 类型定义 (6行)
└── form/           # 表单构建器子模块
    ├── mod.rs              # Form 模块声明和导出
    ├── builder.rs          # FormBuilder 主实现 (391行)
    ├── group-_builder.rs     # GroupBuilder 实现 (137行)
    ├── field-_builder.rs     # FieldBuilder 实现 (182行)
    ├── condition-_evaluator.rs # 条件评估器 (58行)
    └── types.rs             # Form 类型定义 (281行)
```

### 依赖模块

- **`inquire` crate**：提供交互式终端 UI 功能
- **`dialoguer` crate**：提供确认对话框和密码输入功能
- **`color-_eyre` crate**：错误处理

### 模块集成

Dialog 模块被所有需要用户交互的命令广泛使用：

- **PR 命令**：使用 `InputDialog` 输入 PR 标题、描述等
- **Jira 命令**：使用 `InputDialog` 输入 Jira ID，使用 `SelectDialog` 选择操作
- **Branch 命令**：使用 `MultiSelectDialog` 选择要清理的分支
- **Config 命令**：使用 `FormBuilder` 构建完整的配置表单，使用 `SelectDialog` 选择配置项
- **Repo 命令**：使用 `FormBuilder` 构建仓库配置表单
- **LLM 命令**：使用 `FormBuilder` 构建 LLM 配置表单
- **Alias 命令**：使用 `FormBuilder` 构建别名配置表单
- **MCP 命令**：使用 `FormBuilder` 构建 MCP 配置表单
- **GitHub 命令**：使用 `SelectDialog` 选择账号
- **Lifecycle 命令**：使用 `ConfirmDialog` 确认操作

---

## 🔄 集成关系

Dialog 模块是 Workflow CLI 的基础设施模块，为所有需要用户交互的命令提供统一的对话框接口。该模块通过以下方式与其他模块集成：

1. **命令层集成**：所有命令层模块通过 Dialog 模块提供的接口进行用户交互
2. **表单构建**：通过 `FormBuilder` 提供复杂的表单构建功能，支持条件显示、验证等
3. **统一体验**：提供统一的用户交互体验，确保所有命令的交互方式一致

### 主要集成场景

- **PR 命令**：使用 `InputDialog` 输入 PR 标题、描述等
- **Jira 命令**：使用 `InputDialog` 输入 Jira ID，使用 `SelectDialog` 选择操作
- **Branch 命令**：使用 `MultiSelectDialog` 选择要清理的分支
- **Config 命令**：使用 `FormBuilder` 构建完整的配置表单
- **LLM 命令**：使用 `FormBuilder` 构建 LLM 配置表单
- **Alias 命令**：使用 `FormBuilder` 构建别名配置表单

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
- `with-_default(default)` - 设置默认值
- `with-_validator(validator)` - 设置验证器
- `allow-_empty(allow)` - 允许空值
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
    .with-_default("user@example.com")
    .prompt()?;

// 带验证器
let age = InputDialog::new("Enter age")
    .with-_validator(|input: &str| {
        if input.parse::<u32>().is-_ok() {
            Ok(())
        } else {
            Err("Please enter a valid number".to-_string())
        }
    })
    .prompt()?;

// 允许空值
let optional = InputDialog::new("Enter value (optional)")
    .allow-_empty(true)
    .prompt()?;
```

#### 2. SelectDialog - 单选对话框

提供单选功能，从选项列表中选择一个选项。

**主要方法**：
- `new(prompt, options)` - 创建新的单选对话框
- `with-_default(index)` - 设置默认选项索引
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
    .with-_default(0)
    .prompt()?;
// selected 是 "Option 1" 或 "Option 2" 或 "Option 3"
```

#### 3. MultiSelectDialog - 多选对话框

提供多选功能，从选项列表中选择多个选项。

**主要方法**：
- `new(prompt, options)` - 创建新的多选对话框
- `with-_default(indices)` - 设置默认选中的选项索引
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
- `with-_default(default)` - 设置默认选择
- `with-_cancel-_message(message)` - 设置取消消息（取消时返回错误）
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
    .with-_default(true)
    .prompt()?;

// 取消时返回错误
ConfirmDialog::new("This operation cannot be undone. Continue?")
    .with-_default(false)
    .with-_cancel-_message("Operation cancelled.")
    .prompt()?;
```

#### 5. FormBuilder - 表单构建器

提供高级表单构建功能，支持 Group/Step/Field 三层结构，可以将复杂的配置流程封装为一个统一的表单。

**核心概念**：
- **Group（组）**：表单的最高层级，可以包含多个步骤，支持必填组和可选组
- **Step（步骤）**：组内的逻辑单元，可以包含多个字段，支持条件执行
- **Field（字段）**：表单的基本输入单元，支持多种字段类型

**主要方法**：
- `new()` - 创建新的表单构建器
- `add-_group(id, builder, config)` - 添加表单组
- `run()` - 执行表单并收集用户输入

**GroupBuilder 方法**：
- `step(builder)` - 添加无条件步骤
- `step-_if(field-_name, value, builder)` - 添加单条件步骤（字段值等于指定值时执行）
- `step-_if-_all(conditions, builder)` - 添加多条件步骤（所有条件都满足时执行，AND 逻辑）
- `step-_if-_any(conditions, builder)` - 添加多条件步骤（任一条件满足时执行，OR 逻辑）
- `step-_if-_dynamic(condition-_fn, builder)` - 添加动态条件步骤（基于运行时值）

**FieldBuilder 方法**：
- `add-_text(name, message)` - 添加文本输入字段
- `add-_password(name, message)` - 添加密码输入字段
- `add-_selection(name, message, choices)` - 添加选择字段
- `add-_confirmation(name, message)` - 添加确认字段
- `required()` - 标记字段为必填
- `default(value)` - 设置字段默认值
- `validate(validator)` - 设置字段验证器
- `allow-_empty(allow)` - 允许字段为空

**特性**：
- 支持 Group/Step/Field 三层结构
- 支持必填组和可选组
- 支持步骤级条件逻辑（step-_if, step-_if-_all, step-_if-_any, step-_if-_dynamic）
- 支持字段级条件逻辑
- 支持多种字段类型（Text, Password, Selection, Confirmation）
- 支持字段验证和默认值
- 链式调用，提供流畅的 API

**使用示例**：

```rust
use workflow::base::dialog::{FormBuilder, GroupConfig};

// 基本用法：必填组
let form-_result = FormBuilder::new()
    .add-_group("jira", |g| {
        g.step(|f| {
            f.add-_text("jira-_email", "Jira email address").required()
        })
        .step(|f| {
            f.add-_text("jira-_service-_address", "Jira service address").required()
        })
    }, GroupConfig::required())
    .run()?;

// 可选组（带标题和描述）
let form-_result = FormBuilder::new()
    .add-_group("llm", |g| {
        g.step(|f| {
            f.add-_selection("llm-_provider", "Select LLM provider",
                vec!["openai".into(), "deepseek".into()])
        })
        .step-_if("llm-_provider", "openai", |f| {
            f.add-_text("openai-_key", "OpenAI API key").required()
        })
        .step-_if("llm-_provider", "deepseek", |f| {
            f.add-_text("deepseek-_key", "DeepSeek API key").required()
        })
    }, GroupConfig::optional()
        .with-_title("LLM/AI Configuration")
        .with-_description("Configure LLM provider and API keys")
        .with-_default-_enabled(true))
    .run()?;

// 多条件步骤
let form-_result = FormBuilder::new()
    .add-_group("advanced", |g| {
        g.step(|f| {
            f.add-_text("provider", "Provider").required()
        })
        .step-_if-_all([
            ("provider", "openai"),
            ("environment", "production")
        ], |f| {
            f.add-_text("api-_key", "Production API key").required()
        })
        .step-_if-_any([
            ("provider", "openai"),
            ("provider", "deepseek")
        ], |f| {
            f.add-_confirmation("use-_proxy", "Use proxy?")
        })
    }, GroupConfig::required())
    .run()?;

// 动态条件步骤
let form-_result = FormBuilder::new()
    .add-_group("dynamic", |g| {
        g.step(|f| {
            f.add-_text("count", "Item count").required()
        })
        .step-_if-_dynamic(|result| {
            result.get("count")
                .and-_then(|v| v.parse::<i32>().ok())
                .map(|n| n > 10)
                .unwrap-_or(false)
        }, |f| {
            f.add-_text("bulk-_discount", "Bulk discount code")
        })
    }, GroupConfig::required())
    .run()?;

// 访问表单结果
let jira-_email = form-_result.get-_required("jira-_email")?;
let llm-_provider = form-_result.get("llm-_provider").cloned();
let use-_proxy = form-_result.get-_bool("use-_proxy").unwrap-_or(false);
```

**架构设计**：

FormBuilder 采用三层构建器模式：

```
FormBuilder
  ↓
GroupBuilder (组构建器)
  ↓
FieldBuilder (字段构建器)
```

**执行流程**：

1. **验证阶段**：检查组 ID 唯一性、步骤非空、字段非空
2. **组执行阶段**：
   - 可选组：先询问用户是否配置
   - 必填组：直接执行
   - 显示组标题和描述（如果有）
3. **步骤执行阶段**：
   - 评估步骤条件（如果有）
   - 如果条件满足，执行步骤内的字段
4. **字段执行阶段**：
   - 评估字段条件（如果有）
   - 如果条件满足，显示对话框收集用户输入
   - 验证字段值（如果有验证器）
   - 存储字段值到结果映射

**条件评估**：

FormBuilder 支持多种条件类型：

- **单条件**：`step-_if(field-_name, value)` - 字段值等于指定值时执行
- **多条件 AND**：`step-_if-_all([...])` - 所有条件都满足时执行
- **多条件 OR**：`step-_if-_any([...])` - 任一条件满足时执行
- **动态条件**：`step-_if-_dynamic(fn)` - 基于运行时值判断

条件操作符：
- `Equals`：等于（不区分大小写）
- `NotEquals`：不等于（不区分大小写）
- `In`：在列表中
- `NotIn`：不在列表中

### 设计模式

#### 链式调用设计

所有对话框支持链式配置，提供更好的代码可读性：

```rust
let result = InputDialog::new("Enter value")
    .with-_default("default")
    .with-_validator(|s| {
        // 验证逻辑
        Ok(())
    })
    .allow-_empty(false)
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
        log-_error!("{}", e);
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

#### 基础对话框流程

```
应用层（命令、模块）
  ↓
Dialog API（InputDialog, SelectDialog, MultiSelectDialog, ConfirmDialog）
  ↓
inquire/dialoguer 库（底层终端 UI）
  ↓
用户交互
  ↓
返回结果或错误
```

#### FormBuilder 流程

```
应用层（命令、模块）
  ↓
FormBuilder API
  ↓
GroupBuilder → StepBuilder → FieldBuilder
  ↓
基础 Dialog API（InputDialog, SelectDialog, ConfirmDialog）
  ↓
inquire/dialoguer 库（底层终端 UI）
  ↓
用户交互
  ↓
条件评估（ConditionEvaluator）
  ↓
返回 FormResult 或错误
```

### 数据流

#### 输入对话框流程

```
InputDialog::new("prompt")
  ↓
with-_default("default")  // 可选
  ↓
with-_validator(|s| {...})  // 可选
  ↓
allow-_empty(true)  // 可选
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
with-_default(0)  // 可选
  ↓
prompt()
  ↓
显示对话框
  ↓
用户选择
  ↓
返回 Result<T>
```

#### FormBuilder 表单流程

```
FormBuilder::new()
  ↓
add-_group("id", |g| {...}, GroupConfig::required())
  ↓
GroupBuilder::step(|f| {...})
  ↓
FieldBuilder::add-_text(...).required()
  ↓
run()
  ↓
验证配置（validate）
  ↓
执行组（可选组先询问）
  ↓
执行步骤（评估条件）
  ↓
执行字段（评估条件，显示对话框）
  ↓
收集用户输入
  ↓
返回 FormResult
```

### 与其他模块的集成

Dialog 模块是基础设施模块，被整个项目广泛使用：

- **CLI 命令层**：所有需要用户交互的命令使用对话框
- **配置管理**：
  - 使用 `FormBuilder` 构建完整的配置表单（`config setup`, `repo setup`, `llm setup`, `mcp setup`, `alias add`）
  - 使用基础对话框获取简单配置值
- **Git 操作**：使用对话框选择分支、确认操作
- **Jira 操作**：使用对话框输入 Jira ID、选择操作
- **PR 操作**：使用对话框输入 PR 信息

**依赖关系**：

```
dialog (基础设施)
  ├── form/ (FormBuilder 子模块)
  │   ├── builder.rs (使用基础对话框)
  │   ├── group-_builder.rs
  │   ├── field-_builder.rs
  │   └── condition-_evaluator.rs
  └── 基础对话框 (InputDialog, SelectDialog, MultiSelectDialog, ConfirmDialog)
  ↓
所有业务模块（commands, lib/*）
```

---

## 📝 扩展性

### 添加新的对话框类型

1. 在 `dialog/` 目录下创建新的模块文件（如 `date-_picker.rs`）
2. 实现对话框结构体和 `prompt()` 方法
3. 在 `mod.rs` 中声明模块并重新导出
4. 在 `src/lib/base/mod.rs` 中添加到全局导出（如果需要）

### 添加新的验证器

验证器是函数类型 `Fn(&str) -> Result<(), String>`，可以轻松添加：

```rust
let validator = |input: &str| -> Result<(), String> {
    if input.len() < 5 {
        Err("Input must be at least 5 characters".to-_string())
    } else {
        Ok(())
    }
};

InputDialog::new("Enter value")
    .with-_validator(validator)
    .prompt()?;
```

### 添加新的表单字段类型

1. 在 `form/types.rs` 中的 `FormFieldType` 枚举添加新类型
2. 在 `form/field-_builder.rs` 中添加对应的 `add-_xxx` 方法
3. 在 `form/builder.rs` 的 `ask-_field` 方法中添加字段类型的处理逻辑

### 添加新的条件操作符

1. 在 `form/types.rs` 中的 `ConditionOperator` 枚举添加新操作符
2. 在 `form/condition-_evaluator.rs` 的 `evaluate` 方法中添加对应的评估逻辑

---

## 📚 相关文档

- [总体架构文档](../architecture.md)
- [TOOLS 模块架构文档](./tools.md)
- [Indicator 模块架构文档](./indicator.md)

---

## 📋 使用示例

### 文本输入

```rust
use workflow::base::dialog::InputDialog;

let name = InputDialog::new("Enter your name")
    .with-_default("John Doe")
    .prompt()?;
```

### 单选

```rust
use workflow::base::dialog::SelectDialog;

let options = vec!["Option 1", "Option 2", "Option 3"];
let selected = SelectDialog::new("Choose an option", options)
    .with-_default(0)
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
    .with-_default(true)
    .prompt()?;
```

### 表单构建器

```rust
use workflow::base::dialog::{FormBuilder, GroupConfig};

let form-_result = FormBuilder::new()
    .add-_group("jira", |g| {
        g.step(|f| {
            f.add-_text("jira-_email", "Jira email address").required()
        })
        .step(|f| {
            f.add-_text("jira-_service-_address", "Jira service address").required()
        })
    }, GroupConfig::required())
    .add-_group("llm", |g| {
        g.step(|f| {
            f.add-_selection("llm-_provider", "Select LLM provider",
                vec!["openai".into(), "deepseek".into()])
        })
        .step-_if("llm-_provider", "openai", |f| {
            f.add-_text("openai-_key", "OpenAI API key").required()
        })
    }, GroupConfig::optional()
        .with-_title("LLM Configuration")
        .with-_default-_enabled(true))
    .run()?;

// 访问结果
let jira-_email = form-_result.get-_required("jira-_email")?;
let llm-_provider = form-_result.get("llm-_provider");
```

---

## ✅ 总结

Dialog 模块为整个项目提供统一的交互式对话框接口：

### 基础对话框组件

1. **InputDialog**：文本输入，支持默认值、验证器、空值处理
2. **SelectDialog**：单选，支持默认选项
3. **MultiSelectDialog**：多选，支持默认选中多个选项
4. **ConfirmDialog**：确认，支持默认选择和取消消息

### 表单构建器（FormBuilder）

5. **FormBuilder**：高级表单构建器，支持：
   - Group/Step/Field 三层结构
   - 必填组和可选组
   - 步骤级和字段级条件逻辑
   - 多种字段类型（Text, Password, Selection, Confirmation）
   - 字段验证和默认值
   - 链式调用 API

**设计优势**：
- ✅ **易用性**：简洁的 API，支持链式调用
- ✅ **一致性**：统一的错误处理方式
- ✅ **类型安全**：使用泛型支持任意类型
- ✅ **灵活性**：支持默认值、验证器、空值处理等多种配置
- ✅ **用户体验**：使用 `inquire` 和 `dialoguer` 提供美观的终端 UI，`ConfirmDialog` 支持单键自动完成
- ✅ **可扩展性**：FormBuilder 支持复杂的条件逻辑和动态表单构建
- ✅ **模块化**：三层构建器模式，职责清晰，易于维护

---

**最后更新**: 2025-12-23
