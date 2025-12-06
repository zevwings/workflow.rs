# Requirements 文档清理指南

## 📋 快速总结

**已实现可清理**: 5 个文档
**需要转换**: 2 个文档
**保留活跃**: 4 个文档

---

## 🎯 执行方案

### 推荐方案：归档（保留历史）

```bash
# 在项目根目录执行
./scripts/reorganize-docs.sh
```

**优点**:
- ✅ 保留完整历史记录
- ✅ 便于回溯和参考
- ✅ 不丢失任何信息

**结果**:
- 已完成文档 → `docs/archive/requirements/`
- 架构相关 → `docs/architecture/lib/`
- 用户指南 → `docs/guides/`

---

### 备选方案：直接删除（清爽）

```bash
# 在项目根目录执行
./scripts/cleanup-completed-docs.sh
```

**优点**:
- ✅ 保持目录清爽
- ✅ 减少维护负担
- ✅ 代码仓库更整洁

**注意**:
- ⚠️ 永久删除文档（但 Git 历史保留）
- ⚠️ 需要输入 "yes" 确认

---

## 📦 将被处理的文档

### ✅ 已完成 - 建议归档/删除

| 文档 | 实现度 | 原因 |
|-----|-------|------|
| `dirs-crate-integration.md` | 80% | dirs 已集成，核心功能完成 |
| `dirs-integration-analysis.md` | 80% | 集成分析已完成 |
| `icloud-storage-analysis.md` | 95% | iCloud 存储已完全实现 |
| `icloud-storage-implementation.md` | 95% | 实施步骤已完成 |
| `implementation-steps.md` | 80% | 大部分步骤已完成 |

### 🔄 转换类型 - 移至其他目录

| 原文档 | 新位置 | 原因 |
|-------|--------|------|
| `icloud-storage-decision-flow.md` | `docs/architecture/lib/ICLOUD_STORAGE_ARCHITECTURE.md` | 适合作为架构文档 |
| `icloud-storage-usage-examples.md` | `docs/guides/ICLOUD_STORAGE_GUIDE.md` | 适合作为用户指南 |

### 🚧 保留 - 继续使用

| 文档 | 状态 | 原因 |
|-----|------|------|
| `dirs-optimization-analysis.md` | 活跃 | 追踪剩余 9 处优化点 |
| `dirs-status-summary.md` | 活跃 | 快速状态概览 |
| `third-party-library-analysis.md` | 活跃 | 第三方库集成追踪 |
| `ui-framework-recommendations.md` | 参考 | 未来 UI 改进指南 |
| `README.md` | 索引 | 文档目录索引 |

---

## 🔍 详细分析

### dirs Crate 文档

**当前实现状态**: ✅ 80% 完成

**已实现**:
```rust
✅ dirs::home_dir() - 统一主目录获取
✅ try_icloud_base_dir() - iCloud 自动检测
✅ config_dir() - 配置目录（支持 iCloud）
✅ workflow_dir() - 工作流目录
✅ work_history_dir() - 工作历史目录
```

**待优化**: 9 处手动环境变量读取（详见 `dirs-optimization-analysis.md`）

**文档处理**:
- `dirs-crate-integration.md` → 归档/删除（主要内容已实现）
- `dirs-integration-analysis.md` → 归档/删除（分析已完成）
- `dirs-optimization-analysis.md` → **保留**（追踪剩余优化）
- `dirs-status-summary.md` → **保留**（状态概览）

---

### iCloud 存储文档

**当前实现状态**: ✅ 95% 完成

**已实现**:
```rust
✅ try_icloud_base_dir() - iCloud 目录检测
✅ is_config_in_icloud() - 判断配置位置
✅ storage_location() - 获取存储位置描述
✅ storage_info() - 详细存储信息
✅ WORKFLOW_DISABLE_ICLOUD - 环境变量控制
```

**文档处理**:
- `icloud-storage-analysis.md` → 归档/删除（功能已完成）
- `icloud-storage-implementation.md` → 归档/删除（实施已完成）
- `icloud-storage-decision-flow.md` → **转为架构文档**（有长期参考价值）
- `icloud-storage-usage-examples.md` → **转为用户指南**（有用户价值）

---

### 其他文档

**`implementation-steps.md`**:
- 状态: 大部分步骤已完成
- 处理: 归档/删除（作为指南的使命已完成）

**`third-party-library-analysis.md`**:
- 状态: 实施中（25% 完成）
- 处理: **保留**（继续追踪第三方库集成）

**`ui-framework-recommendations.md`**:
- 状态: 未实施（0% 完成）
- 处理: **保留**（未来 UI 改进参考）

---

## 📊 清理前后对比

### 清理前（11 个文档）

```
docs/requirements/
├── dirs-crate-integration.md              # 已完成
├── dirs-integration-analysis.md           # 已完成
├── dirs-optimization-analysis.md          # 活跃
├── dirs-status-summary.md                 # 活跃
├── icloud-storage-analysis.md             # 已完成
├── icloud-storage-decision-flow.md        # 已完成
├── icloud-storage-implementation.md       # 已完成
├── icloud-storage-usage-examples.md       # 已完成
├── implementation-steps.md                # 已完成
├── third-party-library-analysis.md        # 活跃
└── ui-framework-recommendations.md        # 参考
```

### 清理后（5 个文档 + 重组）

