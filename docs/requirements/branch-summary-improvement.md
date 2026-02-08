# 分支总结流程改进分析

基于 `docs/analysis/branch-summary-workflow.md`（人工流程文档）与 `crates/services/src/summary/` （代码实现）的对比分析，识别出 5 个可参考文档流程进行改进的方向。

---

## 一、现状对比

### 文档流程（V1 人工流程）

```
确定基准分支 → 收集基础信息 → 逐文件查看 diff → 撰写总结
```

- 线性、平铺式，面向人工操作
- 采集维度丰富（提交历史、未提交变更、分叉点）
- 分析策略单一（所有文件一视同仁）
- 输出为自由 Markdown，按功能域归纳

### 代码实现（V2 自动化 pipeline）

```
prepare(数据采集) → Stage1(文件分类) → Stage2(4路分类分析) → Stage3(全局总结)
```

- 分治式，面向 LLM 自动化
- 采集维度有缺失（只取 HEAD commit，忽略工作区状态）
- 分析策略精细（按文件性质分 4 路，批量操作只采样）
- 输出为结构化 JSON，按文件类型归纳

### 核心差异

| 维度 | 文档流程 | 代码实现 |
|------|---------|---------|
| 提交历史 | `git log BASE..HEAD` 完整采集 | 仅 `get_commit_info("HEAD")` 单条 |
| 工作区状态 | `git status --short` 纳入考虑 | 完全忽略 |
| 分析深度 | 关注「与基准行为差异」 | 关注「做了什么、怎么做的」 |
| 非业务文件 | 轻量处理（列出 + 用途简述） | 全量 diff 发送 LLM |
| 输出归纳方式 | 按功能域（跨文件类型） | 按文件类型（features/fixes/refactors/...） |

---

## 二、改进方案

### 改进 1：引入提交历史链

**问题**：`prepare()` 只获取 HEAD 单条 commit，Stage3 LLM 无法理解变更的演进脉络。

**文档中的做法**：

```bash
git log BASE..HEAD --oneline
```

采集分支上的完整提交列表，包含提交数量和每条提交的 subject。

**可用接口**：`GitRepository` trait 已有 `commits_to_merge(source, target) -> Vec<String>` 方法。

**修改方案**：

1. `AnalysisContext` 增加字段：

```rust
struct AnalysisContext {
    // ... 现有字段 ...
    /// 分支提交历史（从旧到新）
    commit_history: Vec<CommitInfo>,
    /// 提交总数
    commit_count: u32,
}
```

2. `prepare()` 中采集提交历史：

```rust
// 获取 commit SHA 列表
let commit_shas = self.git_repo
    .commits_to_merge(&current_branch, &base_branch)?;
let commit_count = commit_shas.len() as u32;

// 逐条获取完整信息（超过 50 条时截断）
let max_history = 50;
let commit_history: Vec<CommitInfo> = commit_shas.iter()
    .take(max_history)
    .filter_map(|sha| self.git_repo.get_commit_info(sha).ok())
    .collect();
```

3. `SummaryAnalyzeInput` 增加字段：

```rust
pub(crate) struct SummaryAnalyzeInput {
    // ... 现有字段 ...
    pub commit_history_summary: String,
    pub commit_count: u32,
}
```

4. `summary/conversation.rs` 的 user prompt 增加段落：

```text
### 提交历史（共 {commit_count} 条）
{commit_history_summary}
```

5. `summary.md` prompt 增加指引：要求 LLM 参考提交历史理解变更演进，判断是「一次性大改」还是「逐步迭代」。

**涉及文件**：

- `crates/services/src/summary/service.rs` — prepare() 增加数据采集
- `crates/services/src/summary/summary/input.rs` — 增加字段
- `crates/services/src/summary/summary/conversation.rs` — user prompt 增加段落
- `crates/services/src/summary/prompt/summary.md` — 增加指引

**复杂度**：中等

---

### 改进 2：未提交变更警告

**问题**：用户在提 PR 前运行总结时，工作区可能还有未暂存的修改，当前流程完全忽略。

**文档中的做法**：

```bash
git status --short
```

将未提交变更一并列出，确保总结与实际状态一致。

**可用接口**：`GitRepository` trait 已有 `get_working_tree_status() -> WorkingTreeStatus` 方法。

**修改方案**：

1. `prepare()` 中增加检查：

