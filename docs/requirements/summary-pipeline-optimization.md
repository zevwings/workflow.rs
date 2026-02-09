# Summary Pipeline 优化方案

基于对 `crates/services/src/summary/` 三阶段分析流水线的审查，结合实际 PR 总结场景中的方法论对比，识别出 7 个可改进方向。

**状态**: ⏳ 待实施
**实现度**: 0%
**优先级**: 高

---

## 一、现状分析

### 当前架构

```
prepare(数据采集)
    │
    ▼
Stage1: FileClassifyService（文件分类）
    │  输入: commit 元数据 + 文件列表（无 diff）
    │  输出: CommitFileClassification（分类 + 分析策略）
    │
    ▼
Stage2: 4 路分类分析（顺序执行）
    │  ├─ BatchAnalyzeService   → batch_group 文件（采样前 3 个 diff）
    │  ├─ LogicAnalyzeService   → focus_group 文件（完整 diff）
    │  ├─ ConfigAnalyzeService  → configuration + documentation 文件
    │  └─ TestAnalyzeService    → tests 文件（完整 diff）
    │
    ▼
Stage3: SummaryAnalyzeService（全局总结）
    输入: Stage1 JSON + Stage2 四项 JSON + 统计 + 提交历史
    输出: CommitSummaryAnalysis（commit message + 结构化总结）
```

### 核心问题

| 问题 | 影响 | 严重度 |
|------|------|--------|
| Stage2 四路分析顺序执行 | 总耗时 = 4 次 LLM 调用之和 | 高 |
| 缺少目录级结构感知 | 大型重构 PR 总结缺乏"模块提取/迁移"等高层视角 | 高 |
| 批量分析采样策略简单 | 取前 3 个文件，可能采到低信息量样本 | 中 |
| Stage1 缺少目录聚合统计 | LLM 需自行从文件列表中推断模块级别模式 | 中 |
| 提交历史仅作背景信息 | 未引导 LLM 从中推断变更演进脉络 | 低 |
| 冗余文件未清理 | `batch/batch_analyze.rs` 是重构遗留 | 低 |
| Stage3 缺少变更模式识别引导 | 重构类 PR 的总结缺乏深度 | 低 |

---

## 二、改进方案

### 改进 1：Stage2 并行执行

**问题**：`run_stage2()` 中四个子服务顺序调用，每次都需等待 LLM 返回后再发起下一次请求。对于需要分析全部四类文件的 PR，总等待时间约为 4 倍单次 LLM 调用耗时。

**分析**：四个子服务之间无数据依赖——它们都只读取 Stage1 的输出和 `AnalysisContext`，互不影响。

**修改方案**：

使用 `tokio::join!` 或 `futures::join!` 并行执行：

```rust
async fn run_stage2(
    &self,
    context: &AnalysisContext,
    classification: &CommitFileClassification,
) -> Result<Stage2Results, ServiceError> {
    let (batch_result, logic_result, config_result, test_result) = tokio::join!(
        self.run_batch_analysis(context, classification),
        self.run_logic_analysis(context, classification),
        self.run_config_analysis(context, classification),
        self.run_test_analysis(context, classification),
    );

    Ok(Stage2Results {
        batch_analysis: batch_result?,
        logic_analysis: logic_result?,
        config_analysis: config_result?,
        test_analysis: test_result?,
    })
}
```

**注意事项**：
- 需确认 `LLMExecutor` 的实现支持并发调用（`Arc<dyn LLMExecutor>` + `Send + Sync`）
- 如果 LLM API 有速率限制，可考虑使用 `Semaphore` 控制并发度
- 某路分析为空（如无测试文件）时直接返回 `"{}"`，不会占用 LLM 资源

**预期效果**：Stage2 总耗时从 `T1 + T2 + T3 + T4` 降到 `max(T1, T2, T3, T4)`，约提升 3-4 倍。

**涉及文件**：

