# 文档重命名待办事项

## 📋 概述

将 `docs/` 和 `analysis/` 目录下的所有文档文件名从 `SCREAMING_SNAKE_CASE`（全大写+下划线）改为 `snake-_case`（小写+下划线），以提升可读性和一致性。

**目录重命名**：`docs/todo/` → `docs/requirements/`

**架构文档重组**：采用**方案 C（混合方案）**，按功能模块组织架构文档，合并 lib 层和 commands 层文档

**命名规则**：
- 架构文档：删除 `_architecture` 后缀（重组后无重名风险）
- 其他文档：保留类型后缀（`_guidelines`、`_todo`、`_analysis` 等）

**示例转换**：
- `ARCHITECTURE.md` → `architecture.md`（总体架构文档，保留原名）
- `DEVELOPMENT_GUIDELINES.md` → `development-_guidelines.md`
- `PR_COMMAND_ARCHITECTURE.md` + `lib/PR_ARCHITECTURE.md` → `pr.md`（合并，删除后缀）
- `lib/GIT_ARCHITECTURE.md` → `git.md`（删除后缀）
- `GIT2_AUTHENTICATION_ANALYSIS.md` → `git2_authentication-_analysis.md`
- `docs/todo/` → `docs/requirements/`

---

## 📊 任务统计

| 目录/操作 | 文档数量 | 状态 |
|---------|---------|------|
| **目录重命名** | | |
| `docs/todo/` → `docs/requirements/` | 1 个目录 | ⏳ 待处理 |
| **架构文档重组** | | |
| Lib + Commands 合并 | ~15 对文档 | ⏳ 待处理 |
| 独立 Lib 文档 | ~10 个 | ⏳ 待处理 |
| 独立 Commands 文档 | ~5 个 | ⏳ 待处理 |
| **文件重命名** | | |
| `docs/architecture/`（重组后） | ~30 | ⏳ 待处理 |
| `docs/guidelines/` | ~20+ | ⏳ 待处理 |
| `docs/requirements/`（重命名后） | 7 | ⏳ 待处理 |
| `analysis/` | 3 | ⏳ 待处理 |
| `analysis/impl/` | 1 | ⏳ 待处理 |
| **总计** | **~70+** | ⏳ 待处理 |

---

## ✅ 任务清单

### 阶段 0：目录重命名（优先执行）

#### 0.1 重命名目录

- [ ] `docs/todo/` → `docs/requirements/`（使用 `git mv` 保留历史）

#### 0.2 更新目录引用

- [ ] 更新 `.cursorrules` 中的所有 `docs/todo/` 引用（~16 处）
  - [ ] 更新文档存放规则中的路径引用
  - [ ] 更新文档命名规范中的路径引用
  - [ ] 更新文档索引规则中的路径引用
  - [ ] 更新文档删除规则中的路径引用
- [ ] 更新 `docs/README.md` 中的目录引用（~1 处）
- [ ] 更新其他文档中的 `docs/todo/` 引用
  - [ ] `docs/guidelines/workflows/references/REVIEW_DOCUMENT_GUIDELINES.md`
  - [ ] `docs/guidelines/DOCUMENT_TIMESTAMP_GUIDELINES.md`
  - [ ] 其他可能包含引用的文档

### 阶段 1：准备工作

- [ ] 创建备份分支（可选，用于安全回滚）
- [ ] 列出所有需要重命名的文件清单
- [ ] 识别所有包含文档链接的文件
- [ ] 检查 CI/CD 流程中是否有硬编码的文档路径
- [ ] 检查代码中是否有硬编码的文档路径引用

### 阶段 2：架构文档重组（方案 C）

#### 2.1 合并 Lib + Commands 文档对

按功能模块合并 lib 层和 commands 层文档，合并后的文档包含两个部分：
- **Lib 层架构**：核心业务逻辑、API 设计、平台抽象
- **Commands 层架构**：命令实现、用户交互、参数解析

**需要合并的文档对**：

