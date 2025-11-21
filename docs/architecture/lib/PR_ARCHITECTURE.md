# PR 模块架构文档

## 📋 概述

PR 模块（`lib/pr/`）是 Workflow CLI 的核心库模块，提供 Pull Request 的平台抽象层。支持 GitHub 和 Codeup 两种代码托管平台，通过 `PlatformProvider` trait 实现统一的平台接口，使用工厂函数实现多态分发。该模块专注于平台 API 的抽象和调用，不涉及命令层的业务逻辑。

**注意**：本文档仅描述 `lib/pr/` 模块的架构。关于 PR 命令层的详细内容，请参考 [PR 命令模块架构文档](../commands/PR_COMMAND_ARCHITECTURE.md)。

**模块统计：**
- 总代码行数：约 2000+ 行
- 文件数量：15+ 个
- 支持平台：GitHub、Codeup
- 主要结构体：`PlatformProvider` trait、`GitHub`、`Codeup`、`PullRequestLLM`

---

## 📁 模块结构

```
src/lib/pr/
├── mod.rs              # PR 模块声明和导出 (18行)
├── platform.rs         # PlatformProvider trait 和工厂函数 (150行)
├── helpers.rs          # PR 辅助函数 (282行)
├── llm.rs              # LLM 功能（PR 标题生成）(253行)
│
├── github/             # GitHub 平台实现
│   ├── mod.rs          # GitHub 模块导出
│   ├── platform.rs    # GitHub 平台实现
│   ├── requests.rs     # GitHub API 请求结构体
│   ├── responses.rs    # GitHub API 响应结构体
│   └── errors.rs       # GitHub 错误处理
│
└── codeup/             # Codeup 平台实现
    ├── mod.rs          # Codeup 模块导出
    ├── platform.rs    # Codeup 平台实现
    ├── requests.rs    # Codeup API 请求结构体
    ├── responses.rs   # Codeup API 响应结构体
    └── errors.rs      # Codeup 错误处理
```

### 依赖模块

- **`lib/git/`**：Git 操作（检测仓库类型，用于工厂函数自动选择平台）
- **`lib/base/llm/`**：AI 功能（PR 标题生成，通过 `llm.rs` 模块封装）
- **`lib/base/http/`**：HTTP 客户端（API 请求）
- **`lib/base/settings/`**：配置管理（环境变量读取，如 `GITHUB_TOKEN`、`CODEUP_PROJECT_ID` 等）

**注意**：PR 模块不直接依赖 Jira、Git 分支操作、工具函数等模块，这些集成由命令层（`commands/pr/`）负责协调。

---

## 🏗️ 架构设计

### 设计原则

1. **平台抽象**：通过 `PlatformProvider` trait 实现统一的平台接口
2. **多态分发**：使用工厂函数 `create_provider()` 实现动态分发
3. **模块化设计**：按平台拆分模块，职责清晰
4. **统一错误处理**：平台特定错误处理统一封装
5. **代码复用**：请求/响应结构体分离，便于维护

### 核心组件

#### 1. 平台抽象层 (`platform.rs`)

**职责**：定义统一的 PR 平台接口和工厂函数

- **`PlatformProvider` trait**：定义所有平台必须实现的 9 个方法
  - `create_pull_request()` - 创建 PR
  - `merge_pull_request()` - 合并 PR
  - `get_pull_request_info()` - 获取 PR 信息
  - `get_pull_request_url()` - 获取 PR URL
  - `get_pull_request_title()` - 获取 PR 标题
  - `get_current_branch_pull_request()` - 获取当前分支的 PR ID
  - `get_pull_requests()` - 列出 PR（可选）
  - `get_pull_request_status()` - 获取 PR 状态
  - `close_pull_request()` - 关闭 PR

- **`create_provider()` 工厂函数**：
  - 自动检测仓库类型（GitHub/Codeup）
  - 返回 `Box<dyn PlatformProvider>` trait 对象
  - 实现真正的多态分发

- **`PullRequestStatus` 结构体**：PR 状态信息（state, merged, merged_at）

