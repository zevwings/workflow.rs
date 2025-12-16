# Jira 模块架构文档

## 📋 概述

Jira 模块是 Workflow CLI 的核心功能之一，提供与 Jira REST API 交互的完整功能，包括用户信息管理、Ticket/Issue 操作、项目状态管理、工作历史记录管理和日志处理等功能。该模块采用分层架构设计，通过统一的 HTTP 客户端和 API 子模块实现代码复用和统一管理。

**模块统计：**
- 总代码行数：约 2800+ 行
- 文件数量：20+ 个
- 主要组件：7 个（JiraHttpClient、API 子模块、业务逻辑层、ConfigManager）
- 支持功能：用户管理、Ticket 操作、状态管理、工作历史、日志处理

---

## 📁 模块结构

### 核心模块文件

```
src/lib/jira/
├── mod.rs              # 模块导出和向后兼容别名 (56行)
├── api/                # API 方法子模块
│   ├── mod.rs          # API 统一入口
│   ├── http_client.rs  # JiraHttpClient (HTTP 层，~155行)
│   ├── issue.rs        # Issue/Ticket 相关 API (~182行)
│   ├── user.rs         # 用户相关 API (~30行)
│   └── project.rs      # 项目相关 API (~60行)
├── config.rs           # ConfigManager (TOML 配置管理器，~148行)
├── client.rs           # JiraClient 包装器（向后兼容，~104行）
├── helpers.rs          # 辅助函数（认证、URL、字符串处理，~178行）
├── types.rs            # 数据模型定义 (~115行)
├── users.rs            # 用户信息管理 (~173行)
├── ticket.rs           # Ticket/Issue 操作 (~201行)
├── status.rs           # 状态管理 (~275行)
├── history.rs          # 工作历史记录管理 (~392行)
└── logs/               # 日志处理模块
    ├── mod.rs          # JiraLogs 结构体定义 (66行)
    ├── constants.rs    # 常量定义 (33行)
    ├── helpers.rs      # 日志处理辅助函数 (178行)
    ├── path.rs         # 路径管理功能 (135行)
    ├── download.rs     # 下载功能 (450行)
    ├── search.rs       # 搜索和查找功能 (187行)
    ├── zip.rs          # ZIP 处理功能 (131行)
    └── clean.rs        # 清理功能 (103行)
```

**总计：约 2800+ 行代码**

### 依赖模块

- **`lib/base/http/`**：HTTP 客户端（统一的 HTTP 请求接口）
- **`lib/base/settings/`**：配置管理（Jira 配置：base_url, email, api_token）
- **`lib/base/util/`**：工具函数（日志输出、字符串处理等）

### 模块集成

- **PR 模块集成** (`lib/pr/`)：
  - 创建 PR 时：`JiraStatus::configure_interactive()` 配置状态，`JiraTicket::transition()` 更新状态，`JiraWorkHistory::write_work_history()` 保存历史
  - 合并 PR 时：`JiraWorkHistory::read_work_history()` 查找 ticket，`JiraTicket::transition()` 更新状态

- **Log 和 Jira 命令集成** (`commands/log/` 和 `commands/jira/`)：
  - 下载日志：`JiraTicket::get_attachments()` 获取附件，`JiraLogs::download_from_jira()` 下载
  - 显示信息：`JiraTicket::get_info()` 获取 ticket 信息

- **HTTP 模块集成** (`lib/base/http/`)：
  - `HttpClient::global()` - 获取全局 HTTP 客户端
  - `HttpResponse::ensure_success()` - 统一错误处理

---

## 🏗️ 架构设计

### 设计原则

1. **分层架构**：HTTP 客户端层 → API 方法层 → 业务逻辑层
2. **统一接口**：所有 HTTP 请求通过 `JiraHttpClient` 统一处理
3. **模块化设计**：按功能域拆分为独立模块，职责清晰
4. **向后兼容**：保持现有公共 API 不变，内部实现可优化
5. **配置驱动**：使用 `ConfigManager` 统一管理 TOML 配置文件

