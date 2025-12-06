# Requirements 文档目录

## 📋 文档状态概览

本目录包含项目的需求分析、设计方案和实施指南文档。

### 文档分类

- ✅ **已实现** - 功能已完成，文档可归档
- 🚧 **实施中** - 功能部分完成，文档仍有参考价值
- ⏳ **待实施** - 功能未开始，文档为规划参考
- 📚 **参考文档** - 永久保留的指南和推荐

---

## 📦 dirs Crate 相关文档

### ✅ 已完成 - 可归档

#### 1. `dirs-crate-integration.md`
- **状态**: ✅ 已实现（核心功能）
- **实现度**: ~80%
- **建议**: 移至 `docs/archive/requirements/` 或删除
- **原因**:
  - dirs crate 已成功集成到 Cargo.toml
  - 核心路径管理已使用 dirs::home_dir()
  - 主要功能已实现
- **保留理由**: 可作为历史记录保留，但不再是活跃需求文档

#### 2. `dirs-integration-analysis.md`
- **状态**: ✅ 已实现
- **实现度**: ~80%
- **建议**: 移至 `docs/archive/requirements/` 或删除
- **原因**:
  - 集成分析已完成
  - 主要优化点已实施
- **保留理由**: 历史参考

### 🚧 实施中 - 保留

#### 3. `dirs-optimization-analysis.md`
- **状态**: 🚧 部分完成
- **实现度**: ~80%
- **建议**: **保留**，继续使用
- **原因**:
  - 核心已完成，但仍有 9 处优化点
  - 提供了具体的优化方案和代码示例
  - 是当前活跃的技术债务追踪文档
- **下一步**: 完成剩余 9 处优化后可归档

#### 4. `dirs-status-summary.md`
- **状态**: 🚧 部分完成
- **实现度**: ~80%
- **建议**: **保留**，继续使用
- **原因**:
  - 提供快速状态概览
  - 包含实施计划和测试清单
  - 是当前活跃的追踪文档
- **下一步**: 完成优化后更新为 "已完成" 状态

---

## ☁️ iCloud 存储相关文档

### ✅ 已实现 - 可归档

#### 5. `icloud-storage-analysis.md`
- **状态**: ✅ 已实现
- **实现度**: ~95%
- **建议**: 移至 `docs/archive/requirements/` 或保留作为文档参考
- **原因**:
  - iCloud 存储功能已完全实现
  - `try_icloud_base_dir()` 已实现
  - `is_config_in_icloud()` 已实现
  - `storage_location()` 已实现
  - 支持环境变量 `WORKFLOW_DISABLE_ICLOUD`
- **保留理由**: 可作为功能文档保留，帮助理解 iCloud 实现逻辑

#### 6. `icloud-storage-decision-flow.md`
- **状态**: ✅ 已实现
- **实现度**: ~95%
- **建议**: 移至 `docs/architecture/` 作为架构文档
- **原因**:
  - 决策流程已完全实现
  - 文档质量高，有助于理解代码逻辑
- **建议**: 重命名为 `ICLOUD_STORAGE_ARCHITECTURE.md` 并移至架构文档目录

#### 7. `icloud-storage-implementation.md`
- **状态**: ✅ 已实现
- **实现度**: ~95%
- **建议**: 移至 `docs/archive/requirements/` 或删除
- **原因**:
  - 实施步骤已完成
  - 作为指南的作用已结束
- **保留理由**: 历史记录

#### 8. `icloud-storage-usage-examples.md`
- **状态**: ✅ 已实现
- **实现度**: ~95%
- **建议**: 移至 `docs/guides/` 作为用户指南
- **原因**:
  - 包含实用的使用示例
  - 对用户有参考价值
- **建议**: 重命名为 `ICLOUD_STORAGE_GUIDE.md` 并作为用户文档保留

---

## 📚 其他需求文档

### 🚧 实施中 - 保留

#### 9. `third-party-library-analysis.md`
- **状态**: 🚧 实施中
- **实现度**: ~25%
- **建议**: **保留**，继续使用
- **当前进度**:
  - ✅ `dirs` - 已完成 80%
  - ⏳ `humansize` - 待实施
  - ⏳ `reqwest-retry` - 待实施
  - ⏳ `tracing` - 待实施
- **下一步**: 继续推进第三方库集成

#### 10. `implementation-steps.md`
- **状态**: 🚧 部分过时
- **实现度**: ~80%
- **建议**: **更新或归档**
- **原因**:
  - 大部分步骤已完成（dirs + iCloud）
  - 部分内容已过时
- **建议**:
  - 选项 A: 更新为当前状态，移除已完成步骤
  - 选项 B: 归档到 `docs/archive/requirements/`

### ⏳ 待实施 - 保留作参考

#### 11. `ui-framework-recommendations.md`
- **状态**: ⏳ 未实施
- **实现度**: 0%
- **建议**: **保留**作为未来参考
- **原因**:
  - 功能未开始实施
  - 是未来 UI 改进的技术选型参考
  - 文档质量高，有参考价值
- **下一步**: 等待 UI 改进需求时使用

---

## 🗂️ 建议的文档重组方案