- `crates/services/src/summary/service.rs` — `run_stage2()` 方法改为并行

**复杂度**：低

---

### 改进 2：引入目录聚类维度

**问题**：Stage1 的分类维度包含 `by_status`（增删改重命名）、`by_nature`（逻辑/配置/测试/文档）、`by_scale`（大/中/小），但**缺少 `by_directory` / `by_module` 维度**。

这导致在大型重构 PR 中，LLM 无法在 Stage1 阶段就识别出"整个 `crates/llm/` 是新增 crate"或"整个 `storage/src/llm/` 被删除"这类模块级别的结构变更。这些信息最终在 Stage3 总结时非常关键。

**实际场景对比**：

人工总结 PR 时，第一步就是看 `git diff --stat` 并**按目录分组**：

```
crates/llm/           → 15 个新增文件，+4265 行（新增 crate）
crates/storage/src/llm/ → 28 个删除文件，-2334 行（删除旧模块）
crates/services/src/summary/ → 40 个新增文件，+3006 行（新增模块）
```

这种目录级别的模式识别，比逐文件分类高效得多。

**修改方案**：

1. `prepare()` 阶段增加目录级预聚合：

```rust
/// 目录级变更统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryStats {
    /// 目录路径（取前 2-3 级）
    pub path: String,
    /// 该目录下的变更文件数
    pub file_count: usize,
    /// 总新增行数
    pub total_additions: u32,
    /// 总删除行数
    pub total_deletions: u32,
    /// 是否全部为新增文件
    pub all_new: bool,
    /// 是否全部为删除文件
    pub all_deleted: bool,
    /// 文件状态分布
    pub status_distribution: HashMap<String, usize>,
}
```

```rust
fn aggregate_by_directory(files: &[CommitFileChange]) -> Vec<DirectoryStats> {
    let mut dir_map: HashMap<String, Vec<&CommitFileChange>> = HashMap::new();

    for file in files {
        // 取前 3 级目录作为分组键
        // 例如 "crates/llm/src/client.rs" → "crates/llm/src"
        let dir_key = file.path
            .rsplitn(2, '/')
            .last()
            .unwrap_or(&file.path)
            .to_string();
        dir_map.entry(dir_key).or_default().push(file);
    }

    dir_map.into_iter()
        .map(|(path, group_files)| {
            let all_new = group_files.iter().all(|f| f.status == "added");
            let all_deleted = group_files.iter().all(|f| f.status == "removed");
            DirectoryStats {
                path,
                file_count: group_files.len(),
                total_additions: group_files.iter().filter_map(|f| f.additions).sum(),
                total_deletions: group_files.iter().filter_map(|f| f.deletions).sum(),
                all_new,
                all_deleted,
                status_distribution: /* 按 status 计数 */,
            }
        })
        .sorted_by(|a, b| b.file_count.cmp(&a.file_count))
        .collect()
}
```

2. `classify/service.rs` 的输入增加目录统计段：

```text
## Directory Statistics

| Directory | Files | +Lines | -Lines | Pattern |
|-----------|-------|--------|--------|---------|
| crates/llm/src | 15 | 4265 | 0 | all_new |
| crates/storage/src/llm | 28 | 0 | 2334 | all_deleted |
| ... | ... | ... | ... | ... |
```

3. `classify_files.md` prompt 增加分类维度：

```markdown
### 4. 按目录聚类 (by_directory)

基于目录统计信息，识别模块级别的变更模式：

- **new_module**: 目录下全部为新增文件 → 新增模块/crate
- **removed_module**: 目录下全部为删除文件 → 移除模块
- **migrated_module**: 一个目录全部删除 + 另一个目录全部新增 → 模块迁移
- **heavy_modification**: 目录下大量修改文件 → 重构热点
```