- [ ] `lib/PR_ARCHITECTURE.md` + `commands/PR_COMMAND_ARCHITECTURE.md` → `pr.md`
- [ ] `lib/JIRA_ARCHITECTURE.md` + `commands/JIRA_COMMAND_ARCHITECTURE.md` → `jira.md`
- [ ] `lib/BRANCH_ARCHITECTURE.md` + `commands/BRANCH_COMMAND_ARCHITECTURE.md` → `branch.md`
- [ ] `lib/COMMIT_ARCHITECTURE.md` + `commands/COMMIT_COMMAND_ARCHITECTURE.md` → `commit.md`
- [ ] `lib/TAG_ARCHITECTURE.md` + `commands/TAG_COMMAND_ARCHITECTURE.md` → `tag.md`
- [ ] `lib/STASH_ARCHITECTURE.md` + `commands/STASH_COMMAND_ARCHITECTURE.md` → `stash.md`
- [ ] `lib/PROXY_ARCHITECTURE.md` + `commands/PROXY_COMMAND_ARCHITECTURE.md` → `proxy.md`
- [ ] `lib/REPO_ARCHITECTURE.md` + `commands/REPO_COMMAND_ARCHITECTURE.md` → `repo.md`
- [ ] `lib/CONFIG_ARCHITECTURE.md` + `commands/CONFIG_COMMAND_ARCHITECTURE.md` → `config.md`
- [ ] `lib/LOG_ARCHITECTURE.md` + `commands/LOG_COMMAND_ARCHITECTURE.md` → `log.md`
- [ ] `lib/LLM_ARCHITECTURE.md` + `commands/LLM_COMMAND_ARCHITECTURE.md` → `llm.md`
- [ ] `lib/LIFECYCLE_ARCHITECTURE.md` + `commands/LIFECYCLE_COMMAND_ARCHITECTURE.md` → `lifecycle.md`
- [ ] `lib/GITHUB_ARCHITECTURE.md` + `commands/GITHUB_COMMAND_ARCHITECTURE.md` → `github.md`
- [ ] `lib/CHECK_ARCHITECTURE.md` + `commands/CHECK_COMMAND_ARCHITECTURE.md` → `check.md`
- [ ] `lib/ALIAS_ARCHITECTURE.md` + `commands/ALIAS_COMMAND_ARCHITECTURE.md` → `alias.md`

**注意**：如果某个模块只有 lib 层或只有 commands 层文档，则直接重命名，无需合并。

#### 2.2 处理独立 Lib 文档

以下文档只有 lib 层，直接重命名（删除 `_architecture` 后缀）：

- [ ] `lib/GIT_ARCHITECTURE.md` → `git.md`
- [ ] `lib/HTTP_ARCHITECTURE.md` → `http.md`
- [ ] `lib/SETTINGS_ARCHITECTURE.md` → `settings.md`
- [ ] `lib/CLI_ARCHITECTURE.md` → `cli.md`
- [ ] `lib/COMPLETION_ARCHITECTURE.md` → `completion.md`
- [ ] `lib/CONCURRENT_ARCHITECTURE.md` → `concurrent.md`
- [ ] `lib/DIALOG_ARCHITECTURE.md` → `dialog.md`
- [ ] `lib/INDICATOR_ARCHITECTURE.md` → `indicator.md`
- [ ] `lib/LOGGER_ARCHITECTURE.md` → `logger.md`
- [ ] `lib/PROMPT_ARCHITECTURE.md` → `prompt.md`
- [ ] `lib/ROLLBACK_ARCHITECTURE.md` → `rollback.md`
- [ ] `lib/SHELL_ARCHITECTURE.md` → `shell.md`
- [ ] `lib/TEMPLATE_ARCHITECTURE.md` → `template.md`
- [ ] `lib/TOOLS_ARCHITECTURE.md` → `tools.md`

#### 2.3 处理独立 Commands 文档

以下文档只有 commands 层，直接重命名（删除 `_architecture` 后缀）：

- [ ] `commands/MIGRATE_COMMAND_ARCHITECTURE.md` → `migrate.md`

#### 2.4 处理其他架构文档

- [ ] `ARCHITECTURE.md` → `architecture.md`（总体架构文档）
- [ ] `CHECK_LOG.md` → `check-_log.md`

### 阶段 3：文件重命名（其他目录）

#### 3.1 `docs/guidelines/` 目录