- **`TYPES_OF_CHANGES` 常量**：PR 变更类型定义

#### 2. GitHub 平台实现 (`github/`)

**职责**：GitHub REST API v3 的完整实现

- **`platform.rs`**：实现 `PlatformProvider` trait
- **`requests.rs`**：GitHub API 请求结构体
- **`responses.rs`**：GitHub API 响应结构体
- **`errors.rs`**：GitHub 特定错误处理

**关键特性**：
- 使用 GitHub REST API v3
- 需要 `GITHUB_TOKEN` 环境变量
- 支持所有 trait 方法

#### 3. Codeup 平台实现 (`codeup/`)

**职责**：Codeup REST API 的完整实现

- **`platform.rs`**：实现 `PlatformProvider` trait
- **`requests.rs`**：Codeup API 请求结构体
- **`responses.rs`**：Codeup API 响应结构体
- **`errors.rs`**：Codeup 特定错误处理

**关键特性**：
- 使用 Codeup REST API
- 需要 `CODEUP_PROJECT_ID`、`CODEUP_CSRF_TOKEN`、`CODEUP_COOKIE` 环境变量
- 支持所有 trait 方法

#### 4. 辅助函数层 (`helpers.rs`)

**职责**：提供 PR 相关的通用辅助函数

**主要函数**：
- `extract_pull_request_id_from_url()` - 从 URL 提取 PR ID
- `extract_github_repo_from_url()` - 从 URL 提取 GitHub 仓库信息
- `generate_branch_name()` - 生成分支名
- `generate_commit_title()` - 生成 commit 标题
- `generate_pull_request_body()` - 生成 PR body
- `get_current_branch_pr_id()` - 获取当前分支的 PR ID
- `detect_repo_type()` - 检测仓库类型（向后兼容）

#### 5. LLM 功能层 (`llm.rs`)

**职责**：提供 PR 标题的 AI 生成功能

- **`PullRequestLLM`**：PR LLM 客户端包装器
- **`PullRequestContent`**：PR 内容结构体
- **主要方法**：`generate_title()` - 从 Jira ticket 描述生成 PR 标题

---

## 🔄 调用流程

### 整体架构流程

```
调用者（命令层或其他模块）
  ↓
lib/pr/platform.rs (工厂函数 create_provider())
  ↓
lib/pr/github/platform.rs 或 lib/pr/codeup/platform.rs (平台实现)
  ↓
lib/base/http/ (HTTP 客户端)
  ↓
GitHub API 或 Codeup API
```

#### 架构流程图

```mermaid
graph TB
    Caller[调用者<br/>命令层或其他模块] --> Factory[lib/pr/platform.rs<br/>create_provider<br/>工厂函数]

    Factory -->|GitHub| GitHub[lib/pr/github/platform.rs<br/>GitHub 实现]
    Factory -->|Codeup| Codeup[lib/pr/codeup/platform.rs<br/>Codeup 实现]

    GitHub --> Http[lib/base/http/<br/>HTTP 客户端]
    Codeup --> Http

    Http --> GitHubAPI[GitHub API]
    Http --> CodeupAPI[Codeup API]

    Factory --> Helpers[lib/pr/helpers.rs<br/>辅助函数]
    Factory --> LLM[lib/pr/llm.rs<br/>LLM 功能]

    style Caller fill:#e1f5ff
    style Factory fill:#e8f5e9
    style GitHub fill:#e3f2fd
    style Codeup fill:#fff3e0
    style Http fill:#f3e5f5
    style Helpers fill:#f3e5f5
    style LLM fill:#f3e5f5
```

### 典型调用示例

#### 1. 创建 PR

```rust
use workflow::pr::create_provider;

let provider = create_provider()?;

// 创建 PR
let pr_url = provider.create_pull_request(
    "Fix bug in login",
    "This PR fixes a bug in the login functionality",
    "feature/fix-login",
    None,
)?;
```

#### 2. 合并 PR

```rust
use workflow::pr::create_provider;

let provider = create_provider()?;

// 检查 PR 状态
let status = provider.get_pull_request_status("123")?;
if !status.merged {
    // 合并 PR
    provider.merge_pull_request("123", true)?;
}
```

