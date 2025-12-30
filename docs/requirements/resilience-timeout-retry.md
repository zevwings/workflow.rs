# Resilience (Timeout & Retry) 需求文档

## 概述

本文档定义了在代码库中使用 `src/lib/base/resilience` 模块的 `timeout` 和 `retry` 机制的需求。

## 目标

1. **统一超时和重试机制**：所有可能卡住或失败的操作都应使用统一的超时和重试机制
2. **提高可靠性**：防止操作无限期卡住，自动重试临时性错误
3. **改善用户体验**：提供清晰的错误信息和重试反馈
4. **平台兼容性**：针对不同平台（Windows、macOS、Linux）提供适当的超时和重试配置

## 原则

### 使用场景分类

1. **HTTP 请求** → 使用 `HttpRetry::retry()`（专门为 HTTP 设计）
   - 适用于所有 HTTP API 调用（GitHub API、Jira API、LLM API）
   - 原因：网络错误、5xx 错误、429 限流可重试
   - 超时：已由 `HttpClient` 提供（默认 30 秒）

2. **命令层非 HTTP 操作** → 使用 `execute_with_timeout_and_retry()`（通用超时+重试）
   - 文件系统操作（文件读写、目录遍历、文件锁）
   - 进程执行操作（脚本执行、命令执行）
   - Git 操作（非网络，如 commit、merge、rebase）
   - Zip/Tar 解压操作

3. **仅超时保护** → 使用 `execute_with_timeout()`（不需要重试）
   - 某些操作只需要超时保护，不需要重试（如 Git 操作）

## 当前状态

### ✅ 已移除的调用

以下位置已移除 `execute_with_timeout`、`execute_with_timeout_and_retry`、`execute_with_retry` 的调用：

1. **`src/lib/repo/config/private.rs`** - 配置文件写入
2. **`src/commands/lifecycle/update.rs`** - 文件下载、解压、脚本执行、清理操作
3. **`tests/common/guards/git_config_guard.rs`** - Git 配置设置
4. **`tests/common/environments/git_test_env.rs`** - Git 测试环境中的 `repo.remote()` 操作
5. **`tests/common/environments/cli_test_env.rs`** - CLI 测试环境中的 `repo.remote()` 操作

### 📋 需要添加的位置

根据 `analysis/resilience_usage_analysis.md` 的分析，以下位置需要添加超时和重试机制：

## 详细需求

### 1. 文件系统操作（高优先级）

#### 1.1 `src/lib/base/fs/directory.rs`

**需求**：为以下方法添加超时和重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `list_dirs()` | 递归遍历目录（`WalkDir`） | 大目录可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `list_files()` | 递归遍历文件（`WalkDir`） | 大目录可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `find_files()` | 递归查找文件（`WalkDir`） | 大目录可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `list_direct_dirs()` | 读取目录条目（`read_dir_safe()`） | Windows 上可能卡住 | `execute_with_timeout_and_retry()` + `default_filesystem_timeout()` |
| `list_direct_files()` | 读取目录条目（`read_dir_safe()`） | Windows 上可能卡住 | `execute_with_timeout_and_retry()` + `default_filesystem_timeout()` |
| `ensure_exists()` | 创建目录（`fs::create_dir_all()`） | Windows 上可能失败（文件锁定） | `execute_with_timeout_and_retry()` + `default_filesystem_timeout()` |

**优先级**：高
**原因**：Windows 上容易卡住或失败，影响用户体验

#### 1.2 `src/lib/base/fs/path.rs`

**需求**：为 `read_dir_safe()` 方法添加超时和重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `read_dir_safe()` | 读取目录条目（`fs::read_dir()`） | Windows 上可能卡住 | `execute_with_timeout_and_retry()` + `default_filesystem_timeout()` |

**优先级**：高
**原因**：Windows 上容易卡住

#### 1.3 `src/lib/base/fs/file.rs`