```rust
let working_status = self.git_repo.get_working_tree_status()
    .map_err(|e| ServiceError::Other(format!("获取工作区状态失败: {}", e)))?;
let has_uncommitted = !working_status.is_clean();
```

2. `AnalysisContext` 增加标记：

```rust
struct AnalysisContext {
    // ... 现有字段 ...
    has_uncommitted_changes: bool,
}
```

3. 传递到 Stage3，在 user prompt 中条件性增加提示：

```text
### 注意
⚠ 工作区存在未提交的变更，以下总结仅覆盖已提交部分。
```

**涉及文件**：

- `crates/services/src/summary/service.rs` — prepare() 增加状态检查
- `crates/services/src/summary/summary/input.rs` — 增加字段
- `crates/services/src/summary/summary/conversation.rs` — 条件性增加提示

**复杂度**：低

---

### 改进 3：行为差异分析维度

**问题**：`analyze_logic.md` prompt 有 6 个分析维度，但没有要求 LLM 归纳「修改前后的行为差异」。

**文档中的做法**（步骤 4.3）：

> 每个功能块内写清：**与基准行为的差异**（例如：原先只发客户端，现在在后端执行并传 session_id）

**修改方案**：

1. `analyze_logic.md` prompt 增加第 7 个分析维度：

```markdown
7. **行为差异**：基于 diff 的 -/+ 行，归纳该文件修改前后的行为变化
   - before: 修改前的行为描述
   - after: 修改后的行为描述
   - reason: 为什么要做这个行为变更
```

