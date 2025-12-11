# Todo 文档目录

## 📋 文档状态概览

本目录包含项目的待办事项、需求分析、设计方案和实施指南文档。

### 文档分类

- ✅ **已实现** - 功能已完成，文档可归档
- 🚧 **实施中** - 功能部分完成，文档仍有参考价值
- ⏳ **待实施** - 功能未开始，文档为规划参考
- 📚 **参考文档** - 永久保留的指南和推荐

---

## 📚 当前待办事项文档

### 📋 分类待办事项文档

#### 1. [`JIRA_TODO.md`](./JIRA_TODO.md)
- **状态**: 🚧 部分完成
- **实现度**: ~40%
- **分类**: JIRA 模块
- **已完成**:
  - ✅ `jira info` - 显示 ticket 基本信息（支持多种输出格式）
  - ✅ `jira changelog` - 显示变更历史（支持字段过滤、多种输出格式）
  - ✅ `jira comments` - 显示评论（支持分页、过滤、多种输出格式）
  - ✅ `jira attachments` - 下载附件
  - ✅ `jira clean` - 清理本地数据
  - ✅ JIRA API：`transition`、`assign`、`add_comment`（已实现，待封装为命令）
  - ✅ PR 创建和合并时自动更新 JIRA 状态
- **待实现**:
  - `jira info` 增强功能（显示更多字段）
  - 新增 JIRA 命令（assign、comment、create、list、watch）
  - JIRA 集成增强（批量操作、自定义工作流规则）
- **优先级**: 高优先级（命令封装、info 增强）


#### 3. [`INTEGRATION_TODO.md`](./INTEGRATION_TODO.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 集成与扩展
- **内容**:
  - 更多平台支持（GitLab、Bitbucket）
  - 通知系统（桌面通知、邮件通知）
- **优先级**: 中优先级

---

## 📁 目录结构

```
docs/todo/
├── README.md                      # 本文件（索引文档）
│
├── JIRA_TODO.md                   # JIRA 模块待办事项（🚧 部分完成）
└── INTEGRATION_TODO.md            # 集成与扩展待办事项
```

---

## 📊 当前统计

| 状态 | 文档数量 | 说明 |
|-----|---------|------|
| 🚧 部分完成 | 1 个 | JIRA 模块已有基础实现 |
| ⏳ 待实施 | 1 个 | 按模块分类的待办事项 |
| **总计** | **2 个** | - |

### 文档列表

#### 分类待办事项文档
1. **JIRA_TODO.md** - JIRA 模块待办事项（🚧 部分完成，~40%）
2. **INTEGRATION_TODO.md** - 集成与扩展待办事项

---

## 📌 总结

### 当前待办事项

本目录包含 2 个文档：

#### 部分完成文档（1 个）
1. **JIRA 模块** (`JIRA_TODO.md`) - 🚧 部分完成（~40%）
   - ✅ 已完成：`jira info`、`jira changelog`、`jira comments`、`jira attachments`、`jira clean`、JIRA API 基础功能
   - ⏳ 待实现：`jira info` 增强、新增命令（assign、comment、create、list、watch）

#### 待实施文档（1 个）
2. **集成与扩展** (`INTEGRATION_TODO.md`)
   - 更多平台支持、通知系统

> **注意**：配置管理相关功能（配置验证、导入/导出、多环境支持）已迁移至需求文档。
> 详见：[配置验证与导入/导出需求文档](../requirements/CONFIG_VALIDATION_AND_IMPORT_EXPORT.md)

### 文档维护

- 定期审查文档状态
- 实施完成后及时归档或删除
- 保持目录整洁，只保留活跃的待办事项

---

---

## 🔗 快速导航

### 按模块查找
- **JIRA 相关** → [`JIRA_TODO.md`](./JIRA_TODO.md) 🚧 部分完成
- **集成与扩展** → [`INTEGRATION_TODO.md`](./INTEGRATION_TODO.md)
- **Git 工作流** → [Git 工作流需求文档](../requirements/GIT_WORKFLOW.md)
- **工作流自动化** → ✅ 已完成（模板系统已实现）
- **别名系统** → [别名系统需求文档](../requirements/ALIAS_SYSTEM.md)
- **配置管理** → [配置验证与导入/导出需求文档](../requirements/CONFIG_VALIDATION_AND_IMPORT_EXPORT.md)

---

**最后更新**: 2025-01-27
**文档维护**: 定期审查，保持目录整洁

---

## 📝 更新说明

### 2025-01-27 更新
- ✅ 更新了 JIRA 和 Git 模块的完成状态
- ✅ 移除了不存在的文档引用
- ✅ 更新了统计信息和目录结构
- ✅ 删除了 CONFIG_TODO.md（配置管理功能已迁移至需求文档）
- ✅ 删除了 ALIAS_TODO.md（别名系统需求已迁移至需求文档）
- ✅ 删除了 GIT_TODO.md（Git 工作流需求已迁移至需求文档）
- ✅ 删除了 WORKFLOW_TODO.md（工作流自动化需求已迁移至需求文档）
- ✅ 删除了 STATS_TODO.md（数据可视化与报告待办事项已删除）
- ✅ 删除了 PERFORMANCE_TODO.md 和 PERFORMANCE_ANALYSIS.md（性能优化相关内容已迁移至需求文档）