- [ ] `CARGO_BLOAT_GUIDELINES.md` → `cargo-_bloat-_guidelines.md`
- [ ] `CI_WORKFLOW_GUIDELINES.md` → `ci-_workflow-_guidelines.md`
- [ ] `DEVELOPMENT_GUIDELINES.md` → `development-_guidelines.md`
- [ ] `DOCUMENT_GUIDELINES.md` → `document-_guidelines.md`
- [ ] `DOCUMENT_TIMESTAMP_GUIDELINES.md` → `document-_timestamp-_guidelines.md`
- [ ] `GITHUB_SETUP_GUIDELINES.md` → `github-_setup-_guidelines.md`
- [ ] `PR_PLATFORM_GUIDELINES.md` → `pr-_platform-_guidelines.md`
- [ ] `TEMPLATE_GUIDELINES.md` → `template-_guidelines.md`
- [ ] `TESTING_GUIDELINES.md` → `testing-_guidelines.md`
- [ ] `workflows/PRE_COMMIT_GUIDELINES.md` → `workflows/pre-_commit-_guidelines.md`
- [ ] `workflows/REVIEW_GUIDELINES.md` → `workflows/review-_guidelines.md`
- [ ] `workflows/references/QUICK_REFERENCE_GUIDELINES.md` → `workflows/references/quick-_reference-_guidelines.md`
- [ ] `workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md` → `workflows/references/review-_architecture-_doc-_guidelines.md`
- [ ] `workflows/references/REVIEW_CLI_GUIDELINES.md` → `workflows/references/review-_cli-_guidelines.md`
- [ ] `workflows/references/REVIEW_CODE_GUIDELINES.md` → `workflows/references/review-_code-_guidelines.md`
- [ ] `workflows/references/REVIEW_DOCUMENT_GUIDELINES.md` → `workflows/references/review-_document-_guidelines.md`
- [ ] `workflows/references/REVIEW_TEST_CASE_GUIDELINES.md` → `workflows/references/review-_test-_case-_guidelines.md`
- [ ] `workflows/references/STYLE_GUIDELINES.md` → `workflows/references/style-_guidelines.md`
- [ ] `workflows/references/TEST_COVERAGE_CHECK_GUIDELINES.md` → `workflows/references/test-_coverage-_check-_guidelines.md`

#### 3.2 `docs/requirements/` 目录（目录重命名后）

- [ ] `COVERAGE_IMPROVEMENT_TODO.md` → `coverage-_improvement-_todo.md`
- [ ] `DOC_CHECK_TODO.md` → `doc-_check-_todo.md`
- [ ] `GIX_MIGRATION_TODO.md` → `gix-_migration-_todo.md`
- [ ] `INTEGRATION_TODO.md` → `integration-_todo.md`
- [ ] `JIRA_TODO.md` → `jira-_todo.md`
- [ ] `TEST_ARCHITECTURE_IMPROVEMENT_TODO.md` → `test-_architecture-_improvement-_todo.md`
- [ ] `DOCUMENT_RENAME_TODO.md` → `document-_rename-_todo.md`（本文件）

#### 3.3 `analysis/` 目录

- [ ] `GIT2_AUTHENTICATION_ANALYSIS.md` → `git2_authentication-_analysis.md`
- [ ] `GIT2_MIGRATION_STATUS.md` → `git2_migration-_status.md`
- [ ] `TEST_ANALYSIS_REPORT.md` → `test-_analysis-_report.md`

#### 3.4 `analysis/impl/` 目录（实现/实施文档）

**注意**：实现文档或实施文档应存放到 `analysis/impl/` 目录下。

- [ ] `GIT2_AUTHENTICATION_IMPLEMENTATION.md` → `analysis/impl/git2_authentication-_implementation.md`（移动到 impl 目录）

### 阶段 4：更新文档引用

#### 4.1 更新索引文件

- [ ] 更新 `docs/README.md` 中的所有链接（~50+ 处）
  - [ ] 更新目录引用：`todo/` → `requirements/`
- [ ] 更新 `docs/requirements/README.md` 中的所有链接（~10+ 处）
  - [ ] 更新文档内部链接（文件名大小写）
