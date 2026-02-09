# 单次提交 Commit Message 生成方案

**状态**: ⏳ 待实施
**实现度**: 0%
**优先级**: 高
**分类**: Commit 功能增强
**创建日期**: 2026-02-09

---

## 📋 目录

- [需求背景](#-需求背景)
- [现状分析](#-现状分析)
- [方案设计](#-方案设计)
- [技术实现](#-技术实现)
- [API 设计](#-api-设计)
- [使用示例](#-使用示例)
- [实施计划](#-实施计划)
- [风险评估](#-风险评估)

---

## 🎯 需求背景

### 问题描述

当前 `crates/services/src/summary` 模块实现了三阶段的 commit 分析流程，专门针对**分支合并场景**（多提交）设计：

```
准备阶段 → 阶段一（文件分类） → 阶段二（分类分析） → 阶段三（全局总结）
              1 次 LLM              4 次 LLM 并行              1 次 LLM
```

**问题**：
- ❌ 对于**单次提交**场景（日常 `git commit`），三阶段流程过于复杂
- ❌ 需要 5-6 次 LLM 调用，耗时 20-30 秒，成本 $0.05-0.25
- ❌ 用户日常提交时不需要如此深度的分析

### 核心需求

提供一个**轻量级的单次提交 commit message 生成服务**：
- ✅ 单次 LLM 调用（~5-10 秒，成本 $0.01-0.05）
- ✅ 支持处理大量文件（50-100+ 个文件）
- ✅ 输出结构化、高质量的 commit message
- ✅ 复用现有的 prompt 设计和 JSON schema
- ✅ 保留三阶段流程用于分支合并场景

---

## 📊 现状分析

### 现有架构

#### 1. 三阶段分析流程 (`CommitSummaryServiceImpl`)

**适用场景**：分支合并（多提交）

```rust
// crates/services/src/summary/service.rs
impl CommitSummaryServiceImpl {
    fn run_analysis(&self, base_branch: Option<&str>) -> Result<CommitSummaryAnalysis> {
        let ctx = self.prepare(base_branch)?;        // 准备：获取 merge diff
        let stage1 = self.run_stage1(&ctx)?;         // 阶段一：文件分类
        let stage2 = self.run_stage2(&ctx, &stage1)?; // 阶段二：4 个并行分析
        // 阶段三：全局总结
        SummaryAnalyzeService::new(self.llm_executor.clone())
            .summarize(input, &ctx.language_code)
    }
}
```

**核心方法**：
- `get_merge_changed_files(&current_branch, &base_branch)` - 获取两分支差异
- `commits_to_merge()` - 获取提交历史
- `get_merge_diff()` - 获取完整 merge diff

**优势**：
- ✅ 深度分析多提交的演进过程
- ✅ 并行执行子服务提升性能
- ✅ 适合大型重构、功能分支合并

**劣势**：
- ❌ 对单次提交来说过于复杂
- ❌ 调用成本高（5-6 次 LLM）

#### 2. Prompt 设计

现有的 `crates/services/src/summary/prompt/summary.md` 提供了高质量的 prompt 模板：
- ✅ 结构化输出（Conventional Commits 格式）
- ✅ 多维度分析（features/fixes/refactors/config/docs/tests）
- ✅ 影响分析和风险评估
- ✅ 支持多语言输出

**复用价值**：该 prompt 可以简化后直接用于单次提交场景。

---

## 🎨 方案设计

### 设计原则

1. **简单优先**：单次 LLM 调用，避免过度工程
2. **复用优先**：复用现有 prompt 和 schema，减少维护成本
3. **清晰边界**：区分"单次提交"和"分支合并"两种场景
4. **扩展性**：为未来的增强功能预留接口

### 方案概览

#### 方案：智能单次 LLM 调用

**核心思路**：
- 利用现代 LLM 的大上下文窗口（Claude Sonnet 200K tokens）
- 在一次调用中完成文件分类、变更分析、message 生成
- 通过智能 prompt 指导 LLM 自动完成结构化分析

**流程图**：

```
┌─────────────────────────────────────────────────────────────┐
│                   CommitMessageService                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
         ┌────────────────────────────────────────┐
         │  1. 获取变更数据（无 LLM 调用）        │
         │     - get_staged_files() 或            │
         │     - get_commit_changed_files()       │
         │     - get_staged_diff() 或             │
         │     - get_commit_diff()                │
         └────────────────────────────────────────┘
                              │
                              ▼
         ┌────────────────────────────────────────┐
         │  2. 格式化输入数据                     │
         │     - 文件列表摘要                     │
         │     - Diff 内容（智能截断）            │
         └────────────────────────────────────────┘
                              │
                              ▼
         ┌────────────────────────────────────────┐
         │  3. 单次 LLM 调用                      │
         │     Prompt 包含：                      │
         │     - 文件分类指导                     │
         │     - 变更模式识别                     │
         │     - 结构化输出 schema                │
         └────────────────────────────────────────┘
                              │
                              ▼
         ┌────────────────────────────────────────┐
         │  4. 解析 JSON 结果                     │
         │     → CommitSummaryAnalysis            │
         └────────────────────────────────────────┘
```

**对比三阶段流程**：

| 维度 | 单次 LLM 调用 | 三阶段流程 |
|------|--------------|-----------|
| **LLM 调用次数** | 1 次 | 5-6 次 |
| **耗时** | ~5-10 秒 | ~20-30 秒 |
| **成本** | $0.01-0.05 | $0.05-0.25 |
| **上下文窗口需求** | ~10-50K tokens | ~5-15K tokens × 6 |
| **准确性** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **复杂度** | 低 | 高 |
| **适用场景** | 单次提交 | 分支合并 |

---

## 🔧 技术实现

### 1. Domain 层：定义服务接口

```rust
// crates/domain/src/commit/service.rs

use crate::errors::ServiceError;
use crate::summary::entity::CommitSummaryAnalysis;

/// Commit Service trait（现有）
pub trait CommitService {
    // ... 现有方法 ...
}

/// Commit Message 生成服务（单次提交场景）
pub trait CommitMessageService {
    /// 为 staged 变更生成 commit message
    ///
    /// 适用于 `git commit` 前的场景，分析当前暂存区的变更。
    fn generate_for_staged(&self) -> Result<CommitSummaryAnalysis, ServiceError>;

    /// 为指定提交生成 commit message
    ///
    /// 适用于分析已有提交的场景，如 `workflow commit analyze <sha>`。
    fn generate_for_commit(&self, commit_ref: &str) -> Result<CommitSummaryAnalysis, ServiceError>;
}
```

```rust
// crates/domain/src/commit/mod.rs
//! Commit 业务域
//!
//! 包含 Commit 相关的实体和服务接口

pub mod entity;
pub mod service;

// Re-export public types
pub use entity::AmendPreview;
pub use service::{CommitMessageService, CommitService};
```

### 2. Services 层：实现服务

#### 文件结构

```
crates/services/src/
├── summary/                 # 现有的三阶段分析服务（保留）
│   ├── mod.rs
│   ├── service.rs           # CommitSummaryServiceImpl（分支合并场景）
│   ├── batch/
│   ├── classify/
│   ├── config/
│   ├── logic/
│   ├── prompt/
│   └── summary/
└── commit/                  # 新增：单次提交服务
    ├── mod.rs
    └── message/             # Commit message 生成服务
        ├── mod.rs
        ├── service.rs       # CommitMessageServiceImpl
        ├── prompt.md        # 简化版 prompt
        └── conversation.rs  # LLM 对话构建器
```

#### 核心服务实现

```rust
// crates/services/src/commit/message/service.rs

use std::sync::Arc;

use domain::commit::CommitMessageService;
use domain::errors::ServiceError;
use domain::git::entity::CommitFileChange;
use domain::summary::entity::CommitSummaryAnalysis;
use domain::GitRepository;
use llm::{LLMConfigContext, LLMExecutor};
use llm::parsers::JsonParser;

use super::CommitMessageConversation;

/// 单次提交 Commit Message 生成服务实现
pub struct CommitMessageServiceImpl {
    git_repo: Arc<dyn GitRepository>,
    llm_executor: Arc<dyn LLMExecutor>,
    llm_context: Arc<dyn LLMConfigContext>,
}

impl CommitMessageServiceImpl {
    pub fn new(
        git_repo: Arc<dyn GitRepository>,
        llm_executor: Arc<dyn LLMExecutor>,
        llm_context: Arc<dyn LLMConfigContext>,
    ) -> Self {
        Self {
            git_repo,
            llm_executor,
            llm_context,
        }
    }

    /// 准备分析上下文（获取变更数据）
    fn prepare_staged(&self) -> Result<AnalysisInput, ServiceError> {
        // 1. 获取 staged 文件列表
        let files = self.git_repo.get_staged_files()
            .map_err(|e| ServiceError::Other(format!("Failed to get staged files: {}", e)))?;

        if files.is_empty() {
            return Err(ServiceError::Other("No staged changes to commit".to_string()));
        }

        // 2. 获取 staged diff
        let diff = self.git_repo.get_staged_diff()
            .map_err(|e| ServiceError::Other(format!("Failed to get staged diff: {}", e)))?
            .unwrap_or_default();

        // 3. 统计信息
        let stats = calculate_statistics(&files);

        Ok(AnalysisInput { files, diff, stats })
    }

    fn prepare_commit(&self, commit_ref: &str) -> Result<AnalysisInput, ServiceError> {
        // 1. 获取提交信息
        let commit_info = self.git_repo.get_commit_info(commit_ref)
            .map_err(|e| ServiceError::Other(format!("Failed to get commit info: {}", e)))?;

        // 2. 获取提交的变更文件
        let files = self.git_repo.get_commit_changed_files(commit_ref)
            .map_err(|e| ServiceError::Other(format!("Failed to get changed files: {}", e)))?;

        // 3. 获取提交的 diff
        let diff = self.git_repo.get_commit_diff(commit_ref)
            .map_err(|e| ServiceError::Other(format!("Failed to get commit diff: {}", e)))?
            .unwrap_or_default();

        // 4. 统计信息
        let stats = calculate_statistics(&files);

        Ok(AnalysisInput { files, diff, stats })
    }

    /// 生成 commit message（核心逻辑）
    fn generate(&self, input: AnalysisInput) -> Result<CommitSummaryAnalysis, ServiceError> {
        // 1. 格式化输入数据
        let file_summary = format_file_summary(&input.files);
        let diff_content = smart_truncate_diff(&input.diff, &input.files, MAX_DIFF_LINES);

        // 2. 构建 LLM 对话
        let language_code = self.llm_context.get_language();
        let conversation = CommitMessageConversation::new(
            file_summary,
            diff_content,
            input.stats,
        );

        // 3. 单次 LLM 调用
        let response = self.llm_executor
            .execute(&conversation, &language_code, "single_commit_message")
            .map_err(|e| ServiceError::Other(e.to_string()))?;

        // 4. 解析结果
        JsonParser::to_model(&response).map_err(|e| {
            ServiceError::Other(format!("Failed to parse commit message: {}", e))
        })
    }
}

impl CommitMessageService for CommitMessageServiceImpl {
    fn generate_for_staged(&self) -> Result<CommitSummaryAnalysis, ServiceError> {
        let input = self.prepare_staged()?;
        self.generate(input)
    }

    fn generate_for_commit(&self, commit_ref: &str) -> Result<CommitSummaryAnalysis, ServiceError> {
        let input = self.prepare_commit(commit_ref)?;
        self.generate(input)
    }
}

// ────────────────────────────────────────────────────────────────
// Helper Types & Functions
// ────────────────────────────────────────────────────────────────

/// 分析输入数据
struct AnalysisInput {
    files: Vec<CommitFileChange>,
    diff: String,
    stats: FileStatistics,
}

/// 文件统计信息
struct FileStatistics {
    total_files: u32,
    added_count: u32,
    modified_count: u32,
    deleted_count: u32,
    renamed_count: u32,
    total_additions: u32,
    total_deletions: u32,
}

/// 最大 diff 行数（避免超出 LLM 上下文窗口）
const MAX_DIFF_LINES: usize = 2000;

/// 计算文件统计信息
fn calculate_statistics(files: &[CommitFileChange]) -> FileStatistics {
    use domain::git::entity::CommitChangeType;

    let mut stats = FileStatistics {
        total_files: files.len() as u32,
        added_count: 0,
        modified_count: 0,
        deleted_count: 0,
        renamed_count: 0,
        total_additions: 0,
        total_deletions: 0,
    };

    for file in files {
        match file.change_type {
            CommitChangeType::Added => stats.added_count += 1,
            CommitChangeType::Deleted => stats.deleted_count += 1,
            CommitChangeType::Modified | CommitChangeType::TypeChanged | CommitChangeType::Copied => {
                stats.modified_count += 1
            }
            CommitChangeType::Renamed => stats.renamed_count += 1,
        }

        stats.total_additions += file.additions.unwrap_or(0);
        stats.total_deletions += file.deletions.unwrap_or(0);
    }

    stats
}

/// 格式化文件列表摘要
fn format_file_summary(files: &[CommitFileChange]) -> String {
    use domain::git::entity::CommitChangeType;

    let mut lines = Vec::new();
    lines.push("## Changed Files\n".to_string());

    for file in files {
        let status = match file.change_type {
            CommitChangeType::Added => "A",
            CommitChangeType::Deleted => "D",
            CommitChangeType::Modified => "M",
            CommitChangeType::Renamed => "R",
            CommitChangeType::Copied => "C",
            CommitChangeType::TypeChanged => "T",
        };

        let additions = file.additions.unwrap_or(0);
        let deletions = file.deletions.unwrap_or(0);

        lines.push(format!(
            "- [{}] {} (+{} -{}) {}",
            status, file.path, additions, deletions,
            infer_file_type(&file.path)
        ));
    }

    lines.join("\n")
}

/// 推断文件类型
fn infer_file_type(path: &str) -> &'static str {
    if path.ends_with("_test.rs") || path.ends_with(".test.ts") || path.contains("/tests/") {
        "[test]"
    } else if path.ends_with(".md") || path.contains("/docs/") {
        "[docs]"
    } else if path.ends_with(".toml") || path.ends_with(".json") || path.ends_with(".yaml") || path.ends_with(".yml") {
        "[config]"
    } else if path.ends_with(".rs") || path.ends_with(".ts") || path.ends_with(".js") || path.ends_with(".go") {
        "[code]"
    } else {
        ""
    }
}

/// 智能截断 diff（保留关键部分）
fn smart_truncate_diff(
    diff: &str,
    files: &[CommitFileChange],
    max_lines: usize,
) -> String {
    let lines: Vec<&str> = diff.lines().collect();

    // 如果 diff 不大，直接返回
    if lines.len() <= max_lines {
        return diff.to_string();
    }

    // 策略：优先保留小文件的完整 diff
    // 大文件只保留前后各 50 行

    // TODO: 实现更智能的截断逻辑
    // 1. 按文件拆分 diff
    // 2. 小文件（< 100 行）保留完整 diff
    // 3. 大文件只保留前后各 50 行 + 函数签名行

    // 当前简化实现：直接截断
    format!(
        "{}\n\n[... Diff truncated: {} lines omitted for brevity ...]",
        lines[..max_lines].join("\n"),
        lines.len() - max_lines
    )
}
```

### 3. Prompt 设计

```markdown
// crates/services/src/commit/message/prompt.md

You are a Git commit message expert. Analyze the provided file changes and diff content to generate a high-quality commit message.

## Input Data

The user will provide:
1. **Changed Files**: List of modified files with change type and line counts
2. **Diff Content**: Full diff of the changes
3. **Statistics**: Overall change statistics

## Analysis Guidelines

### 1. Automatic File Classification

Classify files into categories (do NOT translate category names):
- **Business Logic**: *.rs (non-test), *.ts (non-.test.ts), *.go, *.py, etc.
- **Configuration**: *.toml, *.json, *.yaml, *.config.*, .env, etc.
- **Tests**: *_test.*, *.test.*, tests/**, __tests__/**
- **Documentation**: *.md, docs/**, README, CHANGELOG
- **Dependencies**: Cargo.toml, package.json, go.mod, requirements.txt

### 2. Identify Change Patterns

Recognize common patterns:
- **New Feature**: Mostly new files + business logic code
- **Bug Fix**: Small modifications + test additions
- **Refactoring**: Large modifications but no functional changes
- **Configuration**: Only config file changes
- **Documentation**: Only documentation updates

### 3. Language Requirements

- Use the language specified in the `LANGUAGE REQUIREMENT` section
- Keep these in original form (do NOT translate):
  - Code identifiers: function/variable/class names
  - File paths and filenames
  - Technical terms: library/framework/protocol names

## Output Requirements

### 1. Commit Title
- Follow Conventional Commits: `<type>(<scope>): <subject>`
- Length: ≤ 50 characters
- Types: feat, fix, refactor, docs, style, test, chore, perf
- Subject: lowercase verb phrase

### 2. Commit Body
- Explain WHY (motivation)
- List WHAT changed (key changes)
- Describe HOW (technical approach)

### 3. Structured Summary
- Categorize changes: features/fixes/refactors/config/docs/tests
- Optional: Group by feature domain if multiple related changes exist

### 4. Impact Analysis
- Breaking changes (if any)
- Affected modules
- Risk assessment
- Testing suggestions

## Output Format

Output strictly in JSON format (schema same as multi-commit analysis):

```json
{
  "commit_message": {
    "title": "feat(auth): add OAuth2.0 support",
    "body": "...",
    "footer": "..."
  },
  "structured_summary": {
    "type": "feat",
    "scope": "auth",
    "subject": "add OAuth2.0 support",
    "main_purpose": "...",
    "key_changes": ["...", "..."],
    "details_by_category": {
      "features": ["..."],
      "fixes": [],
      "refactors": [],
      "config": ["..."],
      "docs": [],
      "tests": ["..."],
      "others": []
    },
    "changes_by_domain": [...]
  },
  "impact_analysis": {...},
  "statistics": {...},
  "metadata": {...}
}
```
```

### 4. GitRepository 扩展

```rust
// crates/domain/src/git/repository.rs

pub trait GitRepository {
    // ... 现有方法 ...

    // ────────────────────────────────────────────────────────────
    // 新增：单次提交相关方法
    // ────────────────────────────────────────────────────────────

    /// 获取暂存区文件列表
    ///
    /// 执行 `git diff --cached --name-status --numstat`
    fn get_staged_files(&self) -> Result<Vec<CommitFileChange>, GitError>;

    /// 获取暂存区完整 diff
    ///
    /// 执行 `git diff --cached`
    fn get_staged_diff(&self) -> Result<Option<String>, GitError>;

    /// 获取指定提交的变更文件列表
    ///
    /// 执行 `git show --name-status --numstat <commit>`
    fn get_commit_changed_files(&self, commit_ref: &str) -> Result<Vec<CommitFileChange>, GitError>;

    /// 获取指定提交的完整 diff
    ///
    /// 执行 `git show <commit>` 或 `git diff <commit>^..<commit>`
    fn get_commit_diff(&self, commit_ref: &str) -> Result<Option<String>, GitError>;
}
```

```rust
// crates/infrastructure/src/git/repository.rs

impl GitRepository for GitRepositoryImpl {
    fn get_staged_files(&self) -> Result<Vec<CommitFileChange>, GitError> {
        // 实现逻辑：解析 git diff --cached --name-status --numstat
        // 复用现有的 parse_diff_files 逻辑
        todo!()
    }

    fn get_staged_diff(&self) -> Result<Option<String>, GitError> {
        // 实现逻辑：执行 git diff --cached
        todo!()
    }

    fn get_commit_changed_files(&self, commit_ref: &str) -> Result<Vec<CommitFileChange>, GitError> {
        // 实现逻辑：解析 git show --name-status --numstat <commit>
        todo!()
    }

    fn get_commit_diff(&self, commit_ref: &str) -> Result<Option<String>, GitError> {
        // 实现逻辑：执行 git show <commit>
        todo!()
    }
}
```

---

## 📦 API 设计

### CLI 命令

#### 1. `workflow commit generate` - 为 staged 变更生成 commit message

```bash
# 基本用法
workflow commit generate

# 指定输出格式
workflow commit generate --format json
workflow commit generate --format plain

# 自动提交
workflow commit generate --auto-commit

# 指定语言
workflow commit generate --language zh-CN
```

**行为**：
1. 检查暂存区是否有变更
2. 调用 `CommitMessageService::generate_for_staged()`
3. 输出生成的 commit message
4. 如果 `--auto-commit`，自动执行 `git commit`

#### 2. `workflow commit analyze` - 分析指定提交

```bash
# 分析 HEAD
workflow commit analyze

# 分析指定提交
workflow commit analyze abc1234

# 输出为 JSON
workflow commit analyze abc1234 --format json
```

**行为**：
1. 调用 `CommitMessageService::generate_for_commit()`
2. 输出详细的结构化分析

### 服务层 API

```rust
// crates/app/src/registry.rs

use domain::commit::CommitMessageService;
use services::commit::message::CommitMessageServiceImpl;

// 在 registry 中注册服务
pub fn get_commit_message_service() -> Arc<dyn CommitMessageService> {
    // 单例模式
    REGISTRY.commit_message_service.get_or_init(|| {
        Arc::new(CommitMessageServiceImpl::new(
            get_git_repository(),
            get_llm_executor(),
            get_llm_config_context(),
        ))
    }).clone()
}
```

---

## 💡 使用示例

### 场景 1：日常提交

```bash
# 1. 修改代码
vim src/main.rs

# 2. 暂存变更
git add src/main.rs

# 3. 生成 commit message
workflow commit generate

# 输出：
# ┌─ Generated Commit Message ─────────────────────────────┐
# │                                                         │
# │  fix(main): correct error handling in startup process  │
# │                                                         │
# │  Fixed a bug where panics during startup were not      │
# │  properly caught, causing unclear error messages.      │
# │                                                         │
# │  - Add Result<> wrapper to init() function             │
# │  - Update error messages for clarity                   │
# │  - Add unit tests for error scenarios                  │
# │                                                         │
# └─────────────────────────────────────────────────────────┘
#
# Use this message? [Y/n]: Y

# 4. 自动提交
git commit -F .git/COMMIT_EDITMSG
```

### 场景 2：大规模重构

```bash
# 1. 完成大规模重构（50+ 文件）
git add .

# 2. 生成详细分析
workflow commit generate --format json > commit_analysis.json

# 3. 查看结构化总结
cat commit_analysis.json | jq '.structured_summary'

# 4. 使用生成的 message
workflow commit generate --auto-commit
```

### 场景 3：分析已有提交

```bash
# 分析最近的提交
workflow commit analyze HEAD

# 分析指定提交
workflow commit analyze abc1234

# 输出详细报告
workflow commit analyze abc1234 --format json > report.json
```

---

## 📅 实施计划

### Phase 1: 基础实现（1-2 天）

**目标**：完成核心服务和基本 CLI 命令

- [ ] **Domain 层**
  - [ ] 创建 `domain/src/commit/service.rs` 文件
  - [ ] 定义 `CommitMessageService` trait
  - [ ] 更新 `domain/src/commit/mod.rs` 导出服务
  - [ ] 扩展 `GitRepository` trait（新增 4 个方法）

- [ ] **Infrastructure 层**
  - [ ] 实现 `get_staged_files()`
  - [ ] 实现 `get_staged_diff()`
  - [ ] 实现 `get_commit_changed_files()`
  - [ ] 实现 `get_commit_diff()`

- [ ] **Services 层**
  - [ ] 创建 `services/src/commit/` 模块目录
  - [ ] 创建 `services/src/commit/message/` 子模块
  - [ ] 实现 `CommitMessageServiceImpl`
  - [ ] 编写 `prompt.md` 和 `conversation.rs`
  - [ ] 实现辅助函数（`format_file_summary`, `smart_truncate_diff`）
  - [ ] 更新 `services/src/lib.rs` 导出服务

- [ ] **App 层**
  - [ ] 实现 `CommitGenerateCommand`
  - [ ] 实现 `CommitAnalyzeCommand`
  - [ ] 更新 `app/src/commands/commit/mod.rs` 导出命令
  - [ ] 在 registry 中注册 `CommitMessageService`
  - [ ] 注册到 CLI

### Phase 2: 功能增强（1-2 天）

**目标**：优化用户体验和输出质量

- [ ] **智能 Diff 截断**
  - [ ] 实现按文件拆分 diff
  - [ ] 小文件保留完整 diff
  - [ ] 大文件只保留关键部分（函数签名、重要变更）

- [ ] **输出格式优化**
  - [ ] 实现 `--format plain` 输出（美化的文本格式）
  - [ ] 实现 `--format json` 输出（完整 JSON）
  - [ ] 实现交互式确认（`Use this message? [Y/n]`）

- [ ] **自动提交**
  - [ ] 实现 `--auto-commit` 功能
  - [ ] 支持 GPG 签名
  - [ ] 支持 `--amend` 选项

### Phase 3: 测试与文档（1 天）

**目标**：确保质量和可维护性

- [ ] **单元测试**
  - [ ] `calculate_statistics` 测试
  - [ ] `format_file_summary` 测试
  - [ ] `smart_truncate_diff` 测试
  - [ ] Mock GitRepository 测试服务逻辑

- [ ] **集成测试**
  - [ ] 测试 `generate_for_staged()` 完整流程
  - [ ] 测试 `generate_for_commit()` 完整流程
  - [ ] 测试不同文件规模场景（1 文件、10 文件、100 文件）

- [ ] **文档**
  - [ ] 更新 `docs/README.md`（添加新命令说明）
  - [ ] 编写 `docs/guides/commit-message-generation.md`
  - [ ] 更新 `crates/services/README.md`（说明 commit 和 summary 服务的区别）
  - [ ] 添加 `crates/services/src/commit/README.md`（说明 commit 服务模块）

### Phase 4: 迭代优化（持续）

**目标**：根据用户反馈持续改进

- [ ] **性能优化**
  - [ ] 监控 LLM 调用耗时
  - [ ] 优化 diff 截断策略
  - [ ] 缓存机制（避免重复分析相同变更）

- [ ] **Prompt 优化**
  - [ ] 收集生成质量反馈
  - [ ] A/B 测试不同 prompt 版本
  - [ ] 针对特定项目类型优化（Rust/TypeScript/Go）

- [ ] **功能扩展**
  - [ ] 支持 Conventional Commits 配置
  - [ ] 支持自定义 commit message 模板
  - [ ] 集成到 Git hooks（pre-commit）

---

## 🚨 风险评估

### 技术风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| **LLM 上下文窗口超限** | 高 | 中 | 实现智能 diff 截断，限制最大 diff 行数 |
| **大规模变更分析质量下降** | 中 | 中 | 提供降级方案：当文件数 > 100 时提示使用三阶段分析 |
| **Prompt 适配性问题** | 中 | 低 | 复用经过验证的 prompt 模板，逐步迭代优化 |
| **Git 命令兼容性** | 低 | 低 | 充分测试不同 Git 版本（2.20+） |

### 业务风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| **生成质量不符合预期** | 高 | 中 | 提供交互式确认，用户可编辑后再提交 |
| **用户习惯变更** | 低 | 低 | 保持现有命令不变，新命令为可选增强功能 |
| **成本考虑** | 中 | 低 | 单次调用成本低（$0.01-0.05），但高频使用需监控 |

### 实施风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| **实施时间超预期** | 中 | 中 | 分阶段实施，Phase 1 可独立交付 |
| **与现有代码冲突** | 低 | 低 | 新功能独立模块，不影响现有三阶段流程 |

---

## 📊 成功指标

### 性能指标

- ✅ 单次 LLM 调用耗时 < 10 秒
- ✅ 支持 100+ 文件的变更分析
- ✅ 成本 < $0.05 per commit

### 质量指标

- ✅ 生成的 commit message 符合 Conventional Commits 规范
- ✅ 结构化总结准确率 > 90%
- ✅ 用户满意度（主观评价）> 4/5

### 采用指标

- ✅ 命令使用频率（目标：> 50% 的 commit 使用 generate 命令）
- ✅ 功能留存率（目标：> 80% 的用户持续使用）

---

## 🔗 相关文档

- [Summary 服务架构](../../crates/services/src/summary/README.md)
- [LLM 集成文档](../../crates/llm/README.md)
- [Git Repository 设计](../../crates/domain/src/git/repository.rs)
- [Conventional Commits 规范](https://www.conventionalcommits.org/)

---

## 📝 变更日志

| 日期 | 版本 | 变更内容 |
|------|------|---------|
| 2026-02-09 | 1.0 | 初始版本，定义单次提交 commit message 生成方案 |

---

**最后更新**: 2026-02-09
