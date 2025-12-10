# Git 工作流待办事项

## 📋 概述

本文档列出 Git 工作流相关的待办功能，包括分支管理和 Commit 管理。

---

## ✅ 已完成功能

- ✅ `branch clean` - 清理本地分支
- ✅ `branch ignore` - 管理分支忽略列表

---

## ❌ 待实现功能

### 1. 分支管理增强

#### 1.4 `branch compare` - 对比分支差异
- ❌ 对比分支差异

**命令示例**：
```bash
workflow branch compare branch1 branch2            # 对比两个分支
workflow branch compare branch1 --base master      # 对比与 base 的差异
workflow branch compare --stat                     # 只显示统计
```

---

### 2. Commit 管理

#### 2.4 `commit history` - 查看 commit 历史
- ❌ 查看 commit 历史（支持过滤）

**命令示例**：
```bash
workflow commit history                            # 查看历史
workflow commit history --author user@example.com  # 按作者过滤
workflow commit history --since "2024-01-01"       # 按时间过滤
workflow commit history --grep "fix"               # 搜索消息
```

---

## 📊 优先级

### 高优先级
1. **分支管理增强**
   - `branch compare` - 对比分支差异

2. **Commit 管理**
   - `commit history` - 查看 commit 历史（过滤）

### 中优先级
（暂无）

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：分支管理增强
   - `branch compare` - 对比分支差异

2. **第二阶段**：Commit 管理
   - `commit history` - 查看 commit 历史

### 技术考虑
1. **Git 操作**：使用 `git2` crate 或直接调用 git 命令
2. **错误处理**：处理 Git 操作失败的情况
3. **交互式选择**：使用 fuzzy finder 提供更好的用户体验
4. **测试**：为新功能添加单元测试和集成测试
5. **文档**：及时更新文档和示例

---

## 📚 相关文档

- [Git 工作流需求文档](../requirements/GIT_WORKFLOW.md) - 已转换为需求文档
- [JIRA 模块待办事项](./JIRA_TODO.md)
- [工作流自动化待办事项](./WORKFLOW_TODO.md)

---

**最后更新**: 2025-12-09
