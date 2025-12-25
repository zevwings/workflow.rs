# Rollback 模块测试覆盖率改进计划

> Rollback 模块测试覆盖率分析与改进方案

**状态**: 📋 待实施
**当前覆盖率**: 3.2% (8/251 行)
**目标覆盖率**: >80%
**需要提升**: +76.8% (+193 行)
**优先级**: ⭐⭐⭐ 高（关键安全功能）

---

## 📊 执行摘要

### 模块概述

Rollback 模块是 Workflow CLI 的关键安全模块，负责：
- **备份管理**：在更新前备份二进制文件和补全脚本
- **回滚操作**：更新失败时恢复备份的文件
- **清理管理**：清理备份文件
- **Shell 重载**：恢复后重新加载 shell 配置

### 代码规模

| 指标 | 数值 |
|------|------|
| 总代码行数 | 514 行 |
| 可测试行数 | 251 行 |
| 已覆盖行数 | 8 行 |
| 未覆盖行数 | 243 行 |
| 测试代码行数 | 0 行 |

### 当前覆盖率

| 文件 | 覆盖率 | 已覆盖/可测试 | 状态 |
|------|--------|---------------|------|
| `rollback.rs` | 3.2% | 8/251 | 🔴 **极低** |
| **总计** | **3.2%** | **8/251** | 🔴 **需改进** |

### 核心问题

1. **几乎完全未测试**（3.2%）：251 行代码中只有 8 行被测试
2. **关键安全功能缺失测试**：回滚是更新失败时的最后保障，必须可靠
3. **文件系统操作未测试**：备份、恢复、清理操作都需要文件系统测试
4. **跨平台兼容性未测试**：不同操作系统的路径和权限处理

---

## 🔍 测试覆盖缺失分析

### 1. rollback.rs - 回滚操作（243 行未覆盖）

**核心功能**：
- 创建备份目录
- 备份二进制文件
- 备份补全脚本
- 恢复二进制文件
- 恢复补全脚本
- 清理备份文件
- Shell 配置重载

**未测试的关键函数**：

```rust
// 备份操作
pub fn create_backup() -> Result<BackupResult>
fn create_backup_dir() -> Result<PathBuf>
fn backup_binaries(backup_dir: &Path, binaries: Vec<String>) -> Result<Vec<(String, PathBuf)>>
fn backup_completions(backup_dir: &Path, completion_dir: &Path) -> Result<Vec<(String, PathBuf)>>

// 恢复操作
pub fn rollback(backup_info: &BackupInfo) -> Result<RollbackResult>
fn restore_binaries(backups: &[(String, PathBuf)]) -> Result<RestoreResult>
fn restore_completions(backups: &[(String, PathBuf)]) -> Result<RestoreResult>

// 清理操作
pub fn cleanup_backup(backup_info: &BackupInfo) -> Result<()>
fn cleanup_backup_dir(backup_dir: &Path) -> Result<()>
```

**测试难点**：
- 需要模拟系统目录（/usr/local/bin）
- 需要测试文件权限和所有权
- 需要测试跨平台路径处理
- 需要模拟 Shell 配置重载
- 需要测试错误恢复场景

---

## 📝 测试改进计划

### 阶段 1：高优先级测试（目标：50% 覆盖率）

#### 1.1 备份操作测试（预计 +80 行覆盖）

**文件**：`tests/rollback/backup.rs`

**测试用例**：
```rust
// 备份目录创建
#[test]
fn test_create_backup_dir() { }

#[test]
fn test_create_backup_dir_already_exists() { }

// 二进制文件备份
#[test]
fn test_backup_binaries_success() { }

#[test]
fn test_backup_binaries_file_not_found() { }

#[test]
fn test_backup_binaries_permission_denied() { }

// 补全脚本备份
#[test]
fn test_backup_completions_success() { }

#[test]
fn test_backup_completions_directory_not_found() { }

// 完整备份流程
#[test]
fn test_create_backup_complete() { }

#[test]
fn test_create_backup_partial_failure() { }
```

