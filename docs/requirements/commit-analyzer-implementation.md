# 提交分析功能：基于现有代码的实现分析

本文档基于 [commit-analyzer-design.md](./commit-analyzer-design.md) 的流程设计，分析在当前项目中如何落地实现：**哪些能力可直接复用、哪些需要新增**。不包含具体代码实现。

---

## 一、可直接复用的部分

### 1. 阶段一输入数据（文件元信息）

| 需求文档字段 | 项目中的对应 | 说明 |
|-------------|--------------|------|
| `commit_id` | `CommitInfo.sha` | 已有 |
| `author` | `CommitInfo.author_email` / `author_name` | 已有 |
| `timestamp` | `CommitInfo.author_time` / `committer_time`（需转 ISO8601） | 已有，需格式化 |
| `files[].path` | `CommitFileChange.path` | 已有 |
| `files[].status` | `CommitFileChange.change_type` | 已有，需映射为字符串（见下） |
| `files[].additions` / `deletions` | `CommitFileChange.additions` / `deletions`（Option） | 已有，storage 已填 |
| `files[].old_path` | `CommitFileChange.old_path` | 已有（重命名） |

**数据获取流程可直接用：**

- `GitRepository::get_commit_info(ref_or_sha)` → `CommitInfo`
- `GitRepository::get_commit_changed_files(ref_or_sha)` → `Vec<CommitFileChange>`

组合两者即可构造阶段一的「文件列表 + 提交元数据」输入，无需改 domain/storage 接口。  
唯一要做的是：把 `CommitChangeType` 映射成设计里的 `"added"` / `"modified"` / `"deleted"` / `"renamed"` 等字符串（以及可选 `"copied"` / `"type_changed"`）。

---

### 2. 阶段二所需的 Diff 数据

| 需求 | 项目中的能力 | 说明 |
|------|--------------|------|
| 整次 commit 的 diff | `GitRepository::get_commit_diff(ref_or_sha)` → `Option<String>` | 已有，返回完整 patch |
| 按文件拆分的 diff | 无单独 API | 需在应用层按 `diff --git a/... b/...` 切分整段 patch，或新增「按路径取 diff」接口 |

设计里阶段二是「按文件/按组」给 LLM 看 diff（批量抽样、核心逻辑、配置等）。当前有「整 commit 的 diff」，没有「单文件 diff」接口，但可以通过**解析完整 patch** 得到每个文件的 diff 片段，无需必改 storage（若以后要性能或过滤，再考虑在 `DiffService` 加按 path 过滤）。

---

### 3. LLM 调用与解析体系

可直接沿用现有模式：

- **领域接口**：`domain::LLMRepository`（在 services 层被依赖）。
- **实现与对话**：`storage::llm` 的 `LLMService` + 各种 `*Conversation`。
- **对话模式**：`LLMConversation`（system/user prompt、`get_execution_params`、`parse_response`）。
- **JSON 输出**：`JsonParser::extract_json` + `JsonParser::to_model<T>` / `to_map`，适合阶段一、二、三的结构化 JSON。
- **已有类似能力**：
  - `generate_commit_message(changes)`：一次 diff 生成一条 commit message，可视为「单阶段、简化版」；三阶段是它的扩展。
  - `summarize_file_change(file_path, file_diff)`：单文件总结，可复用于阶段二「核心逻辑」或「单文件深度分析」。
  - `create_pr_content` / `summarize_pr` / `reword_pr`：都是「结构化输入 + 可选 JSON」；与阶段三的「全局总结 + 影响分析」思路一致，可复用同一套 client/限流/错误处理。

因此：**阶段一（分类）、阶段二（分类分析）、阶段三（全局总结）都可以做成新的 Conversation + 新 LLMRepository 方法（或新服务接口），沿用现有 LLM 基础设施。**

---

### 4. 应用层入口与 Git 访问