**需求**：为以下方法添加超时和重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `FileReader::open()` | 打开文件（`File::open()`） | Windows 上可能卡住（文件锁定） | `execute_with_timeout_and_retry()` + `default_filesystem_timeout()` |
| `FileReader::to_string()` | 读取文件内容（`fs::read_to_string()`） | 大文件可能卡住，Windows 上可能失败 | `execute_with_timeout_and_retry()` + `default_filesystem_timeout()` |
| `FileReader::lines()` | 读取文件行（`BufReader::lines()`） | 大文件可能卡住，Windows 上可能失败 | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `FileReader::bytes()` | 读取文件字节（`read_to_end()`） | 大文件可能卡住，Windows 上可能失败 | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `FileWriter::write()` | 写入文件（`fs::write()`） | Windows 上可能失败（文件锁定） | `execute_with_timeout_and_retry()` + `default_filesystem_timeout()` |
| `FileWriter::write_bytes()` | 写入文件字节（`fs::write()`） | Windows 上可能失败（文件锁定） | `execute_with_timeout_and_retry()` + `default_filesystem_timeout()` |

**优先级**：高
**原因**：文件操作是核心功能，Windows 上容易失败

#### 1.4 `src/lib/jira/attachments/clean.rs`

**需求**：为以下方法添加超时保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `calculate_directory_size()` | 递归遍历目录计算大小（`WalkDir`） | 大目录可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `list_directory_contents()` | 递归列出目录内容（`WalkDir`） | 大目录可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |

**优先级**：中
**原因**：主要用于清理操作，频率较低

---

### 2. 进程执行操作（中优先级）

#### 2.1 `src/lib/git/pre_commit.rs`

**需求**：为 `run_pre_commit()` 方法添加超时和重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `run_pre_commit()` | 执行 pre-commit hooks（`Command::new().output()`） | hooks 可能卡住或执行时间很长 | `execute_with_timeout_and_retry()` + `default_script_timeout()` |

**优先级**：中
**原因**：hooks 可能卡住，但频率较低
**注意**：已有手动重试逻辑（line 100-160），应该统一使用 `execute_with_timeout_and_retry()`

#### 2.2 `src/lib/rollback/rollback.rs`

**需求**：为 `create_backup()` 方法添加重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `create_backup()` | 设置文件执行权限（`chmod`） | 可能失败（权限问题） | `execute_with_retry()`（不需要超时，操作很快） |

**优先级**：低
**原因**：操作很快，只需要重试

#### 2.3 `src/commands/lifecycle/install.rs`

**需求**：为 `install_binary()` 方法添加超时和重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `install_binary()` | 复制文件和设置权限（`sudo cp`, `sudo chmod`） | 可能失败（权限问题、文件锁定） | `execute_with_timeout_and_retry()` + `default_filesystem_timeout()` |

**优先级**：中
**原因**：安装操作需要可靠性

#### 2.4 `src/lib/base/system/platform.rs`

**需求**：为 `detect_libc_type()` 方法添加超时保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `detect_libc_type()` | 执行 `ldd` 命令检测链接类型 | 可能卡住或失败 | `execute_with_timeout()` + `default_script_timeout()` |

**优先级**：低
**原因**：主要用于检测，频率很低

---

### 3. Git 操作（非网络）（中优先级）

#### 3.1 `src/lib/git/client/repository.rs`

**需求**：为以下方法添加超时保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `open()` | 打开 Git 仓库（`Repository::open()`） | Windows 上可能卡住（触发 DNS 解析） | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `open_at()` | 打开指定路径的 Git 仓库（`Repository::open()`） | Windows 上可能卡住（触发 DNS 解析） | `execute_with_timeout()` + `default_filesystem_timeout()` |

**优先级**：中
**原因**：Windows 上可能卡住，但频率较低
**注意**：测试中已有超时保护，生产代码中可能需要