4. `CommitFileClassification` entity 增加字段：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitFileClassification {
    // ... 现有字段 ...
    #[serde(default)]
    pub directory_patterns: Vec<DirectoryPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryPattern {
    pub directory: String,
    pub pattern: String,  // new_module / removed_module / migrated_module / heavy_modification
    pub description: String,
    #[serde(default)]
    pub related_directory: Option<String>,  // 迁移场景的对应目录
}
```

**涉及文件**：

- `crates/services/src/summary/service.rs` — `prepare()` 增加 `aggregate_by_directory()`
- `crates/services/src/summary/classify/service.rs` — 输入增加目录统计
- `crates/services/src/summary/prompt/classify_files.md` — 增加目录聚类维度
- `crates/domain/src/summary/entity.rs` — 增加 `DirectoryPattern` struct

**复杂度**：中等

---

### 改进 3：优化批量分析的采样策略

**问题**：`BatchAnalyzeService::analyze()` 对 `batch_group` 文件取**前 3 个**的 diff 作为样本。这种简单的取前 N 策略有两个缺陷：

1. 前 3 个文件可能都是纯重命名（diff 为空或极短），信息量低
2. 前 3 个文件可能都在同一子目录下，缺乏代表性

**实际场景**：在前述 PR 中，批量操作涉及 `crates/toolkit/src/http/` → `crates/http/src/` 的文件迁移。其中大部分文件是纯移动（0 行变更），但少数文件有实质改动（如 `client.rs` 有 +60/-60 行）。如果恰好采到 3 个纯移动文件，LLM 会错误认为"所有文件只是移动，无内容变更"。

**修改方案**：

```rust
fn select_representative_samples(
    batch_files: &[String],
    file_diffs: &HashMap<String, String>,
    files: &[CommitFileChange],
    max_samples: usize,
) -> Vec<&String> {
    // 策略 1：按变更量降序排序，优先取有实质变更的文件
    let mut scored: Vec<(&String, u32)> = batch_files.iter()
        .map(|path| {
            let change_size = files.iter()
                .find(|f| &f.path == path)
                .map(|f| f.additions.unwrap_or(0) + f.deletions.unwrap_or(0))
                .unwrap_or(0);
            (path, change_size)
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));

    // 策略 2：确保样本来自不同子目录（多样性）
    let mut selected = Vec::new();
    let mut seen_dirs: HashSet<String> = HashSet::new();

    for (path, _score) in &scored {
        let dir = path.rsplitn(2, '/').last().unwrap_or(path).to_string();
        if selected.len() < max_samples {
            if !seen_dirs.contains(&dir) || selected.len() < max_samples / 2 {
                selected.push(*path);
                seen_dirs.insert(dir);
            }
        }
    }

    // 补齐到 max_samples
    for (path, _) in &scored {
        if selected.len() >= max_samples { break; }
        if !selected.contains(path) {
            selected.push(*path);
        }
    }

    selected
}
```

**同时在 user prompt 中标注采样统计**：

```text
## Batch Files Overview
Total files in batch: {total}
Sampled: {sample_count} (selected by change size + directory diversity)
Files with zero changes: {zero_change_count}

## Sample Files
(以下为采样文件的 diff)
```

这让 LLM 知道还有多少文件未被采样，以及未采样文件的大致情况。

**涉及文件**：

- `crates/services/src/summary/batch/service.rs` — 替换采样逻辑

**复杂度**：低

---

### 改进 4：提交历史演进分析引导

**问题**：提交历史以 `sha[..8] - message_first_line` 格式平铺在 Stage3 的 user prompt 中，但 `summary.md` prompt 没有明确引导 LLM 从中提取**变更的演进脉络**。

**实际场景**：前述 PR 的提交历史为：

```
329a8b4 # workflow.toml / jira.toml / binary 目录迁移
b1cc96e # path 统一
4ff570f # path 统一
fc8211f # 代码优化
91c5d68 # 文档/排除代码
476708d # 提交代码的内容
69fcf9a # 重命名
dc9eccf # 迁移
5394679 feat(pr): IOSNAT-30274: workflow 重构 (#268)
```

从中可以清晰看出重构的步骤：先迁移路径 → 再统一 path → 优化代码 → 重命名 → 最终合并。这种演进脉络对理解 PR 整体意图至关重要。

**修改方案**：

在 `summary.md` 中增加引导段落：

```markdown
### 提交历史分析

请基于提交历史推断变更的**演进过程**：

1. **识别阶段**：将提交序列分为几个逻辑阶段（如"基础设施搭建 → 功能迁移 → 清理优化"）
2. **推断意图**：从提交顺序理解作者的重构/开发思路
3. **判断模式**：
   - 多条提交围绕同一目标 → 逐步迭代
   - 单条大提交 → 一次性变更
   - 先增后删的模式 → 可能是模块迁移（先在新位置创建，再删除旧位置）

将演进分析融入 `structured_summary.primary_purpose` 和 `structured_summary.changes` 中。
```

**涉及文件**：

- `crates/services/src/summary/prompt/summary.md` — 增加提交历史分析引导

**复杂度**：低

---

### 改进 5：Stage3 增加变更模式识别引导

**问题**：`summary.md` prompt 要求 LLM 生成 `details_by_category`（按 features/fixes/refactors 分类），但未引导识别跨文件类型的**高层变更模式**。

**实际场景**：前述 PR 的核心模式是"模块提取"——将 `toolkit/src/http/` 提升为独立的 `crates/http/` crate，涉及文件移动（重命名）、依赖配置修改（Cargo.toml）、引用路径更新（多个 Rust 文件）等跨类型变更。按 features/refactors 分类会把这个统一意图拆散。

**修改方案**：

1. 在 `summary.md` 中增加变更模式识别的引导：

```markdown
### 变更模式识别

在生成总结前，请先识别变更是否匹配以下常见模式：

| 模式 | 特征 | 总结策略 |
|------|------|---------|
| **模块提取** | 一个目录全新增 + 另一个目录全删除 + Cargo.toml 变更 | 强调"将 X 提取为独立模块" |
| **功能迁移** | 大量文件从目录 A 移到目录 B + 引用路径更新 | 强调"将 X 功能从 A 迁移到 B" |
| **接口统一** | 多个文件修改同一类 import/use 路径 | 强调"统一使用新的 X 接口" |
| **分层重构** | domain/services/storage 层间的代码移动 | 强调"调整 X 在架构中的层次" |
| **批量更新** | 大量文件做相似的小改动 | 强调"批量更新 X（共 N 个文件）" |
| **新功能开发** | 新增多个文件 + 无删除 + 业务逻辑代码为主 | 按功能域描述新增能力 |

将识别到的模式融入 `structured_summary.primary_purpose` 中，使总结更具洞察力。
```

2. `CommitSummaryAnalysis` entity 增加可选字段：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredSummary {
    // ... 现有字段 ...
    /// 识别到的变更模式
    #[serde(default)]
    pub change_patterns: Vec<ChangePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePattern {
    /// 模式名称
    pub pattern: String,
    /// 模式描述
    pub description: String,
    /// 涉及的关键目录/模块
    #[serde(default)]
    pub involved_modules: Vec<String>,
}
```

**涉及文件**：

- `crates/services/src/summary/prompt/summary.md` — 增加模式识别引导
- `crates/domain/src/summary/entity.rs` — 增加 `ChangePattern` struct

**复杂度**：中等（prompt 设计需要调优以确保 LLM 准确识别模式）

---

### 改进 6：清理冗余文件

**问题**：`batch/batch_analyze.rs` 包含了与 `batch/conversation.rs` + `batch/service.rs` 功能重复的旧版代码（内联的 `BatchAnalyzeConversation` 和 helper 函数）。`batch/mod.rs` 未引用此文件，是重构遗留物。

**修改方案**：删除 `crates/services/src/summary/batch/batch_analyze.rs`。

**涉及文件**：

- `crates/services/src/summary/batch/batch_analyze.rs` — 删除

**复杂度**：极低

---

### 改进 7：配置/文档文件差异化处理优化

**问题**：`ConfigAnalyzeService` 当前对 `documentation` 类文件和 `configuration` 类文件使用相同的处理方式——都发送完整 diff。但文档文件（如 README.md、CONTRIBUTING.md）的 diff 通常很长且信息密度低，消耗大量 token 但对代码分析价值不大。

**说明**：此改进在 `branch-summary-improvement.md` 的改进 4 中已有描述。确认当前代码是否已实现文档文件的轻量化处理。如已实现则跳过，如未实现则按该文档方案执行。

**涉及文件**：

- `crates/services/src/summary/config/service.rs` — 分离配置/文档处理逻辑
- `crates/services/src/summary/prompt/analyze_config.md` — 调整指引

**复杂度**：低

---

## 三、实施优先级

按价值/成本比排序：

| 优先级 | 改进项 | 价值 | 成本 | 理由 |
|--------|--------|------|------|------|
| **P0** | 1. Stage2 并行执行 | 极高 | 低 | 性能提升 3-4x，改动集中在一个方法 |
| **P0** | 6. 清理冗余文件 | 低 | 极低 | 一键删除，消除代码困惑 |
| **P1** | 2. 目录聚类维度 | 高 | 中 | 大型 PR 总结质量大幅提升 |
| **P1** | 3. 采样策略优化 | 中 | 低 | 改动集中在一个函数，提升批量分析准确性 |
| **P2** | 4. 提交历史演进分析 | 中 | 低 | 仅改 prompt，无代码变更 |
| **P2** | 5. 变更模式识别 | 高 | 中 | prompt 设计需要调优和验证 |
| **P2** | 7. 文档轻量化处理 | 中 | 低 | 参照已有方案实施 |

### 建议实施路径

```
Phase 1（快速见效）
├─ 改进 1: Stage2 并行执行
└─ 改进 6: 清理冗余文件
    ↓
Phase 2（质量提升）
├─ 改进 2: 目录聚类维度
├─ 改进 3: 采样策略优化
└─ 改进 7: 文档轻量化
    ↓
Phase 3（深度优化）
├─ 改进 4: 提交历史演进分析
└─ 改进 5: 变更模式识别
```

---

## 四、涉及文件汇总

```
# Domain 层（实体定义）
crates/domain/src/summary/entity.rs                    → 改进 2、5

# Services 层 — 核心编排
crates/services/src/summary/service.rs                 → 改进 1、2

# Services 层 — 子服务
crates/services/src/summary/batch/service.rs           → 改进 3
crates/services/src/summary/batch/batch_analyze.rs     → 改进 6（删除）
crates/services/src/summary/classify/service.rs        → 改进 2
crates/services/src/summary/config/service.rs          → 改进 7

# Prompt 模板
crates/services/src/summary/prompt/classify_files.md   → 改进 2
crates/services/src/summary/prompt/analyze_config.md   → 改进 7
crates/services/src/summary/prompt/summary.md          → 改进 4、5
```

总计 **9 个文件**（3 个 prompt 模板 + 5 个 Rust 源文件 + 1 个删除），不改变现有三阶段 pipeline 架构。

---

## 五、与已有优化方案的关系

本文档与 `branch-summary-improvement.md` 是互补关系：

| 维度 | branch-summary-improvement | 本文档 |
|------|----------------------------|--------|
| 关注点 | 数据采集层（输入质量） | 分析策略层（处理效率与质量） |
| 核心改进 | 提交历史链、未提交变更、行为差异 | 并行执行、目录聚类、采样策略、模式识别 |
| 重叠项 | 提交历史链（改进 1）≈ 本文档改进 4 的基础 | 文档轻量化（改进 7）≈ 该文档改进 4 |

建议两个文档的重叠项统一实施，避免重复工作。

---

**创建时间**: 2026-02-08
**最后更新**: 2026-02-08