#### 3. 获取 PR 信息

```rust
use workflow::pr::create_provider;

let provider = create_provider()?;

// 获取当前分支的 PR ID
if let Some(pr_id) = provider.get_current_branch_pull_request()? {
    // 获取 PR 详细信息
    let info = provider.get_pull_request_info(&pr_id)?;
    println!("PR URL: {}", info.url);
}
```

---

## 📦 模块职责

### PlatformProvider Trait

**职责**：定义统一的 PR 平台接口

**核心方法**：
- `create_pull_request()` - 创建 PR，返回 PR URL
- `merge_pull_request()` - 合并 PR
- `get_pull_request_info()` - 获取 PR 详细信息
- `get_pull_request_url()` - 获取 PR URL
- `get_pull_request_title()` - 获取 PR 标题
- `get_current_branch_pull_request()` - 获取当前分支的 PR ID
- `get_pull_requests()` - 列出 PR（可选方法）
- `get_pull_request_status()` - 获取 PR 状态
- `close_pull_request()` - 关闭 PR

**设计优势**：
- 使用实例方法（`&self`），支持 trait 对象
- 通过工厂函数实现多态分发
- 消除调用层的代码重复

### GitHub 平台实现

**职责**：GitHub REST API v3 的完整实现

**核心功能**：
- 实现所有 `PlatformProvider` trait 方法
- 统一的 HTTP 请求处理
- GitHub 特定的错误处理
- 请求/响应结构体分离

**使用场景**：
- 自动检测到 GitHub 仓库时使用
- 需要 `GITHUB_TOKEN` 环境变量

### Codeup 平台实现

**职责**：Codeup REST API 的完整实现

**核心功能**：
- 实现所有 `PlatformProvider` trait 方法
- 统一的 HTTP 请求处理
- Codeup 特定的错误处理
- 请求/响应结构体分离

**使用场景**：
- 自动检测到 Codeup 仓库时使用
- 需要 `CODEUP_PROJECT_ID`、`CODEUP_CSRF_TOKEN`、`CODEUP_COOKIE` 环境变量

### Helpers 模块

**职责**：提供 PR 相关的通用辅助函数

**核心功能**：
- URL 解析（提取 PR ID、仓库信息）
- 分支名和 commit 标题生成
- PR body 生成
- 仓库类型检测（向后兼容）

**使用场景**：
- 可以被任何调用者使用（命令层或其他模块）
- 平台无关的通用逻辑

### LLM 模块

**职责**：提供 PR 标题的 AI 生成功能

**核心功能**：
- 从 Jira ticket 描述生成简洁的英文 PR 标题
- 使用统一的 LLM 客户端
- 错误处理和回退机制

**使用场景**：
- PR 创建时自动生成标题
- 如果 AI 生成失败，回退到手动输入

---

## 🔗 与其他模块的集成

### Git 集成

PR 模块依赖 Git 模块进行仓库类型检测：

**关键方法**：
- `GitRepo::detect_repo_type()` - 检测仓库类型（GitHub/Codeup），用于工厂函数自动选择平台实现

### HTTP 集成

PR 模块依赖 HTTP 客户端进行 API 调用：

**关键方法**：
- `HttpClient` - 统一的 HTTP 客户端，用于发送 API 请求

### LLM 集成

PR 模块提供 LLM 功能用于生成 PR 标题：

**关键方法**：
- `PullRequestLLM::generate_title()` - 从 Jira ticket 描述生成简洁的英文 PR 标题
- 依赖 `lib/base/llm/` 模块的 LLM 客户端

**注意**：PR 模块本身不直接集成 Jira、Git 分支操作、工具函数等，这些集成由命令层（`commands/pr/`）负责协调。PR 模块专注于平台 API 的抽象和调用。

---

## 🎯 设计模式

### 1. 策略模式

通过 `PlatformProvider` trait 实现平台抽象，不同平台有不同的实现策略。

