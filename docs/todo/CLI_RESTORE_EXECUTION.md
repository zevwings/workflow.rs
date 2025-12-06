# CLI 恢复执行清单

## ✅ 决策确认

**完全移除 ratatui 和 crossterm，不再使用 TUI 功能**

**Git 工作流**：从 master 创建新分支，包含 feature/ratatui 和当前内容

---

## 📋 执行步骤

### 步骤 1：准备和创建新分支（5 分钟）

#### 方案 A：使用 merge（推荐）⭐

```bash
# 1. 确保当前修改已提交或暂存
git status
git add -A
git stash  # 如果不想提交，可以暂存

# 2. 切换到 master 分支
git checkout master

# 3. 确保 master 是最新的
git pull origin master  # 如果有远程仓库

# 4. 从 master 创建新分支（用于恢复传统 CLI）
git checkout -b feature/restore-cli

# 5. 合并 feature/ratatui 的内容到新分支
git merge feature/ratatui --no-commit

# 6. 如果有暂存的修改，恢复
git stash pop  # 如果有暂存的修改

# 7. 提交合并结果
git add -A
git commit -m "merge: merge feature/ratatui into restore-cli branch

- Include all changes from feature/ratatui
- Ready to remove ratatui and restore traditional CLI"
```

---

#### 方案 B：使用 cherry-pick（精确控制）

如果需要选择性应用提交：

```bash
# 1. 确保当前修改已提交或暂存
git status
git add -A
git stash  # 如果不想提交，可以暂存

# 2. 切换到 master 分支
git checkout master
git pull origin master  # 如果有远程仓库

# 3. 从 master 创建新分支
git checkout -b feature/restore-cli

# 4. 查看 feature/ratatui 的提交列表
git log feature/ratatui --oneline

# 5. Cherry-pick 所有提交（从 master 到 feature/ratatui）
git cherry-pick master..feature/ratatui

# 或者 cherry-pick 特定提交
# git cherry-pick <commit-hash1> <commit-hash2> ...

# 6. 如果有暂存的修改，恢复
git stash pop

# 7. 提交（如果有未提交的修改）
git add -A
git commit -m "cherry-pick: include feature/ratatui changes, ready to restore CLI"
```

**选择建议**：
- **推荐使用 merge**（方案 A）- 简单直接，包含所有内容
- **如果需要选择性应用**，使用 cherry-pick（方案 B）

**分支结构**：
```
master
  ├── feature/ratatui (保留，包含 ratatui 实现)
  └── feature/restore-cli (新分支，将恢复传统 CLI)
```

**说明**：
- ✅ 从干净的 master 基础开始
- ✅ 包含 feature/ratatui 的所有修改
- ✅ 包含当前工作目录的修改
- ✅ 保留 feature/ratatui 分支完整
- ✅ 在新分支上恢复传统 CLI

---

### 步骤 1 替代方案：如果当前有未提交修改

```bash
# 1. 在 feature/ratatui 分支上提交当前修改
git checkout feature/ratatui
git add -A
git commit -m "WIP: current changes before restore"

# 2. 切换到 master
git checkout master
git pull origin master  # 如果有远程仓库

# 3. 创建新分支
git checkout -b feature/restore-cli

# 4. 合并 feature/ratatui
git merge feature/ratatui --no-edit

# 现在新分支包含 feature/ratatui 的所有内容
```

---

### 步骤 2：修改依赖（已完成 ✅）

**Cargo.toml** 已更新：
- ✅ 恢复：`colored = "2.1"`, `dialoguer = "0.11"`, `indicatif = "0.17"`
- ✅ 移除：`ratatui = "0.27"`, `crossterm = "0.28"`

---

### 步骤 3：移除代码目录（5 分钟）

```bash
# 移除 UI 目录
rm -rf src/lib/base/ui/

# 检查 logging 目录是否被使用
grep -r "use.*logging" src/ || echo "logging module not used"
grep -r "logging::" src/ || echo "logging module not used"

# 如果未被使用，移除
rm -rf src/lib/logging/
```

---

### 步骤 4：编译验证（5 分钟）

```bash
# 更新依赖
cargo update

# 编译验证
cargo build

# 如果编译失败，查看错误信息并修复
```

---

### 步骤 5：修复编译错误（如有需要）

**可能的问题**：

1. **如果 `src/lib/logging/` 被使用**：
   - 需要修复 `src/lib/logging/logger.rs`，移除 ratatui 依赖
   - 或使用 `src/lib/base/util/logger.rs`（传统 CLI 版本）

2. **如果有地方引用了 `ui` 模块**：
   - 搜索并移除相关引用
   - 使用传统 CLI 库替代

---

### 步骤 6：功能验证（10 分钟）

```bash
# 测试交互式命令
workflow github list
workflow github add
workflow jira info
workflow log download

# 测试进度条
workflow update

# 运行测试
cargo test
```

---

### 步骤 7：提交更改

```bash
# 检查更改
git status
git diff

# 提交恢复操作
git add -A
git commit -m "restore: remove ratatui and restore traditional CLI

- Remove ratatui and crossterm dependencies
- Remove src/lib/base/ui/ directory
- Remove src/lib/logging/ directory (if unused)
- Restore colored, dialoguer, indicatif dependencies
- Reduce binary size by ~700-1100 KB"
```

**提交历史**：
```
feature/restore-cli
  ├── merge: include feature/ratatui changes
  └── restore: remove ratatui and restore traditional CLI
```

---

## ✅ 完成标准

- [ ] `cargo build` 成功
- [ ] `cargo test` 通过
- [ ] 所有交互式命令正常工作
- [ ] 代码已提交

---

## 📝 快速参考

**移除的依赖**：
- `ratatui = "0.27"`
- `crossterm = "0.28"`

**恢复的依赖**：
- `colored = "2.1"`
- `dialoguer = "0.11"`
- `indicatif = "0.17"`

**移除的目录**：
- `src/lib/base/ui/`
- `src/lib/logging/`（如果未被使用）