- `app/src/commands/commit/` 下已有 `diff.rs`、`files.rs`，分别演示 `get_commit_diff` 和 `get_commit_changed_files`。
- 通过 `get_git_repository()` 拿到 `GitRepository`，即可在同一命令或新命令里串联：  
  `get_commit_info` + `get_commit_changed_files` → 阶段一输入；`get_commit_diff` → 阶段二/三的 diff 来源。

这些都可以直接作为「提交分析」命令的入口和数据源。

---

## 二、需要补齐或新增的部分

### 1. 领域层（domain）

| 缺失项 | 说明 |
|--------|------|
| **阶段一输出类型** | 设计中的「分类结果」JSON（`categories`、`patterns`、`analysis_strategy`、`summary`）在 domain 中没有对应实体；需新增例如 `CommitFileClassification`（或按你命名习惯）及子结构，供 services/storage 返回和后续阶段使用。 |
| **阶段二各分析结果类型** | 批量分析、核心逻辑分析、配置/文档分析、测试分析各自有设计中的 JSON 结构；需在 domain 中定义对应实体（如 `CommitBatchAnalysis`、`CommitLogicAnalysis` 等），便于类型安全和跨层传递。 |
| **阶段三输出类型** | 设计中的「commit_message + structured_summary + impact_analysis + statistics + metadata」在 domain 中没有统一类型；需新增一个「提交分析结果」聚合类型（例如 `CommitAnalysisResult`），包含 title/body/footer、关键变更、影响分析、统计信息等。 |
| **LLM 接口扩展** | `LLMRepository` 目前只有 `generate_commit_message(changes)`。要实现三阶段，需要新增例如：`classify_commit_files(...)`、`analyze_commit_batch(...)`、`analyze_commit_logic(...)`、`analyze_commit_config(...)`、`analyze_commit_tests(...)`、`summarize_commit_analysis(...)` 等（或合并为少量粗粒度方法），入参/出参使用上面新增的 domain 类型。 |

不要求「一次全做」：可以按阶段一 → 阶段二 → 阶段三逐步加实体和接口。

---

### 2. 存储 / LLM 实现层（storage）

| 缺失项 | 说明 |
|--------|------|
| **阶段一 Conversation + Prompt** | 新建一个「文件分类」Conversation：输入 = 序列化后的文件列表 + 提交元数据（与设计中的 JSON 一致）；输出 = 阶段一 JSON；用 `JsonParser::to_model` 反序列化到 domain 的「分类结果」类型。Prompt 可直接采用设计文档 3.3 的模板（或精简版）。 |
| **阶段二多路 Conversation + Prompt** | 设计中有 2.1 批量、2.2 核心逻辑、2.3 配置文档、2.4 测试四类；每类对应一个 Conversation + 一个 prompt 模板（或共用一个模板通过参数区分），输出用 JSON 解析到对应 domain 实体。 |
| **阶段三 Conversation + Prompt** | 输入 = 阶段一 + 阶段二的结果（+ 统计）；输出 = 设计中的阶段三 JSON；同样用 `JsonParser::to_model` 解析到 `CommitAnalysisResult`。 |
| **按文件拆分 diff** | 若不在 git 层加「按 path 的 diff」接口，则需在调用 LLM 前，从 `get_commit_diff()` 返回的完整 patch 中，按 `diff --git a/x b/x` 拆成「路径 → diff 片段」的 map，供阶段二按「重点分析组 / 抽样文件」传入对应 diff。可放在 app 或 services 的工具函数中。 |
| **并发与策略** | 设计中的「阶段二并行分析」：当前 LLM 调用是同步的；若要对「批量 / 核心 / 配置 / 测试」并行，需要在 services 层用 `tokio` 或线程池并发调用多个 `LLMRepository` 方法，并聚合结果后再调阶段三。 |

---

### 3. 服务层（services）

