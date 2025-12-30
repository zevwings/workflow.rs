# HTTP 重试机制需求文档

> 本文档描述为 Jira API、GitHub API 和 LLM API 添加 HTTP 重试机制的需求。

---

## 📋 目录

- [概述](#-概述)
- [当前状态](#-当前状态)
- [需求分析](#-需求分析)
- [实施计划](#-实施计划)
- [优先级](#-优先级)
- [实现建议](#-实现建议)
- [注意事项](#-注意事项)
- [相关文档](#-相关文档)

---

## 📋 概述

### 背景

当前代码库中的 HTTP API 调用（Jira API、GitHub API、LLM API）缺乏统一的重试机制，导致在网络不稳定或遇到临时性错误（如 5xx 服务器错误、429 限流、网络超时等）时，操作容易失败。

### 目标

为所有 HTTP API 调用添加统一的重试机制，提高系统的可靠性和容错能力：

- ✅ 统一使用 `HttpRetry::retry()` 包装所有 HTTP 请求
- ✅ 自动处理网络错误、5xx 服务器错误、429 限流等可重试错误
- ✅ 提供清晰的错误消息和重试日志
- ✅ 支持用户交互式确认（可选）
- ✅ 使用指数退避算法，避免对服务器造成过大压力

### 原则

- **HTTP 请求** → 使用 `HttpRetry::retry()`（专门为 HTTP 设计）
- **命令层非 HTTP 操作** → 使用 `execute_with_timeout_and_retry()`（通用超时+重试）

---

## 📊 当前状态

- **状态**: ⏳ 待实施
- **实现度**: 0%
- **优先级**: ⭐⭐⭐ 高
- **分类**: 基础设施改进

### 现状分析

1. **Jira API** (`src/lib/jira`)
   - ❌ 10 个 HTTP 方法未使用重试机制
   - ❌ 附件下载有手动重试逻辑，但未统一

2. **GitHub API** (`src/lib/pr/github/platform.rs`)
   - ❌ 21 个 HTTP 方法未使用重试机制
   - ❌ 所有 GitHub API 调用都直接使用 `HttpClient`，无重试

3. **LLM API** (`src/lib/base/llm/client.rs`)
   - ❌ 1 个 HTTP 方法未使用重试机制
   - ❌ 使用 `reqwest::Client` 直接发送请求，无重试

### 已有基础设施

- ✅ `HttpRetry::retry()` - HTTP 重试工具已实现
- ✅ `HttpRetryConfig` - 重试配置已实现
- ✅ 智能错误判断 - 可自动识别可重试错误
- ✅ 指数退避算法 - 已实现
- ✅ 用户交互支持 - 已实现

---

## 🔍 需求分析

### 1. Jira API (`src/lib/jira`)

#### 1.1 `src/lib/jira/api/issue.rs`

需要添加重试的方法（7 个）：

1. **`get_issue()`** (line 69-82)
   - HTTP 方法：`GET`
   - 用途：获取 issue 信息
   - 重试原因：网络错误、5xx 错误可重试

2. **`get_issue_transitions()`** (line 107-133)
   - HTTP 方法：`GET`
   - 用途：获取 issue 的可用 transitions
   - 重试原因：网络错误、5xx 错误可重试

3. **`transition_issue()`** (line 145-163)
   - HTTP 方法：`POST`
   - 用途：更新 issue 状态
   - 重试原因：网络错误、5xx 错误可重试

4. **`assign_issue()`** (line 175-192)
   - HTTP 方法：`PUT`
   - 用途：分配 issue 给用户
   - 重试原因：网络错误、5xx 错误可重试

5. **`add_issue_comment()`** (line 203-220)
   - HTTP 方法：`POST`
   - 用途：添加评论到 issue
   - 重试原因：网络错误、5xx 错误可重试

6. **`upload_attachment()`** (line 260-336)
   - HTTP 方法：`POST` (multipart)
   - 用途：上传附件到 issue
   - 重试原因：网络错误、5xx 错误可重试
   - ⚠️ 特殊处理：大文件上传可能需要更长的超时时间

7. **`get_issue_changelog()`** (line 338-365)
   - HTTP 方法：`GET`
   - 用途：获取 issue 的变更历史
   - 重试原因：网络错误、5xx 错误可重试

#### 1.2 `src/lib/jira/api/project.rs`

需要添加重试的方法（1 个）：

8. **`get_project_statuses()`** (line 31-50)
   - HTTP 方法：`GET`
   - 用途：获取项目的状态列表
   - 重试原因：网络错误、5xx 错误可重试
   - ⚠️ 注意：已有 10 秒超时设置

#### 1.3 `src/lib/jira/api/user.rs`

需要添加重试的方法（1 个）：

9. **`get_current_user()`** (line 22-29)
   - HTTP 方法：`GET`
   - 用途：获取当前 Jira 用户信息
   - 重试原因：网络错误、5xx 错误可重试

#### 1.4 `src/lib/jira/attachments/http_client.rs`

需要添加重试的方法（1 个）：

10. **`download_file()`** (line 37-127)
    - HTTP 方法：`GET` (stream)
    - 用途：下载附件文件
    - 重试原因：网络错误、5xx 错误可重试
    - ⚠️ 特殊处理：已有手动重试逻辑（CloudFront URL），需要统一使用 `HttpRetry::retry()`

### 2. GitHub API (`src/lib/pr/github/platform.rs`)

需要添加重试的方法（21 个）：

1. **`create_pull_request()`** (line 35-73)
   - HTTP 方法：`POST`
   - 用途：创建 Pull Request
   - 重试原因：网络错误、5xx 错误、429 限流可重试

2. **`merge_pull_request()`** (line 76-145)
   - HTTP 方法：`PUT`
   - 用途：合并 Pull Request
   - 重试原因：网络错误、5xx 错误、429 限流可重试
   - ⚠️ 注意：包含删除分支的额外请求（line 121, 137）

3. **`get_pull_request_info()`** (line 145-163)
   - HTTP 方法：`GET`
   - 用途：获取 PR 信息（JSON 格式）
   - 重试原因：网络错误、5xx 错误、429 限流可重试

4. **`get_pull_request_url()`** (line 164-171)
   - HTTP 方法：`GET`
   - 用途：获取 PR URL
   - 重试原因：网络错误、5xx 错误、429 限流可重试

5. **`get_pull_request_title()`** (line 172-179)
   - HTTP 方法：`GET`
   - 用途：获取 PR 标题
   - 重试原因：网络错误、5xx 错误、429 限流可重试

6. **`get_pull_request_body()`** (line 180-187)
   - HTTP 方法：`GET`
   - 用途：获取 PR 正文
   - 重试原因：网络错误、5xx 错误、429 限流可重试

7. **`get_pull_request_status()`** (line 188-199)
   - HTTP 方法：`GET`
   - 用途：获取 PR 状态
   - 重试原因：网络错误、5xx 错误、429 限流可重试

8. **`get_pull_requests()`** (line 200-273)
   - HTTP 方法：`GET`
   - 用途：获取 PR 列表
   - 重试原因：网络错误、5xx 错误、429 限流可重试
   - ⚠️ 注意：包含两个请求（open 和 all）

9. **`get_current_branch_pull_request()`** (line 225-293)
   - HTTP 方法：`GET`
   - 用途：获取当前分支的 PR
   - 重试原因：网络错误、5xx 错误、429 限流可重试
   - ⚠️ 注意：包含多个请求（head 和 all）

10. **`get_pull_request_diff()`** (line 296-358)
    - HTTP 方法：`GET`
    - 用途：获取 PR diff
    - 重试原因：网络错误、5xx 错误、429 限流可重试
    - ⚠️ 注意：可能触发 fallback 方法（406 错误）

11. **`close_pull_request()`** (line 360-389)
    - HTTP 方法：`PATCH`
    - 用途：关闭 Pull Request
    - 重试原因：网络错误、5xx 错误、429 限流可重试

12. **`add_comment()`** (line 391-425)
    - HTTP 方法：`POST`
    - 用途：添加评论到 PR
    - 重试原因：网络错误、5xx 错误、429 限流可重试

13. **`approve_pull_request()`** (line 427-488)
    - HTTP 方法：`POST`
    - 用途：批准 Pull Request
    - 重试原因：网络错误、5xx 错误、429 限流可重试

14. **`update_pr_base()`** (line 490-515)
    - HTTP 方法：`PATCH`
    - 用途：更新 PR 的 base 分支
    - 重试原因：网络错误、5xx 错误、429 限流可重试

15. **`update_pull_request()`** (line 518-554)
    - HTTP 方法：`PATCH`
    - 用途：更新 PR 的标题和/或描述
    - 重试原因：网络错误、5xx 错误、429 限流可重试

16. **`get_repository_info()`** (line 629-639)
    - HTTP 方法：`GET`
    - 用途：获取仓库信息
    - 重试原因：网络错误、5xx 错误、429 限流可重试

17. **`get_pull_requests_raw()`** (line 651-712)
    - HTTP 方法：`GET`
    - 用途：获取 PR 列表原始数据
    - 重试原因：网络错误、5xx 错误、429 限流可重试

18. **`fetch_pr_info_internal()`** (line 714-735)
    - HTTP 方法：`GET`
    - 用途：获取 PR 信息（内部方法）
    - 重试原因：网络错误、5xx 错误、429 限流可重试

19. **`get_user_info()`** (line 746-762)
    - HTTP 方法：`GET`
    - 用途：获取 GitHub 用户信息
    - 重试原因：网络错误、5xx 错误、429 限流可重试

20. **`get_pull_request_diff_fallback()`** (line 768-856)
    - HTTP 方法：`GET`
    - 用途：获取 PR diff（fallback 方法）
    - 重试原因：网络错误、5xx 错误、429 限流可重试
    - ⚠️ 注意：包含多个请求（files 和 diff）

21. **`get_pull_request_files_internal()`** (line 866-890)
    - HTTP 方法：`GET`
    - 用途：获取 PR 文件列表（内部方法）
    - 重试原因：网络错误、5xx 错误、429 限流可重试

### 3. LLM API (`src/lib/base/llm/client.rs`)

需要添加重试的方法（1 个）：

1. **`call()`** (line 77-134)
   - HTTP 方法：`POST`
   - 用途：调用 LLM API
   - 重试原因：网络错误、5xx 错误、429 限流可重试
   - ⚠️ 注意：当前使用 `reqwest::Client` 直接发送请求，已有 60 秒超时

---

## 📋 实施计划

### 阶段 1：Jira API（优先级：高）

**目标**：为所有 Jira API 调用添加重试机制

**任务**：
1. 修改 `src/lib/jira/api/issue.rs` - 7 个方法
2. 修改 `src/lib/jira/api/project.rs` - 1 个方法
3. 修改 `src/lib/jira/api/user.rs` - 1 个方法
4. 修改 `src/lib/jira/attachments/http_client.rs` - 1 个方法（统一手动重试逻辑）

**预计工作量**：2-3 天

### 阶段 2：GitHub API（优先级：高）

**目标**：为所有 GitHub API 调用添加重试机制

**任务**：
1. 修改 `src/lib/pr/github/platform.rs` - 21 个方法

**预计工作量**：3-4 天

### 阶段 3：LLM API（优先级：高）

**目标**：为 LLM API 调用添加重试机制

**任务**：
1. 修改 `src/lib/base/llm/client.rs` - 1 个方法

**预计工作量**：1 天

### 总计

- **需要修改的方法数**：32 个
- **需要修改的文件数**：5 个
- **预计总工作量**：6-8 天

---

## ⭐ 优先级

### 高优先级（立即实施）

所有模块都是高优先级，因为：

1. **可靠性提升**：网络操作容易遇到临时性错误，重试机制可以显著提高成功率
2. **用户体验**：减少因网络问题导致的失败，提升用户体验
3. **系统稳定性**：提高系统在恶劣网络环境下的稳定性

### 优先级排序

1. **LLM API** - 用户直接调用，失败影响明显
2. **GitHub API** - PR 操作频繁，失败影响工作流
3. **Jira API** - 集成功能，失败影响自动化流程

---

## 🔧 实现建议

### 1. 统一导入

在每个文件中添加：

```rust
use crate::base::http::retry::{HttpRetry, HttpRetryConfig};
```

### 2. 统一配置

使用默认配置或创建模块特定的配置：

```rust
// 使用默认配置
let config = HttpRetryConfig::new();

// 或自定义配置（针对特殊场景）
let mut config = HttpRetryConfig::new();
config.max_retries = 3;
config.initial_delay = 1;
config.max_delay = 30;
config.interactive = false; // 非交互式场景
```

### 3. 包装 HTTP 请求

将所有 `client.get/post/put/delete/patch()` 调用包装在 `HttpRetry::retry()` 中：

```rust
// 示例：GET 请求
let result = HttpRetry::retry(
    || {
        let client = HttpClient::global()?;
        let auth = jira_auth_config()?;
        let config = RequestConfig::<Value, Value>::new().auth(auth);
        let response = client.get(&url, config)?;
        response
            .ensure_success()?
            .as_json()
            .wrap_err("Failed to parse response")
    },
    &HttpRetryConfig::new(),
    "Get issue",
)?;
```

```rust
// 示例：POST 请求
let result = HttpRetry::retry(
    || {
        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<_, Value>::new()
            .body(&request)
            .headers(&headers);
        let response = client.post(&url, config)?;
        let response_data: CreatePullRequestResponse =
            response.ensure_success_with(handle_github_error)?.as_json()?;
        Ok(response_data.html_url)
    },
    &HttpRetryConfig::new(),
    "Create pull request",
)?;
```

### 4. 特殊处理场景

#### 4.1 大文件上传

对于 `upload_attachment()` 等大文件上传操作，可能需要更长的超时时间：

```rust
let mut config = HttpRetryConfig::new();
config.max_retries = 2; // 减少重试次数，避免重复上传大文件
config.initial_delay = 2; // 增加初始延迟
```

#### 4.2 流式下载

对于 `download_file()` 等流式下载操作，需要确保重试时不会重复下载：

```rust
// 注意：流式下载的重试需要重新开始下载
// HttpRetry 会自动处理，但需要确保输出文件路径可以覆盖
```

#### 4.3 429 限流

`HttpRetry` 已经自动处理 429 错误，但可能需要调整重试延迟：

```rust
// HttpRetry 会自动识别 429 错误并重试
// 默认配置已经足够，无需特殊处理
```

### 5. 避免嵌套重试

某些方法内部已经调用了其他需要重试的方法，需要避免嵌套重试：

```rust
// ❌ 错误示例：嵌套重试
pub fn get_issue_attachments(ticket: &str) -> Result<Vec<JiraAttachment>> {
    HttpRetry::retry(
        || {
            let issue = Self::get_issue(ticket)?; // get_issue 内部已经有重试
            Ok(issue.fields.attachment.unwrap_or_default())
        },
        &config,
        "Get issue attachments",
    )?
}

// ✅ 正确示例：只在最底层添加重试
pub fn get_issue_attachments(ticket: &str) -> Result<Vec<JiraAttachment>> {
    let issue = Self::get_issue(ticket)?; // get_issue 内部已经有重试
    Ok(issue.fields.attachment.unwrap_or_default())
}
```

---

## ⚠️ 注意事项

### 1. 不要重复重试

某些方法内部已经调用了其他需要重试的方法，需要避免嵌套重试。在实施时，需要：

- 分析调用链，确保只在最底层添加重试
- 对于内部方法（如 `fetch_pr_info_internal()`），如果只在内部使用，可以添加重试
- 对于公共方法（如 `get_pull_request_info()`），如果内部调用了其他方法，需要评估是否需要重试

### 2. 错误处理

确保错误消息清晰，包含操作名称：

```rust
HttpRetry::retry(
    || { /* ... */ },
    &config,
    "Get issue", // 清晰的操作名称
)?
```

### 3. 性能考虑

重试会增加延迟，但对于网络操作是必要的：

- 默认配置：3 次重试，初始延迟 1 秒，指数退避
- 对于频繁调用的方法，可以考虑减少重试次数
- 对于关键操作，可以增加重试次数

### 4. 测试

添加重试后需要测试网络错误场景：

- 模拟网络超时
- 模拟 5xx 服务器错误
- 模拟 429 限流错误
- 验证重试逻辑正确性

### 5. 日志和监控

重试机制会产生日志，需要确保：

- 日志级别适当（trace/debug）
- 包含重试次数和延迟信息
- 便于问题排查

---

## 📚 相关文档

- [HTTP Retry 使用分析](../../analysis/http_retry_usage_analysis.md) - 详细的技术分析文档
- [HTTP 架构文档](../../docs/architecture/http.md) - HTTP 模块架构说明
- [Resilience 模块文档](../../docs/architecture/resilience.md) - 超时和重试机制说明

---

## 📊 进度跟踪

### 待实施

- [ ] 阶段 1：Jira API（10 个方法）
- [ ] 阶段 2：GitHub API（21 个方法）
- [ ] 阶段 3：LLM API（1 个方法）

### 已完成

- ✅ 需求分析和文档编写
- ✅ 技术方案确定

---

**最后更新**：2025-01-XX
**文档版本**：1.0

