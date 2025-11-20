# 回滚模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的回滚模块架构，包括更新失败时的备份和恢复机制。该模块负责在更新操作前备份当前版本的二进制文件和补全脚本，并在更新失败时自动恢复备份的文件。

**模块统计：**
- 总代码行数：约 400 行
- 文件数量：2 个核心文件
- 主要组件：2 个（RollbackManager, BackupInfo）
- 备份内容：二进制文件（workflow, pr, qk）和补全脚本文件

---

## 📁 模块结构

### 核心模块文件

```
src/lib/rollback/
├── mod.rs                  # 模块声明和导出
└── rollback.rs             # 回滚管理器（备份、恢复、清理）
```

### 命令封装层

```
src/commands/update.rs      # 更新命令（使用 RollbackManager）
```

### 依赖模块

- **`lib/completion/files.rs`**：获取所有补全脚本文件列表（`get_all_completion_files()`）
- **`lib/base/settings/paths.rs`**：路径管理（`Paths::completion_dir()`）
- **`lib/base/shell/detect.rs`**：Shell 检测（`Detect::shell()`）

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
用户输入
  ↓
main.rs (CLI 入口，参数解析)
  ↓
commands/update.rs (命令封装层)
  ↓
RollbackManager (回滚管理层)
  ↓
文件系统操作（备份/恢复/清理）
```

### 更新流程（包含回滚机制）

```
commands/update.rs::UpdateCommand::run()
  ↓
  1. RollbackManager::create_backup()                  # 创建备份
     ├─ RollbackManager::create_backup_dir()           # 创建备份目录
     ├─ RollbackManager::backup_binaries()              # 备份二进制文件
     │   └─ sudo cp /usr/local/bin/{binary} {backup_dir}/
     └─ RollbackManager::backup_completions()           # 备份补全脚本
         └─ fs::copy() {completion_dir}/{file} {backup_dir}/
  ↓
  2. 执行更新操作（下载、验证、安装）
  ↓
  3. 根据更新结果：
     ├─ 更新成功：
     │   └─ RollbackManager::cleanup_backup()           # 清理备份
     └─ 更新失败：
         ├─ RollbackManager::rollback()                 # 执行回滚
         │   ├─ RollbackManager::restore_binaries()     # 恢复二进制文件
         │   │   └─ sudo cp {backup_dir}/{binary} /usr/local/bin/
         │   └─ RollbackManager::restore_completions()  # 恢复补全脚本
         │       └─ fs::copy() {backup_dir}/{file} {completion_dir}/
         └─ RollbackManager::cleanup_backup()           # 清理备份
```

### 备份流程

```
create_backup()
  ↓
  1. create_backup_dir()
     └─ 在临时目录创建唯一备份目录（workflow-backup-{timestamp}）
  ↓
  2. backup_binaries(backup_dir, ["workflow", "pr", "qk"])
     ├─ 遍历二进制文件列表
     ├─ 检查文件是否存在
     ├─ 使用 sudo cp 复制到备份目录
     └─ 设置执行权限
  ↓
  3. backup_completions(backup_dir, completion_dir)
     ├─ 获取所有补全脚本文件列表（所有 shell 类型）
     ├─ 遍历文件列表
     ├─ 检查文件是否存在
     └─ 使用 fs::copy 复制到备份目录
  ↓
  4. 返回 BackupInfo
```

### 回滚流程

```
rollback(backup_info)
  ↓
  1. restore_binaries(backup_info.binary_backups)
     ├─ 遍历备份的二进制文件列表
     ├─ 检查备份文件是否存在
     ├─ 使用 sudo cp 恢复到 /usr/local/bin
     └─ 设置执行权限
  ↓
  2. restore_completions(backup_info.completion_backups, completion_dir)
     ├─ 确保补全脚本目录存在
     ├─ 遍历备份的补全脚本文件列表
     ├─ 检查备份文件是否存在
     └─ 使用 fs::copy 恢复到补全脚本目录
  ↓
  3. 返回成功
```

### 清理流程

```
cleanup_backup(backup_info)
  ↓
  1. 检查备份目录是否存在
  ↓
  2. fs::remove_dir_all(backup_info.backup_dir)
  ↓
  3. 返回成功
```

---

## 📊 数据流

### 备份数据流

```
/usr/local/bin/workflow, pr, qk (二进制文件)
  ↓