**优势**：
- 添加新平台只需实现 trait，无需修改命令层代码
- 命令层代码与具体平台解耦

### 2. 工厂模式

通过 `create_provider()` 工厂函数实现多态分发。

**优势**：
- 自动检测仓库类型
- 返回 trait 对象，实现真正的多态
- 消除命令层的重复代码

### 3. 依赖注入

通过 trait 和模块化设计，命令层依赖抽象的 `PlatformProvider`，而不是具体的平台实现。

**优势**：
- 符合依赖倒置原则
- 提高代码可测试性
- 降低耦合度

---

## 🔍 错误处理

### 分层错误处理

1. **平台层**：平台特定的错误处理（GitHub/Codeup）
2. **HTTP 层**：HTTP 请求错误、网络错误
3. **业务层**：API 响应错误、数据解析错误

### 容错机制

- **仓库类型未知**：工厂函数返回明确的错误提示
- **API 调用失败**：平台实现层提供详细的错误信息
- **数据解析失败**：返回结构化的错误信息

### 平台特定错误处理

- **GitHub**：解析 GitHub API 错误响应，提供详细的错误信息
- **Codeup**：解析 Codeup API 错误响应，提供详细的错误信息

每个平台实现都有自己的错误处理模块（`errors.rs`），统一封装平台特定的错误类型。

---

## 📊 数据流

### 创建 PR 数据流

```mermaid
flowchart LR
    Caller[调用者<br/>提供参数] --> Factory[工厂函数<br/>create_provider]
    Factory --> Platform{平台选择}
    Platform -->|GitHub| GitHub[GitHub 实现<br/>构建请求]
    Platform -->|Codeup| Codeup[Codeup 实现<br/>构建请求]
    GitHub --> Http[HTTP 客户端<br/>发送请求]
    Codeup --> Http
    Http --> GitHubAPI[GitHub API]
    Http --> CodeupAPI[Codeup API]
    GitHubAPI --> Response[返回 PR URL]
    CodeupAPI --> Response

    style Caller fill:#e1f5ff
    style Factory fill:#e8f5e9
    style GitHub fill:#e3f2fd
    style Codeup fill:#fff3e0
    style Http fill:#f3e5f5
    style Response fill:#c8e6c9
```

### 获取 PR 信息数据流

```mermaid
flowchart LR
    Caller[调用者<br/>提供 PR ID] --> Factory[工厂函数<br/>create_provider]
    Factory --> Platform{平台选择}
    Platform -->|GitHub| GitHub[GitHub 实现<br/>构建请求]
    Platform -->|Codeup| Codeup[Codeup 实现<br/>构建请求]
    GitHub --> Http[HTTP 客户端<br/>发送请求]
    Codeup --> Http
    Http --> GitHubAPI[GitHub API]
    Http --> CodeupAPI[Codeup API]
    GitHubAPI --> Parse[解析响应]
    CodeupAPI --> Parse
    Parse --> Response[返回 PR 信息]

    style Caller fill:#e1f5ff
    style Factory fill:#e8f5e9
    style GitHub fill:#e3f2fd
    style Codeup fill:#fff3e0
    style Http fill:#f3e5f5
    style Parse fill:#fff9c4
    style Response fill:#c8e6c9
```

---

## 📝 扩展性

### 添加新平台

1. 在 `lib/pr/` 下创建新的平台目录（如 `gitlab/`）
2. 创建以下文件：
   - `mod.rs` - 模块导出
   - `platform.rs` - 实现 `PlatformProvider` trait
   - `requests.rs` - API 请求结构体
   - `responses.rs` - API 响应结构体
   - `errors.rs` - 错误处理
3. 在 `lib/pr/platform.rs` 的 `create_provider()` 函数中添加新平台的分支
4. 在 `lib/git/repo.rs` 中添加仓库类型检测逻辑
5. 在 `lib/pr/mod.rs` 中导出新平台

