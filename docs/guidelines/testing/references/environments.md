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

### 迁移到使用标准 Fixture

#### 迁移原则

**何时应该迁移**：

✅ **适合迁移**:
- 测试只需要基础环境（不需要特殊配置）
- 测试可以并行执行
- 测试重复创建相同的环境

❌ **不适合迁移**:
- 测试需要特殊的环境配置
- 测试需要动态创建不同的环境
- 测试需要访问环境变量的可变引用

#### 迁移优先级

1. **高优先级**: 高频使用的测试文件（使用 `CliTestEnv::new()` 或 `GitTestEnv::new()` 超过 5 次）
2. **中优先级**: 中等频率使用的测试文件（3-5 次）
3. **低优先级**: 低频使用的测试文件（1-2 次）

#### 迁移步骤

**步骤 1: 添加必要的导入**

```rust
// 添加 rstest 导入
use rstest::rstest;

// 导入需要的 Fixture
use crate::common::fixtures::{cli_env, cli_env_with_git, git_repo_with_commit};
```

**步骤 2: 将 `#[test]` 改为 `#[rstest]`**

```rust
// 之前
#[test]
fn test_something() -> Result<()> {
    let env = CliTestEnv::new()?;
    // ...
}

// 之后
#[rstest]
fn test_something(cli_env: CliTestEnv) -> Result<()> {
    // cli_env 已经创建好了
    // ...
}
```

**步骤 3: 移除手动创建环境的代码**

```rust
// 之前
let env = CliTestEnv::new()?;
env.init_git_repo()?;

// 之后（使用 cli_env_with_git fixture）
// 不需要手动创建，fixture 已经初始化了 Git 仓库
```

**步骤 4: 更新函数签名**

```rust
// 之前
fn test_something() -> Result<()> {
    // ...
}

// 之后
fn test_something(cli_env: CliTestEnv) -> Result<()> {
    // ...
}
```

#### 迁移示例

**示例 1: 基础 CLI 环境测试**

**之前**:
```rust
#[test]
fn test_path_exists() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("test.txt");
    fs::write(&file_path, "test")?;

    let access = PathAccess::new(&file_path);
    assert!(access.exists());

    Ok(())
}
```

**之后**:
```rust
use rstest::rstest;
use crate::common::fixtures::cli_env;

#[rstest]
fn test_path_exists(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    let file_path = cli_env.path().join("test.txt");
    fs::write(&file_path, "test")?;

    let access = PathAccess::new(&file_path);
    assert!(access.exists());

    Ok(())
}
```

**优势**:
- ✅ 减少代码行数
- ✅ 环境自动创建和清理
- ✅ 支持并行执行

**示例 2: Git 仓库测试**

**之前**:
```rust
#[test]
fn test_branch_exists() -> Result<()> {
    let _env = GitTestEnv::new()?;

    let current_branch = GitBranch::current_branch()?;
    let exists = GitBranch::has_local_branch(&current_branch).unwrap_or(false);

    assert!(exists);
    Ok(())
}
```

**之后**:
```rust
use rstest::rstest;
use crate::common::fixtures::git_repo_with_commit;

#[rstest]
fn test_branch_exists(git_repo_with_commit: GitTestEnv) -> Result<()> {
    let current_branch = GitBranch::current_branch()?;
    let exists = GitBranch::has_local_branch(&current_branch).unwrap_or(false);

    assert!(exists);
    Ok(())
}
```

**优势**:
- ✅ Git 仓库已初始化
- ✅ 已有初始提交
- ✅ 代码更简洁

**示例 3: CLI 环境 + Git 仓库测试**

**之前**:
```rust
#[test]
fn test_check_has_last_commit() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    env.init_git_repo()?
        .create_file("test.txt", "test")?
        .create_commit("Initial commit")?;

    let _guard = CurrentDirGuard::new(env.path())?;
    let result = check_has_last_commit();

    assert!(result.is_ok());
    Ok(())
}
```

**之后**:
```rust
use rstest::rstest;
use crate::common::fixtures::cli_env_with_git;

#[rstest]
fn test_check_has_last_commit(cli_env_with_git: CliTestEnv) -> color_eyre::Result<()> {
    // cli_env_with_git 已经初始化了 Git 仓库
    // 但还没有 commit，需要手动创建
    cli_env_with_git
        .create_file("test.txt", "test")?
        .create_commit("Initial commit")?;

    let _guard = CurrentDirGuard::new(cli_env_with_git.path())?;
    let result = check_has_last_commit();

    assert!(result.is_ok());
    Ok(())
}
```

#### 常见场景

**场景 1: 需要空 Git 仓库（无 commit）**

**解决方案**: 使用 `cli_env_with_git` fixture，它只初始化 Git 仓库，不创建 commit。

```rust
#[rstest]
fn test_empty_git_repo(cli_env_with_git: CliTestEnv) -> Result<()> {
    // Git 仓库已初始化，但没有 commit
    // 可以直接测试空仓库场景
    Ok(())
}
```

**场景 2: 需要带 commit 的 Git 仓库**

**解决方案**: 使用 `git_repo_with_commit` fixture。

```rust
#[rstest]
fn test_with_commits(git_repo_with_commit: GitTestEnv) -> Result<()> {
    // Git 仓库已初始化，且有初始 commit
    // 可以直接测试有 commit 的场景
    Ok(())
}
```