```
docs/requirements/
├── README.md                              # 新增：文档索引
├── dirs-optimization-analysis.md          # 保留：活跃
├── dirs-status-summary.md                 # 保留：活跃
├── third-party-library-analysis.md        # 保留：活跃
└── ui-framework-recommendations.md        # 保留：参考

docs/archive/requirements/                 # 新增：归档目录
├── README.md
├── dirs-crate-integration.md
├── dirs-integration-analysis.md
├── icloud-storage-analysis.md
├── icloud-storage-implementation.md
└── implementation-steps.md

docs/architecture/lib/
└── ICLOUD_STORAGE_ARCHITECTURE.md         # 转换自 decision-flow

docs/guides/
└── ICLOUD_STORAGE_GUIDE.md                # 转换自 usage-examples
```

---

## 🚀 执行步骤

### 方案 1: 归档（推荐）

```bash
# 1. 查看当前状态
ls -la docs/requirements/

# 2. 执行重组脚本
./scripts/reorganize-docs.sh

# 3. 检查结果
git status
git diff --staged

# 4. 提交更改
git commit -m "docs: reorganize requirements documents

- Archive completed requirements (dirs, iCloud)
- Move icloud-storage-decision-flow to architecture docs
- Move icloud-storage-usage-examples to user guides
- Keep active requirements (optimization, third-party)
"

# 5. 推送（可选）
git push
```

### 方案 2: 直接删除

```bash
# 1. 查看当前状态
ls -la docs/requirements/

# 2. 执行清理脚本（需要确认）
./scripts/cleanup-completed-docs.sh

# 3. 检查结果
git status
git diff --staged

# 4. 提交更改
git commit -m "docs: remove completed requirements documents

- Remove completed dirs integration documents
- Remove completed iCloud storage documents
- Move decision-flow to architecture docs
- Move usage-examples to user guides
"

# 5. 推送（可选）
git push
```

---

## ✅ 验证清理结果

### 检查清单

- [ ] requirements 目录仅保留活跃文档
- [ ] 已完成文档已归档或删除
- [ ] 架构文档已正确转换
- [ ] 用户指南已正确转换
- [ ] README.md 索引已更新
- [ ] Git 历史记录完整

### 快速验证命令

```bash
# 查看 requirements 目录
echo "=== Requirements 目录 ==="
ls -1 docs/requirements/

# 查看归档目录（如果使用归档方案）
echo "=== Archive 目录 ==="
ls -1 docs/archive/requirements/ 2>/dev/null || echo "（未使用归档方案）"

# 查看架构文档
echo "=== Architecture 目录 ==="
ls -1 docs/architecture/lib/ | grep ICLOUD

# 查看用户指南
echo "=== Guides 目录 ==="
ls -1 docs/guides/ | grep ICLOUD
```

---

## 🎯 预期收益

### 文档组织

- ✅ 清晰分离：活跃需求 vs 已完成需求
- ✅ 易于查找：文档分类明确
- ✅ 减少混淆：不会误认为未实现

### 维护成本

- ✅ 减少 50% 需求文档数量
- ✅ 聚焦活跃需求
- ✅ 降低文档维护负担

### 代码仓库

- ✅ 更整洁的文档目录
- ✅ 更清晰的文档结构
- ✅ 更好的开发者体验

---

## 📝 后续维护

### 完成 dirs 优化后

当 `dirs-optimization-analysis.md` 中的 9 处优化完成后：

```bash
# 归档优化文档
git mv docs/requirements/dirs-optimization-analysis.md \
       docs/archive/requirements/

git mv docs/requirements/dirs-status-summary.md \
       docs/archive/requirements/

git commit -m "docs: archive completed dirs optimization documents"
```

### 完成第三方库集成后

当 `third-party-library-analysis.md` 中的所有库都集成后：

```bash
# 归档分析文档
git mv docs/requirements/third-party-library-analysis.md \
       docs/archive/requirements/

git commit -m "docs: archive completed third-party library analysis"
```

---

## 🆘 常见问题

### Q: 删除文档会永久丢失吗？

A: 不会。Git 会保留完整历史记录。可以随时通过 `git log` 和 `git checkout` 恢复。

### Q: 归档和删除如何选择？

A:
- **归档**：适合想保留完整历史、便于回溯的情况
- **删除**：适合追求代码仓库整洁、信任 Git 历史的情况

推荐使用**归档**方案，除非你确定不需要快速访问这些文档。

### Q: 如果误删除了文档怎么办？

A:
```bash
# 撤销提交前的暂存
git reset HEAD docs/requirements/

# 恢复已删除的文件
git checkout -- docs/requirements/filename.md

# 如果已提交，回退提交
git revert HEAD
```

### Q: 能否手动选择要清理的文档？

A: 可以。打开脚本文件，修改 `DELETE_DOCS` 或 `ARCHIVE_DOCS` 数组，移除你想保留的文档。

---

## 📚 相关资源

- **文档索引**: `docs/requirements/README.md`
- **重组脚本**: `scripts/reorganize-docs.sh`
- **清理脚本**: `scripts/cleanup-completed-docs.sh`
- **dirs 优化**: `docs/requirements/dirs-optimization-analysis.md`
- **第三方库**: `docs/requirements/third-party-library-analysis.md`

---

**最后更新**: 2025-12-06
**推荐操作**: 使用归档方案（`reorganize-docs.sh`）