sudo cp (复制)
  ↓
{temp_dir}/workflow-backup-{timestamp}/workflow, pr, qk
  ↓
BackupInfo.binary_backups

~/.workflow/completions/* (补全脚本文件)
  ↓
fs::copy (复制)
  ↓
{temp_dir}/workflow-backup-{timestamp}/*.bash, _*, etc.
  ↓
BackupInfo.completion_backups
```

### 恢复数据流

```
BackupInfo.binary_backups
  ↓
sudo cp (恢复)
  ↓
/usr/local/bin/workflow, pr, qk

BackupInfo.completion_backups
  ↓
fs::copy (恢复)
  ↓
~/.workflow/completions/*
```

---

## 🎯 设计模式

### 1. 单一职责原则（SRP）

每个组件只负责一个明确的功能：
- `RollbackManager`：只负责备份、恢复和清理
- `BackupInfo`：只负责存储备份信息

### 2. 资源管理模式

使用 `BackupInfo` 结构体管理备份资源，确保备份和恢复的一致性。

### 3. 错误处理模式

- **备份失败**：记录警告，继续更新（但无法回滚）
- **回滚失败**：记录错误，提示用户手动恢复
- **清理失败**：记录警告，不影响主流程

---

## 🔍 核心数据结构

### BackupInfo（结构体）

```rust
#[derive(Debug)]
pub struct BackupInfo {
    /// 备份目录
    pub backup_dir: PathBuf,
    /// 备份的二进制文件路径
    binary_backups: Vec<(String, PathBuf)>, // (binary_name, backup_path)
    /// 备份的补全脚本路径
    completion_backups: Vec<(String, PathBuf)>, // (completion_name, backup_path)
}
```

**字段说明**：
- `backup_dir`：备份目录路径（公开字段，用于错误提示）
- `binary_backups`：二进制文件备份列表（私有字段）
- `completion_backups`：补全脚本备份列表（私有字段）

**设计说明**：
- 使用 `Vec<(String, PathBuf)>` 存储文件名和备份路径的映射
- 便于恢复时根据文件名找到对应的备份文件

### RollbackManager（结构体）

```rust
pub struct RollbackManager;
```

**方法**：
- `create_backup()` - 创建备份（公开方法）
- `rollback(backup_info)` - 执行回滚（公开方法）
- `cleanup_backup(backup_info)` - 清理备份（公开方法）
- `create_backup_dir()` - 创建备份目录（私有方法）
- `backup_binaries(backup_dir, binaries)` - 备份二进制文件（私有方法）
- `backup_completions(backup_dir, completion_dir)` - 备份补全脚本（私有方法）
- `restore_binaries(backups)` - 恢复二进制文件（私有方法）
- `restore_completions(backups, completion_dir)` - 恢复补全脚本（私有方法）

---

## 🔗 与其他模块的集成

### Completion 模块

- **`lib/completion/files.rs`**：`get_all_completion_files()`
  - 获取所有 shell 类型的所有补全脚本文件名列表
  - 用于备份时确定需要备份的文件

### 路径管理

- **`lib/base/settings/paths.rs`**：`Paths`
  - `completion_dir()` - 获取补全脚本目录路径

### Shell 检测

- **`lib/base/shell/detect.rs`**：`Detect`
  - `shell()` - 检测当前 shell 类型（用于确定补全脚本目录）

### 更新命令

- **`commands/update.rs`**：`UpdateCommand`
  - 在更新前调用 `create_backup()`
  - 在更新成功时调用 `cleanup_backup()`
  - 在更新失败时调用 `rollback()` 和 `cleanup_backup()`

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **功能层**：文件操作错误、权限错误、系统调用错误

### 容错机制

- **备份失败**：
  - 记录警告信息
  - 继续更新流程（但无法回滚）
  - 提示用户如果更新失败需要手动恢复

- **回滚失败**：
  - 记录错误信息
  - 提示用户系统可能处于不一致状态
  - 显示备份位置，提示用户手动恢复

- **清理失败**：
  - 记录警告信息
  - 不影响主流程
  - 备份目录会在系统清理临时文件时自动删除

### 权限处理

- **二进制文件备份/恢复**：
  - 使用 `sudo cp` 命令（需要用户输入密码）
  - 使用 `sudo chmod` 设置执行权限

- **补全脚本备份/恢复**：
  - 使用 `fs::copy`（不需要特殊权限）
  - 补全脚本目录通常在用户主目录下

---

## 📝 扩展性

### 添加新的备份内容

1. 在 `BackupInfo` 中添加新的备份列表字段
2. 在 `create_backup()` 中添加新的备份逻辑
3. 在 `rollback()` 中添加新的恢复逻辑

**示例**：
```rust
pub struct BackupInfo {
    pub backup_dir: PathBuf,
    binary_backups: Vec<(String, PathBuf)>,
    completion_backups: Vec<(String, PathBuf)>,
    config_backups: Vec<(String, PathBuf)>, // 新增配置备份
}

impl RollbackManager {
    fn backup_configs(backup_dir: &Path) -> Result<Vec<(String, PathBuf)>> {
        // 备份配置文件
    }
}
```

### 添加备份验证

1. 在备份后验证备份文件的完整性
2. 在恢复前验证备份文件的存在和完整性

**示例**：
```rust
fn verify_backup(backup_info: &BackupInfo) -> Result<()> {
    // 验证所有备份文件存在
    // 验证文件完整性（可选：校验和）
}
```

---

## 🎨 代码质量特性

### 已实现的优化

1. **职责分离**：
   - `RollbackManager` 只负责备份、恢复和清理
   - `BackupInfo` 只负责存储备份信息

2. **错误处理**：
   - 备份失败不影响更新流程
   - 回滚失败提供清晰的错误提示和手动恢复建议

3. **资源管理**：
   - 使用 `BackupInfo` 统一管理备份资源
   - 确保备份和恢复的一致性

4. **可扩展性**：
   - 易于添加新的备份内容
   - 易于添加备份验证逻辑

---

## 📚 相关文档

- [主架构文档](./ARCHITECTURE.md)
- [安装/卸载模块架构文档](./INSTALL_ARCHITECTURE.md)
- [Completion 模块架构文档](./COMPLETION_ARCHITECTURE.md)

---

## 🔄 使用场景

### 正常更新场景

1. **更新前**：
   - 用户运行 `workflow update`
   - 系统自动创建备份（二进制文件和补全脚本）

2. **更新中**：
   - 下载新版本
   - 验证文件完整性
   - 安装新版本

3. **更新成功**：
   - 验证安装结果
   - 清理备份目录
   - 提示更新完成

### 更新失败场景

1. **更新失败**：
   - 检测到更新失败（下载失败、验证失败、安装失败等）
   - 自动执行回滚

2. **回滚过程**：
   - 恢复二进制文件到备份版本
   - 恢复补全脚本到备份版本
   - 清理备份目录

3. **回滚成功**：
   - 提示回滚完成
   - 系统恢复到更新前的状态

4. **回滚失败**：
   - 提示回滚失败
   - 显示备份位置
   - 提示用户手动恢复

---

## 💡 设计决策

### 为什么备份到临时目录？

- **原因**：临时目录通常有足够的空间，且系统会自动清理
- **好处**：不需要用户手动清理备份文件
- **位置**：`{temp_dir}/workflow-backup-{timestamp}`

### 为什么使用时间戳命名备份目录？

- **原因**：避免备份目录冲突
- **好处**：支持多次备份（虽然通常只需要一次）
- **格式**：`workflow-backup-{unix_timestamp}`

### 为什么备份所有 shell 类型的补全脚本？

- **原因**：用户可能在不同 shell 环境下使用
- **好处**：确保回滚时恢复所有补全脚本
- **实现**：使用 `get_all_completion_files()` 获取所有文件

### 为什么备份失败时继续更新？

- **原因**：备份失败可能是权限问题，但不影响更新本身
- **好处**：不阻止用户更新
- **代价**：如果更新失败，无法自动回滚（但可以手动恢复）

### 为什么使用 sudo 备份/恢复二进制文件？

- **原因**：二进制文件安装在 `/usr/local/bin`，需要 root 权限
- **好处**：确保备份和恢复的完整性
- **代价**：需要用户输入密码

---

## 🔒 安全性考虑

### 备份文件权限

- **备份目录**：使用系统临时目录，权限由系统管理
- **备份文件**：保持原文件的权限（二进制文件设置执行权限）

### 备份文件清理

- **正常情况**：更新成功或回滚成功后自动清理
- **异常情况**：系统临时目录清理机制会自动清理旧备份

### 备份文件位置

- **位置**：系统临时目录（`std::env::temp_dir()`）
- **访问**：只有当前用户和 root 可以访问
- **清理**：系统自动清理或手动清理

