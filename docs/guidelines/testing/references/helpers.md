# 测试辅助工具指南

> 本文档介绍测试辅助工具的使用方法，包括 CliCommandBuilder 和 TestDataGenerator。

---

## 📋 目录

- [概述](#-概述)
- [CliCommandBuilder（CLI命令构建器）](#1-clicommandbuildercli命令构建器)
- [TestDataGenerator（测试数据生成器）](#2-testdatagenerator测试数据生成器)
- [路径获取函数](#3-路径获取函数)
- [最佳实践](#4-最佳实践)

---

## 📋 概述

测试辅助工具提供便捷的测试辅助功能，简化测试代码编写：

- **CliCommandBuilder**：简化CLI命令测试的构建器，提供流畅的API
- **TestDataGenerator**：生成测试用的数据，减少样板代码
- **路径获取函数**：统一的路径获取函数，使用 `dirs` crate 并支持测试环境隔离

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

## 3. 路径获取函数

测试中使用统一的路径获取函数，这些函数使用 `dirs` crate 并支持测试环境隔离。

### 功能特性

- ✅ **统一性**：所有路径获取使用统一的方式
- ✅ **跨平台兼容性**：自动处理不同平台的路径差异
- ✅ **测试环境隔离**：优先使用环境变量（支持测试隔离），然后回退到 `dirs` crate
- ✅ **与源代码一致**：与源代码中的 `Paths::home_dir()` 行为一致

### 可用的路径获取函数

- **`test_home_dir()`** - 获取主目录（测试环境感知）
- **`test_config_dir()`** - 获取配置目录（测试环境感知）
- **`test_data_dir()`** - 获取数据目录（测试环境感知）
- **`test_cache_dir()`** - 获取缓存目录（测试环境感知）

### 基本使用

```rust
use tests::common::helpers::{test_home_dir, test_config_dir};
use tests::common::guards::EnvGuard;

#[test]
fn test_example() -> color_eyre::Result<()> {
    let mut guard = EnvGuard::new();
    guard.set("HOME", "/test/isolated/home");

    // 使用统一的路径获取函数
    let home = test_home_dir()?;
    let config_dir = test_config_dir()?;

    // 验证路径正确
    assert_eq!(home, PathBuf::from("/test/isolated/home"));
    assert!(config_dir.to_string_lossy().contains(".workflow"));

    Ok(())
}
```

### API 参考

#### test_home_dir()

获取主目录（测试环境感知）。

**行为**：
- 优先使用环境变量（`HOME`/`USERPROFILE`），支持测试隔离
- 回退到 `dirs::home_dir()`
- 与源代码中的 `Paths::home_dir()` 行为一致

**返回**：`color_eyre::Result<PathBuf>`

**使用示例**：
```rust
use tests::common::helpers::test_home_dir;

#[test]
fn test_home_dir() -> color_eyre::Result<()> {
    let mut guard = EnvGuard::new();
    guard.set("HOME", "/test/home");

    let home = test_home_dir()?;
    assert_eq!(home, PathBuf::from("/test/home"));
    Ok(())
}
```

#### test_config_dir()

获取配置目录（测试环境感知）。

**行为**：
- 优先使用 `WORKFLOW_CONFIG_DIR` 环境变量
- 否则使用标准配置目录（`~/.workflow/config`）

**返回**：`color_eyre::Result<PathBuf>`

**使用示例**：
```rust
use tests::common::helpers::test_config_dir;

#[test]
fn test_config_dir() -> color_eyre::Result<()> {
    let config_dir = test_config_dir()?;
    assert!(config_dir.to_string_lossy().contains(".workflow"));
    assert!(config_dir.to_string_lossy().contains("config"));
    Ok(())
}
```

#### test_data_dir()

获取数据目录（测试环境感知）。

**平台差异**：
- **Windows**: `%LOCALAPPDATA%` 或 `dirs::data_local_dir()`
- **Unix**: `$XDG_DATA_HOME` 或 `~/.local/share`

**返回**：`color_eyre::Result<PathBuf>`

#### test_cache_dir()

获取缓存目录（测试环境感知）。

**平台差异**：
- **Unix**: `$XDG_CACHE_HOME` 或 `~/.cache`
- **Windows**: `%LOCALAPPDATA%` 下的缓存目录

**返回**：`color_eyre::Result<PathBuf>`

### 使用示例

#### 基本路径获取

```rust
use tests::common::helpers::{test_home_dir, test_config_dir};

#[test]
fn test_basic_paths() -> color_eyre::Result<()> {
    let home = test_home_dir()?;
    let config_dir = test_config_dir()?;

    // 使用路径进行测试
    assert!(home.exists() || !home.exists()); // 主目录可能不存在但路径有效
    assert!(config_dir.to_string_lossy().contains(".workflow"));

    Ok(())
}
```

#### 结合测试环境隔离使用

```rust
use tests::common::helpers::test_home_dir;
use tests::common::guards::EnvGuard;

#[test]
fn test_with_isolation() -> color_eyre::Result<()> {
    let mut guard = EnvGuard::new();
    guard.set("HOME", "/test/isolated/home");

    // test_home_dir() 会返回测试隔离的路径
    let home = test_home_dir()?;
    assert_eq!(home, PathBuf::from("/test/isolated/home"));

    // guard drop 后，环境变量自动恢复
    Ok(())
}
```

#### 获取配置文件路径

```rust
use tests::common::helpers::test_config_dir;

#[test]
fn test_config_file_path() -> color_eyre::Result<()> {
    let config_dir = test_config_dir()?;
    let config_file = config_dir.join("workflow.toml");

    // 使用配置文件路径进行测试
    // ...

    Ok(())
}
```

### 注意事项

#### 1. 测试环境隔离

这些函数优先使用环境变量（支持测试隔离），然后回退到 `dirs` crate：

```rust
// ✅ 正确：使用 EnvGuard 设置环境变量
let mut guard = EnvGuard::new();
guard.set("HOME", "/test/home");
let home = test_home_dir()?; // 返回 "/test/home"

// ❌ 错误：直接使用 dirs::home_dir()，不支持测试隔离
let home = dirs::home_dir().unwrap(); // 返回真实系统路径
```

#### 2. 与源代码的一致性

这些函数的行为与源代码中的 `Paths::home_dir()` 一致：

```rust
// 源代码中的实现
pub(crate) fn home_dir() -> Result<PathBuf> {
    // 优先检查环境变量
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home));
    }
    // 回退到 dirs::home_dir()
    dirs::home_dir().wrap_err("Cannot determine home directory")
}

// 测试中的实现（行为一致）
pub fn test_home_dir() -> color_eyre::Result<PathBuf> {
    // 优先检查环境变量（与源代码一致）
    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home));
    }
    // 回退到 dirs::home_dir()（与源代码一致）
    dirs::home_dir().ok_or_else(|| {
        color_eyre::eyre::eyre!("Cannot determine home directory")
    })
}
```

#### 3. 不应该使用这些函数的场景

**临时目录**：
```rust
// ✅ 正确：临时目录应继续使用 std::env::temp_dir() 或 tempfile
let temp_dir = std::env::temp_dir();
let temp_file = tempfile::tempdir()?;
```

**当前目录**：
```rust
// ✅ 正确：当前目录应继续使用 std::env::current_dir()
let current_dir = std::env::current_dir()?;
```

**测试基础设施路径**：
```rust
// ✅ 正确：CliTestEnv 和 TestIsolation 创建的路径不需要使用这些函数
let env = CliTestEnv::new()?;
let project_path = env.project_path(); // 使用 CliTestEnv 提供的方法
```

#### 4. 特殊情况：需要真实系统路径

如果测试需要访问**真实的系统路径**（而不是测试隔离路径），应该继续使用 `dirs::home_dir()`，并添加注释说明原因：

```rust
// 注意：这里使用 dirs::home_dir() 而不是 test_home_dir()，
// 因为此函数的目的是从真实的全局 Git 配置复制到测试隔离的配置中。
// 即使测试环境设置了 HOME 环境变量，我们也需要访问真实的系统主目录。
let global_config = dirs::home_dir()
    .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get home directory"))?
    .join(".gitconfig");
```

### 迁移指南

#### 迁移前

```rust
// 直接使用 dirs::home_dir()
let global_config = dirs::home_dir()
    .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get home directory"))?
    .join(".gitconfig");
```

#### 迁移后（如果不需要真实系统路径）

```rust
use tests::common::helpers::test_home_dir;

let global_config = test_home_dir()?
    .join(".gitconfig");
```

#### 迁移后（如果需要真实系统路径）

```rust
// 注意：这里使用 dirs::home_dir() 而不是 test_home_dir()，
// 因为需要访问真实的系统路径
let global_config = dirs::home_dir()
    .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get home directory"))?
    .join(".gitconfig");
```

---

## 4. 最佳实践

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

**最后更新**: 2025-12-28

