# Mock服务器使用指南

> 本文档详细说明 Mock 服务器的使用方法。

---

## 📋 目录

- [MockServer基本使用](#1-mockserver基本使用)
- [创建Mock端点](#2-创建mock端点)
- [模拟错误情况](#3-模拟错误情况)
- [验证Mock调用](#4-验证mock调用)
- [Mock服务器最佳实践](#5-mock服务器最佳实践)

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

---

## 3. 模拟错误情况

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

## 4. 验证Mock调用

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

## 5. Mock服务器最佳实践

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
