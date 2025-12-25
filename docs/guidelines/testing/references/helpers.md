# 测试辅助工具指南

> 本文档介绍测试辅助工具的使用方法，包括 CliCommandBuilder 和 TestDataGenerator。

---

## 📋 目录

- [概述](#-概述)
- [CliCommandBuilder（CLI命令构建器）](#1-clicommandbuildercli命令构建器)
- [TestDataGenerator（测试数据生成器）](#2-testdatagenerator测试数据生成器)
- [最佳实践](#3-最佳实践)

---

## 📋 概述

测试辅助工具提供便捷的测试辅助功能，简化测试代码编写：

- **CliCommandBuilder**：简化CLI命令测试的构建器，提供流畅的API
- **TestDataGenerator**：生成测试用的数据，减少样板代码

---

## 1. CliCommandBuilder（CLI命令构建器）

`CliCommandBuilder` 是CLI命令测试的构建器，简化CLI命令的执行和断言。

### 功能特性

- ✅ 流畅的API：链式调用，代码可读性强
- ✅ 类型安全：编译时检查，避免运行时错误
- ✅ 便捷的断言：提供 `assert_success()`、`assert_failure()` 等方法

### 基本使用

```rust
use tests::common::CliCommandBuilder;

#[test]
fn test_cli_command() {
    CliCommandBuilder::new()
        .arg("config")
        .arg("show")
        .assert_success();
}
```

### API 参考

#### 创建方法

- `new() -> Self`：创建新的命令构建器

#### 配置方法

- `arg<S: AsRef<OsStr>>(arg: S) -> Self`：添加单个参数
- `args<I, S>(args: I) -> Self`：添加多个参数
- `env<K, V>(key: K, val: V) -> Self`：设置环境变量
- `current_dir<P: AsRef<Path>>(dir: P) -> Self`：设置工作目录

#### 断言方法

- `assert_success() -> Assert`：执行命令并断言成功
- `assert_failure() -> Assert`：执行命令并断言失败
- `assert() -> Assert`：执行命令并返回断言对象

### 使用示例

#### 基本命令测试

```rust
use tests::common::CliCommandBuilder;

#[test]
fn test_config_show() {
    CliCommandBuilder::new()
        .arg("config")
        .arg("show")
        .assert_success();
}
```

#### 带参数的命令测试

```rust
use tests::common::CliCommandBuilder;

#[test]
fn test_with_args() {
    CliCommandBuilder::new()
        .args(&["config", "set", "jira.url", "https://test.atlassian.net"])
        .assert_success();
}
```

#### 设置环境变量

```rust
use tests::common::CliCommandBuilder;

#[test]
fn test_with_env() {
    CliCommandBuilder::new()
        .arg("config")
        .arg("show")
        .env("HOME", "/tmp/test")
        .assert_success();
}
```

#### 设置工作目录

```rust
use tests::common::CliCommandBuilder;
use tests::common::environments::CliTestEnv;

#[test]
fn test_with_work_dir() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;

    CliCommandBuilder::new()
        .arg("config")
        .arg("show")
        .current_dir(env.path())
        .assert_success();

    Ok(())
}
```

#### 验证输出

```rust
use tests::common::CliCommandBuilder;

#[test]
fn test_output() {
    let assert = CliCommandBuilder::new()
        .arg("config")
        .arg("show")
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("jira"));
}
```

#### 验证错误输出

```rust
use tests::common::CliCommandBuilder;

#[test]
fn test_error_output() {
    CliCommandBuilder::new()
        .arg("invalid")
        .arg("command")
        .assert_failure()
        .stderr(predicates::str::contains("error"));
}
```

---

## 2. TestDataGenerator（测试数据生成器）

`TestDataGenerator` 提供测试数据的生成方法，减少样板代码。

### 功能特性

- ✅ 提供常用测试数据模板
- ✅ 易于扩展：可以添加新的数据生成方法

### 基本使用

```rust
use tests::common::TestDataGenerator;

#[test]
fn test_config_generation() {
    let config_content = TestDataGenerator::config_content();

    // 使用生成的配置内容
    assert!(config_content.contains("jira"));
    assert!(config_content.contains("github"));
}
```

### API 参考

#### 数据生成方法

- `config_content() -> String`：生成测试用的配置内容

### 使用示例

#### 生成配置文件

```rust
use tests::common::TestDataGenerator;
use tests::common::environments::CliTestEnv;

#[test]
fn test_with_generated_config() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;

    // 生成配置内容
    let config_content = TestDataGenerator::config_content();

    // 创建配置文件
    env.create_config(&config_content)?;

    Ok(())
}
```

### 扩展指南

如果需要添加新的数据生成方法，可以在 `TestDataGenerator` 实现中添加：

```rust
impl TestDataGenerator {
    /// 生成新的测试数据
    pub fn new_data_type() -> String {
        // 返回生成的测试数据
        r#"
        {
            "field1": "value1",
            "field2": "value2"
        }
        "#
        .to_string()
    }
}
```

---

## 3. 最佳实践

### 1. 使用 CliCommandBuilder 简化命令测试

```rust
// ✅ 推荐：使用 CliCommandBuilder
#[test]
fn test_command() {
    CliCommandBuilder::new()
        .arg("config")
        .arg("show")
        .assert_success();
}

// ❌ 不推荐：直接使用 assert_cmd::Command
#[test]
fn test_command() {
    let mut cmd = assert_cmd::Command::new("workflow");
    cmd.args(&["config", "show"]);
    cmd.assert().success();
}
```

### 2. 结合测试环境使用

```rust
// ✅ 推荐：结合 CliTestEnv 使用
#[test]
fn test_with_env() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;

    CliCommandBuilder::new()
        .arg("config")
        .arg("show")
        .current_dir(env.path())
        .assert_success();

    Ok(())
}
```

### 3. 使用 TestDataGenerator 减少样板代码

```rust
// ✅ 推荐：使用 TestDataGenerator
#[test]
fn test_with_generated_data() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    let config_content = TestDataGenerator::config_content();
    env.create_config(&config_content)?;
    Ok(())
}

// ❌ 不推荐：硬编码测试数据
#[test]
fn test_with_hardcoded_data() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    env.create_config(r#"[jira]
url = "https://test.atlassian.net"
username = "test@example.com"

[github]
token = "test_token"
"#)?;
    Ok(())
}
```

### 4. 验证输出内容

```rust
// ✅ 推荐：验证输出内容
#[test]
fn test_output_verification() {
    let assert = CliCommandBuilder::new()
        .arg("config")
        .arg("show")
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("jira"))
        .stdout(predicates::str::contains("github"));
}
```

---

## 相关文档

- [测试环境工具指南](./environments.md) - 测试环境工具详细使用方法
- [测试工具指南](./tools.md) - 其他测试工具
- [测试编写规范](../writing.md) - 测试编写规范

---

**最后更新**: 2025-12-25