### 核心组件

#### 1. HTTP 客户端层 (`api/http_client.rs`)

**职责**：提供统一的 Jira REST API 请求接口

- **`JiraHttpClient`**：Jira HTTP 客户端
  - 单例模式（`OnceLock`），线程安全
  - 缓存认证信息和客户端实例
  - 统一错误处理（`HttpResponse::ensure_success()`）
  - 支持 GET、POST、PUT 请求

**主要方法**：
- `global()` - 获取全局单例
- `get<T>(path)` - 执行 GET 请求
- `post<Req, Resp>(path, body)` - 执行 POST 请求
- `put<Req, Resp>(path, body)` - 执行 PUT 请求

**关键特性**：
- 自动添加 Basic Authentication
- 自动构建完整的 API URL
- 统一的错误处理和上下文信息

**使用场景**：
- 所有 Jira API 调用都通过此客户端
- API 层的基础设施

#### 2. API 方法层 (`api/`)

**职责**：提供所有 Jira REST API 方法的统一接口

##### `issue.rs` - JiraIssueApi

**主要方法**：
- `get_issue()` - 获取 issue 信息
- `get_issue_attachments()` - 获取附件列表
- `get_issue_transitions()` - 获取可用 transitions
- `transition_issue()` - 更新 issue 状态
- `assign_issue()` - 分配 issue 给用户
- `add_issue_comment()` - 添加评论

##### `user.rs` - JiraUserApi

**主要方法**：
- `get_current_user()` - 获取当前用户信息

##### `project.rs` - JiraProjectApi

**主要方法**：
- `get_project_statuses()` - 获取项目状态列表

#### 3. 配置管理层 (`config.rs`)

**职责**：提供统一的 TOML 配置文件读写功能

- **`ConfigManager<T>`**：泛型配置管理器
  - `read()` - 读取配置（文件不存在时返回默认值）
  - `write()` - 写入配置
  - `update()` - 更新配置（读取→修改→写入）
  - 自动设置文件权限（Unix 系统 600 权限）

**关键特性**：
- 泛型设计，支持任意配置类型
- 自动创建目录
- 原子性更新（读取→修改→写入）
- 类型安全

**使用场景**：
- 管理 `jira-users.toml`
- 管理 `jira-status.toml`
- 可扩展到其他配置文件

#### 4. 业务逻辑层

**职责**：提供高级业务功能，封装 API 调用

##### `users.rs` - JiraUsers

**职责**：用户信息获取和本地缓存

**主要方法**：
- `get()` - 获取当前用户信息（优先从本地缓存读取）

**关键特性**：
- 使用 `ConfigManager<JiraUsersConfig>` 管理用户配置
- 配置文件：`~/.workflow/config/jira-users.toml`

##### `ticket.rs` - JiraTicket

**职责**：Ticket/Issue 操作

**主要方法**：
- `get_info()` - 获取 ticket 信息
- `get_attachments()` - 获取附件列表
- `transition()` - 更新 ticket 状态
- `assign()` - 分配 ticket 给用户
- `add_comment()` - 添加评论

##### `status.rs` - JiraStatus

**职责**：状态管理（项目状态获取、状态配置）

**主要方法**：
- `configure_interactive()` - 交互式配置状态映射
- `read_pull_request_created_status()` - 读取 PR 创建时的状态
- `read_pull_request_merged_status()` - 读取 PR 合并时的状态

**关键特性**：
- 使用 `ConfigManager<JiraStatusMap>` 管理状态配置
- 配置文件：`~/.workflow/config/jira-status.toml`

##### `history.rs` - JiraWorkHistory

**职责**：工作历史记录管理（PR 创建/合并记录）

