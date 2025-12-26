# 测试编写规范

> 本文档定义测试编写的具体规范和最佳实践。

---

## 📋 目录

- [测试编写规范](#-测试编写规范)
- [编写测试最佳实践](#-编写测试最佳实践)
  - [1. 测试命名规范](#1-测试命名规范)
  - [2. 测试结构（AAA 模式）](#2-测试结构aaa-模式)
  - [3. 测试独立性](#3-测试独立性)
  - [4. 测试覆盖原则](#4-测试覆盖原则)
  - [5. 测试数据管理](#5-测试数据管理)
  - [6. Mock 使用原则](#6-mock-使用原则)
  - [7. 断言最佳实践](#7-断言最佳实践)
  - [8. 参数化测试](#8-参数化测试)
  - [9. 测试迁移](#9-测试迁移)
  - [10. 测试文档](#10-测试文档)
- [被忽略测试文档规范](#-被忽略测试文档规范)

---

## ✅ 测试编写规范

### 1. 测试结构

每个测试应包含：
- **Arrange**：准备测试数据和环境
- **Act**：执行被测试的功能
- **Assert**：验证结果

```rust
#[test]
fn test_parse_ticket_id() {
    // Arrange
    let input = "PROJ-123";

    // Act
    let result = parse_ticket_id(input);

    // Assert
    assert_eq!(result, Some("PROJ-123"));
}
```

### 2. 错误处理测试

为错误情况编写测试：

```rust
#[test]
fn test_parse_ticket_id_invalid() {
    assert_eq!(parse_ticket_id("invalid"), None);
    assert_eq!(parse_ticket_id(""), None);
}
```

### 错误处理最佳实践

#### 使用 `Result<()>` 返回类型

```rust
// ✅ 推荐：使用 Result<()> 返回类型
#[test]
fn test_example() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    env.create_file("test.txt", "content")?;
    Ok(())
}

// ❌ 不推荐：使用 () 返回类型和 expect()
#[test]
fn test_example() {
    let env = CliTestEnv::new().expect("Failed to create env");
    env.create_file("test.txt", "content").expect("Failed to create file");
}
```

#### 使用 `?` 操作符

```rust
// ✅ 推荐：使用 ? 操作符
let result = function_that_may_fail()?;

// ❌ 不推荐：使用 unwrap/expect
let result = function_that_may_fail().unwrap();
```

**优势**：
- ✅ 错误传播清晰
- ✅ 代码简洁
- ✅ 符合 Rust 最佳实践
- ✅ 自动传播错误上下文，提供更多信息

#### Fixture函数中的错误处理

```rust
// ✅ 推荐：Fixture 失败应该 panic，但错误信息要详细
#[fixture]
pub fn git_repo_with_commit() -> GitTestEnv {
    GitTestEnv::new()
        .unwrap_or_else(|e| panic!("Failed to create git test env: {}", e))
}

// 注意：Fixture 创建失败应该 panic（测试环境问题）
// 但测试逻辑中的错误仍使用 Result<()>
#[rstest]
fn test_something(git_repo_with_commit: GitTestEnv) -> Result<()> {
    // git_repo_with_commit 创建失败会 panic（这是期望的）
    // 但测试逻辑中的错误应该返回 Result
    let file = fs::read_to_string("missing.txt")?; // 使用 ?
    Ok(())
}
```

#### 测试辅助函数中的错误处理

```rust
// ✅ 推荐：返回 Result
pub fn load_fixture(name: &str) -> color_eyre::Result<String> {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);

    fs::read_to_string(&fixture_path)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to load fixture {}: {}", name, e))
}

// ❌ 不推荐：使用unwrap/expect
pub fn load_fixture(name: &str) -> String {
    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}
```

#### Option 类型处理

对于 `Option` 类型，如果确实不应该为 `None`，使用 `expect()` 也是合理的。但应该：
- 提供详细的错误信息
- 考虑是否可以返回 `Result` 让调用者处理

```rust
// ✅ 可以接受：提供详细错误信息
let parent = file_path.parent()
    .expect("File path should have a parent directory");

// ✅ 更好的方式：返回 Result
let parent = file_path.parent()
    .ok_or_else(|| color_eyre::eyre::eyre!("File path should have a parent directory"))?;
```

### 3. 边界条件测试

测试边界条件和极端情况：

```rust
#[test]
fn test_parse_ticket_id_boundary() {
    // 最小长度
    assert_eq!(parse_ticket_id("A-1"), Some("A-1"));
    // 最大长度
    assert_eq!(parse_ticket_id("VERY-LONG-PROJECT-NAME-123"), Some("VERY-LONG-PROJECT-NAME-123"));
}
```

---

## ✍️ 编写测试最佳实践

### 1. 测试命名规范

**描述性命名**：
- ✅ 使用描述性的测试名称，说明测试的内容和预期结果
- ✅ 使用 `test_` 前缀或 `#[test]` 属性
- ✅ 测试名称应包含：被测试的功能、输入条件、预期结果

```rust
// ✅ 好的命名
#[test]
fn test_parse_ticket_id_with_valid_input() {}

#[test]
fn test_parse_ticket_id_with_invalid_input_returns_none() {}

// ❌ 不好的命名
#[test]
fn test1() {}

#[test]
fn test_parse() {}
```

### 2. 测试结构（AAA 模式）

**Arrange-Act-Assert 模式**：
```rust
#[test]
fn test_example() {
    // Arrange: 准备测试数据和环境
    let input = "PROJ-123";
    let expected = Some("PROJ-123");

    // Act: 执行被测试的功能
    let result = parse_ticket_id(input);

    // Assert: 验证结果
    assert_eq!(result, expected);
}
```

### 3. 测试独立性

**每个测试应独立**：
- ✅ 每个测试应独立运行，不依赖其他测试
- ✅ 每个测试应使用独立的数据和环境
- ✅ 测试之间不应共享状态

```rust
// ✅ 好的做法：每个测试独立
#[test]
fn test_parse_ticket_id_1() {
    let result = parse_ticket_id("PROJ-123");
    assert_eq!(result, Some("PROJ-123"));
}

#[test]
fn test_parse_ticket_id_2() {
    let result = parse_ticket_id("PROJ-456");
    assert_eq!(result, Some("PROJ-456"));
}

// ❌ 不好的做法：测试之间共享状态
static mut COUNTER: i32 = 0;

#[test]
fn test_1() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);
}

#[test]
fn test_2() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 2);  // 依赖 test_1
}
```

### 4. 测试覆盖原则

**测试覆盖重点**：
- ✅ **成功路径**：测试正常流程
- ✅ **错误路径**：测试错误处理和边界条件
- ✅ **边界条件**：测试边界值和极端情况
- ✅ **集成场景**：测试模块间交互

### 5. 测试数据管理

**使用 Fixtures**：
```rust
// ✅ 使用 fixtures 目录中的测试数据
use std::fs;

#[test]
fn test_parse_pr_response() {
    let data = fs::read_to_string("tests/fixtures/sample_github_pr.json")
        .expect("Failed to read fixture");
    // 使用测试数据
}
```

**使用测试数据工厂**：
```rust
// ✅ 使用测试数据工厂生成测试数据
use tests::common::test_data_factory::TestDataFactory;

#[test]
fn test_with_factory() {
    let pr = TestDataFactory::github_pr()
        .with_id(123)
        .with_title("Test PR")
        .build();
    // 使用生成的测试数据
}
```

### 6. Mock 使用原则

**何时使用 Mock**：
- ✅ 测试需要调用外部 API（GitHub、Jira 等）
- ✅ 测试需要模拟网络请求和响应
- ✅ 测试需要避免依赖外部服务
- ✅ 测试需要模拟错误情况

**Mock 使用规范**：
```rust
// ✅ 使用 MockServer 包装器
use crate::common::http_helpers::MockServer;

#[test]
fn test_api_call() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建 Mock
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // ...

    // MockServer 会在 Drop 时自动清理环境变量
}
```

### 7. 断言最佳实践

**使用清晰的断言**：
```rust
// ✅ 使用描述性的断言消息
assert_eq!(result, expected, "Failed to parse ticket ID: {}", input);

// ✅ 使用专门的断言工具
use pretty_assertions::assert_eq;  // 显示彩色 diff

// ❌ 避免模糊的断言
assert!(result.is_some());  // 不够清晰
```

### 8. 参数化测试

参数化测试允许你使用不同的输入值运行同一个测试函数，从而减少重复代码并提高测试覆盖率。

#### 何时使用参数化测试

✅ **适合使用参数化测试的场景**：
- 多个相似测试函数（测试相同的功能，只是输入不同）
- 表格驱动测试（需要测试多种输入组合）
- 边界值测试（测试多个边界值和正常值）
- 枚举值测试（测试枚举的所有变体）

❌ **不适合使用参数化测试的场景**：
- 测试不同的错误场景（不同的错误需要不同的断言和验证逻辑）
- 需要不同设置的测试（每个测试需要不同的环境设置或fixture配置）
- 测试执行顺序重要（测试之间有依赖关系）

#### 基本用法

```rust
use rstest::rstest;

#[rstest]
#[case("input1", "expected1")]
#[case("input2", "expected2")]
#[case("input3", "expected3")]
fn test_function_with_various_inputs(
    #[case] input: &str,
    #[case] expected: &str,
) {
    let result = function_under_test(input);
    assert_eq!(result, expected);
}
```

#### 使用 `#[values]` 进行简单参数化

```rust
#[rstest]
fn test_with_multiple_values(
    #[values(1, 2, 3, 4, 5)] value: i32,
) {
    assert!(value > 0);
}
```

#### 组合 Fixture 和参数

```rust
use rstest::rstest;
use crate::common::fixtures::cli_env;

#[rstest]
fn test_cli_with_different_configs(
    cli_env: CliTestEnv,
    #[values(
        "[jira]\nurl = \"test1\"",
        "[jira]\nurl = \"test2\""
    )] config: &str,
) -> Result<()> {
    cli_env.create_config(config)?;
    // 测试代码
    Ok(())
}
```

#### 参数化测试最佳实践

**1. 测试函数命名**：
```rust
// ✅ 好的命名
#[rstest]
fn test_http_method_from_str_with_valid_methods_parses_correctly(...)

// ❌ 不好的命名
#[rstest]
fn test_http(...)
```

**2. 文档注释**：
```rust
/// 测试 HTTP 方法解析（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 HttpMethod::from_str() 能够正确解析所有有效的 HTTP 方法字符串。
///
/// ## 测试场景
/// 测试所有标准 HTTP 方法：GET, POST, PUT, DELETE, PATCH
#[rstest]
#[case("GET", HttpMethod::Get)]
// ...
```

**3. Case 注释**：
```rust
#[rstest]
#[case("hello", true)]  // 有效输入
#[case("  world  ", true)]  // 带空格的有效输入
#[case("", false)]  // 空字符串
#[case("   ", false)]  // 只有空格
fn test_validator(...)
```

**4. 保持测试独立**：
```rust
// ✅ 好的做法：每个 case 独立
#[rstest]
#[case("input1", "expected1")]
#[case("input2", "expected2")]
fn test_independent_cases(...)

// ❌ 不好的做法：case 之间有依赖
#[rstest]
#[case("input1", "expected1")]  // 这个 case 修改了全局状态
#[case("input2", "expected2")]  // 这个 case 依赖上面的状态
fn test_dependent_cases(...)
```

#### 常见模式

**验证器测试**：
```rust
#[rstest]
#[case("valid", true)]
#[case("invalid", false)]
#[case("", false)]
fn test_validator(
    #[case] input: &str,
    #[case] should_be_valid: bool,
) {
    let validator = create_validator();
    let result = validator(input);
    assert_eq!(result.is_ok(), should_be_valid);
}
```

**枚举值测试**：
```rust
#[rstest]
#[case(HttpMethod::Get, "GET")]
#[case(HttpMethod::Post, "POST")]
#[case(HttpMethod::Put, "PUT")]
fn test_enum_display(
    #[case] method: HttpMethod,
    #[case] expected: &str,
) {
    assert_eq!(format!("{}", method), expected);
}
```

### 9. 测试文档

**为复杂测试添加注释**：
```rust
#[test]
fn test_complex_scenario() {
    // 测试场景：当用户输入无效的 ticket ID 时，
    // 系统应该返回 None 并记录错误日志

    let input = "INVALID";
    let result = parse_ticket_id(input);

    assert_eq!(result, None);
    // 验证错误日志已记录
}
```

---

## 🚫 被忽略测试文档规范

对于使用 `#[ignore]` 标记的测试，必须添加完整的文档注释。

### 统一文档格式

所有被忽略的测试都应该包含以下5个部分的文档注释：

```rust
/// 测试标题（简短描述测试内容）
///
/// ## 测试目的
/// 验证/测试...（说明测试验证什么功能）
///
/// ## 为什么被忽略
/// - **主要原因**: ...
/// - **次要原因**: ...
/// - **使用场景**: ...
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_function_name -- --ignored
/// ```
/// （如适用）额外的运行说明或交互步骤
///
/// ## 测试场景
/// 1. ...
/// 2. ...
/// 3. ...
///
/// ## 预期行为
/// - ...
/// - ...
#[test]
#[ignore] // 简短原因
fn test_function_name() {
    // 测试代码
}
```

### 常见忽略原因

**1. 用户交互测试**：
- **需要用户交互**: 测试需要用户在终端中进行交互操作
- **CI环境不支持**: 自动化CI环境无法提供交互式输入

**2. 网络请求测试**：
- **需要网络连接**: 测试需要实际的网络连接到外部API
- **需要API密钥**: 需要有效的API密钥或认证凭据
- **CI成本考虑**: 避免在CI中产生API调用费用

**3. 时间相关测试**：
- **涉及真实时间延迟**: 测试需要等待实际的时间流逝
- **测试运行时间长**: 完整测试需要较长时间
- **CI时间限制**: 避免在CI中占用过多时间

**4. 修改系统配置的测试**：
- **修改系统文件**: 测试会修改用户的配置文件
- **安全风险**: 避免在CI或开发环境中意外修改配置

详细的被忽略测试规范请参考 [被忽略测试规范](./references/ignored-tests.md)。

---

## 相关文档

- [测试组织规范](./organization.md) - 测试组织结构和命名约定
- [测试命令参考](./commands.md) - 常用测试命令
- [测试工具指南](./references/tools.md) - 测试工具使用
- [被忽略测试规范](./references/ignored-tests.md) - 被忽略测试的完整规范

---

**最后更新**: 2025-01-27

