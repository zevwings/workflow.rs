# Prompt 管理模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Prompt 管理模块架构，包括：
- Prompt 文件的加载和管理
- 文件缓存机制（避免重复读取）
- 线程安全的 Prompt 管理
- 扁平化文件结构设计

该模块为整个应用提供统一的 Prompt 管理基础设施，支持从文件加载 Prompt，便于维护和版本控制。

**模块统计：**
- 总代码行数：约 263 行
- 文件数量：2 个核心文件（manager.rs, mod.rs）
- 主要组件：1 个（PromptManager）
- Prompt 文件：`prompts/` 目录下的 Markdown 文件

---

## 📁 模块结构

### 核心模块文件

```
src/lib/base/prompt/
├── mod.rs          # 模块声明和导出 (27行)
├── manager.rs      # PromptManager 实现 (238行)
└── prompts/        # Prompt 文件目录
    └── generate_branch.system.md  # 示例 Prompt 文件
```

### 依赖模块

- **`anyhow`**：错误处理
- **`std::collections::HashMap`**：缓存存储
- **`std::sync::{Mutex, OnceLock}`**：线程安全

### 模块集成

#### PR 模块

- **`lib/pr/llm.rs`**：PR LLM 服务
  - `PromptManager::load("generate_branch.system.md")` - 加载 system prompt

#### 使用场景

- **Prompt 加载**：所有需要 LLM prompt 的模块通过 `PromptManager::load()` 加载
- **文件管理**：Prompt 文件统一放在 `prompts/` 目录下，便于维护

---

## 🏗️ 架构设计

### 设计原则

1. **文件优先**：Prompt 从文件加载，便于维护和版本控制
2. **扁平化结构**：所有 Prompt 文件直接放在 `prompts/` 目录下，不使用子文件夹
3. **缓存机制**：使用内存缓存避免重复读取文件，提高性能
4. **线程安全**：使用 `Mutex` 和 `OnceLock` 保证线程安全
5. **完整文件名**：必须使用完整文件名（包含扩展名），如 `"generate_branch.system.md"`

### 核心组件

#### 1. PromptManager (`manager.rs`)

**职责**：提供统一的 Prompt 加载和管理功能

**主要方法**：

- `load(name: &str) -> Result<String>` - 从文件加载 Prompt（文件不存在会返回错误）
- `load_or_default<F>(name: &str, default_fn: F) -> Result<String>` - 从文件加载 Prompt，如果文件不存在则使用默认值
- `clear_cache()` - 清除所有缓存的 Prompt（主要用于测试）
- `load_from_file(name: &str) -> Result<String>` - 从文件加载 Prompt（内部方法）
- `name_to_path(name: &str) -> Result<PathBuf>` - 将 Prompt 名称转换为文件路径（内部方法）
- `get_from_cache(name: &str) -> Option<String>` - 从缓存获取 Prompt（内部方法）
- `put_to_cache(name: &str, content: &str)` - 将 Prompt 存入缓存（内部方法）

**关键特性**：

- ✅ **文件加载**：从 `prompts/` 目录加载 Prompt 文件
- ✅ **缓存机制**：使用 `HashMap` 缓存已加载的 Prompt，避免重复读取文件
- ✅ **线程安全**：使用 `Mutex` 和 `OnceLock` 保证线程安全
- ✅ **路径管理**：使用 `env!("CARGO_MANIFEST_DIR")` 在编译时确定文件路径
- ✅ **文件格式支持**：支持 `.md` 和 `.txt` 格式（优先 `.md`）

**使用场景**：

- LLM 模块加载 system prompt
- 需要从文件加载 prompt 的所有场景

### 设计模式

#### 1. 单例模式

使用 `OnceLock` 和 `Mutex` 实现线程安全的单例缓存：

```rust
fn prompt_cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}
```

**优势**：
- 线程安全：使用 `Mutex` 保护共享状态
- 懒加载：使用 `OnceLock` 实现懒初始化
- 性能优化：避免重复读取文件

#### 2. 策略模式

支持两种加载策略：
- `load()` - 强制从文件加载（文件不存在返回错误）
- `load_or_default()` - 从文件加载，失败则使用默认值

**优势**：
- 灵活性：根据场景选择不同的加载策略
- 容错性：`load_or_default` 提供默认值回退

### 错误处理

#### 分层错误处理

1. **文件加载错误**：文件不存在或读取失败
2. **路径解析错误**：文件路径构建失败

#### 容错机制

- **文件不存在**：`load()` 返回错误，`load_or_default()` 使用默认值
- **缓存失效**：缓存不存在时自动从文件加载
- **路径错误**：使用 `env!("CARGO_MANIFEST_DIR")` 确保路径正确

---

## 🔄 调用流程与数据流

### 整体架构流程

```
调用 PromptManager::load()
  ↓
检查缓存（get_from_cache）
  ↓
缓存命中？ → 是 → 返回缓存内容
  ↓ 否
从文件加载（load_from_file）
  ↓
路径转换（name_to_path）
  ↓
读取文件（fs::read_to_string）
  ↓
存入缓存（put_to_cache）
  ↓
返回 Prompt 内容
```

### 典型调用示例

#### 1. 从文件加载 Prompt