**主要方法**：
- `read_work_history()` - 读取工作历史记录（通过 PR ID 查找 Jira ticket）
- `find_pr_id_by_branch()` - 根据分支名查找 PR ID
- `write_work_history()` - 写入工作历史记录
- `update_work_history_merged()` - 更新工作历史记录的合并时间
- `delete_work_history_entry()` - 删除工作历史记录条目

**关键特性**：
- JSON 格式，按仓库分别存储
- 文件位置：`~/.workflow/work-history/{repo_id}.json`

##### `logs/` - JiraLogs

**职责**：日志处理（下载、搜索、查找、清理）

**主要方法**：
- `download_from_jira()` - 从 Jira 下载日志附件
- `find_request_id()` - 查找请求 ID
- `extract_response_content()` - 提取响应内容
- `search_keyword()` - 搜索关键词
- `clean_dir()` - 清理日志目录

**关键特性**：
- 统一接口，状态缓存
- 详细架构参见 [日志命令模块架构文档](../commands/LOG_COMMAND_ARCHITECTURE.md) 和 [Jira 命令模块架构文档](../commands/JIRA_COMMAND_ARCHITECTURE.md)

#### 5. 数据模型层 (`types.rs`)

**职责**：定义所有 Jira API 相关的数据结构

- `JiraIssue` - Issue 信息
- `JiraUser` - 用户信息
- `JiraAttachment` - 附件信息
- `JiraComment` - 评论信息
- `JiraTransition` - 状态转换信息

#### 6. 工具层

##### `helpers.rs` - 辅助函数

**主要函数**：
- `get_auth()` - 获取认证信息
- `get_base_url()` - 获取基础 URL
- `extract_jira_project()` - 提取项目名
- `extract_jira_ticket_id()` - 提取 ticket ID
- `validate_jira_ticket_format()` - 验证 ticket 格式
- `sanitize_email_for_filename()` - 清理邮箱用于文件名

##### `client.rs` - JiraClient

**职责**：向后兼容包装器

**关键特性**：
- 所有方法委托到对应的功能模块
- 保持现有公共 API 不变

### 设计模式

#### 1. 分层架构模式

**HTTP 客户端层** → **API 方法层** → **业务逻辑层**

- **HTTP 客户端层**：`JiraHttpClient` 负责 HTTP 请求封装
- **API 方法层**：`JiraIssueApi`、`JiraUserApi`、`JiraProjectApi` 负责具体 API 调用
- **业务逻辑层**：`JiraUsers`、`JiraTicket`、`JiraStatus` 负责高级业务功能

**优势**：
- 职责清晰，易于维护
- 可以独立测试每一层
- 易于扩展新功能

#### 2. 单例模式

**`JiraHttpClient`** 使用 `OnceLock` 实现线程安全的单例。

**优势**：
- 减少重复的认证信息获取
- 复用 HTTP 客户端实例
- 提升性能

#### 3. 配置管理模式

**`ConfigManager<T>`** 泛型配置管理器，统一管理 TOML 配置文件。

**优势**：
- 代码复用
- 类型安全
- 统一行为（文件权限、默认值处理等）

#### 4. 适配器模式

**`JiraClient`** 向后兼容包装器，所有方法委托到对应的功能模块。

**优势**：
- 保持现有 API 不变
- 内部实现可优化
- 平滑迁移

### 错误处理

#### 分层错误处理

1. **HTTP 层**：`HttpResponse::ensure_success()` 统一检查 HTTP 状态码
2. **API 层**：使用 `anyhow::Context` 添加上下文信息
3. **业务层**：提供清晰的错误消息和恢复建议

#### 容错机制

- **配置不存在**：使用默认值，不报错
- **API 调用失败**：返回详细的错误信息，包含状态码和响应体
- **文件操作失败**：提供清晰的错误提示和手动操作建议
- **网络错误**：可选的重试机制（通过 `HttpRetry`）

---

## 🔄 调用流程与数据流

### 整体架构流程