**示例**：
```rust
// lib/pr/platform.rs
pub fn create_provider() -> Result<Box<dyn PlatformProvider>> {
    match GitRepo::detect_repo_type()? {
        RepoType::GitHub => Ok(Box::new(GitHub)),
        RepoType::Codeup => Ok(Box::new(Codeup)),
        RepoType::GitLab => Ok(Box::new(GitLab)),  // 新增
        RepoType::Unknown => anyhow::bail!("Unsupported repository type"),
    }
}
```

### 添加新的辅助函数

1. 在 `lib/pr/helpers.rs` 中添加新函数
2. 在 `lib/pr/mod.rs` 中导出（如需要）
3. 更新文档

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [PR 命令模块架构文档](../commands/PR_COMMAND_ARCHITECTURE.md) - PR 命令层详情
- [Jira 模块架构文档](./JIRA_ARCHITECTURE.md) - Jira 集成详情
- [Git 模块架构文档](./GIT_ARCHITECTURE.md) - Git 操作详情
- [LLM 模块架构文档](./LLM_ARCHITECTURE.md) - AI 功能详情

---

## 📋 使用示例

### 基本使用

```rust
use workflow::pr::create_provider;

// 创建平台提供者（自动检测仓库类型）
let provider = create_provider()?;

// 创建 PR
let pr_url = provider.create_pull_request(
    "Fix bug in login",
    "This PR fixes a bug in the login functionality",
    "feature/fix-login",
    None,
)?;

// 获取 PR 信息
let info = provider.get_pull_request_info("123")?;

// 合并 PR
provider.merge_pull_request("123", true)?;

// 关闭 PR
provider.close_pull_request("123")?;
```

### 获取当前分支的 PR

```rust
use workflow::pr::create_provider;

let provider = create_provider()?;

// 获取当前分支的 PR ID
if let Some(pr_id) = provider.get_current_branch_pull_request()? {
    println!("Current branch has PR: {}", pr_id);

    // 获取 PR 状态
    let status = provider.get_pull_request_status(&pr_id)?;
    println!("PR status: {}, merged: {}", status.state, status.merged);
}
```

### 列出 PR

```rust
use workflow::pr::create_provider;

let provider = create_provider()?;

// 列出所有打开的 PR
let prs = provider.get_pull_requests(Some("open"), Some(10))?;
println!("{}", prs);
```

### 使用辅助函数

```rust
use workflow::pr::helpers::{
    generate_branch_name,
    generate_commit_title,
    generate_pull_request_body,
};

// 生成分支名
let branch_name = generate_branch_name("PROJ-123", "Add new feature", None)?;

// 生成 commit 标题
let commit_title = generate_commit_title("PROJ-123", "Add new feature", false)?;

// 生成 PR body
let pr_body = generate_pull_request_body(
    "This is a new feature",
    &["New feature (non-breaking change which adds functionality)"],
    Some("PROJ-123"),
    None,
)?;
```

### 使用 LLM 生成标题

```rust
use workflow::pr::PullRequestLLM;

let llm = PullRequestLLM::new()?;
let title = llm.generate_title("PROJ-123", "This is a description of the feature")?;
println!("Generated title: {}", title);
```

---

## ✅ 总结

PR 模块采用清晰的分层架构设计：

1. **平台抽象层**：`PlatformProvider` trait 定义统一的平台接口
2. **工厂函数**：`create_provider()` 实现多态分发，自动检测仓库类型
3. **平台实现层**：GitHub 和 Codeup 分别实现 trait，模块化组织
4. **辅助函数层**：提供通用的 PR 相关辅助函数
5. **LLM 功能层**：提供 PR 标题的 AI 生成功能

**设计优势**：
- ✅ **多态支持**：通过 trait 对象实现真正的多态
- ✅ **代码复用**：消除调用层的重复代码
- ✅ **易于扩展**：添加新平台只需实现 trait
- ✅ **模块化**：按平台拆分，职责清晰
- ✅ **类型安全**：使用 trait 和类型系统保证类型安全
- ✅ **平台无关**：调用者无需关心具体平台实现

通过平台抽象和工厂模式，实现了代码复用、易于维护和扩展的目标。命令层（`commands/pr/`）使用本模块提供的接口，实现了完整的 PR 生命周期管理功能。