#### 3.2 `src/lib/git/cherry_pick.rs`

**需求**：为以下方法添加超时保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `cherry_pick()` | 执行 cherry-pick（`merge_trees()`, `commit()`） | 大文件或复杂合并可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `cherry_pick_no_commit()` | 执行 cherry-pick（不提交） | 大文件或复杂合并可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `cherry_pick_continue()` | 继续 cherry-pick（`commit()`） | 可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |

**优先级**：中
**原因**：复杂操作可能卡住，但频率较低

#### 3.3 `src/lib/git/branch.rs`

**需求**：为以下方法添加超时保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `rebase_onto()` | 执行 rebase（`rebase()`, `rebase.next()`, `rebase.commit()`） | 复杂 rebase 可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |
| `rebase_range()` | 执行范围 rebase（`rebase()`, `rebase.next()`, `rebase.commit()`） | 复杂 rebase 可能卡住 | `execute_with_timeout()` + `default_filesystem_timeout()` |

**优先级**：中
**原因**：复杂操作可能卡住，但频率较低

---

### 4. Zip/Tar 解压操作（高优先级）

#### 4.1 `src/lib/base/zip/zip_impl.rs`

**需求**：为 `extract_zip()` 方法添加超时和重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `extract_zip()` | 解压 zip 文件 | 大文件可能卡住 | `execute_with_timeout_and_retry()` + `default_extract_timeout()` |

**优先级**：高
**原因**：大文件可能卡住，用户等待时间长
**注意**：已在 `update.rs:519` 中使用，但这里是底层实现，可能需要单独保护

#### 4.2 `src/lib/base/zip/tar.rs`

**需求**：为 `extract_tar_gz()` 方法添加超时和重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `extract_tar_gz()` | 解压 tar.gz 文件 | 大文件可能卡住 | `execute_with_timeout_and_retry()` + `default_extract_timeout()` |

**优先级**：高
**原因**：大文件可能卡住，用户等待时间长
**注意**：已在 `update.rs:522` 中使用，但这里是底层实现，可能需要单独保护

#### 4.3 `src/lib/jira/attachments/zip.rs`

**需求**：为以下方法添加超时和重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `extract_zip()` | 解压 zip 文件（Jira 附件） | 大文件可能卡住 | `execute_with_timeout_and_retry()` + `default_extract_timeout()` |
| `merge_split_zip_files()` | 合并分片 zip 文件（`WalkDir` 遍历 + 文件操作） | 大文件可能卡住，文件操作可能失败 | `execute_with_timeout_and_retry()` + `default_extract_timeout()` |

**优先级**：高
**原因**：Jira 附件可能很大，用户等待时间长

#### 4.4 `src/lib/jira/attachments/download.rs`

**需求**：为 `download_and_extract()` 方法中的解压操作添加超时和重试保护

| 方法 | 操作 | 风险 | 建议方案 |
|------|------|------|----------|
| `download_and_extract()` | 解压下载的 zip 文件 | 大文件可能卡住 | `execute_with_timeout_and_retry()` + `default_extract_timeout()` |

**优先级**：高
**原因**：大文件可能卡住，用户等待时间长

---

## 实施计划

### 阶段 1：高优先级（立即实施）

1. **文件系统操作**（15 个方法）
   - `src/lib/base/fs/directory.rs` - 6 个方法
   - `src/lib/base/fs/path.rs` - 1 个方法
   - `src/lib/base/fs/file.rs` - 6 个方法
   - `src/lib/jira/attachments/clean.rs` - 2 个方法

2. **Zip/Tar 解压操作**（5 个方法）
   - `src/lib/base/zip/zip_impl.rs` - 1 个方法
   - `src/lib/base/zip/tar.rs` - 1 个方法
   - `src/lib/jira/attachments/zip.rs` - 2 个方法
   - `src/lib/jira/attachments/download.rs` - 1 个方法

