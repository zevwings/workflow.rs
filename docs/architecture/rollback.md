# 回滚模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的回滚模块架构，包括更新失败时的备份和恢复机制。该模块负责在更新操作前备份当前版本的二进制文件和补全脚本，并在更新失败时自动恢复备份的文件。

**模块统计：**
- 总代码行数：约 455 行（rollback.rs: 450 行，mod.rs: 5 行）
- 文件数量：2 个核心文件
- 主要组件：2 个（RollbackManager, BackupInfo）
- 备份内容：二进制文件（workflow）和补全脚本文件

---

## 📁 模块结构

### 核心模块文件

```
src/lib/rollback/
├── mod.rs                  # 模块声明和导出
└── rollback.rs             # 回滚管理器（备份、恢复、清理）
```

### 依赖模块

- **`lib/completion/files.rs`**：获取所有补全脚本文件列表（`get-_all-_completion-_files()`）
- **`lib/base/settings/paths.rs`**：路径管理（`Paths::completion-_dir()`, `Paths::config-_file()`）
- **`lib/base/shell/detect.rs`**：Shell 检测（`Detect::shell()`）
- **`lib/base/shell/reload.rs`**：Shell 配置重新加载（`Reload::shell()`）

---

## 🏗️ 架构设计

### 组件职责分离

模块采用职责分离的设计模式，每个组件负责单一职责：

#### 1. RollbackManager（结构体）

- **职责**：提供备份和恢复功能，用于更新失败时的回滚操作
- **功能**：
  - 创建备份（备份二进制文件和补全脚本）
  - 执行回滚（恢复备份的文件）
  - 清理备份（删除备份目录）

#### 2. BackupInfo（结构体）

- **职责**：存储备份信息
- **功能**：
  - 存储备份目录路径
  - 存储备份的二进制文件路径列表
  - 存储备份的补全脚本路径列表

---

## 🔄 调用流程

### 整体架构流程

```
调用者（命令层或其他模块）
  ↓
RollbackManager (回滚管理层)
  ↓
文件系统操作（备份/恢复/清理）
```

### 备份流程

```
RollbackManager::create-_backup()
  ↓
  1. RollbackManager::create-_backup-_dir()           # 创建备份目录
  2. RollbackManager::backup-_binaries()              # 备份二进制文件
     └─ sudo cp /usr/local/bin/{binary} {backup-_dir}/
  3. RollbackManager::backup-_completions()           # 备份补全脚本
     └─ fs::copy() {completion-_dir}/{file} {backup-_dir}/
```

### 回滚流程

```
RollbackManager::rollback(backup-_info)
  ↓
  1. RollbackManager::restore-_binaries()             # 恢复二进制文件
     └─ sudo cp {backup-_dir}/{binary} /usr/local/bin/
  2. RollbackManager::restore-_completions()          # 恢复补全脚本
     └─ fs::copy() {backup-_dir}/{file} {completion-_dir}/
  3. 尝试重新加载 shell 配置（可选）
     └─ Reload::shell()                              # 重新加载 shell 配置
```

### 备份流程

```
create-_backup()
  ↓
  1. create-_backup-_dir()
     └─ 在临时目录创建唯一备份目录（workflow-backup-{timestamp}）
  ↓
  2. backup-_binaries(backup-_dir, ["workflow"])
     ├─ 遍历二进制文件列表
     ├─ 检查文件是否存在
     ├─ 使用 sudo cp 复制到备份目录
     └─ 设置执行权限
  ↓
  3. backup-_completions(backup-_dir, completion-_dir)
     ├─ 获取所有补全脚本文件列表（所有 shell 类型）
     ├─ 遍历文件列表
     ├─ 检查文件是否存在
     └─ 使用 fs::copy 复制到备份目录
  ↓
  4. 返回 BackupInfo
```

### 回滚流程