```rust
use workflow::base::prompt::PromptManager;

// 加载 Prompt（文件不存在会返回错误）
let prompt = PromptManager::load("generate_branch.system.md")?;
```

**流程**：
1. 检查缓存中是否有 `"generate_branch.system.md"`
2. 如果缓存未命中，调用 `name_to_path()` 构建文件路径
3. 使用 `fs::read_to_string()` 读取文件
4. 将内容存入缓存
5. 返回 Prompt 内容

#### 2. 从文件加载 Prompt（带默认值回退）

```rust
// 加载 Prompt，如果文件不存在则使用默认值
let prompt = PromptManager::load_or_default("generate_branch.system.md", || {
    "Default system prompt".to_string()
})?;
```

**流程**：
1. 检查缓存
2. 如果缓存未命中，尝试从文件加载
3. 如果文件不存在，调用 `default_fn()` 生成默认值
4. 将默认值存入缓存
5. 返回 Prompt 内容

### 数据流

```
Prompt 文件 (prompts/generate_branch.system.md)
  ↓
PromptManager::load()
  ↓
缓存检查 (HashMap<String, String>)
  ↓
文件读取 (fs::read_to_string)
  ↓
缓存存储
  ↓
返回 Prompt 字符串
```

---

## 📝 文件结构

### Prompt 文件组织

Prompt 文件应放在以下位置，使用扁平化结构（无子文件夹）：

```
src/lib/base/prompt/prompts/
├── generate_branch.system.md    # 生成分支名的 system prompt
├── generate_branch.user.md       # 生成分支名的 user prompt（可选）
└── ...
```

### 文件命名规则

- **扁平化结构**：所有 Prompt 文件直接放在 `prompts/` 目录下，不使用子文件夹
- **命名格式**：使用点号分隔，格式为 `{功能}.{类型}.{扩展名}`（如 `generate_branch.system.md`）
- 支持两种文件格式：
  - **`.md`** (推荐)：Markdown 格式，更易读和维护
  - **`.txt`** (向后兼容)：纯文本格式
- **重要**：调用时必须使用完整文件名（包含扩展名），如 `"generate_branch.system.md"`
- 文件路径：`prompts/generate_branch.system.md` 或 `prompts/generate_branch.system.txt`

### 路径解析

Prompt 文件路径在编译时确定，使用 `env!("CARGO_MANIFEST_DIR")` 获取项目根目录：

```rust
let manifest_dir = env!("CARGO_MANIFEST_DIR");
let base_path = Path::new(manifest_dir);
let file_path = base_path.join(format!("src/lib/base/prompt/prompts/{}", name));
```

---

## 📝 扩展性

### 添加新的 Prompt 文件

1. 在 `src/lib/base/prompt/prompts/` 目录下创建新的 Prompt 文件
2. 使用命名格式：`{功能}.{类型}.md`（如 `generate_pr_title.system.md`）
3. 在代码中使用 `PromptManager::load()` 加载：

```rust
let prompt = PromptManager::load("generate_pr_title.system.md")?;
```

### 修改现有 Prompt

1. 直接编辑 `prompts/` 目录下的 Markdown 文件
2. 重新编译项目（缓存会在编译时更新）

### 添加新的加载策略

如果需要添加新的加载策略，可以在 `PromptManager` 中添加新的方法：

```rust
impl PromptManager {
    pub fn load_with_fallback<F, G>(name: &str, fallback_fn: F, default_fn: G) -> Result<String>
    where
        F: FnOnce() -> Result<String>,
        G: FnOnce() -> String,
    {
        // 实现新的加载策略
    }
}
```

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [LLM 模块架构文档](./LLM_ARCHITECTURE.md) - LLM 客户端使用 Prompt
- [PR 模块架构文档](./PR_ARCHITECTURE.md) - PR 模块使用 Prompt 生成分支名和 PR 标题

---

## 📋 使用示例

### 基本使用

```rust
use workflow::base::prompt::PromptManager;

// 加载 Prompt（文件不存在会返回错误）
let prompt = PromptManager::load("generate_branch.system.md")?;
```

### 带默认值回退

```rust
// 加载 Prompt，如果文件不存在则使用默认值
let prompt = PromptManager::load_or_default("generate_branch.system.md", || {
    "Default system prompt".to_string()
})?;
```

### 清除缓存（主要用于测试）

```rust
// 清除所有缓存的 Prompt
PromptManager::clear_cache();
```

### 在 PR 模块中使用

```rust
// src/lib/pr/llm.rs
use workflow::base::prompt::PromptManager;

let system_prompt = PromptManager::load("generate_branch.system.md")
    .with_context(|| "Failed to load system prompt from file: generate_branch.system.md")?;
```

---

## ⚠️ 注意事项

1. **编译时路径**：Prompt 文件路径在编译时确定，使用 `env!("CARGO_MANIFEST_DIR")`
2. **文件不存在**：使用 `load()` 时，如果文件不存在会返回错误；使用 `load_or_default()` 时，如果文件不存在会使用默认值
3. **缓存机制**：Prompt 会被缓存，修改文件后需要重新编译才能生效
4. **线程安全**：所有操作都是线程安全的，可以在多线程环境中使用
5. **完整文件名**：调用时必须使用完整文件名（包含扩展名），如 `"generate_branch.system.md"`