- [ ] 更新 `docs/migration/README.md` 中的链接
- [ ] 更新 `docs/guidelines/workflows/README.md` 中的所有链接（~15+ 处）

#### 4.2 更新文档内部交叉引用

- [ ] 搜索所有 `.md` 文件中的文档链接引用
- [ ] 更新文档内部的交叉引用链接
- [ ] 验证所有链接的有效性

### 阶段 5：验证和测试

- [ ] 验证所有重命名后的文件存在且内容完整
- [ ] 验证所有链接正常工作（无 404 错误）
- [ ] 检查 Git 历史记录（使用 `git mv` 保留历史）
- [ ] 运行文档检查工具（如有）
- [ ] 检查 CI/CD 流程是否正常

---

## 🔧 实施步骤

### 步骤 0：目录重命名（优先执行）

```bash
# 重命名目录（保留 Git 历史）
git mv docs/todo docs/requirements
```

### 步骤 1：架构文档重组

#### 1.1 合并 Lib + Commands 文档

对于需要合并的文档对，创建新的合并文档：

```bash
# 示例：合并 PR 模块文档
# 1. 读取 lib/PR_ARCHITECTURE.md 和 commands/PR_COMMAND_ARCHITECTURE.md
# 2. 创建新的 pr.md（删除 _architecture 后缀），包含两部分：
#    - Lib 层架构部分
#    - Commands 层架构部分
# 3. 删除旧的 lib/PR_ARCHITECTURE.md 和 commands/PR_COMMAND_ARCHITECTURE.md
```

**合并文档结构模板**：

```markdown
# {模块名} 模块架构文档

## 📋 概述
- 模块功能概述
- Lib 层和 Commands 层的整体说明

## 📁 Lib 层架构（核心业务逻辑）
- 模块结构
- API 设计
- 业务逻辑实现
- 平台抽象设计

## 📁 Commands 层架构（命令封装）
- 命令结构
- 用户交互
- 参数解析
- 与 Lib 层的集成

## 🔄 集成关系
- Lib 层和 Commands 层如何协作
- 数据流向
```

**文档命名**：删除 `_architecture` 后缀，使用简洁的模块名（如 `pr.md`、`jira.md`、`git.md`）

#### 1.2 重命名独立文档

对于只有 lib 层或只有 commands 层的文档，直接重命名（删除 `_architecture` 后缀）：

```bash
# 示例：重命名独立 Lib 文档
git mv docs/architecture/lib/GIT_ARCHITECTURE.md docs/architecture/git.md
```

### 步骤 2：使用 Git 重命名文件（保留历史）

推荐使用 `git mv` 命令重命名文件，这样可以保留 Git 历史记录：

```bash
# 示例：重命名单个文件
git mv docs/architecture/architecture.md docs/architecture/architecture.md

# 批量重命名脚本示例（需要根据实际情况调整）
find docs/architecture -name "*.md" -type f | while read file; do
    dir=$(dirname "$file")
    old-_name=$(basename "$file")
    new-_name=$(echo "$old-_name" | tr '[:upper:]' '[:lower:]')
    if [ "$old-_name" != "$new-_name" ]; then
        git mv "$file" "$dir/$new-_name"
    fi
done
```

### 步骤 3：更新文档链接和引用

使用搜索替换工具更新所有文档中的链接引用：

```bash
# 1. 更新目录引用：docs/todo/ -> docs/requirements/
# 在 .cursorrules、docs/README.md 等文件中

# 2. 更新文件名引用：ARCHITECTURE.md -> architecture.md
# PR_ARCHITECTURE.md -> pr-_architecture.md
# 等等...
```

### 步骤 4：验证链接

使用工具验证所有链接的有效性：

```bash
# 可以使用 markdown-link-check 等工具
# 或者手动检查关键链接
```

---

## ⚠️ 注意事项

1. **执行顺序**：
   - **阶段 0**：目录重命名（`docs/todo/` → `docs/requirements/`）
   - **阶段 2**：架构文档重组（合并 lib 和 commands 文档）
   - **阶段 3**：文件重命名（其他目录）
   - **阶段 4**：更新文档引用
   - **阶段 5**：验证和测试