| 缺失项 | 说明 |
|--------|------|
| **提交分析用例/服务** | 没有「编排三阶段」的用例：先取 commit 信息 + 文件列表 → 调阶段一 → 根据分类结果决定调用阶段二的哪些分支（以及传入哪些文件/抽样）→ 收集阶段二结果 → 调阶段三。需要新建一个「CommitAnalyzerService」或类似，依赖 `GitRepository` 和 `LLMRepository`，实现上述流程。 |
| **策略与分支逻辑** | 根据阶段一的 `analysis_strategy`（批量处理组、重点分析组、可跳过组）决定：哪些文件走批量抽样、哪些走核心逻辑分析、哪些走配置/测试、哪些跳过；需要明确「同一文件归属多类」时的优先级（例如优先「核心逻辑」）。 |
| **错误与降级** | 设计未定义：某阶段 LLM 超时/限流/JSON 解析失败时是否重试、是否降级为「仅阶段一 + 简单 generate_commit_message」等；建议在 services 层明确策略（至少：失败返回错误 vs 降级）。 |

---

### 4. 应用层（app）

| 缺失项 | 说明 |
|--------|------|
| **新命令或子命令** | 将「提交分析」暴露为子命令（例如 `workflow commit analyze [ref]`），调用上面的 CommitAnalyzerService，接收 ref（默认 HEAD），输出设计中的结构化结果（或只输出 commit message / Markdown 报告）。 |
| **输出格式** | 设计支持 JSON / Markdown / Git Message；当前没有统一展示层，需要根据子命令参数（如 `--format=json|markdown|message`）格式化并打印。 |

---

### 5. 与设计文档的细微差异

| 点 | 说明 |
|----|------|
| **status 字符串** | 设计用 `"added"`/`"modified"`/`"deleted"`/`"renamed"`；项目用 `CommitChangeType` 枚举。在组装阶段一输入 JSON 时做一次映射即可，无需改 domain 枚举。 |
| **TypeChanged / Copied** | 设计未单独列，但 domain 有；可映射为 `"modified"` 或单独 `"type_changed"`/`"copied"`，在 prompt 里简单说明即可。 |
| **单文件 diff** | 设计按文件给 diff；项目只有整 commit diff。用「解析完整 patch 按文件切分」即可满足实现，无需立刻改 Git 接口。 |

---

## 三、小结表

| 层次 | 可直接使用 | 缺失/需新增 |
|------|------------|-------------|
| **domain** | `CommitInfo`、`CommitFileChange`、`CommitChangeType`；`LLMRepository` 及现有方法 | 阶段一/二/三的结果实体；`LLMRepository` 上新方法（分类、多类分析、全局总结） |
| **storage (git)** | `get_commit_info`、`get_commit_changed_files`、`get_commit_diff` | 可选：按路径返回 diff（非必须，可先解析 patch） |
| **storage (llm)** | `LLMConversation`、`JsonParser`、现有 client/服务/对话模式 | 阶段一/二/三的 Conversation + prompt 模板；新 LLMService 方法对接新 Conversation |
| **services** | 无现成「提交分析」用例 | 新增 CommitAnalyzerService：编排三阶段、策略分支、错误/降级 |
| **app** | `get_git_repository()`、commit 相关命令结构 | 新子命令（如 `commit analyze`）、输出格式化（JSON/Markdown/message） |

---

## 四、实施顺序建议

1. **domain**：定义阶段一/二/三的实体与 `LLMRepository` 新方法签名。
2. **storage (llm)**：实现各阶段 Conversation + prompt，并在 LLMService/LLMRepository 中对接。
3. **services**：实现 CommitAnalyzerService（编排三阶段、策略分支、可选并发与降级）。
4. **app**：新增 `commit analyze` 子命令与输出格式（JSON/Markdown/message）。
5. **diff 按文件切分**：在 services 或 app 侧用工具函数从完整 patch 切分；若后续有性能需求再考虑在 storage 的 DiffService 增加按 path 的接口。

---

## 参考

- 流程与 Prompt 设计：[commit-analyzer-design.md](./commit-analyzer-design.md)
- 需求文档索引：[README.md](./README.md)