```
rollback(backup-_info)
  ↓
  1. restore-_binaries(backup-_info.binary-_backups)
     ├─ 遍历备份的二进制文件列表
     ├─ 检查备份文件是否存在
     ├─ 使用 sudo cp 恢复到 /usr/local/bin
     └─ 设置执行权限
  ↓
  2. restore-_completions(backup-_info.completion-_backups, completion-_dir)
     ├─ 确保补全脚本目录存在
     ├─ 遍历备份的补全脚本文件列表
     ├─ 检查备份文件是否存在
     └─ 使用 fs::copy 恢复到补全脚本目录
  ↓
  3. 尝试重新加载 shell 配置（可选）
     ├─ 检测当前 shell 类型
     ├─ 调用 Reload::shell() 重新加载配置
     └─ 如果失败，记录警告并提供手动重新加载命令
  ↓
  4. 返回成功
```

### 清理流程

```
cleanup-_backup(backup-_info)
  ↓
  1. 检查备份目录是否存在
  ↓
  2. fs::remove-_dir-_all(backup-_info.backup-_dir)
  ↓
  3. 返回成功
```

---

## 📊 数据流

### 备份数据流

```
/usr/local/bin/workflow (二进制文件)
  ↓
sudo cp (复制)
  ↓
{temp-_dir}/workflow-backup-{timestamp}/workflow
  ↓
BackupInfo.binary-_backups

~/.workflow/completions/* (补全脚本文件)
  ↓
fs::copy (复制)
  ↓
{temp-_dir}/workflow-backup-{timestamp}/*.bash, _*, etc.
  ↓
BackupInfo.completion-_backups
```

### 恢复数据流

```
BackupInfo.binary-_backups
  ↓
sudo cp (恢复)
  ↓
/usr/local/bin/workflow

BackupInfo.completion-_backups
  ↓
fs::copy (恢复)
  ↓
~/.workflow/completions/*
```

---

## 📝 扩展性

### 添加新的备份内容

1. 在 `BackupInfo` 中添加新的备份列表字段
2. 在 `create-_backup()` 中添加新的备份逻辑
3. 在 `rollback()` 中添加新的恢复逻辑

**示例**：
```rust
pub struct BackupInfo {
    pub backup-_dir: PathBuf,
    binary-_backups: Vec<(String, PathBuf)>,
    completion-_backups: Vec<(String, PathBuf)>,
    config-_backups: Vec<(String, PathBuf)>, // 新增配置备份
}

impl RollbackManager {
    fn backup-_configs(backup-_dir: &Path) -> Result<Vec<(String, PathBuf)>> {
        // 备份配置文件
    }
}
```

### 添加备份验证

1. 在备份后验证备份文件的完整性
2. 在恢复前验证备份文件的存在和完整性

**示例**：
```rust
fn verify-_backup(backup-_info: &BackupInfo) -> Result<()> {
    // 验证所有备份文件存在
    // 验证文件完整性（可选：校验和）
}
```

---

## 📚 相关文档

- [总体架构文档](../architecture.md)
- [生命周期管理命令模块架构文档](../commands/LIFECYCLE_COMMAND_architecture.md)
- [Completion 模块架构文档](./COMPLETION_architecture.md)
- [Shell 检测与管理模块架构文档](./SHELL_architecture.md)

---

## 📋 使用示例

### 基本使用

```rust
use workflow::rollback::RollbackManager;

// 创建备份
let backup-_info = RollbackManager::create-_backup()?;

// 执行更新操作...
match update-_result {
    Ok(_) => {
        // 更新成功，清理备份
        RollbackManager::cleanup-_backup(&backup-_info)?;
    }
    Err(e) => {
        // 更新失败，执行回滚
        RollbackManager::rollback(&backup-_info)?;
    }
}
```

---

## ✅ 总结

Rollback 模块采用清晰的资源管理设计：

1. **单一职责**：RollbackManager 只负责备份、恢复和清理
2. **资源管理**：BackupInfo 统一管理备份资源
3. **容错机制**：备份失败不阻止更新，回滚失败提供手动恢复建议

**设计优势**：
- ✅ **安全性**：使用临时目录存储备份，自动清理
- ✅ **可靠性**：完整的错误处理和容错机制
- ✅ **易用性**：简单的 API，自动管理备份生命周期
- ✅ **可扩展性**：易于添加新的备份内容

---

**最后更新**: 2025-12-16