### 方案 1: 创建归档目录

```
docs/
├── requirements/           # 活跃需求文档
│   ├── README.md          # 本文件
│   ├── dirs-optimization-analysis.md      # 🚧 保留
│   ├── dirs-status-summary.md             # 🚧 保留
│   ├── third-party-library-analysis.md    # 🚧 保留
│   └── ui-framework-recommendations.md    # ⏳ 保留
│
├── archive/               # 归档文档（已完成的需求）
│   └── requirements/
│       ├── dirs-crate-integration.md           # ✅ 归档
│       ├── dirs-integration-analysis.md        # ✅ 归档
│       ├── icloud-storage-analysis.md          # ✅ 归档
│       ├── icloud-storage-implementation.md    # ✅ 归档
│       └── implementation-steps.md             # ✅ 归档
│
├── architecture/          # 架构文档
│   └── lib/
│       └── ICLOUD_STORAGE_ARCHITECTURE.md  # 从 icloud-storage-decision-flow.md 重命名
│
└── guides/                # 用户指南
    └── ICLOUD_STORAGE_GUIDE.md             # 从 icloud-storage-usage-examples.md 重命名
```

### 方案 2: 直接删除

**可以删除的文档**（已完成且无长期参考价值）:
- `dirs-crate-integration.md` - 信息已过时
- `dirs-integration-analysis.md` - 分析已完成
- `icloud-storage-implementation.md` - 实施指南已用完

**应该保留/转换的文档**:
- `icloud-storage-analysis.md` → 保留作为功能文档
- `icloud-storage-decision-flow.md` → 转为架构文档
- `icloud-storage-usage-examples.md` → 转为用户指南

---

## 📝 执行建议

### 立即执行（推荐方案 1）

#### 步骤 1: 创建目录结构

```bash
# 创建归档目录
mkdir -p docs/archive/requirements
mkdir -p docs/guides

# 创建 README
cp docs/requirements/README.md docs/archive/requirements/README.md
```

#### 步骤 2: 移动已完成文档

```bash
# 移动到归档
mv docs/requirements/dirs-crate-integration.md docs/archive/requirements/
mv docs/requirements/dirs-integration-analysis.md docs/archive/requirements/
mv docs/requirements/icloud-storage-analysis.md docs/archive/requirements/
mv docs/requirements/icloud-storage-implementation.md docs/archive/requirements/
mv docs/requirements/implementation-steps.md docs/archive/requirements/
```

#### 步骤 3: 转换文档类型

```bash
# 转换为架构文档
mv docs/requirements/icloud-storage-decision-flow.md \
   docs/architecture/lib/ICLOUD_STORAGE_ARCHITECTURE.md

# 转换为用户指南
mv docs/requirements/icloud-storage-usage-examples.md \
   docs/guides/ICLOUD_STORAGE_GUIDE.md
```

#### 步骤 4: 更新文档索引

更新以下文档的链接：
- `docs/README.md` - 主文档索引
- `docs/architecture/ARCHITECTURE.md` - 架构文档索引
- 其他引用这些文档的地方

#### 步骤 5: 提交更改

```bash
git add -A
git commit -m "docs: reorganize requirements documents

- Archive completed requirements (dirs, iCloud)
- Move icloud-storage-decision-flow to architecture docs
- Move icloud-storage-usage-examples to user guides
- Keep active requirements (optimization, third-party)
"
```

---

## 🎯 完成剩余任务后的清理

### dirs 优化完成后（预计 1 小时后）

```bash
# 更新状态为已完成
# 然后归档
mv docs/requirements/dirs-optimization-analysis.md docs/archive/requirements/
mv docs/requirements/dirs-status-summary.md docs/archive/requirements/
```

### 第三方库集成完成后（预计数周后）

```bash
# 归档分析文档
mv docs/requirements/third-party-library-analysis.md docs/archive/requirements/
```

---

## 📊 当前统计

| 状态 | 文档数量 | 建议动作 |
|-----|---------|----------|
| ✅ 已完成可归档 | 5 个 | 移至 `archive/` |
| 🚧 实施中保留 | 4 个 | 保留在 `requirements/` |
| ⏳ 待实施参考 | 1 个 | 保留在 `requirements/` |
| 📚 转换类型 | 2 个 | 转为架构/指南文档 |
| **总计** | **11 个** | - |

---

## 📌 总结

### 立即行动

1. **归档 5 个已完成文档** - 移至 `docs/archive/requirements/`
2. **转换 2 个文档** - 转为架构文档和用户指南
3. **保留 4 个活跃文档** - 继续作为需求追踪

### 保持活跃的文档

- `dirs-optimization-analysis.md` - 追踪剩余 9 处优化
- `dirs-status-summary.md` - 快速状态概览
- `third-party-library-analysis.md` - 第三方库集成追踪
- `ui-framework-recommendations.md` - 未来 UI 改进参考

### 预期收益

- ✅ 清理完成的需求文档
- ✅ 保持文档目录的清晰性
- ✅ 保留有价值的架构和用户文档
- ✅ 明确当前活跃的需求

---

**更新时间**: 2025-12-06
**文档维护**: 定期审查，保持目录整洁