```
调用者（命令层或其他模块）
  ↓
lib/jira/*.rs (业务逻辑层)
  ├── users.rs → JiraUsers::get()
  ├── ticket.rs → JiraTicket::get_info()
  ├── status.rs → JiraStatus::configure_interactive()
  └── history.rs → JiraWorkHistory::read_work_history()
  ↓
lib/jira/api/*.rs (API 方法层)
  ├── issue.rs → JiraIssueApi::get_issue()
  ├── user.rs → JiraUserApi::get_current_user()
  └── project.rs → JiraProjectApi::get_project_statuses()
  ↓
lib/jira/api/http_client.rs (HTTP 客户端层)
  └── JiraHttpClient::global() → get()/post()/put()
  ↓
lib/base/http/ (基础 HTTP 层)
  └── HttpClient::global()
```

### 典型调用示例

#### 1. 获取 Ticket 信息

```
JiraTicket::get_info("PROJ-123")
  ↓
JiraIssueApi::get_issue("PROJ-123")
  ↓
JiraHttpClient::global()?.get("issue/PROJ-123?fields=*all&expand=renderedFields")
  ↓
HttpClient::global()?.get(url, config)
```

#### 2. 更新 Ticket 状态

```
JiraTicket::transition("PROJ-123", "Done")
  ↓
JiraIssueApi::transition_issue("PROJ-123", transition_id)
  ↓
JiraHttpClient::global()?.post("issue/PROJ-123/transitions", body)
```

#### 3. 读取工作历史记录

```
JiraWorkHistory::read_work_history(pr_id, repository)
  ↓
读取 ~/.workflow/work-history/{repo}.json
```

### 数据流

#### 配置文件位置

- **用户配置**：`~/.workflow/config/jira-users.toml`
  - 格式：TOML 数组表（`[[users]]`）
  - 内容：用户邮箱、account_id、display_name

- **状态配置**：`~/.workflow/config/jira-status.toml`
  - 格式：TOML 表（`[PROJ]`）
  - 内容：PR 创建/合并时的目标状态

- **工作历史**：`~/.workflow/work-history/{repo_id}.json`
  - 格式：JSON 对象（PR ID → Entry）
  - 内容：Jira ticket、PR URL、时间戳、分支名等

#### 配置文件示例

**jira-users.toml**：
```toml
[[users]]
email = "user@example.com"
account_id = "628d9616269a9a0068f27e0c"
display_name = "User Name"
```

**jira-status.toml**：
```toml
[PROJ]
created-pr = "In Progress"
merged-pr = "Done"
```

**work-history/{repo_id}.json**：
```json
{
  "456": {
    "jira_ticket": "PROJ-123",
    "pull_request_url": "https://github.com/xxx/pull/456",
    "created_at": "2024-01-15T10:30:00Z",
    "merged_at": null,
    "repository": "github.com/xxx/yyy",
    "branch": "feature/PROJ-123-add-feature"
  }
}
```

#### 模块依赖关系

```
mod.rs
  ├── api/ (HTTP 客户端层和 API 方法层)
  │   ├── http_client.rs → helpers.rs, lib/base/http/
  │   ├── issue.rs → http_client.rs, types.rs
  │   ├── user.rs → http_client.rs, types.rs
  │   └── project.rs → http_client.rs
  ├── config.rs (配置管理层，无依赖)
  ├── helpers.rs (工具层，无依赖)
  ├── types.rs (数据模型层，无依赖)
  ├── users.rs → api/user.rs, config.rs, helpers.rs
  ├── ticket.rs → api/issue.rs, types.rs
  ├── status.rs → api/project.rs, config.rs, helpers.rs
  ├── history.rs → lib/base/settings/paths.rs
  ├── logs/ (日志处理模块)
  │   ├── mod.rs → lib/base/settings/
  │   ├── download.rs → constants.rs, helpers.rs, zip.rs
  │   ├── search.rs → constants.rs, helpers.rs
  │   └── ...
  └── client.rs → users.rs, ticket.rs, types.rs
```

**依赖关系清晰，无循环依赖**

