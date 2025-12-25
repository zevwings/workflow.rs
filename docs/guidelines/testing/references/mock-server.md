# Mock服务器使用指南

> 本文档详细说明 Mock 服务器的使用方法。

---

## 📋 目录

- [MockServer基本使用](#1-mockserver基本使用)
- [创建Mock端点](#2-创建mock端点)
- [高级Mock方法](#3-高级mock方法)
- [预设Mock端点](#4-预设mock端点)
- [模拟错误情况](#5-模拟错误情况)
- [验证Mock调用](#6-验证mock调用)
- [Mock服务器最佳实践](#7-mock服务器最佳实践)

---

## 1. MockServer基本使用

### 创建 Mock 服务器

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_api_call() {
    // 创建 Mock 服务器
    let mut mock_server = MockServer::new();

    // 设置 API 基础 URL
    mock_server.setup_github_base_url();
    // 或
    mock_server.setup_jira_base_url();
}
```

---

## 2. 创建Mock端点

### 使用 server 属性创建 Mock

`MockServer` 提供了 `server` 属性，可以直接访问底层的 `mockito::Server`：

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_basic_mock() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 使用 server 属性创建 Mock
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // ...
}
```

---

## 3. 高级Mock方法

`MockServer` 提供了便捷的高级方法，简化常见Mock场景的创建。

### mock_github_pr - GitHub PR Mock

创建GitHub PR相关的Mock端点，自动匹配GitHub API的请求头：

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_github_pr() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建GitHub PR Mock
    mock_server
        .mock_github_pr(
            "POST",
            "/repos/owner/repo/pulls",
            r#"{"number": 123, "title": "Test PR"}"#,
            201,
        );

    // 执行测试
    // ...
}
```

### mock_jira_issue - Jira Issue Mock

创建Jira Issue相关的Mock端点，自动匹配Jira API的请求头：

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_jira_issue() {
    let mut mock_server = MockServer::new();
    mock_server.setup_jira_base_url();

    // 创建Jira Issue Mock
    mock_server
        .mock_jira_issue(
            "GET",
            "/rest/api/3/issue/PROJ-123",
            r#"{"key": "PROJ-123", "fields": {"summary": "Test Issue"}}"#,
            200,
        );

    // 执行测试
    // ...
}
```

### mock_error_response - 错误响应Mock

创建标准化的错误响应Mock：

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_error_response() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建错误响应Mock
    mock_server
        .mock_error_response(
            "GET",
            "/api/endpoint",
            "Not Found",
            404,
        );

    // 执行测试
    // ...
}
```

---

## 4. 预设Mock端点

`MockServer` 提供了预设的Mock端点方法，进一步简化常见场景的Mock设置。

### GitHub API 预设

#### setup_github_create_pr_success

设置GitHub创建PR成功响应：

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_create_pr() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 设置创建PR成功响应
    mock_server
        .setup_github_create_pr_success("owner", "repo", 123);

    // 执行测试
    // ...
}
```

#### setup_github_get_pr

设置GitHub获取PR信息响应：

```rust
use crate::common::http_helpers::MockServer;
use serde_json::json;

#[test]
fn test_get_pr() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 设置获取PR响应
    let pr_data = json!({
        "number": 123,
        "title": "Test PR",
        "html_url": "https://github.com/owner/repo/pull/123",
        "state": "open"
    });
    mock_server
        .setup_github_get_pr("owner", "repo", 123, &pr_data);

    // 执行测试
    // ...
}
```

#### setup_github_error

设置GitHub错误响应：

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_github_error() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 设置错误响应
    mock_server
        .setup_github_error("/api/endpoint", 404, "Not Found");

    // 执行测试
    // ...
}
```

### Jira API 预设

#### setup_jira_get_issue_success

设置Jira获取Issue成功响应：

```rust
use crate::common::http_helpers::MockServer;
use serde_json::json;

#[test]
fn test_get_jira_issue() {
    let mut mock_server = MockServer::new();
    mock_server.setup_jira_base_url();

    // 设置获取Issue响应
    let issue_data = json!({
        "key": "PROJ-123",
        "fields": {
            "summary": "Test Issue",
            "status": {"name": "In Progress"}
        }
    });
    mock_server
        .setup_jira_get_issue_success("PROJ-123", &issue_data);

    // 执行测试
    // ...
}
```

#### setup_jira_issue_not_found

设置Jira Issue不存在响应：

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_jira_issue_not_found() {
    let mut mock_server = MockServer::new();
    mock_server.setup_jira_base_url();

    // 设置Issue不存在响应
    mock_server
        .setup_jira_issue_not_found("PROJ-123");

    // 执行测试
    // ...
}
```

---

## 5. 模拟错误情况

### 基本 Mock 端点

```rust
#[test]
fn test_get_request() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建 Mock 端点
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // ...
}
```

### 带条件的 Mock 端点

```rust
use mockito::Matcher;

#[test]
fn test_with_conditions() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建带条件的 Mock
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .match_header("authorization", Matcher::Regex(r"token .+".to_string()))
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("key".to_string(), "value".to_string()),
        ]))
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // ...
}
```

### 模拟网络错误

```rust
#[test]
fn test_network_error() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 模拟 500 错误
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(500)
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    // 测试错误处理
    // ...
}
```

### 模拟超时

```rust
#[test]
fn test_timeout() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 模拟延迟响应（测试超时处理）
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .with_header("content-type", "application/json")
        .create();

    // 测试超时处理逻辑
    // ...
}
```

---

## 6. 验证Mock调用

### 验证 Mock 是否被调用

```rust
#[test]
fn test_verify_mock() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建 Mock 并保存引用
    let mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // ...

    // 验证 Mock 被调用
    mock.assert();
}
```

---

## 7. Mock服务器最佳实践

### 每个测试独立 Mock

```rust
// ✅ 好的做法：每个测试创建独立的 Mock 服务器
#[test]
fn test_1() {
    let mut mock_server = MockServer::new();
    // ...
}

#[test]
fn test_2() {
    let mut mock_server = MockServer::new();
    // ...
}
```

### 自动清理

```rust
// ✅ MockServer 会在 Drop 时自动清理环境变量
#[test]
fn test_with_auto_cleanup() {
    let mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    // 测试代码
    // MockServer 会在测试结束时自动清理环境变量
}
```

---

## 相关文档

- [测试工具指南](./tools.md) - 其他测试工具
- [测试编写规范](../writing.md) - 测试编写规范

---

**最后更新**: 2025-12-25
