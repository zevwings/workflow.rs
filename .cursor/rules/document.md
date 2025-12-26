# Document Generation Rules

> **⚠️ Synchronization Note**: This file must be kept in sync with its Chinese version at `docs/cursorrules/document.md`. When modifying this file, you must immediately update the corresponding Chinese version.

---

## 🚫 Document Generation Rules (Mandatory)

### Core Principle

**Default: No Document Generation**: Unless the user uses explicit keywords, do NOT generate any `.md` document files.

### 🔄 Per-Message Independent Evaluation Rule

**Critical Principle**: Each user message must be evaluated **independently** for document generation eligibility. Previous document generation in the same chat session does NOT grant permission for subsequent automatic generation.

**Rules**:
- ✅ **Every message**: Must pass ALL FOUR checks independently
- ❌ **Never assume**: "User wanted documents before, so they want them now"
- ❌ **Never continue**: "I generated a document earlier, so I'll generate more"
- ✅ **Always check**: Does THIS specific message contain explicit keywords?

**Example**:
```
Message 1: "generate test report" → ✅ Generate document
Message 2: "analyze failure reasons" → ❌ NO document, provide analysis in chat
Message 3: "summarize" → ❌ NO document, provide summary in chat
Message 4: "generate summary report" → ✅ Generate document (new explicit request)
```

### 🚫 Context Independence Rule

**Critical Principle**: Chat context, conversation history, or previous document generation does NOT override the mandatory four-check process.

**Prohibited Assumptions**:
- ❌ "User asked for analysis before, so they want analysis documents now"
- ❌ "I generated documents earlier, so I should continue generating"
- ❌ "This is a follow-up, so I should generate a document"
- ❌ "User seems to want documentation based on context"

**Required Behavior**:
- ✅ Check THIS message for explicit keywords
- ✅ Apply blacklist rules to THIS message
- ✅ Ignore previous document generation history
- ✅ Treat each message as a fresh request

### ✅ Allowed Document Generation (Whitelist)

**ONLY** generate documents when users use these explicit keywords:

**Direct Commands**:
- "generate document", "create document", "write a document"
- "generate report", "create report", "write a report"
- "save as document", "save as file", "output to file"
- "record to document", "write to document"

**Explicit Requests**:
- "help me create a... document"
- "can you generate a... report"
- "please write a... documentation"

**Exceptions**:
- When executing `review.md` or `pre-commit.md` workflows (explicitly defined below)

**⚠️ Important**: If the user doesn't use the above keywords, **DO NOT generate documents**!

### ❌ Prohibited Auto-Generation Scenarios (Blacklist)

The following scenarios **STRICTLY PROHIBIT** automatic document generation. Provide text responses in chat only:

**Analysis Requests**:
- "analyze...", "check...", "inspect...", "diagnose...", "investigate..."
- ❌ Wrong: Auto-generate `analysis/xxx-analysis.md`
- ✅ Correct: Provide analysis in chat, ask "Need me to generate a document?"

**Summary Requests**:
- "summarize...", "sum up...", "consolidate...", "outline..."
- ❌ Wrong: Auto-generate `analysis/xxx-summary.md`
- ✅ Correct: Provide summary text in chat

**Completion Requests**:
- "complete...", "implement...", "do...", "handle..."
- ❌ Wrong: Think "complete" means "code + report"
- ✅ Correct: "complete" means code only, summarize in chat

**Progress Requests**:
- "how's the progress", "what's done", "status update", "what's completed"
- ❌ Wrong: Auto-generate `analysis/xxx-progress.md`
- ✅ Correct: Report progress in chat

**Review Requests**:
- "review...", "examine...", "look at...", "check out..."
- ❌ Wrong: Auto-generate `analysis/xxx-review.md`
- ✅ Correct: Provide review feedback in chat
- ⚠️ Exception: When executing `review.md` workflow (explicitly defined below)

### 📋 Mandatory Pre-Generation Checklist (Enhanced)

AI must pass ALL FOUR checks **independently** before generating ANY `.md` document:

**Important Reminders**:
- ⚠️ **Re-check every message**, do not skip checks because documents were generated before
- ⚠️ **Context does not affect judgment**, do not assume user intent based on conversation history
- ⚠️ **Independent evaluation**, treat each message as an independent request

**Check 1: Explicit Request**
- [ ] Does **THIS message** use a whitelist keyword?
  - ✅ Yes → Continue to Check 2
  - ❌ No → **PROHIBIT generation**, provide text response in chat
  - ⚠️ **Note**: Cannot assume this message contains keywords just because previous messages did