**场景 3: 需要多个环境**

**解决方案**: 可以在一个测试中使用多个 fixture。

```rust
#[rstest]
fn test_multiple_envs(
    cli_env: CliTestEnv,
    git_repo_with_commit: GitTestEnv,
) -> Result<()> {
    // 可以使用多个环境
    Ok(())
}
```

**场景 4: 需要特殊配置**

**解决方案**: 如果 fixture 不满足需求，可以继续使用手动创建，或创建自定义 fixture。

```rust
// 选项 1: 继续手动创建（如果配置复杂）
#[test]
fn test_special_config() -> Result<()> {
    let mut env = CliTestEnv::new()?;
    env.env_guard().set("SPECIAL_VAR", "value");
    // ...
}

// 选项 2: 创建自定义 fixture（如果经常使用）
#[fixture]
fn cli_env_with_special_config() -> CliTestEnv {
    let mut env = CliTestEnv::new().expect("Failed to create env");
    env.env_guard().set("SPECIAL_VAR", "value");
    env
}
```

#### 迁移注意事项

**1. 错误处理**

Fixture 使用 `expect()`:
- Fixture 创建失败应该 panic（测试环境问题）
- 测试逻辑中的错误仍使用 `Result<()>`

```rust
#[rstest]
fn test_something(cli_env: CliTestEnv) -> Result<()> {
    // cli_env 创建失败会 panic（这是期望的）
    // 但测试逻辑中的错误应该返回 Result
    let file = fs::read_to_string("missing.txt")?; // 使用 ?
    Ok(())
}
```

**2. 并行执行**

Fixture 支持并行执行:
- 所有标准 Fixture 都使用隔离的环境
- 可以安全地并行执行

```rust
// 这些测试可以并行执行
#[rstest]
fn test_1(cli_env: CliTestEnv) -> Result<()> { Ok(()) }

#[rstest]
fn test_2(cli_env: CliTestEnv) -> Result<()> { Ok(()) }
```

**3. 环境变量访问**

如果需要设置环境变量:
- 使用 `env.env_guard().set()` 方法
- 注意需要可变引用

```rust
#[rstest]
fn test_with_env(mut cli_env: CliTestEnv) -> Result<()> {
    cli_env.env_guard().set("TEST_VAR", "value");
    // ...
    Ok(())
}
```

**4. 工作目录切换**

如果需要切换工作目录:
- 使用 `CurrentDirGuard`（如果测试需要）
- 或者使用绝对路径（推荐）

```rust
#[rstest]
fn test_with_dir_switch(cli_env: CliTestEnv) -> Result<()> {
    let _guard = CurrentDirGuard::new(cli_env.path())?;
    // 当前工作目录已切换到 cli_env.path()
    Ok(())
}
```

#### 迁移检查清单

迁移测试时，请检查：

- [ ] 添加了 `use rstest::rstest;`
- [ ] 导入了需要的 Fixture
- [ ] 将 `#[test]` 改为 `#[rstest]`
- [ ] 移除了手动创建环境的代码
- [ ] 更新了函数签名（添加 fixture 参数）
- [ ] 测试仍然通过
- [ ] 代码更简洁

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

### 6. 测试隔离性

#### 文件锁保护

避免并发测试写入同一个配置文件：

```rust
// ✅ 生产代码已实现文件锁保护
// 测试代码无需额外处理，但应了解原理

// 文件锁确保并发安全
let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(&config_path)?;
file.lock_exclusive()?;
// ... 读写操作 ...
file.unlock()?;
```

#### 环境变量隔离

每个测试应使用独立的环境变量：

```rust
// ✅ 推荐：每个测试独立环境
#[test]
fn test_1() -> color_eyre::Result<()> {
    let mut env = CliTestEnv::new()?;
    env.env_guard().set("TEST_VAR", "value1");
    // ...
    Ok(())
}

#[test]
fn test_2() -> color_eyre::Result<()> {
    let mut env = CliTestEnv::new()?;
    env.env_guard().set("TEST_VAR", "value2");  // 独立环境
    // ...
    Ok(())
}

// ❌ 不推荐：共享环境变量
static mut TEST_VAR: Option<String> = None;
```

#### 测试隔离性检查清单

- [ ] 每个测试使用独立的测试环境（`CliTestEnv` 或 `GitTestEnv`）
- [ ] 测试之间不共享状态（全局变量、静态变量等）
- [ ] 测试可以并行运行（使用 `cargo test -- --test-threads=4`）
- [ ] 测试结束后自动清理资源（依赖RAII）
- [ ] 文件操作使用文件锁保护（生产代码）

#### 并发测试建议

**对于 CI/CD**：
- 可以使用 `--test-threads=1` 确保稳定性（如果遇到并发问题）
- 大多数情况下，并行运行是安全的

**对于本地开发**：
- 可以并行运行以加快速度（默认行为）
- 如果遇到并发问题，检查测试隔离性

---

## 相关文档

- [测试工具指南](./tools.md) - 其他测试工具
- [Mock服务器使用指南](./mock-server.md) - Mock服务器详细使用方法
- [测试编写规范](../writing.md) - 测试编写规范
- [测试组织规范](../organization.md) - 测试组织结构

---

**最后更新**: 2025-01-27

