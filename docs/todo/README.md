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
- **实现度**: ~30%
- **分类**: JIRA 模块
- **已完成**:
  - ✅ `jira info` - 显示 ticket 基本信息（支持 comments、changelog、多种输出格式）
  - ✅ `jira attachments` - 下载附件
  - ✅ `jira clean` - 清理本地数据
  - ✅ JIRA API：`transition`、`assign`、`add_comment`（已实现，待封装为命令）
  - ✅ PR 创建和合并时自动更新 JIRA 状态
- **待实现**:
  - `jira info` 增强功能（显示更多字段）
  - 新增 JIRA 命令（assign、comment、create、list、watch）
  - JIRA 集成增强（批量操作、自定义工作流规则）
- **优先级**: 高优先级（命令封装、info 增强）

#### 2. [`GIT_TODO.md`](./GIT_TODO.md)
- **状态**: 🚧 部分完成
- **实现度**: ~20%
- **分类**: Git 工作流
- **已完成**:
  - ✅ `branch clean` - 清理本地分支
  - ✅ `branch ignore` - 管理分支忽略列表（add、remove、list）
  - ✅ `branch prefix` - 管理分支前缀（set、get、remove）
- **待实现**:
  - 分支管理增强（create、switch、rename、compare、sync）
  - Commit 管理（amend、squash、reword、history）
  - Stash 管理（list、apply、drop、pop）
- **优先级**: 高优先级（分支创建、切换、commit 管理）

#### 3. [`WORKFLOW_TODO.md`](./WORKFLOW_TODO.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 工作流自动化
- **内容**:
  - 模板系统（PR 模板、Commit 模板、分支命名模板）
  - 钩子系统（Pre-commit、Post-merge、Pre-push）
- **优先级**: 高优先级（模板系统）

#### 4. [`PERFORMANCE_TODO.md`](./PERFORMANCE_TODO.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 性能优化
- **内容**:
  - 缓存机制（API 响应缓存、本地数据缓存、智能刷新）
  - 并发处理（并行下载、批量 API 调用）
- **优先级**: 高优先级（API 响应缓存、并行下载）

#### 5. [`STATS_TODO.md`](./STATS_TODO.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 数据可视化与报告
- **内容**:
  - 统计报告（PR 统计、JIRA 统计、Git 统计）
  - 图表输出（ASCII 图表、导出为图片）
  - 时间线视图
- **优先级**: 中优先级

#### 6. [`INTEGRATION_TODO.md`](./INTEGRATION_TODO.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 集成与扩展
- **内容**:
  - 更多平台支持（GitLab、Bitbucket）
  - 通知系统（桌面通知、邮件通知）
- **优先级**: 中优先级

#### 7. [`ALIAS_TODO.md`](./ALIAS_TODO.md)
- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: 别名系统
- **内容**:
  - 别名配置（在配置文件中定义别名）
  - 别名展开（支持命令参数传递、别名嵌套）
  - 别名管理命令（list、add、remove）
- **优先级**: 中优先级

---

## 📁 目录结构

```
docs/todo/
├── README.md                      # 本文件（索引文档）
│
├── JIRA_TODO.md                   # JIRA 模块待办事项（🚧 部分完成）
├── GIT_TODO.md                    # Git 工作流待办事项（🚧 部分完成）
├── WORKFLOW_TODO.md               # 工作流自动化待办事项
├── PERFORMANCE_TODO.md            # 性能优化待办事项
├── STATS_TODO.md                  # 数据可视化与报告待办事项
├── INTEGRATION_TODO.md            # 集成与扩展待办事项
└── ALIAS_TODO.md                  # 别名系统待办事项
```

---

## 📊 当前统计

| 状态 | 文档数量 | 说明 |
|-----|---------|------|
| 🚧 部分完成 | 2 个 | JIRA、Git 模块已有基础实现 |
| ⏳ 待实施 | 5 个 | 按模块分类的待办事项 |
| **总计** | **7 个** | - |

### 文档列表

#### 分类待办事项文档
1. **JIRA_TODO.md** - JIRA 模块待办事项（🚧 部分完成，~30%）
2. **GIT_TODO.md** - Git 工作流待办事项（🚧 部分完成，~20%）
3. **WORKFLOW_TODO.md** - 工作流自动化待办事项
4. **PERFORMANCE_TODO.md** - 性能优化待办事项
5. **STATS_TODO.md** - 数据可视化与报告待办事项
6. **INTEGRATION_TODO.md** - 集成与扩展待办事项
7. **ALIAS_TODO.md** - 别名系统待办事项

---

## 📌 总结

### 当前待办事项

本目录包含 7 个文档：

#### 部分完成文档（2 个）
1. **JIRA 模块** (`JIRA_TODO.md`) - 🚧 部分完成（~30%）
   - ✅ 已完成：`jira info`、`jira attachments`、`jira clean`、JIRA API 基础功能
   - ⏳ 待实现：`jira info` 增强、新增命令（assign、comment、create、list、watch）

2. **Git 工作流** (`GIT_TODO.md`) - 🚧 部分完成（~20%）
   - ✅ 已完成：`branch clean`、`branch ignore`、`branch prefix`
   - ⏳ 待实现：分支管理增强、Commit 管理、Stash 管理

#### 待实施文档（5 个）
3. **工作流自动化** (`WORKFLOW_TODO.md`)
   - 模板系统、钩子系统、批量操作

4. **性能优化** (`PERFORMANCE_TODO.md`)
   - 缓存机制、并发处理

5. **数据可视化与报告** (`STATS_TODO.md`)
   - 统计报告、图表输出、时间线视图

6. **集成与扩展** (`INTEGRATION_TODO.md`)
   - 更多平台支持、通知系统

7. **别名系统** (`ALIAS_TODO.md`)
   - 别名配置、别名展开、别名管理命令

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
- **Git 相关** → [`GIT_TODO.md`](./GIT_TODO.md) 🚧 部分完成
- **工作流自动化** → [`WORKFLOW_TODO.md`](./WORKFLOW_TODO.md)
- **性能优化** → [`PERFORMANCE_TODO.md`](./PERFORMANCE_TODO.md)
- **数据可视化** → [`STATS_TODO.md`](./STATS_TODO.md)
- **集成与扩展** → [`INTEGRATION_TODO.md`](./INTEGRATION_TODO.md)
- **别名系统** → [`ALIAS_TODO.md`](./ALIAS_TODO.md)
- **配置管理** → [配置验证与导入/导出需求文档](../requirements/CONFIG_VALIDATION_AND_IMPORT_EXPORT.md)

---

**最后更新**: 2025-01-27
**文档维护**: 定期审查，保持目录整洁

---

## 📝 更新说明

### 2025-01-27 更新
- ✅ 更新了 JIRA 和 Git 模块的完成状态
- ✅ 添加了 ALIAS_TODO.md 文档
- ✅ 移除了不存在的文档引用
- ✅ 更新了统计信息和目录结构
- ✅ 删除了 CONFIG_TODO.md（配置管理功能已迁移至需求文档）