**工作量估计**：3-4 天

#### 1.2 恢复操作测试（预计 +70 行覆盖）

**文件**：`tests/rollback/restore.rs`

**测试用例**：
```rust
// 二进制文件恢复
#[test]
fn test_restore_binaries_success() { }

#[test]
fn test_restore_binaries_file_not_found() { }

#[test]
fn test_restore_binaries_permission_denied() { }

// 补全脚本恢复
#[test]
fn test_restore_completions_success() { }

#[test]
fn test_restore_completions_partial_failure() { }

// 完整回滚流程
#[test]
fn test_rollback_complete() { }

#[test]
fn test_rollback_partial_failure() { }

#[test]
fn test_rollback_shell_reload() { }
```

**工作量估计**：3-4 天

### 阶段 2：中优先级测试（目标：70% 覆盖率）

#### 2.1 清理操作测试（预计 +30 行覆盖）

**文件**：`tests/rollback/cleanup.rs`

**测试用例**：
```rust
#[test]
fn test_cleanup_backup_success() { }

#[test]
fn test_cleanup_backup_dir_not_found() { }

#[test]
fn test_cleanup_backup_permission_denied() { }

#[test]
fn test_cleanup_backup_dir() { }
```

**工作量估计**：1-2 天

#### 2.2 错误处理测试（预计 +30 行覆盖）

**文件**：`tests/rollback/error_handling.rs`

**测试用例**：
```rust
#[test]
fn test_backup_with_invalid_paths() { }

#[test]
fn test_restore_with_corrupted_backup() { }

#[test]
fn test_rollback_with_missing_backup_info() { }
```

**工作量估计**：1-2 天

### 阶段 3：完善测试（目标：>80% 覆盖率）

#### 3.1 跨平台测试（预计 +20 行覆盖）

**文件**：`tests/rollback/platform_specific.rs`

**测试用例**：
```rust
#[cfg(target_os = "windows")]
#[test]
fn test_backup_windows_paths() { }

#[cfg(unix)]
#[test]
fn test_backup_unix_paths() { }

#[test]
fn test_path_handling_cross_platform() { }
```

**工作量估计**：1-2 天

#### 3.2 边界情况测试（预计 +13 行覆盖）

**文件**：`tests/rollback/edge_cases.rs`

**测试用例**：
```rust
#[test]
fn test_backup_empty_binary_list() { }

#[test]
fn test_backup_empty_completion_list() { }

#[test]
fn test_restore_empty_backup() { }
```

**工作量估计**：0.5 天

---

## 🎯 实施优先级

### P0 - 立即实施（2 周内）

| 任务 | 预计覆盖提升 | 工作量 |
|------|-------------|--------|
| 备份操作测试 | +31.9% | 3-4 天 |
| 恢复操作测试 | +27.9% | 3-4 天 |

**预期结果**：覆盖率从 3.2% 提升到 63.0%

### P1 - 短期实施（1 个月内）

| 任务 | 预计覆盖提升 | 工作量 |
|------|-------------|--------|
| 清理操作测试 | +12.0% | 1-2 天 |
| 错误处理测试 | +12.0% | 1-2 天 |

**预期结果**：覆盖率从 63.0% 提升到 87.0%

### P2 - 中期实施（2 个月内）

| 任务 | 预计覆盖提升 | 工作量 |
|------|-------------|--------|
| 跨平台测试 | +8.0% | 1-2 天 |
| 边界情况测试 | +5.2% | 0.5 天 |

**预期结果**：覆盖率从 87.0% 提升到 >100%（实际约 90-95%，考虑到部分代码难以测试）

---

## 📚 相关文档

- [测试覆盖度提升综合方案](./test-coverage-improvement.md)
- [Rollback 模块架构](../../architecture/rollback.md)（如果存在）

---

**最后更新**: 2025-12-25