---

## 📋 使用示例

### 基本使用

```rust
use workflow::jira::{JiraTicket, JiraStatus, JiraWorkHistory};

// 获取 ticket 信息
let issue = JiraTicket::get_info("PROJ-123")?;

// 更新 ticket 状态
JiraTicket::transition("PROJ-123", "Done")?;

// 配置状态映射
JiraStatus::configure_interactive("PROJ")?;

// 读取工作历史记录
let ticket = JiraWorkHistory::read_work_history("456", Some("github.com/xxx/yyy"))?;
```

### 使用 JiraClient（向后兼容）

```rust
use workflow::jira::Jira;

// 获取 ticket 信息
let issue = Jira::get_ticket_info("PROJ-123")?;

// 更新 ticket 状态
Jira::move_ticket("PROJ-123", "Done")?;
```

### 使用 ConfigManager

```rust
use workflow::jira::ConfigManager;
use crate::base::settings::paths::Paths;

let config_path = Paths::jira_users_config()?;
let manager = ConfigManager::<JiraUsersConfig>::new(config_path);

// 读取配置
let config = manager.read()?;

// 更新配置
manager.update(|config| {
    config.users.push(new_user);
})?;
```

---

## 📝 扩展性

### 添加新的 API 方法

1. 在对应的 API 子模块中添加方法（`api/issue.rs`、`api/user.rs`、`api/project.rs`）
2. 在业务逻辑层添加封装方法（如需要）
3. 更新文档

**示例**：
```rust
// api/issue.rs
impl JiraIssueApi {
    pub fn delete_issue(issue_key: &str) -> Result<()> {
        let client = JiraHttpClient::global()?;
        client.delete(&format!("issue/{}", issue_key))?;
        Ok(())
    }
}
```

### 添加新的配置类型

1. 定义配置结构体（实现 `Serialize`、`Deserialize`、`Default`）
2. 使用 `ConfigManager<T>` 管理配置
3. 在 `mod.rs` 中导出（如需要）

**示例**：
```rust
#[derive(Serialize, Deserialize, Default)]
pub struct MyConfig {
    pub field: String,
}

let manager = ConfigManager::<MyConfig>::new(config_path);
let config = manager.read()?;
```

### 添加新的业务功能

1. 创建新的模块文件（如 `new_feature.rs`）
2. 实现业务逻辑，使用 API 层的方法
3. 在 `mod.rs` 中导出

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [PR 模块架构文档](./PR_ARCHITECTURE.md) - PR 模块如何使用 Jira 集成
- [日志命令模块架构文档](../commands/LOG_COMMAND_ARCHITECTURE.md) - 日志命令层如何使用 Jira 日志处理模块
- [Jira 命令模块架构文档](../commands/JIRA_COMMAND_ARCHITECTURE.md) - Jira 命令层如何使用 Jira 集成
- [HTTP 模块架构文档](./HTTP_ARCHITECTURE.md) - HTTP 客户端详情

---

## ✅ 总结

Jira 模块采用清晰的分层架构设计：

1. **HTTP 客户端层**：`JiraHttpClient` 提供统一的 HTTP 请求接口
2. **API 方法层**：按功能模块化组织所有 Jira REST API 方法
3. **业务逻辑层**：提供高级业务功能，封装 API 调用
4. **配置管理层**：`ConfigManager` 统一管理 TOML 配置文件
5. **工具层**：辅助函数和向后兼容包装器

**设计优势**：
- ✅ **职责清晰**：分层架构，每层职责明确
- ✅ **代码复用**：统一的 HTTP 客户端和配置管理器
- ✅ **易于维护**：模块化设计，低耦合
- ✅ **易于扩展**：添加新功能只需在对应层添加代码
- ✅ **向后兼容**：保持现有 API 不变，内部实现可优化

通过模块化设计和统一接口，实现了代码复用、易于维护和扩展的目标。

---

**最后更新**: 2025-12-16
