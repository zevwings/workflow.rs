# Prompt 管理模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Prompt 管理模块架构，包括：
- Prompt 作为编译时常量嵌入到二进制文件中
- 多语言支持（语言增强功能）
- Prompt 生成函数（根据语言动态生成）

该模块为整个应用提供统一的 Prompt 管理基础设施，Prompt 内容作为编译时常量直接嵌入到二进制文件中，便于维护和版本控制。

**模块统计：**
- 总代码行数：约 143 行（summarize-_pr.system.rs）+ 约 200 行（generate-_branch.system.rs）
- 文件数量：2 个 Prompt 文件（generate-_branch.system.rs, summarize-_pr.system.rs）
- 主要组件：2 个（GENERATE_BRANCH_SYSTEM_PROMPT 常量，generate-_summarize-_pr-_system-_prompt 函数）
- 语言支持：通过 `lib/base/llm/languages.rs` 提供多语言支持

---

## 📁 模块结构

### 核心模块文件

```
src/lib/base/prompt/
├── mod.rs                      # 模块声明和导出 (28行)
├── generate-_branch.system.rs   # 生成分支名的 system prompt (约 200 行)
└── summarize-_pr.system.rs      # PR 总结的 system prompt (143行)
```

### 依赖模块

- **`lib/base/llm/languages.rs`**：多语言支持（`get-_language-_requirement` 函数）
- **`lib/base/llm/mod.rs`**：语言相关 API 重新导出

### 模块集成

#### PR 模块

- **`lib/pr/llm.rs`**：PR LLM 服务
  - `GENERATE_BRANCH_SYSTEM_PROMPT` - 使用编译时嵌入的 prompt 常量
  - `generate-_summarize-_pr-_system-_prompt(language)` - 根据语言生成 PR 总结 prompt

#### 使用场景

- **Prompt 使用**：所有需要 LLM prompt 的模块直接使用编译时嵌入的常量或函数
- **多语言支持**：通过 `get-_language-_requirement` 函数增强 prompt 中的语言要求

---

## 🏗️ 架构设计

### 设计原则

1. **编译时嵌入**：Prompt 作为编译时常量直接嵌入到二进制文件中，无需运行时文件读取
2. **多语言支持**：通过 `get-_language-_requirement` 函数根据语言动态增强 prompt
3. **类型安全**：使用 Rust 常量（`pub const`）和函数，编译时检查
4. **易于维护**：Prompt 内容在源代码中，便于版本控制和代码审查
5. **性能优化**：编译时嵌入，无需运行时文件 I/O 操作

### 核心组件

#### 1. GENERATE_BRANCH_SYSTEM_PROMPT (`generate-_branch.system.rs`)

**职责**：提供生成分支名和 PR 标题的 system prompt

**关键特性**：

- ✅ **编译时常量**：使用 `pub const` 定义，编译时嵌入到二进制文件
- ✅ **直接使用**：无需文件加载，直接使用常量
- ✅ **类型安全**：编译时检查，确保 prompt 内容正确

**使用场景**：

- PR 创建时生成分支名和 PR 标题
- 通过 `PullRequestLLM::generate()` 调用

#### 2. generate-_summarize-_pr-_system-_prompt (`summarize-_pr.system.rs`)

**职责**：根据语言生成 PR 总结的 system prompt

**主要方法**：

- `generate-_summarize-_pr-_system-_prompt(language: &str) -> String` - 根据语言代码生成 system prompt

**关键特性**：

- ✅ **多语言支持**：支持多种语言（en, zh-CN, zh-TW, ja, ko, de, fr, es, pt, ru 等）
- ✅ **语言增强**：通过 `get-_language-_requirement` 函数增强 prompt 中的语言要求
- ✅ **动态生成**：根据语言代码动态生成包含语言要求的 prompt
- ✅ **详细指导**：包含详细的要求分析、功能说明、用户场景等指导

**使用场景**：

- PR 总结时生成多语言的总结文档
- 通过 `PullRequestLLM::summarize-_pr()` 调用

### 设计模式

#### 1. 编译时常量模式

使用 Rust 的 `pub const` 定义编译时常量：

```rust
pub const GENERATE_BRANCH_SYSTEM_PROMPT: &str = r#"..."#;
```