**预计工作量**：2-3 天

### 阶段 2：中优先级（建议实施）

3. **进程执行操作**（4 个方法）
   - `src/lib/git/pre_commit.rs` - 1 个方法
   - `src/lib/rollback/rollback.rs` - 1 个方法
   - `src/commands/lifecycle/install.rs` - 1 个方法
   - `src/lib/base/system/platform.rs` - 1 个方法

4. **Git 操作（非网络）**（7 个方法）
   - `src/lib/git/client/repository.rs` - 2 个方法
   - `src/lib/git/cherry_pick.rs` - 3 个方法
   - `src/lib/git/branch.rs` - 2 个方法

**预计工作量**：2-3 天

---

## 实施指南

### 1. 文件系统操作

统一使用 `execute_with_timeout_and_retry()`：

```rust
use crate::base::resilience::{execute_with_timeout_and_retry, default_filesystem_timeout, TimeoutConfig, RetryConfig};

let timeout_config = TimeoutConfig::new(default_filesystem_timeout()).with_platform_specific();
let retry_config = RetryConfig::platform_default();

execute_with_timeout_and_retry(
    timeout_config,
    retry_config,
    || {
        // 文件系统操作
        fs::read_to_string(&path)?
    },
    "Read file",
)?
```

### 2. 进程执行操作

使用 `execute_with_timeout_and_retry()` + `default_script_timeout()`：

```rust
use crate::base::resilience::{execute_with_timeout_and_retry, default_script_timeout, TimeoutConfig, RetryConfig};

let timeout_config = TimeoutConfig::new(default_script_timeout()).with_platform_specific();
let retry_config = RetryConfig::platform_default();

execute_with_timeout_and_retry(
    timeout_config,
    retry_config,
    || {
        let output = Command::new("pre-commit").arg("run").output()?;
        // 处理输出...
        Ok(())
    },
    "Run pre-commit hooks",
)?
```

### 3. Git 操作（非网络）

使用 `execute_with_timeout()` + `default_filesystem_timeout()`：

```rust
use crate::base::resilience::{execute_with_timeout, default_filesystem_timeout, TimeoutConfig};

let timeout_config = TimeoutConfig::new(default_filesystem_timeout()).with_platform_specific();

execute_with_timeout(
    timeout_config,
    || {
        Repository::open(".")?
    },
)?
```

### 4. Zip/Tar 解压操作

使用 `execute_with_timeout_and_retry()` + `default_extract_timeout()`：

```rust
use crate::base::resilience::{execute_with_timeout_and_retry, default_extract_timeout, TimeoutConfig, RetryConfig};

let timeout_config = TimeoutConfig::new(default_extract_timeout()).with_platform_specific();
let retry_config = RetryConfig::platform_default();

execute_with_timeout_and_retry(
    timeout_config,
    retry_config,
    || {
        Unzip::extract_zip(&zip_path, &output_dir)?
    },
    "Extract zip",
)?
```

---

## 注意事项

1. **避免嵌套重试**：某些方法内部调用了其他需要重试的方法，需要避免嵌套重试
2. **性能考虑**：超时和重试会增加延迟，但对于可能卡住的操作是必要的
3. **Windows 特殊处理**：Windows 上的文件系统操作更容易卡住，需要更长的超时时间
4. **大文件处理**：大文件操作（解压、读取）需要更长的超时时间
5. **测试覆盖**：添加超时和重试后需要测试各种场景

---

## 相关文档

- [Resilience 使用分析](../../analysis/resilience_usage_analysis.md) - 详细的分析文档
- [HTTP Retry 需求文档](./http-retry.md) - HTTP 请求的重试机制需求
- [Resilience 模块文档](../../docs/architecture/resilience.md) - 超时和重试机制说明

---

## 更新历史

- **2025-01-XX**：创建初始需求文档，基于 `analysis/resilience_usage_analysis.md` 的分析结果