2. **Git 历史**：使用 `git mv` 而不是 `mv`，以保留文件历史记录（目录和文件都需要）
3. **`.cursorrules` 更新**：目录重命名后，必须更新 `.cursorrules` 中的所有路径引用（~16 处）
4. **外部引用**：检查是否有外部文档或代码引用这些文档路径
5. **CI/CD**：检查 CI/CD 流程中是否有硬编码的文档路径
6. **搜索替换**：使用精确匹配，避免误替换其他内容（如代码中的字符串）
7. **大小写敏感**：某些文件系统（如 macOS）默认不区分大小写，需要注意
8. **README.md**：保持 `README.md` 文件名不变（约定）
9. **目录重命名影响**：`docs/todo/` → `docs/requirements/` 会影响所有引用该目录的文档和配置文件

---

## 📝 更新规范

完成重命名后，需要更新 `.cursorrules` 中的文档命名规范：

**当前规范**（需要更新）：
- 架构文档：`{MODULE}_ARCHITECTURE.md`（Lib 层）或 `{MODULE}_COMMAND_ARCHITECTURE.md`（命令层）
- 指南文档：`{TOPIC}_GUIDELINES.md`
- TODO 文档：`{TOPIC}_TODO.md`
- 分析文档：`{TOPIC}_ANALYSIS.md` 或 `{TOPIC}_ANALYSIS_REPORT.md`

**更新后规范**：
- 架构文档：`{module}.md`（按功能模块组织，包含 Lib 层和 Commands 层，删除 `_architecture` 后缀）
- 指南文档：`{topic}_guidelines.md`
- 需求文档：`{topic}_todo.md`（存放到 `docs/requirements/`）
- 分析文档：`{topic}_analysis.md` 或 `{topic}_analysis-_report.md`（存放到 `analysis/`）
- 实现文档：`{topic}_implementation.md` 或 `{topic}_impl.md`（存放到 `analysis/impl/`）

**目录路径更新**：
- `docs/todo/` → `docs/requirements/`

**架构文档组织方式**：
- 采用**方案 C（混合方案）**，按功能模块组织
- 每个功能模块一个文档，包含 Lib 层和 Commands 层两部分
- 文档命名：`{module}.md`（如 `pr.md`、`jira.md`、`git.md`）
- **删除 `_architecture` 后缀**：重组后所有架构文档都在 `docs/architecture/` 目录下，无子目录，无重名风险

---

## 🔗 相关文档

- [文档编写指南](../guidelines/DOCUMENT_GUIDELINES.md)
- [开发规范](../guidelines/DEVELOPMENT_GUIDELINES.md)
- [文档索引](../README.md)

## 📝 目录重命名详细说明

### 重命名原因

将 `docs/todo/` 重命名为 `docs/requirements/` 的原因：
- 更准确地反映文档内容（需求、待办事项、实施计划）
- 与常见的项目文档结构保持一致
- 提升目录名称的可读性和专业性

### 需要更新的引用位置

1. **`.cursorrules`**（~16 处）：
   - 文档存放规则中的路径引用
   - 文档命名规范中的路径引用
   - 文档索引规则中的路径引用
   - 文档删除规则中的路径引用

2. **`docs/README.md`**（~1 处）：
   - 待办事项文档目录引用

3. **其他文档**：
   - `docs/guidelines/workflows/references/REVIEW_DOCUMENT_GUIDELINES.md`
   - `docs/guidelines/DOCUMENT_TIMESTAMP_GUIDELINES.md`
   - 其他可能包含 `docs/todo/` 引用的文档

### 重命名后的目录结构

#### `docs/requirements/` 目录

```
docs/requirements/
├── README.md                      # 索引文档
├── coverage-_improvement-_todo.md
├── doc-_check-_todo.md
├── document-_rename-_todo.md        # 本文件
├── gix-_migration-_todo.md
├── integration-_todo.md
├── jira-_todo.md
└── test-_architecture-_improvement-_todo.md
```

#### `docs/architecture/` 目录（重组后）