2. `LogicFileAnalysis` entity 增加字段：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicFileAnalysis {
    // ... 现有字段 ...
    #[serde(default)]
    pub behavior_diff: BehaviorDiff,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BehaviorDiff {
    #[serde(default)]
    pub before: String,
    #[serde(default)]
    pub after: String,
    #[serde(default)]
    pub reason: String,
}
```

3. `analyze_logic.md` 的 JSON 输出格式增加对应字段：

```json
{
  "files": [
    {
      // ... 现有字段 ...
      "behavior_diff": {
        "before": "修改前的行为描述",
        "after": "修改后的行为描述",
        "reason": "行为变更的原因"
      }
    }
  ]
}
```

**涉及文件**：

- `crates/services/src/summary/prompt/analyze_logic.md` — 增加分析维度
- `crates/domain/src/summary/entity.rs` — 增加 BehaviorDiff struct

**复杂度**：低。改动面最小，但对审查者的价值最高。

---

### 改进 4：文档文件轻量化处理

**问题**：`ConfigAnalyzeService` 对 `documentation` 类文件也发送完整 diff，浪费 token。

**文档中的做法**（步骤 3）：

> 对**非业务文件**（如 `.cursor/skills/*.md`、文档）：仅需在总结中列出「新增/修改了哪些文件、用途简述」。

**修改方案**：

在 `config/service.rs` 中分离 `configuration` 和 `documentation` 的处理逻辑：

```rust
pub fn analyze(...) -> Result<String, ServiceError> {
    let config_paths: Vec<&String> = stage1.categories.by_nature.configuration.iter().collect();
    let doc_paths: Vec<&String> = stage1.categories.by_nature.documentation.iter().collect();

    if config_paths.is_empty() && doc_paths.is_empty() {
        return Ok("{}".to_string());
    }

    // 配置文件：保持现有逻辑，发送完整 diff
    let mut config_parts = String::new();
    for path in &config_paths {
        let additions = files.iter().find(|f| f.path == **path)
            .and_then(|f| f.additions).unwrap_or(0);
        let deletions = files.iter().find(|f| f.path == **path)
            .and_then(|f| f.deletions).unwrap_or(0);
        let diff = file_diffs.get(*path).map(String::as_str).unwrap_or("");
        config_parts.push_str(&format!(
            "\n### {}\n变更：+{} -{}\n\n```diff\n{}\n```\n\n---\n",
            path, additions, deletions, diff
        ));
    }

    // 文档文件：只发送路径 + 变更统计（不发完整 diff）
    let mut doc_parts = String::new();
    for path in &doc_paths {
        let additions = files.iter().find(|f| f.path == **path)
            .and_then(|f| f.additions).unwrap_or(0);
        let deletions = files.iter().find(|f| f.path == **path)
            .and_then(|f| f.deletions).unwrap_or(0);
        let status = if additions > 0 && deletions == 0 { "新增" }
                     else if additions == 0 && deletions > 0 { "删除" }
                     else { "修改" };
        doc_parts.push_str(&format!("- {} [{}] (+{} -{})\n", path, status, additions, deletions));
    }

    let user_prompt = format!(
        "## 配置文件变更\n{}\n\n## 文档文件变更（仅列表，无需深入分析内容）\n{}\n",
        config_parts, doc_parts
    );
    // ...
}
```

同时微调 `analyze_config.md` prompt，告诉 LLM 文档文件只有摘要信息，只需归纳「新增/修改了什么文档」。

**涉及文件**：

- `crates/services/src/summary/config/service.rs` — 分离处理逻辑
- `crates/services/src/summary/prompt/analyze_config.md` — 微调指引

**复杂度**：低

---

### 改进 5：按功能域归纳

**问题**：Stage3 输出的 `details_by_category` 按文件类型分（features/fixes/refactors/...），无法揭示跨类型变更的统一意图。

**文档中的做法**（步骤 4.3）：

> 不按「文件」平铺，而是按**功能域**归纳（例如：AM 日历接入、日历 update、错误处理、工具链配置）。

**修改方案**：

1. `StructuredSummary` entity 增加字段：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredSummary {
    // ... 现有字段 ...
    #[serde(default)]
    pub changes_by_domain: Vec<FeatureDomain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDomain {
    /// 功能域名称（如 "HTTP 客户端统一化"、"LLM 集成"）
    #[serde(default)]
    pub domain: String,
    /// 该功能域的目的
    #[serde(default)]
    pub purpose: String,
    /// 涉及的文件路径
    #[serde(default)]
    pub files: Vec<String>,
    /// 跨类型的变更描述
    #[serde(default)]
    pub changes: Vec<String>,
}
```

2. `summary.md` prompt 增加输出段和指引：

```json
"changes_by_domain": [
  {
    "domain": "功能域名称",
    "purpose": "该功能域的整体目的",
    "files": ["涉及的文件路径列表"],
    "changes": ["该功能域下的变更描述，可跨 features/config/tests 等类别"]
  }
]
```

指引 LLM：从 Stage2 的 4 路分析结果中识别出 2-5 个功能域，将相关的业务代码、配置、测试变更聚合到同一个域下。与 `details_by_category` 互补，不重复。

**涉及文件**：

- `crates/domain/src/summary/entity.rs` — 增加 FeatureDomain struct
- `crates/services/src/summary/prompt/summary.md` — 增加输出段和指引

**复杂度**：中等。prompt 设计需要引导 LLM 正确识别功能域边界。

---

## 三、实施优先级

按价值/成本比排序：

| 优先级 | 改进项 | 价值 | 成本 | 理由 |
|--------|--------|------|------|------|
| **P0** | 3. 行为差异分析 | 高 | 低 | 改一个 prompt + 一个 entity，审查者直接受益 |
| **P0** | 2. 未提交变更警告 | 中 | 极低 | 几行代码，防止产出误导性总结 |
| **P1** | 1. 提交历史链 | 高 | 中 | 需要数据采集和传递，但能显著提升总结质量 |
| **P1** | 4. 文档轻量化 | 中 | 低 | 节省 token 成本，大变更场景效果明显 |
| **P2** | 5. 功能域归纳 | 高 | 中 | prompt 设计有挑战，但输出维度最有价值 |

建议先完成 P0（改进 2 + 3），快速验证效果后再推进 P1 和 P2。

---

## 四、涉及文件汇总

```
# Domain 层（实体定义）
crates/domain/src/summary/entity.rs             → 改进 3、5

# Services 层 — 核心编排
crates/services/src/summary/service.rs           → 改进 1、2

# Services 层 — 子服务
crates/services/src/summary/config/service.rs    → 改进 4

# Services 层 — 输入/对话
crates/services/src/summary/summary/input.rs     → 改进 1、2
crates/services/src/summary/summary/conversation.rs → 改进 1、2

# Prompt 模板
crates/services/src/summary/prompt/analyze_logic.md  → 改进 3
crates/services/src/summary/prompt/analyze_config.md → 改进 4
crates/services/src/summary/prompt/summary.md        → 改进 1、5
```

总计 **8 个文件**（3 个 prompt 模板 + 5 个 Rust 源文件），不影响现有三阶段 pipeline 架构。

---

**最后更新**: 2026-02-08
