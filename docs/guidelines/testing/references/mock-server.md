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
- [动态响应生成（模板系统）](#7-动态响应生成模板系统)
- [请求验证](#8-请求验证)
- [Mock场景预设库](#9-mock场景预设库)
- [Mock服务器最佳实践](#10-mock服务器最佳实践)

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

## 7. 动态响应生成（模板系统）

`MockServer` 支持使用模板创建动态响应，避免硬编码JSON字符串。

### 使用模板创建Mock

```rust
use crate::common::http_helpers::MockServer;
use std::collections::HashMap;

#[test]
fn test_pr_with_template() -> color_eyre::Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建变量映射
    let mut vars = HashMap::new();
    vars.insert("pr_number".to_string(), "123".to_string());
    vars.insert("owner".to_string(), "test-owner".to_string());
    vars.insert("repo".to_string(), "test-repo".to_string());

    // 使用模板创建Mock端点
    mock_server.mock_with_template(
        "GET",
        "/repos/{owner}/{repo}/pulls/{pr_number}",
        r#"{
            "number": {{pr_number}},
            "title": "Test PR",
            "owner": "{{owner}}",
            "repo": "{{repo}}",
            "html_url": "https://github.com/{{owner}}/{{repo}}/pull/{{pr_number}}"
        }"#,
        vars,
        200,
    );

    // 执行测试...
    // let response = client.get_pr("test-owner", "test-repo", 123)?;
    // assert_eq!(response.number, 123);

    Ok(())
}
```

### 模板语法

- `{{variable_name}}` - 变量替换
- 支持JSON格式
- 支持嵌套对象和数组

### 使用响应生成器（高级）

```rust
use crate::common::mock_templates::{ResponseGenerator, TemplateResponseGenerator, MockRequest};
use std::collections::HashMap;

#[test]
fn test_with_response_generator() -> color_eyre::Result<()> {
    let mut vars = HashMap::new();
    vars.insert("status".to_string(), "success".to_string());

    let generator: Box<dyn ResponseGenerator> = Box::new(
        TemplateResponseGenerator::new(
            r#"{"status": "{{status}}"}"#.to_string(),
            vars,
        )
    );

    let request = MockRequest::new("GET".to_string(), "/api/test".to_string());
    let response = generator.generate(&request)?;

    assert!(response.contains("success"));
    Ok(())
}
```

---

## 8. 请求验证

`MockServer` 提供了请求验证功能，确保Mock端点接收到正确的请求。

### 验证请求头和请求体

```rust
use crate::common::http_helpers::MockServer;
use crate::common::mock_validators::RequestValidator;
use std::collections::HashMap;

#[test]
fn test_with_validation() -> color_eyre::Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建请求验证器
    let validator = RequestValidator::new()
        .require_header("authorization", r"token .+")
        .require_header("accept", "application/vnd.github.v3+json")
        .require_body_json(r#"{"title": ".+", "head": ".+", "base": ".+"}"#)
        .require_query_param("draft", "false");

    // 创建带验证的Mock端点
    // 注意：mockito 本身不支持请求验证，验证需要在测试代码中手动进行
    mock_server.mock_with_template(
        "POST",
        "/repos/owner/repo/pulls",
        r#"{"number": 123, "title": "Test PR"}"#,
        HashMap::new(),
        201,
    );

    // 在测试中手动验证请求
    // let request = build_request(...);
    // let validation_result = validator.validate(&request);
    // assert!(validation_result.is_valid());

    Ok(())
}
```

### 验证路径参数和查询参数

```rust
use crate::common::mock_validators::RequestValidator;
use crate::common::mock_templates::MockRequest;

#[test]
fn test_param_validation() -> color_eyre::Result<()> {
    let validator = RequestValidator::new()
        .require_path_param("pr_number", "123")
        .require_query_param("state", "open");

    let mut request = MockRequest::new("GET".to_string(), "/pulls/123".to_string());
    request.path_params.insert("pr_number".to_string(), "123".to_string());
    request.query_params.insert("state".to_string(), "open".to_string());

    let result = validator.validate(&request);
    assert!(result.is_valid());
    Ok(())
}
```

---

## 9. Mock场景预设库

`MockServer` 提供了预设场景库，可以快速设置常见的Mock场景。

### 加载和使用场景

```rust
use crate::common::http_helpers::MockServer;
use std::path::PathBuf;

#[test]
fn test_with_scenario() -> color_eyre::Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 加载预设场景
    let scenario_path = PathBuf::from("tests/fixtures/mock_scenarios/github/pr_workflow.json");
    mock_server.load_scenario(&scenario_path)?;

    // 现在所有场景中定义的Mock端点都已配置好
    // 执行测试...
    // let pr = client.create_pr(...)?;
    // let pr_info = client.get_pr(...)?;

    Ok(())
}
```

### 使用场景管理器（高级）

```rust
use crate::common::mock_scenarios::MockScenarioManager;
use crate::common::http_helpers::MockServer;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_with_scenario_manager() -> color_eyre::Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    let mut manager = MockScenarioManager::new(
        PathBuf::from("tests/fixtures")
    );

    // 加载场景
    let scenario_path = PathBuf::from("tests/fixtures/mock_scenarios/github/pr_workflow.json");
    manager.load_scenario(&scenario_path)?;

    // 设置变量
    let mut vars = HashMap::new();
    vars.insert("owner".to_string(), "test-owner".to_string());
    vars.insert("repo".to_string(), "test-repo".to_string());
    vars.insert("pr_number".to_string(), "123".to_string());
    vars.insert("title".to_string(), "Test PR".to_string());

    // 应用场景
    manager.apply_scenario(&mut mock_server, "github_pr_workflow", Some(&vars))?;

    // 执行测试...
    Ok(())
}
```

### 创建自定义场景

创建 `tests/fixtures/mock_scenarios/custom/my_scenario.json`:

```json
{
  "name": "my_custom_scenario",
  "description": "自定义测试场景",
  "mocks": [
    {
      "method": "GET",
      "path": "/api/test/{id}",
      "response": {
        "template": "{\"id\": {{id}}, \"status\": \"ok\"}",
        "status": 200
      },
      "validation": {
        "required_headers": ["authorization"],
        "required_query_params": ["format"]
      }
    }
  ]
}
```

### 完整示例：使用模板和验证

```rust
use crate::common::http_helpers::MockServer;
use crate::common::mock_validators::RequestValidator;
use std::collections::HashMap;

#[test]
fn test_with_template_and_validation() -> color_eyre::Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建验证器
    let validator = RequestValidator::new()
        .require_header("authorization", r"token .+")
        .require_body_json(r#"{"title": ".+"}"#);

    // 使用模板创建Mock
    let mut vars = HashMap::new();
    vars.insert("pr_number".to_string(), "456".to_string());

    mock_server.mock_with_template(
        "POST",
        "/repos/owner/repo/pulls",
        r#"{"number": {{pr_number}}, "title": "New PR"}"#,
        vars,
        201,
    );

    // 在测试中验证请求
    // let request = build_request(...);
    // let result = validator.validate(&request);
    // assert!(result.is_valid(), "{}", result.to_report());

    Ok(())
}
```

---

## 10. Mock服务器最佳实践

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

### Mock使用建议

- **优先使用场景预设库**：对于常见的工作流程，使用预设场景比手动创建Mock更高效
- **使用模板系统**：当需要动态数据时，使用模板系统而不是硬编码响应
- **验证请求**：对于重要的API调用，使用请求验证确保测试的正确性

### 模板系统使用建议

- **使用模板变量**：对于需要动态数据的场景，使用模板变量而不是硬编码值
- **合理组织变量**：将相关的变量组织在一起，使用有意义的变量名
- **验证模板语法**：确保模板语法正确，避免运行时错误

### 场景预设库使用建议

- **优先使用预设场景**：对于常见的工作流程，优先使用预设场景
- **创建自定义场景**：对于项目特定的工作流程，创建自定义场景
- **场景版本管理**：为场景文件添加版本号，便于追踪和更新

---

## 相关文档

- [测试工具指南](./tools.md) - 其他测试工具
- [测试编写规范](../writing.md) - 测试编写规范

---

**最后更新**: 2025-01-27