```
docs/architecture/
├── architecture.md                # 总体架构文档（保留原名）
├── check-_log.md                   # Check Log 文档（非架构文档，保留原名）
├── pr.md                         # PR 模块（合并 lib + commands）
├── jira.md                       # Jira 模块（合并 lib + commands）
├── branch.md                     # Branch 模块（合并 lib + commands）
├── commit.md                     # Commit 模块（合并 lib + commands）
├── tag.md                        # Tag 模块（合并 lib + commands）
├── stash.md                      # Stash 模块（合并 lib + commands）
├── proxy.md                      # Proxy 模块（合并 lib + commands）
├── repo.md                       # Repo 模块（合并 lib + commands）
├── config.md                     # Config 模块（合并 lib + commands）
├── log.md                        # Log 模块（合并 lib + commands）
├── llm.md                        # LLM 模块（合并 lib + commands）
├── lifecycle.md                  # Lifecycle 模块（合并 lib + commands）
├── github.md                     # GitHub 模块（合并 lib + commands）
├── check.md                      # Check 模块（合并 lib + commands）
├── alias.md                      # Alias 模块（合并 lib + commands）
├── migrate.md                    # Migrate 模块（独立 commands）
├── git.md                        # Git 模块（独立 lib）
├── http.md                       # HTTP 模块（独立 lib）
├── settings.md                   # Settings 模块（独立 lib）
├── cli.md                        # CLI 模块（独立 lib）
├── completion.md                 # Completion 模块（独立 lib）
├── concurrent.md                 # Concurrent 模块（独立 lib）
├── dialog.md                     # Dialog 模块（独立 lib）
├── indicator.md                 # Indicator 模块（独立 lib）
├── logger.md                     # Logger 模块（独立 lib）
├── prompt.md                     # Prompt 模块（独立 lib）
├── rollback.md                   # Rollback 模块（独立 lib）
├── shell.md                      # Shell 模块（独立 lib）
├── template.md                   # Template 模块（独立 lib）
└── tools.md                      # Tools 模块（独立 lib）
```

**注意**：
- 重组后，`lib/` 和 `commands/` 子目录将被移除，所有架构文档直接位于 `docs/architecture/` 目录下
- **删除 `_architecture` 后缀**：由于所有架构文档都在同一目录下，无重名风险，可以删除后缀使文档名更简洁
- `architecture.md` 是总体架构文档，保留原名以区分

#### `analysis/impl/` 目录（实现/实施文档）

```
analysis/impl/
└── git2_authentication-_implementation.md  # 实现文档（从 analysis/ 移动）
```

**注意**：
- `analysis/impl/` 目录用于存放实现文档或实施文档
- 实现文档命名：`{topic}_implementation.md` 或 `{topic}_impl.md`
- 这些文档是临时实现文档，不是参考文档，可以随时删除

---

## 📝 架构文档重组详细说明

### 重组原因

采用**方案 C（混合方案）**的原因：
- **用户视角**：用户更关心功能模块的整体架构，而不是 lib 和 commands 的技术分层
- **完整性**：一个文档包含该功能的全部信息（Lib 层 + Commands 层）
- **减少重复**：避免在两个文档中重复说明相同功能
- **便于查找**：用户只需查找一个文档即可了解完整功能

### 文档结构

合并后的架构文档应包含以下部分：

1. **概述**：模块功能概述、Lib 层和 Commands 层的整体说明
2. **Lib 层架构**：核心业务逻辑、API 设计、平台抽象
3. **Commands 层架构**：命令实现、用户交互、参数解析
4. **集成关系**：Lib 层和 Commands 层如何协作、数据流向
5. **相关文档**：相关文档链接

### 合并策略

- **有对应关系的文档对**：合并为一个文档（如 `lib/PR_ARCHITECTURE.md` + `commands/PR_COMMAND_ARCHITECTURE.md`）
- **只有 Lib 层的文档**：直接重命名，保留 Lib 层内容（如 `lib/GIT_ARCHITECTURE.md`）
- **只有 Commands 层的文档**：直接重命名，保留 Commands 层内容（如 `commands/MIGRATE_COMMAND_ARCHITECTURE.md`）

---

**创建时间**: 2025-01-27
**最后更新**: 2025-01-27
**状态**: ⏳ 待实施
**优先级**: 中优先级（代码质量改进）