**优势**：
- 零运行时开销：编译时嵌入，无需运行时文件 I/O
- 类型安全：编译时检查，确保 prompt 内容正确
- 易于维护：Prompt 内容在源代码中，便于版本控制

#### 2. 函数式生成模式

使用函数根据参数动态生成 prompt：

```rust
pub fn generate-_summarize-_pr-_system-_prompt(language: &str) -> String {
    let base-_prompt = r#"..."#;
    get-_language-_requirement(base-_prompt, language)
}
```

**优势**：
- 灵活性：根据语言动态生成不同的 prompt
- 可扩展性：易于添加新的语言支持
- 统一管理：所有语言增强逻辑集中在一个函数中

### 错误处理

#### 语言代码处理

1. **语言代码验证**：如果提供的语言代码不在支持列表中，使用英文作为默认语言
2. **语言查找**：通过 `find-_language()` 函数查找支持的语言
3. **默认回退**：如果找不到匹配的语言，使用英文的默认 instruction

#### 容错机制

- **语言代码不匹配**：自动回退到英文
- **语言增强失败**：使用基础 prompt，不包含语言要求增强

---

## 🔄 调用流程与数据流

### 整体架构流程

#### 1. 使用编译时常量（GENERATE_BRANCH_SYSTEM_PROMPT）

```
使用 GENERATE_BRANCH_SYSTEM_PROMPT 常量
  ↓
直接使用编译时嵌入的 prompt 内容
  ↓
返回 Prompt 字符串
```

#### 2. 使用函数生成（generate-_summarize-_pr-_system-_prompt）

```
调用 generate-_summarize-_pr-_system-_prompt(language)
  ↓
查找语言（find-_language(language)）
  ↓
获取语言 instruction（get-_language-_instruction(language)）
  ↓
增强 prompt（get-_language-_requirement(base-_prompt, language)）
  ↓
返回增强后的 Prompt 字符串
```

### 典型调用示例

#### 1. 使用编译时常量

```rust
use workflow::base::prompt::GENERATE_BRANCH_SYSTEM_PROMPT;

// 直接使用编译时嵌入的 prompt
let system-_prompt = GENERATE_BRANCH_SYSTEM_PROMPT.to-_string();
```

**流程**：
1. 直接使用常量，无需任何运行时操作
2. 编译时已嵌入到二进制文件
3. 零运行时开销

#### 2. 使用函数生成（多语言支持）

```rust
use workflow::base::prompt::generate-_summarize-_pr-_system-_prompt;

// 根据语言生成 prompt
let system-_prompt = generate-_summarize-_pr-_system-_prompt("zh-CN");
```

**流程**：
1. 调用函数，传入语言代码
2. 查找支持的语言（`find-_language()`）
3. 获取语言 instruction（`get-_language-_instruction()`）
4. 增强基础 prompt（`get-_language-_requirement()`）
5. 返回包含语言要求的完整 prompt

### 数据流

#### 编译时常量流程

```
源代码中的 const 定义
  ↓
编译时嵌入到二进制文件
  ↓
运行时直接使用
```

#### 函数生成流程

```
语言代码（如 "zh-CN"）
  ↓
find-_language() 查找语言
  ↓
get-_language-_instruction() 获取 instruction
  ↓
get-_language-_requirement() 增强 prompt
  ↓
返回增强后的 Prompt 字符串
```

---

## 📝 文件结构

### Prompt 文件组织

Prompt 文件作为 Rust 源文件，直接放在 `src/lib/base/prompt/` 目录下：

```
src/lib/base/prompt/
├── mod.rs                      # 模块声明和导出
├── generate-_branch.system.rs   # 生成分支名的 system prompt（编译时常量）
└── summarize-_pr.system.rs     # PR 总结的 system prompt（函数生成）
```

### 文件命名规则

- **Rust 源文件**：所有 Prompt 文件使用 `.rs` 扩展名
- **命名格式**：使用点号分隔，格式为 `{功能}.{类型}.rs`（如 `generate-_branch.system.rs`）
- **编译时嵌入**：使用 `pub const` 定义编译时常量，或使用函数动态生成
- **模块路径**：使用 `#[path]` 属性指定文件路径

### 文件内容结构

#### 编译时常量（generate-_branch.system.rs）

```rust
pub const GENERATE_BRANCH_SYSTEM_PROMPT: &str = r#"..."#;
```