**Check 2: Blacklist Scenario**
- [ ] Is **THIS message** a blacklist scenario?
  - ❌ Yes → **PROHIBIT generation**, provide text response in chat
  - ✅ No → Continue to Check 3
  - ⚠️ **Note**: Even if previous messages generated documents, if this message is a blacklist scenario, generation is still prohibited

**Check 3: Exception Rule**
- [ ] Is this a Review Workflow auto-report?
  - ✅ Yes → Can generate
  - ❌ No → Continue to Check 4

**Check 4: Double Confirmation**
- [ ] If passed all checks above, ask before generating:
  ```
  Ready to generate document:
  - [document path]

  Confirm generation?
  ```
  - ⚠️ **Note**: Even if documents were generated before, confirmation is still required

**Only after passing ALL FOUR checks can documents be generated!**

### 🔍 Pre-Generation Self-Check Mechanism

**Before generating any document, AI must ask itself**:

1. **Keyword Check**:
   - "Did the user use explicit keywords in THIS message?"
   - "If this were the first message, would I generate a document?"

2. **Scenario Check**:
   - "Is this message a blacklist scenario?"
   - "Am I assuming the user wants a document?"

3. **Context Check**:
   - "Am I assuming the user wants a document because of previous messages?"
   - "Am I continuing to generate because I generated documents before?"

4. **Independence Check**:
   - "If I ignore the previous conversation history, does THIS message alone satisfy the generation conditions?"
   - "Am I making assumptions based on context?"

**If ANY answer is unclear or "no"** → **PROHIBIT document generation**

### 📚 Correct vs Incorrect Examples

**Scenario 1: User says "analyze test failure reasons"**

❌ **Wrong Approach**:
```
1. Run analysis
2. Auto-generate analysis/test-failure-analysis.md
3. Tell user: "Report generated"
```

✅ **Correct Approach**:
```
1. Run analysis
2. **In chat** provide analysis:

   🔍 Test Failure Analysis

   Failed tests: 27
   Main cause: Global state pollution (set_current_dir)

   Specific issues:
   - 54 tests use set_current_dir
   - Some tests don't restore directory
   - Causes subsequent tests to run in wrong directory

   Suggested fixes:
   1. Create CurrentDirGuard tool
   2. Use RAII pattern for auto-restore
   3. Remove redundant set_current_dir calls

   Need me to generate a detailed report?

3. Wait for explicit confirmation
```

**Scenario 2: User says "complete Branch Sync tests"**

❌ **Wrong Approach** (This violation):
```
1. Implement code ✅
2. Auto-generate branch-sync-complete-report.md ❌
3. Auto-generate test-failure-diagnosis.md ❌
4. Auto-generate branch-sync-final-summary.md ❌
```

✅ **Correct Approach**:
```
1. Implement code ✅
2. **In chat** summarize:

   🎉 Branch Sync Tests Complete!

   ✅ Completed:
   - GitTestEnv tool (166 lines)
   - 4 test cases
   - GitHub API Mock
   - CurrentDirGuard tool

   📊 Test Results:
   - Standalone: ✅ All pass
   - Full suite: Failures reduced from 27 to 15

   💡 Issues identified with solutions provided

   Need me to generate a complete implementation report?

3. Wait for explicit confirmation
```

**Scenario 3: User says "generate test coverage report"**

✅ **Correct Approach** (Explicit keyword):
```
1. Confirm: User used "generate report" keyword ✅
2. Not a blacklist scenario ✅
3. Directly generate analysis/test-coverage-report.md
4. Tell user: "Report generated"
```

### 🔐 Review Workflow Report Generation Rules

**Special Exception**: When executing these workflows, **MUST** auto-generate reports:

- **Comprehensive Deep Review** (`docs/guidelines/development/workflows/review.md`):
  - Must auto-generate `report/review-report-{timestamp}.md` after execution
- **Pre-commit Check** (`docs/guidelines/development/workflows/pre-commit.md`):
  - Must auto-generate `report/pre-commit-report-{timestamp}.md` after execution

**Timestamp Format**: `YYYY-MM-DD_HH-MM-SS` (e.g., `2025-12-23_22-55-15`)
**Report Content**: Generate complete reports per workflow document templates
**Report Location**: All reports must be saved in `report/` directory

### ⚠️ Violation Handling

If AI violates document generation rules:

1. **Immediately stop** current behavior
2. **Acknowledge error**: "I violated the document generation rules"
3. **Explain reason**: Why the violation occurred
4. **Propose remedy**:
   - Delete violating documents?
   - Consolidate content into chat?
5. **Ensure understanding**: Restate rules, confirm no repeat

**Example**:
```
🚨 I made a serious mistake!

Violation: Auto-generated 24 analysis documents
Rules broken: Generated documents without explicit request

Root cause analysis:
- Mistakenly thought "complete task" includes "generate report"
- Didn't check for explicit keywords
- Violated blacklist scenario rules
- Made assumptions based on context (common violation cause)

Suggested remedy:
1. Delete these 24 violating documents
2. Keep code implementation
3. Strictly follow four-check checklist going forward
4. Evaluate each message independently, do not make assumptions based on context

Should I delete these documents immediately?
```

### 🔒 Enforcement Guarantee

These rules are **mandatory**, not suggestions. AI must strictly comply and may NOT bypass them for "user convenience", "task completion", "providing complete solution", or any other reason.

## 📁 Document Classification and Storage

**Core Principle**: **All generated document files are automatically categorized into corresponding directories based on document type; if the type cannot be determined, prioritize checking for keywords like "分析" (analysis), matching then store in `analysis/`, otherwise default to `docs/requirements/`.**

**Important Rule**: Document files are stored in corresponding directories by type. Creating document files arbitrarily in the project root or other locations is prohibited.

### Document Classification Table

| Document Type | Directory | Naming Pattern | Keywords (保留中文关键词) |
|--------------|-----------|----------------|---------------------------|
| Architecture Documents | `docs/architecture/` | `{TOPIC}.md` | 架构、架构设计、Architecture、模块架构、系统设计 |
| Guidelines Documents | `docs/guidelines/` | `{TOPIC}.md` | 指南、规范、Guidelines、开发规范、测试规范、文档规范 |
| Migration Documents | `docs/migration/` | `{TOPIC}.md` | 迁移、Migration、版本升级、配置迁移 |
| TODO Documents | `docs/requirements/` | `{TOPIC}.md` | TODO、待办、待实现、计划 |
| Requirement Documents | `docs/requirements/` | `{TOPIC}.md` | 需求、需求分析、功能需求、需求文档 |
| Analysis Documents | `analysis/` | `{TOPIC}.md` | 分析、ANALYSIS、问题分析、技术分析、测试分析、代码分析、性能分析、架构分析、设计分析、代码审查分析、问题诊断 |
| Report Documents | `report/` | `{TOPIC}.md` | 分析报告、检查报告、代码分析、质量报告（from pre-commit.md） |

**Note**: Requirement analysis, feature descriptions, implementation plans, and other unclassified documents default to `docs/requirements/` directory.

### Document Storage Decision Process

1. Check if the user explicitly specified document type or storage location
2. If type is specified, automatically categorize according to the above classification rules
3. **Important Restrictions**:
   - **Analysis Documents** (`analysis/`): **Must** only generate when user explicitly requests analysis documents, cannot auto-judge based on keywords
   - **Requirement Documents** (`docs/requirements/`): **Must** only generate when user explicitly requests requirement documents, cannot auto-judge based on keywords
4. If type is not specified, check keywords in document content:
   - Contains "TODO", "待办", "待实现", "计划" (plan), "需求" (requirement), "需求分析", "功能需求", "需求文档" → `docs/requirements/`
   - Other unclassified documents → `docs/requirements/`
5. If type cannot be determined, default to `docs/requirements/`

### Document Naming Standards

All documents use the `{TOPIC}.md` format.

**Important**:
- Use `kebab-case` for topic names (e.g., `test-coverage.md`, `jira.md`)

Refer to the **Document Classification Table** above for document type classification and storage locations.

### Creating New Documents

- **Document Writing Guidelines**: Use templates to create new documents (reference `docs/guidelines/document.md`)
  - Select appropriate template based on document type (architecture, guidelines, requirements, review workflow, development workflow, review guide documents)
  - Follow document writing standards and chapter checklist
- **Document Timestamp**: Add "Last Updated" timestamp at the end of documents (reference `docs/guidelines/document-timestamp.md`)
  - Format: `**最后更新**: YYYY-MM-DD`
  - Location: End of document, after separator line
  - **Important**: When updating document content, must update the timestamp at the end of the document to the current date
- **Document Standards**: Ensure documents follow project document writing standards

---

**Last Updated**: 2025-12-25

