# PR 平台新增指南

> 本文档描述如何为 Workflow CLI 添加新的 PR 平台支持（如 GitLab、Bitbucket 等）。

---

## 📋 目录

- [概述](#-概述)
- [架构设计](#-架构设计)
- [实现步骤](#-实现步骤)
- [需要修改的文件](#-需要修改的文件)
- [实现细节](#-实现细节)
- [测试](#-测试)
- [示例：添加 GitLab 支持](#-示例添加-gitlab-支持)
- [检查清单](#-检查清单)

---

## 📋 概述

### 设计原则

PR 模块采用 **策略模式（Strategy Pattern）** 设计，通过 `PlatformProvider` trait 定义统一的接口，不同平台（GitHub、GitLab 等）实现各自的逻辑。

### 核心组件

1. **`PlatformProvider` trait** (`src/lib/pr/platform.rs`)
   - 定义所有 PR 平台必须实现的共同方法
   - 提供平台无关的 PR 操作接口

2. **平台实现** (`src/lib/pr/{platform}/`)
   - 每个平台有独立的目录和实现
   - 实现 `PlatformProvider` trait 的所有方法

3. **平台工厂** (`src/lib/pr/platform.rs::create_provider()`)
   - 根据仓库类型自动创建对应的平台提供者

### 当前支持

- ✅ **GitHub** - 完全支持
- ❌ **Codeup** - 已移除支持（保留枚举值用于检测，但不支持 PR 功能）

---

## 🏗️ 架构设计

### 模块结构

```
src/lib/pr/
├── mod.rs                    # 模块导出
├── platform.rs              # PlatformProvider trait 和工厂函数
├── helpers.rs                # 通用辅助函数
├── body_parser.rs            # PR body 解析
├── github/                   # GitHub 平台实现
│   ├── mod.rs
│   ├── platform.rs          # GitHub 实现 PlatformProvider
│   ├── requests.rs          # API 请求结构
│   ├── responses.rs         # API 响应结构
│   └── errors.rs            # 错误处理
└── {new_platform}/          # 新平台实现（需要创建）
    ├── mod.rs
    ├── platform.rs
    ├── requests.rs
    ├── responses.rs
    └── errors.rs
```

### 数据流

```
用户命令
  ↓
create_provider()  # 根据 RepoType 创建平台实例
  ↓
PlatformProvider trait 方法
  ↓
具体平台实现（GitHub/GitLab/etc.）
  ↓
HTTP API 调用
  ↓
返回结果
```

---

## 🔧 实现步骤

### 步骤 1：添加仓库类型检测

#### 1.1 在 `src/lib/git/types.rs` 中添加新的 `RepoType` 变体

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoType {
    /// GitHub 仓库
    GitHub,
    /// 阿里云 Codeup 仓库
    Codeup,
    /// GitLab 仓库  // 新增
    GitLab,
    /// 未知类型的仓库
    Unknown,
}
```

#### 1.2 在 `src/lib/git/repo.rs` 中添加 URL 检测逻辑

在 `parse_repo_type_from_url()` 函数中添加新平台的 URL 匹配规则：

```rust
fn parse_repo_type_from_url(url: &str) -> RepoType {
    // 检查 GitHub
    if url.contains("github.com")
        || url.starts_with("git@github")
        || url.starts_with("ssh://git@github")
    {
        RepoType::GitHub
    }
    // 检查 GitLab  // 新增
    else if url.contains("gitlab.com")
        || url.starts_with("git@gitlab")
        || url.starts_with("ssh://git@gitlab")
    {
        RepoType::GitLab
    }
    else if url.contains("codeup.aliyun.com") {
        RepoType::Codeup
    } else {
        RepoType::Unknown
    }
}
```

**注意**：如果新平台支持多种 URL 格式（SSH、HTTPS、自定义域名），需要添加相应的检测逻辑。

---

### 步骤 2：创建新平台模块目录

在 `src/lib/pr/` 下创建新平台的目录结构：

```bash
mkdir -p src/lib/pr/gitlab
```

创建以下文件：
- `src/lib/pr/gitlab/mod.rs` - 模块声明和导出
- `src/lib/pr/gitlab/platform.rs` - 平台实现（实现 `PlatformProvider` trait）
- `src/lib/pr/gitlab/requests.rs` - API 请求结构体
- `src/lib/pr/gitlab/responses.rs` - API 响应结构体
- `src/lib/pr/gitlab/errors.rs` - 错误处理

---

### 步骤 3：实现请求和响应结构

#### 3.1 实现 `requests.rs`

定义 API 请求所需的结构体，使用 `serde::Serialize`：

```rust
use serde::Serialize;

/// 创建 Merge Request 请求
#[derive(Debug, Serialize)]
pub struct CreateMergeRequestRequest {
    pub title: String,
    pub body: String,
    pub source_branch: String,
    pub target_branch: String,
    // 根据平台 API 添加其他字段
}

/// 合并 Merge Request 请求
#[derive(Debug, Serialize)]
pub struct MergeMergeRequestRequest {
    pub merge_commit_message: Option<String>,
    pub should_remove_source_branch: Option<bool>,
    // 根据平台 API 添加其他字段
}
```

#### 3.2 实现 `responses.rs`

定义 API 响应结构体，使用 `serde::Deserialize`：

```rust
use serde::Deserialize;

/// 创建 Merge Request 响应
#[derive(Debug, Deserialize)]
pub struct CreateMergeRequestResponse {
    pub web_url: String,  // GitLab 使用 web_url，GitHub 使用 html_url
    pub iid: u64,         // GitLab 使用 iid，GitHub 使用 number
}

/// Merge Request 信息
#[derive(Debug, Deserialize)]
pub struct MergeRequestInfo {
    pub iid: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub merged: bool,
    pub merged_at: Option<String>,
    pub web_url: String,
    pub source_branch: String,
    pub target_branch: String,
    pub author: Option<GitLabUser>,
}
```

**注意**：不同平台的字段名可能不同，需要根据实际 API 文档调整。

#### 3.3 实现 `errors.rs`

实现平台特定的错误处理：

```rust
use crate::base::http::HttpResponse;
use anyhow::Error;
use serde::Deserialize;

/// GitLab 错误响应结构
#[derive(Debug, Deserialize)]
pub struct GitLabErrorResponse {
    pub message: String,
    pub error: Option<String>,
}

/// 格式化 GitLab 错误信息
pub fn format_error(error: &GitLabErrorResponse, response: &HttpResponse) -> Error {
    let msg = format!(
        "GitLab API error: {} (Status: {})",
        error.message, response.status
    );
    anyhow::anyhow!(msg)
}

/// 处理 GitLab API 错误
pub fn handle_gitlab_error(response: &HttpResponse) -> Result<(), Error> {
    if response.is_success() {
        return Ok(());
    }

    let error: GitLabErrorResponse = response.as_json()?;
    Err(format_error(&error, response))
}
```

---

### 步骤 4：实现 `PlatformProvider` trait

在 `src/lib/pr/gitlab/platform.rs` 中实现所有必需的方法：

```rust
use crate::pr::platform::{PlatformProvider, PullRequestStatus};
use crate::pr::PullRequestRow;
use anyhow::{Context, Result};

/// GitLab 平台实现
pub struct GitLab;

impl PlatformProvider for GitLab {
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String> {
        // 1. 获取项目信息（owner/repo 或 project_id）
        let (project_id, _) = Self::get_project_info()?;

        // 2. 确定目标分支
        let base_branch = target_branch
            .map(|s| s.to_string())
            .unwrap_or_else(|| GitBranch::get_default_branch()?);

        // 3. 构建 API URL
        let url = format!("{}/projects/{}/merge_requests", Self::base_url(), project_id);

        // 4. 构建请求体
        let request = CreateMergeRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            source_branch: source_branch.to_string(),
            target_branch: base_branch,
        };

        // 5. 发送 HTTP 请求
        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<_, Value>::new()
            .body(&request)
            .headers(&headers);

        let response = client.post(&url, config)?;
        let response_data: CreateMergeRequestResponse = response
            .ensure_success_with(handle_gitlab_error)?
            .as_json()?;

        // 6. 返回 PR URL
        Ok(response_data.web_url)
    }

    // 实现其他必需的方法...
    fn merge_pull_request(&self, pull_request_id: &str, delete_branch: bool) -> Result<()> {
        // 实现合并逻辑
    }

    fn get_pull_request_info(&self, pull_request_id: &str) -> Result<String> {
        // 实现获取 PR 信息逻辑
    }

    // ... 其他方法
}
```

**关键点**：

1. **必需方法**：必须实现 `PlatformProvider` trait 中的所有方法
2. **可选方法**：`get_pull_requests()` 和 `get_pull_request_diff()` 有默认实现，如果平台不支持可以保持默认
3. **错误处理**：使用 `anyhow::Context` 提供清晰的错误信息
4. **认证**：从 `Settings` 获取 API token，参考 GitHub 实现的 `get_headers()` 方法

---

### 步骤 5：添加辅助方法

在平台实现中添加内部辅助方法：

```rust
impl GitLab {
    /// 获取 GitLab API 基础 URL
    fn base_url() -> &'static str {
        "https://gitlab.com/api/v4"  // 或从配置读取
    }

    /// 创建 API 请求的 headers
    fn get_headers(token: Option<&str>) -> Result<HeaderMap> {
        let token = token
            .or_else(|| {
                let settings = Settings::get();
                settings.gitlab.get_current_token()  // 需要在 Settings 中添加
            })
            .context("GitLab API token is not configured")?;

        // 构建 headers...
    }

    /// 获取项目信息（project_id 和 owner/repo）
    fn get_project_info() -> Result<(String, String)> {
        // 从 Git remote URL 解析项目信息
    }

    /// 获取 Merge Request 信息（内部方法）
    fn fetch_mr_info_internal(mr_iid: u64) -> Result<MergeRequestInfo> {
        // 实现获取 MR 信息的逻辑
    }
}
```

---

### 步骤 6：更新模块导出

#### 6.1 更新 `src/lib/pr/gitlab/mod.rs`

```rust
pub mod errors;
pub mod platform;
pub mod requests;
pub mod responses;

pub use errors::{format_error, GitLabError, GitLabErrorResponse};
pub use platform::GitLab;
pub use responses::GitLabUser;
```

#### 6.2 更新 `src/lib/pr/mod.rs`

```rust
pub mod body_parser;
pub mod github;
pub mod gitlab;  // 新增
pub mod helpers;
pub mod llm;
pub mod platform;
pub mod table;

// 导出
pub use github::errors::{GitHubError, GitHubErrorResponse};
pub use github::{GitHub, GitHubUser};
pub use gitlab::errors::{GitLabError, GitLabErrorResponse};  // 新增
pub use gitlab::{GitLab, GitLabUser};  // 新增
pub use platform::{create_provider, PlatformProvider, PullRequestStatus, TYPES_OF_CHANGES};
// ... 其他导出
```

#### 6.3 更新 `src/lib/pr/platform.rs`

在 `create_provider()` 函数中添加新平台的分支：

```rust
pub fn create_provider() -> Result<Box<dyn PlatformProvider>> {
    match GitRepo::detect_repo_type()? {
        RepoType::GitHub => Ok(Box::new(GitHub)),
        RepoType::GitLab => Ok(Box::new(GitLab)),  // 新增
        RepoType::Codeup => {
            anyhow::bail!("Codeup support has been removed. Only GitHub and GitLab are currently supported.")
        }
        RepoType::Unknown => {
            anyhow::bail!("Unsupported repository type. Only GitHub and GitLab are currently supported.")
        }
    }
}
```

---

### 步骤 7：添加配置支持（可选）

如果新平台需要配置（如 API token、自定义域名等），需要在 Settings 中添加：

#### 7.1 在 `src/lib/base/settings/settings.rs` 中添加配置结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    /// GitLab API tokens（支持多账号）
    #[serde(default)]
    pub tokens: Vec<String>,

    /// 当前激活的账号索引
    #[serde(default)]
    pub current: Option<usize>,

    /// 自定义 GitLab 实例 URL（用于自托管 GitLab）
    #[serde(default)]
    pub base_url: Option<String>,
}

impl GitLabConfig {
    /// 获取当前激活的 token
    pub fn get_current_token(&self) -> Option<&str> {
        // 实现逻辑
    }
}
```

#### 7.2 在 `Settings` 结构体中添加字段

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // ... 现有字段
    pub gitlab: GitLabConfig,
}
```

---

### 步骤 8：更新辅助函数（如需要）

如果 `src/lib/pr/helpers.rs` 中有平台特定的逻辑，需要更新：

```rust
pub fn detect_repo_type() -> Result<RepoType> {
    GitRepo::detect_repo_type()
}

// 如果新平台需要特殊的 URL 解析逻辑，添加相应的辅助函数
pub fn extract_gitlab_repo_from_url(url: &str) -> Result<String> {
    // 实现 GitLab URL 解析
}
```

---

## 📝 需要修改的文件

### 必须修改的文件

1. **`src/lib/git/types.rs`**
   - 添加新的 `RepoType` 变体

2. **`src/lib/git/repo.rs`**
   - 在 `parse_repo_type_from_url()` 中添加 URL 检测逻辑

3. **`src/lib/pr/{platform}/`**（新建目录）
   - `mod.rs` - 模块声明
   - `platform.rs` - 实现 `PlatformProvider` trait
   - `requests.rs` - API 请求结构
   - `responses.rs` - API 响应结构
   - `errors.rs` - 错误处理

4. **`src/lib/pr/mod.rs`**
   - 添加新平台模块声明
   - 导出新平台的公共类型

5. **`src/lib/pr/platform.rs`**
   - 在 `create_provider()` 中添加新平台分支

### 可选修改的文件

6. **`src/lib/base/settings/settings.rs`**（如果需要配置）
   - 添加新平台的配置结构

7. **`src/lib/pr/helpers.rs`**（如果需要特殊处理）
   - 添加平台特定的辅助函数

8. **`src/lib.rs`**（如果需要导出到库的公共 API）
   - 添加新平台的导出

### 测试文件（推荐）

9. **`tests/pr/{platform}.rs`**（新建测试文件）
   - 为新平台创建单元测试

10. **`tests/pr/mod.rs`**
    - 添加新平台的测试模块声明

11. **`tests/integration_test.rs`**（可选）
    - 添加集成测试

---

## 🔍 实现细节

### API 认证

不同平台的认证方式可能不同：

- **GitHub**: 使用 `Bearer {token}` 在 `Authorization` header
- **GitLab**: 使用 `PRIVATE-TOKEN {token}` 或 `Bearer {token}`
- **Bitbucket**: 使用 `Basic {base64(username:password)}` 或 OAuth

参考 GitHub 实现的 `get_headers()` 方法，根据平台文档实现认证。

### PR ID 格式

不同平台使用不同的 PR ID 格式：

- **GitHub**: 数字 ID（如 `123`）
- **GitLab**: IID（Internal ID，如 `42`），不是全局唯一的
- **Bitbucket**: 数字 ID（如 `123`）

在实现 `get_pull_request_info()` 等方法时，需要根据平台特性处理 ID。

### 分支命名

某些平台对分支名有特殊要求：

- **GitHub**: 支持 `/` 分隔的分支名，需要使用 `owner:branch` 格式
- **GitLab**: 支持 `/` 分隔的分支名，直接使用即可

### 合并方法

不同平台支持的合并方法可能不同：

- **GitHub**: `merge`、`squash`、`rebase`
- **GitLab**: `merge`、`squash`、`rebase`、`fast-forward`
- **Bitbucket**: `merge_commit`、`squash`、`fast_forward`

在实现 `merge_pull_request()` 时，需要根据平台支持的方法选择。

### 错误处理

不同平台的错误响应格式不同，需要：

1. 定义平台特定的错误响应结构
2. 实现 `format_error()` 函数格式化错误信息
3. 在 HTTP 响应处理中使用 `ensure_success_with()` 方法

---

## 🧪 测试

### 测试文件位置

测试文件应放在 `tests/pr/` 目录下：

```
tests/pr/
├── mod.rs              # 测试模块声明
├── github.rs           # GitHub 平台测试
├── gitlab.rs           # GitLab 平台测试（新增）
├── body_parser.rs      # PR body 解析测试
└── table.rs            # PR 表格测试
```

### 单元测试

为每个平台实现创建单元测试：

#### 1. 创建测试文件

在 `tests/pr/` 目录下创建新平台的测试文件（如 `tests/pr/gitlab.rs`）：

```rust
// tests/pr/gitlab.rs
use workflow::pr::gitlab::GitLab;
use workflow::pr::platform::PlatformProvider;

#[test]
fn test_create_merge_request() {
    // 测试创建 MR
    // 注意：需要使用 mock 或测试环境，避免实际调用 API
}

#[test]
fn test_merge_merge_request() {
    // 测试合并 MR
}
```

#### 2. 更新测试模块声明

在 `tests/pr/mod.rs` 中添加新平台的测试模块：

```rust
// tests/pr/mod.rs
pub mod body_parser;
pub mod github;
pub mod gitlab;  // 新增
pub mod table;
```

### 集成测试

在 `tests/integration_test.rs` 中添加集成测试：

```rust
#[test]
fn test_gitlab_platform() {
    // 测试 GitLab 平台的完整流程
    // 包括：创建、查询、合并等操作
}
```

### Mock 测试

使用 HTTP mock 库（如 `mockito`）模拟 API 响应，避免实际调用外部 API：

```rust
use mockito::{mock, Server};

#[test]
fn test_create_merge_request_with_mock() {
    let mut server = Server::new();

    // 创建 mock 响应
    let mock = server
        .mock("POST", "/api/v4/projects/123/merge_requests")
        .with_status(201)
        .with_body(r#"{"web_url": "https://gitlab.com/owner/repo/-/merge_requests/1"}"#)
        .create();

    // 执行测试
    // ...

    mock.assert();
}
```

### 测试覆盖

确保测试覆盖以下场景：

- ✅ 创建 PR/MR
- ✅ 获取 PR/MR 信息
- ✅ 合并 PR/MR
- ✅ 关闭 PR/MR
- ✅ 添加评论
- ✅ 错误处理（API 错误、网络错误等）
- ✅ 边界情况（空分支名、无效 ID 等）

---

## 📚 示例：添加 GitLab 支持

### 完整的文件结构

```
src/lib/pr/gitlab/
├── mod.rs              # 模块导出
├── platform.rs         # GitLab 实现 PlatformProvider
├── requests.rs         # API 请求结构
├── responses.rs        # API 响应结构
└── errors.rs           # 错误处理
```

### 关键代码片段

#### `platform.rs` 示例

```rust
impl PlatformProvider for GitLab {
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String> {
        let (project_id, _) = Self::get_project_info()?;
        let base_branch = target_branch
            .map(|s| s.to_string())
            .unwrap_or_else(|| GitBranch::get_default_branch()?);

        let url = format!("{}/projects/{}/merge_requests", Self::base_url(), project_id);
        let request = CreateMergeRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            source_branch: source_branch.to_string(),
            target_branch: base_branch,
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<_, Value>::new()
            .body(&request)
            .headers(&headers);

        let response = client.post(&url, config)?;
        let response_data: CreateMergeRequestResponse = response
            .ensure_success_with(handle_gitlab_error)?
            .as_json()?;

        Ok(response_data.web_url)
    }

    // ... 其他方法实现
}
```

---

## ✅ 检查清单

在完成新平台实现后，使用以下清单检查：

### 代码实现

- [ ] 在 `src/lib/git/types.rs` 中添加新的 `RepoType` 变体
- [ ] 在 `src/lib/git/repo.rs` 中添加 URL 检测逻辑
- [ ] 创建新平台目录 `src/lib/pr/{platform}/`
- [ ] 实现 `requests.rs`（所有 API 请求结构）
- [ ] 实现 `responses.rs`（所有 API 响应结构）
- [ ] 实现 `errors.rs`（错误处理和格式化）
- [ ] 实现 `platform.rs`（所有 `PlatformProvider` trait 方法）
- [ ] 在 `src/lib/pr/mod.rs` 中导出新平台
- [ ] 在 `src/lib/pr/platform.rs` 的 `create_provider()` 中添加分支

### 配置（如需要）

- [ ] 在 `src/lib/base/settings/settings.rs` 中添加配置结构
- [ ] 实现配置的序列化/反序列化
- [ ] 实现 `get_current_token()` 等方法

### 测试

- [ ] 创建测试文件 `tests/pr/{platform}.rs`
- [ ] 在 `tests/pr/mod.rs` 中添加测试模块声明
- [ ] 编写单元测试（覆盖主要功能）
- [ ] 编写集成测试（可选）
- [ ] 使用 mock 测试避免实际 API 调用
- [ ] 所有测试通过（`cargo test`）

### 文档

- [ ] 为新平台添加代码注释
- [ ] 更新相关架构文档（如 `PR_ARCHITECTURE.md`）
- [ ] 更新 README（如需要）

### 代码质量

- [ ] 运行 `cargo fmt` 格式化代码
- [ ] 运行 `cargo clippy` 检查代码质量
- [ ] 修复所有警告和错误
- [ ] 遵循项目的错误处理规范

---

## 🔗 相关文档

- [开发规范](./DEVELOPMENT_GUIDELINES.md) - 代码风格和最佳实践
- [PR 模块架构文档](../architecture/lib/PR_ARCHITECTURE.md) - PR 模块的详细架构
- [Git 模块架构文档](../architecture/lib/GIT_ARCHITECTURE.md) - Git 模块的详细架构

---

## 📝 注意事项

1. **API 版本**：不同平台可能有不同的 API 版本，需要在实现时指定正确的版本
2. **速率限制**：注意平台的 API 速率限制，必要时实现重试逻辑
3. **自托管实例**：如果平台支持自托管（如 GitLab），需要支持自定义 base URL
4. **向后兼容**：添加新平台时，确保不影响现有平台的功能
5. **错误消息**：提供清晰、用户友好的错误消息，帮助用户快速定位问题

---

*最后更新：2024-12*