#### 函数生成（summarize-_pr.system.rs）

```rust
pub fn generate-_summarize-_pr-_system-_prompt(language: &str) -> String {
    let base-_prompt = r#"..."#;
    get-_language-_requirement(base-_prompt, language)
}
```

---

## 📝 扩展性

### 添加新的 Prompt 常量

1. 在 `src/lib/base/prompt/` 目录下创建新的 Rust 源文件（如 `new-_feature.system.rs`）
2. 使用 `pub const` 定义编译时常量：

```rust
pub const NEW_FEATURE_SYSTEM_PROMPT: &str = r#"..."#;
```

3. 在 `mod.rs` 中声明模块并重新导出：

```rust
#[path = "new-_feature.system.rs"]
pub mod new-_feature-_system;

pub use new-_feature-_system::NEW_FEATURE_SYSTEM_PROMPT;
```

### 添加新的 Prompt 生成函数

1. 在 `src/lib/base/prompt/` 目录下创建新的 Rust 源文件（如 `new-_feature.system.rs`）
2. 实现生成函数：

```rust
use crate::base::llm::get-_language-_requirement;

pub fn generate-_new-_feature-_system-_prompt(language: &str) -> String {
    let base-_prompt = r#"..."#;
    get-_language-_requirement(base-_prompt, language)
}
```

3. 在 `mod.rs` 中声明模块并重新导出：

```rust
#[path = "new-_feature.system.rs"]
pub mod new-_feature-_system;

pub use new-_feature-_system::generate-_new-_feature-_system-_prompt;
```

### 修改现有 Prompt

1. 直接编辑 `src/lib/base/prompt/` 目录下的 Rust 源文件
2. 重新编译项目（编译时常量会在编译时更新）

### 添加新的语言支持

1. 在 `src/lib/base/llm/languages.rs` 中的 `SUPPORTED_LANGUAGES` 数组添加新语言：

```rust
SupportedLanguage {
    code: "new-lang",
    name: "New Language",
    native-_name: "新语言",
    instruction-_template: "**所有输出必须使用新语言。**",
},
```

2. 语言系统会自动支持新语言，无需修改 Prompt 文件

---

## 📚 相关文档

- [主架构文档](../architecture.md)
- [LLM 模块架构文档](./LLM_architecture.md) - LLM 客户端使用 Prompt
- [PR 模块架构文档](./PR_architecture.md) - PR 模块使用 Prompt 生成分支名和 PR 标题

---

## 📋 使用示例

### 使用编译时常量

```rust
use workflow::base::prompt::GENERATE_BRANCH_SYSTEM_PROMPT;

// 直接使用编译时嵌入的 prompt
let system-_prompt = GENERATE_BRANCH_SYSTEM_PROMPT.to-_string();
```

### 使用函数生成（多语言支持）

```rust
use workflow::base::prompt::generate-_summarize-_pr-_system-_prompt;

// 根据语言生成 prompt
let system-_prompt = generate-_summarize-_pr-_system-_prompt("zh-CN");
```

### 在 PR 模块中使用

#### 生成分支名和 PR 标题

```rust
// src/lib/pr/llm.rs
use workflow::base::prompt::GENERATE_BRANCH_SYSTEM_PROMPT;

let system-_prompt = GENERATE_BRANCH_SYSTEM_PROMPT.to-_string();
```

#### 生成 PR 总结

```rust
// src/lib/pr/llm.rs
use workflow::base::prompt::generate-_summarize-_pr-_system-_prompt;

let language = "zh-CN"; // 或从配置/参数获取
let system-_prompt = generate-_summarize-_pr-_system-_prompt(language);
```

---

## ⚠️ 注意事项

1. **编译时嵌入**：Prompt 内容在编译时嵌入到二进制文件，修改后需要重新编译才能生效
2. **语言代码**：如果提供的语言代码不在支持列表中，会自动回退到英文
3. **性能优化**：编译时常量零运行时开销，函数生成也无需文件 I/O 操作
4. **类型安全**：使用 Rust 常量，编译时检查，确保 prompt 内容正确
5. **多语言支持**：通过 `get-_language-_requirement` 函数增强 prompt 中的语言要求，确保 LLM 按照指定语言生成内容

---

**最后更新**: 2025-12-16
