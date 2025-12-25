# 测试环境工具指南

> 本文档介绍统一测试环境工具的使用方法，包括 TestIsolation、CliTestEnv 和 GitTestEnv。

---

## 📋 目录

- [概述](#-概述)
- [TestIsolation（统一测试隔离管理器）](#1-testisolation统一测试隔离管理器)
- [CliTestEnv（CLI测试环境）](#2-clitestenvcli测试环境)
- [GitTestEnv（Git测试环境）](#3-gittestenvgit测试环境)
- [迁移指南](#4-迁移指南)
- [最佳实践](#5-最佳实践)

---

## 📋 概述

项目提供了统一的测试环境工具，基于 `TestIsolation` 构建，提供完全隔离的测试环境：

- **TestIsolation**：底层隔离管理器，提供工作目录、环境变量、Git配置和Mock服务器的隔离
- **CliTestEnv**：CLI测试环境，基于 `TestIsolation`，提供便捷的文件和配置管理
- **GitTestEnv**：Git测试环境，基于 `TestIsolation`，自动初始化Git仓库并配置测试用户

### 核心特性

- ✅ **完全隔离**：每个测试运行在独立的临时目录中，不会影响实际仓库
- ✅ **RAII模式**：自动清理资源，测试结束后自动恢复环境
- ✅ **线程安全**：支持并行测试执行
- ✅ **可配置**：支持链式调用，灵活配置隔离级别

---

## 1. TestIsolation（统一测试隔离管理器）

`TestIsolation` 是底层测试隔离管理器，提供完全隔离的测试环境。

### 功能特性

- **独立的工作目录**：自动创建临时目录并切换工作目录
- **隔离的环境变量**：使用 `EnvGuard` 自动恢复环境变量
- **独立的Git配置**：可选的 `GitConfigGuard`，隔离Git配置
- **独立的Mock服务器**：可选的 `MockServer`，用于HTTP API Mock

### 基本使用

```rust
use tests::common::TestIsolation;

#[test]
fn test_basic_isolation() -> color_eyre::Result<()> {
    let isolation = TestIsolation::new()?;

    // 测试代码在完全隔离的环境中运行
    let work_dir = isolation.work_dir();
    assert!(work_dir.exists());

    // isolation 在测试结束时自动清理
    Ok(())
}
```

### 链式调用

```rust
use tests::common::TestIsolation;

#[test]
fn test_with_git_and_mock() -> color_eyre::Result<()> {
    let mut isolation = TestIsolation::new()?
        .with_git_config()?      // 启用Git配置隔离
        .with_mock_server()?;     // 启用Mock服务器

    // 设置Git配置
    isolation.git_config_guard().unwrap().set("user.name", "Test User")?;

    // 设置Mock服务器
    let mock_server = isolation.mock_server_mut().unwrap();
    mock_server.setup_github_base_url();

    Ok(())
}
```

### API 参考

#### 创建方法

- `new() -> Result<Self>`：创建基础隔离环境
- `with_git_config() -> Result<Self>`：启用Git配置隔离
- `with_mock_server() -> Result<Self>`：启用Mock服务器

#### 访问方法

- `work_dir() -> PathBuf`：获取工作目录路径
- `env_guard() -> &mut EnvGuard`：获取环境变量守卫（用于设置环境变量）
- `git_config_guard() -> Option<&mut GitConfigGuard>`：获取Git配置守卫
- `mock_server() -> Option<&MockServer>`：获取Mock服务器引用
- `mock_server_mut() -> Option<&mut MockServer>`：获取Mock服务器可变引用

### 使用示例

#### 设置环境变量

```rust
use tests::common::TestIsolation;

#[test]
fn test_with_env_vars() -> color_eyre::Result<()> {
    let mut isolation = TestIsolation::new()?;

    // 设置环境变量
    isolation.env_guard().set("TEST_VAR", "test_value");

    // 验证环境变量已设置
    assert_eq!(std::env::var("TEST_VAR")?, "test_value");

    // 测试结束时自动恢复
    Ok(())
}
```

#### 使用Mock服务器

```rust
use tests::common::TestIsolation;

#[test]
fn test_with_mock_server() -> color_eyre::Result<()> {
    let mut isolation = TestIsolation::new()?.with_mock_server()?;

    let mock_server = isolation.mock_server_mut().unwrap();
    mock_server.setup_github_base_url();

    // 创建Mock端点
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试...

    Ok(())
}
```

---

## 2. CliTestEnv（CLI测试环境）

`CliTestEnv` 是基于 `TestIsolation` 的CLI测试环境，提供便捷的文件和配置管理。

### 功能特性

- ✅ 完全隔离的测试环境
- ✅ 支持Git仓库初始化（可选）
- ✅ 便捷的文件和配置管理
- ✅ RAII模式自动清理

### 基本使用

```rust
use tests::common::environments::CliTestEnv;

#[test]
fn test_cli_command() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;

    // 创建文件
    env.create_file("test.txt", "content")?;

    // 创建配置文件
    env.create_config(r#"[jira]
url = "https://test.atlassian.net"
"#)?;

    Ok(())
}
```

### 初始化Git仓库

```rust
use tests::common::environments::CliTestEnv;

#[test]
fn test_with_git_repo() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;

    // 初始化Git仓库
    env.init_git_repo()?;

    // 创建文件并提交
    env.create_file("test.txt", "content")?;
    env.create_commit("Initial commit")?;

    Ok(())
}
```

### API 参考

#### 创建方法

- `new() -> Result<Self>`：创建CLI测试环境

#### 文件管理

- `create_file(path: &str, content: &str) -> Result<&Self>`：创建文件
- `create_config(content: &str) -> Result<&Self>`：创建配置文件（`.workflow/workflow.toml`）

#### Git操作

- `init_git_repo() -> Result<&Self>`：初始化Git仓库
- `create_commit(message: &str) -> Result<&Self>`：创建Git提交

#### 访问方法

- `path() -> PathBuf`：获取工作目录路径
- `env_guard() -> &mut EnvGuard`：获取环境变量守卫

### 使用示例

#### 链式调用

```rust
use tests::common::environments::CliTestEnv;

#[test]
fn test_chain_calls() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?
        .init_git_repo()?
        .create_file("test.txt", "content")?
        .create_commit("Add test file")?;

    // 验证文件存在
    assert!(env.path().join("test.txt").exists());

    Ok(())
}
```

#### 设置环境变量

```rust
use tests::common::environments::CliTestEnv;

#[test]
fn test_with_env() -> color_eyre::Result<()> {
    let mut env = CliTestEnv::new()?;

    // 设置环境变量
    env.env_guard().set("HOME", "/tmp/test");

    Ok(())
}
```

---

## 3. GitTestEnv（Git测试环境）

`GitTestEnv` 是基于 `TestIsolation` 的Git测试环境，自动初始化Git仓库并配置测试用户。

### 功能特性

- ✅ 完全隔离的Git仓库（不会影响实际仓库）
- ✅ 自动初始化Git仓库（默认分支为 `main`）
- ✅ 自动配置测试用户（`Test User <test@example.com>`）
- ✅ 自动创建初始提交
- ✅ RAII模式自动清理

### 隔离性说明

**重要**：`GitTestEnv` 在临时目录中创建Git仓库，完全隔离于实际仓库：

- 测试运行在独立的临时目录中
- Git命令作用域仅限于临时目录
- 测试结束后自动清理，不会影响实际仓库
- 支持并行测试执行

### 基本使用

```rust
use tests::common::environments::GitTestEnv;

#[test]
fn test_git_operations() -> color_eyre::Result<()> {
    let env = GitTestEnv::new()?;

    // 创建分支
    env.create_branch("feature/test")?;

    // 切换分支
    env.checkout("feature/test")?;

    // 创建测试提交
    env.make_test_commit("test.txt", "content", "test commit")?;

    Ok(())
}
```

### API 参考

#### 创建方法

- `new() -> Result<Self>`：创建Git测试环境（自动初始化Git仓库）

#### 分支操作

- `create_branch(branch_name: &str) -> Result<()>`：创建新分支
- `checkout(branch_name: &str) -> Result<()>`：切换分支
- `checkout_new_branch(branch_name: &str) -> Result<()>`：创建并切换到新分支
- `current_branch() -> Result<String>`：获取当前分支名

#### 文件操作

- `create_file(filename: &str, content: &str) -> Result<()>`：创建文件
- `add_and_commit(message: &str) -> Result<()>`：添加并提交更改
- `make_test_commit(filename: &str, content: &str, message: &str) -> Result<()>`：创建文件并提交

#### Git命令

- `run_git_command(args: &[&str]) -> Result<()>`：执行Git命令

#### 访问方法

- `path() -> PathBuf`：获取仓库路径

### 使用示例

#### 分支操作

```rust
use tests::common::environments::GitTestEnv;

#[test]
fn test_branch_operations() -> color_eyre::Result<()> {
    let env = GitTestEnv::new()?;

    // 创建并切换到新分支
    env.checkout_new_branch("feature/test")?;

    // 验证当前分支
    assert_eq!(env.current_branch()?, "feature/test");

    // 创建提交
    env.make_test_commit("test.txt", "content", "Add test file")?;

    Ok(())
}
```

#### 执行自定义Git命令

```rust
use tests::common::environments::GitTestEnv;

#[test]
fn test_custom_git_command() -> color_eyre::Result<()> {
    let env = GitTestEnv::new()?;

    // 执行自定义Git命令
    env.run_git_command(&["log", "--oneline"])?;

    Ok(())
}
```

---

## 4. 迁移指南

### 从旧版 CliTestEnv 迁移

**旧版代码**：
```rust
use tests::common::CliTestEnv;

#[test]
fn test_old() {
    let env = CliTestEnv::new();
    // ...
}
```

**新版代码**：
```rust
use tests::common::environments::CliTestEnv;

#[test]
fn test_new() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    // ...
}
```

**主要变化**：
- 导入路径：`tests::common::CliTestEnv` → `tests::common::environments::CliTestEnv`
- 返回类型：`CliTestEnv` → `Result<CliTestEnv>`
- 不再需要手动使用 `CurrentDirGuard`（已内置）

### 从旧版 GitTestEnv 迁移

**旧版代码**：
```rust
use tests::common::GitTestEnv;

#[test]
fn test_old() {
    let env = GitTestEnv::new();
    // ...
}
```

**新版代码**：
```rust
use tests::common::environments::GitTestEnv;

#[test]
fn test_new() -> color_eyre::Result<()> {
    let env = GitTestEnv::new()?;
    // ...
}
```

**主要变化**：
- 导入路径：`tests::common::GitTestEnv` → `tests::common::environments::GitTestEnv`
- 返回类型：`GitTestEnv` → `Result<GitTestEnv>`
- 不再需要手动使用 `CurrentDirGuard`（已内置）
- Git仓库自动初始化，无需手动调用 `init()`

### 常见迁移问题

#### 问题1：测试函数返回类型

**错误**：
```rust
#[test]
fn test_example() {
    let env = CliTestEnv::new()?;  // 错误：? 操作符需要 Result 返回类型
}
```

**解决**：
```rust
#[test]
fn test_example() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    Ok(())
}
```

#### 问题2：手动管理当前目录

**旧版代码**：
```rust
use tests::common::helpers::CurrentDirGuard;

#[test]
fn test_old() {
    let _guard = CurrentDirGuard::new(temp_dir.path()).unwrap();
    // ...
}
```

**新版代码**：
```rust
use tests::common::environments::CliTestEnv;

#[test]
fn test_new() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;  // 自动管理当前目录
    // ...
}
```

---

## 5. 最佳实践

### 1. 选择合适的测试环境

- **CLI测试**：使用 `CliTestEnv`
- **Git操作测试**：使用 `GitTestEnv`
- **底层隔离需求**：直接使用 `TestIsolation`

### 2. 使用链式调用

```rust
// ✅ 推荐：链式调用，代码简洁
let env = CliTestEnv::new()?
    .init_git_repo()?
    .create_file("test.txt", "content")?
    .create_commit("Add test file")?;

// ❌ 不推荐：多次调用，代码冗长
let env = CliTestEnv::new()?;
env.init_git_repo()?;
env.create_file("test.txt", "content")?;
env.create_commit("Add test file")?;
```

### 3. 利用RAII自动清理

```rust
// ✅ 推荐：依赖RAII自动清理
#[test]
fn test_example() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    // 测试代码
    Ok(())  // env 自动清理
}

// ❌ 不推荐：手动清理（不需要）
#[test]
fn test_example() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    // 测试代码
    drop(env);  // 不需要，RAII会自动处理
    Ok(())
}
```

### 4. 测试隔离性

```rust
// ✅ 推荐：每个测试独立环境
#[test]
fn test_1() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    // ...
}

#[test]
fn test_2() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;  // 独立环境
    // ...
}

// ❌ 不推荐：共享环境（可能导致测试相互影响）
static mut ENV: Option<CliTestEnv> = None;
```

### 5. 错误处理

```rust
// ✅ 推荐：使用 ? 操作符和 Result 返回类型
#[test]
fn test_example() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    env.create_file("test.txt", "content")?;
    Ok(())
}

// ❌ 不推荐：使用 unwrap()（测试失败时信息不清晰）
#[test]
fn test_example() {
    let env = CliTestEnv::new().unwrap();
    env.create_file("test.txt", "content").unwrap();
}
```

---

## 相关文档

- [测试工具指南](./tools.md) - 其他测试工具
- [Mock服务器使用指南](./mock-server.md) - Mock服务器详细使用方法
- [测试编写规范](../writing.md) - 测试编写规范
- [测试组织规范](../organization.md) - 测试组织结构

---

**最后更新**: 2025-12-25

