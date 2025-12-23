# Workflow CLI - Cursor AI 规则

> **⚠️ 重要提示**：修改此文件时，必须同时更新英文版本 `.cursorrules` 以保持两个版本同步。

## ⚠️ 重要提示

**代码和文档生成规则**：
- **如果没有明确说明，不需要生成代码和文档**
- 只有在用户明确要求生成代码或文档时，才执行相应的操作
- 对于分析、解释、讨论类的问题，仅提供文字回答，不要自动生成代码或文档文件
- 如果用户只是询问问题或需要解释，不要主动创建文件或修改代码
- **禁止性规则**：**在没有用户明确指示需要生成代码或文档的情况下，禁止通过 AI 自动生成任何代码文件或文档文件。仅提供分析、解释、建议等文字回答。**
- **分析类问题规则**：**对于分析、讨论、询问类的问题，只提供文字回答和分析，不要自动生成分析文档文件。只有在用户明确要求生成文档时，才创建文档文件。**
- **代码修改规则**：**禁止在没有用户明确同意或说明的情况下修改现有代码文件。如需修改代码，必须先获得用户同意或用户明确说明需要修改。**
- **文档修改规则**：**禁止在没有用户明确同意或说明的情况下修改现有文档文件。如需修改文档，必须先获得用户同意或用户明确说明需要修改。**

## 📋 项目概述

这是一个用 Rust 编写的 CLI 工具，用于自动化开发工作流，提供 PR 管理、Jira 集成、日志处理等功能。

### 架构设计

项目采用三层架构：
- **CLI 入口层** (`bin/`, `main.rs`): 命令行参数解析和命令分发
- **命令封装层** (`commands/`): CLI 命令封装，处理用户交互
- **核心业务逻辑层** (`lib/`): 所有业务逻辑实现

详细的模块架构信息请参考 `docs/architecture/architecture.md`。

## 📐 开发规范与指南

**重要**：所有开发规范和指南请严格遵循 `docs/guidelines/development.md` 中的详细说明。

### 开发规范引用

**必须遵守的开发规范文档**：`docs/guidelines/development.md`

该文档包含完整的开发规范，包括代码风格、错误处理、文档规范、命名规范、模块组织、Git 工作流、提交规范、测试规范、代码审查、依赖管理、开发工具等。

### 快速参考

以下是最常用的规范快速参考（**详细说明和完整规范请查看 `docs/guidelines/development.md`**）：

- **代码格式化**：`cargo fmt`（提交前必须运行）
- **代码检查**：`cargo clippy -- -D warnings`（所有警告必须修复）
- **命名约定**：模块/函数/变量 `snake_case`，类型/Trait `PascalCase`，常量 `SCREAMING_SNAKE_CASE`
- **导入顺序**：标准库 → 第三方库 → 项目内部
- **错误处理**：使用 `anyhow::Result<T>` 和 `Context` 添加上下文
- **文档注释**：所有公共 API 必须有 `///` 文档注释，包含参数、返回值、错误、示例

### 代码生成规则

**重要**：只有在用户明确要求生成代码时，才执行代码生成操作。

生成代码时必须严格遵循 `docs/guidelines/development.md` 中的所有开发规范，包括：

- **代码风格**：遵循项目的命名约定和代码风格，使用 `rustfmt` 格式化，通过 `clippy` 检查
- **错误处理**：使用 `anyhow::Result<T>`，使用 `Context` 添加上下文，考虑所有错误情况
- **文档注释**：为新功能添加完整的文档注释（包含参数、返回值、错误、示例）
- **文档编写**：如果生成文档，必须参考 `docs/guidelines/document.md` 选择合适的模板，并在文档末尾添加"最后更新"时间戳（参考 `docs/guidelines/document-timestamp.md`）
- **模块组织**：遵循三层架构，正确组织模块和依赖关系
- **测试**：为新功能添加单元测试和集成测试（如适用）
- **提交规范**：如果涉及 Git 提交，遵循 Conventional Commits 格式
- **代码质量**：保持代码简洁和可读性，遵循 Rust 最佳实践（如使用 `Result` 类型、避免 `unwrap()` 等）
- **代码审查**：确保代码符合审查清单要求

### 添加新功能

1. 在 `lib/` 中实现核心业务逻辑
2. 在 `commands/` 中添加 CLI 命令封装
3. 在 `main.rs` 中注册命令
4. 添加测试用例（参考测试规范）
5. 更新文档
6. **扩展 PR 平台支持**：如需添加新的 PR 平台（如 GitLab、Bitbucket），参考 `docs/guidelines/pr-platform.md`

### Git 工作流和提交规范

**必须遵循**：`docs/guidelines/development.md` 中的 Git 工作流和提交规范。

- **分支策略**：使用 `feature/*`、`fix/*`、`hotfix/*` 分支
- **提交格式**：使用 Conventional Commits 格式（`<type>(<scope>): <subject>`）
- **提交类型**：`feat`、`fix`、`docs`、`style`、`refactor`、`test`、`chore`、`perf`、`ci`
- **提交信息要求**：主题行不超过 50 个字符，使用祈使语气

### 测试规范

**详细规范**：请参考 `docs/guidelines/development.md` 和 `docs/guidelines/testing.md`。

- 单元测试放在对应模块的 `#[cfg(test)]` 模块中
- 集成测试放在 `tests/` 目录
- 使用 `cargo test` 运行测试
- 目标覆盖率：> 80%，关键业务逻辑：> 90%

### 代码审查

**审查清单**：请参考 `docs/guidelines/development.md` 中的代码审查部分。

**代码审查工作流**：
- **提交前检查**（5-15分钟）：参考 `docs/guidelines/workflows/pre-commit.md`
- **综合深入检查**（2-4小时）：参考 `docs/guidelines/workflows/review.md`（功能完成后、定期审查、重大重构前）

提交 PR 前，确保：
- [ ] 代码已格式化（`cargo fmt`）
- [ ] 通过 Clippy 检查（`cargo clippy`）
- [ ] 所有测试通过（`cargo test`）
- [ ] 添加了必要的文档注释
- [ ] 遵循了错误处理规范
- [ ] 提交信息符合规范

### 依赖管理

**详细原则**：请参考 `docs/guidelines/development.md` 中的依赖管理部分。

- 使用 `cargo add <package-name>` 添加依赖
- 优先使用稳定版本的 crate
- 避免不必要的依赖
- 添加新依赖前，考虑是否真的需要、是否有更轻量的替代方案

### 开发工具

**必需工具和常用命令**：请参考 `docs/guidelines/development.md` 中的开发工具部分。

**CI/CD 配置**：
- **CI 工作流配置**：参考 `docs/guidelines/ci-workflow.md`
- **GitHub 配置**：参考 `docs/guidelines/github-setup.md`（Secrets、Variables、分支保护规则等）

**性能分析工具**：
- **二进制大小分析**：参考 `docs/guidelines/cargo-bloat.md`（使用 `cargo-bloat` 分析二进制文件大小）

安装开发工具：`make setup`

## 📝 文档文件管理

**核心原则**：**所有生成的文档文件，根据文档类型自动归类到对应目录；如果无法确定类型，优先检查是否包含"分析"等关键词，匹配则存放到 `analysis/`，否则默认存放到 `docs/requirements/`。**

**重要规则**：文档文件根据类型存放在对应目录，禁止在项目根目录或其他位置随意创建文档文件。

### 文档分类表

| 文档类型 | 目录 | 命名模式 | 关键词（保留中文关键词） |
|---------|------|---------|-------------------------|
| 架构设计文档 | `docs/architecture/` | `{TOPIC}.md` | 架构、架构设计、Architecture、模块架构、系统设计 |
| 开发指南和规范文档 | `docs/guidelines/` | `{TOPIC}.md` | 指南、规范、Guidelines、开发规范、测试规范、文档规范 |
| 版本迁移文档 | `docs/migration/` | `{TOPIC}.md` | 迁移、Migration、版本升级、配置迁移 |
| TODO 文档 | `docs/requirements/` | `{TOPIC}.md` | TODO、待办、待实现、计划 |
| 需求文档 | `docs/requirements/` | `{TOPIC}.md` | 需求、需求分析、功能需求、需求文档 |
| 技术分析文档 | `analysis/` | `{TOPIC}.md` | 分析、ANALYSIS、问题分析、技术分析、测试分析、代码分析、性能分析、架构分析、设计分析、代码审查分析、问题诊断 |
| 实现/实施文档 | `analysis/impl/` | `{TOPIC}.md` | 实现、实施、IMPLEMENTATION、IMPL、实现文档、实施文档、实现方案、实施方案 |
| 分析报告文档 | `report/` | `{TOPIC}.md` | 分析报告、检查报告、代码分析、质量报告（from pre-commit.md） |

**注意**：需求分析、功能说明、实现计划等未明确分类的文档，默认存放到 `docs/requirements/` 目录。

### 文档存放决策流程

1. 检查用户是否明确指定了文档类型或存放位置
2. 如果指定了类型，根据上述分类规则自动归类
3. **重要限制**：
   - **分析文档**（`analysis/`）：**必须**用户明确指明需要生成分析文档时才生成，不能根据关键词自动判断
   - **需求文档**（`docs/requirements/` 中的需求文档）：**必须**用户明确指明需要生成需求文档时才生成，不能根据关键词自动判断
4. 如果未指定类型，检查文档内容中的关键词：
   - 包含"TODO"、"待办"、"待实现"、"计划"、"需求"、"需求分析"、"功能需求"、"需求文档"等关键词 → `docs/requirements/`
   - 其他未明确分类的文档 → `docs/requirements/`
5. 如果无法确定类型，默认存放到 `docs/requirements/`

### 文档命名规范

所有文档都使用 `{TOPIC}.md` 格式。请参考上方的**文档分类表**查看文档类型分类和存储位置。

### 创建新文档时的注意事项

- **文档编写指南**：使用模板创建新文档（参考 `docs/guidelines/document.md`）
  - 根据文档类型选择合适的模板（架构文档、指南文档、需求文档、工作流文档、检查指南文档）
  - 遵循文档编写规范和章节检查清单
- **文档时间戳**：在文档末尾添加"最后更新"时间戳（参考 `docs/guidelines/document-timestamp.md`）
  - 格式：`**最后更新**: YYYY-MM-DD`
  - 位置：文档末尾，分隔线之后
- **文档索引**：更新相应的文档索引（如适用）：
  - TODO 文档：只在 `docs/requirements/README.md` 中索引
  - 其他参考文档：在 `docs/README.md` 中索引
- **文档规范**：确保文档遵循项目的文档编写规范

### 文档索引规则

- **禁止索引**：`analysis/`、`analysis/impl/` 和 `report/` 目录下的文档**绝对不应该**被索引到 `docs/README.md` 或项目根目录 `README.md` 中
- 这些目录包含**临时分析文档和实现文档**，不是参考文档，不需要在文档索引中展示
- **需求文档索引限制**：`docs/requirements/` 目录下的需求文档**只需要**在 `docs/requirements/README.md` 中索引，**不需要**在 `docs/README.md` 或项目根目录 `README.md` 中索引
- **参考文档**：`docs/` 目录下的其他文档（架构文档、指南文档、迁移文档）是参考文档，应该被索引到 `docs/README.md` 中

### 文档删除规则

- **临时文档（可直接删除）**：
  - `analysis/` 目录下的所有文档都是临时技术分析，可以随时删除
  - `analysis/impl/` 目录下的所有文档都是临时实现文档，可以随时删除
  - `report/` 目录下的所有文档都是临时分析报告，可以随时删除
  - 这些文档用于开发过程中的分析、实现和记录，不需要长期保留
- **参考文档（删除需谨慎）**：
  - `docs/` 目录下的文档是**参考文档和架构文档**，删除需要非常注意
  - 包括：架构文档（`docs/architecture/`）、指南文档（`docs/guidelines/`）、迁移文档（`docs/migration/`）、待办文档（`docs/requirements/`）
  - 这些文档是项目的知识库和参考材料，删除前必须确认不再需要
  - **特别说明**：TODO 文档虽然是参考文档，但只在 `docs/requirements/README.md` 中维护索引，不在主文档索引中展示

## ⚙️ 注意事项

1. **跨平台支持**：项目支持 macOS、Linux、Windows，注意平台特定代码
2. **剪贴板功能**：Linux ARM64 和 musl 静态链接版本不支持剪贴板功能
3. **配置文件**：配置文件存储在 `~/.workflow/config/workflow.toml`（macOS/Linux）或 `%APPDATA%\workflow\config\workflow.toml`（Windows）
4. **错误处理**：所有错误都应该提供清晰的错误消息和上下文
5. **日志**：使用 `tracing` 进行日志记录，支持不同日志级别
6. **GitHub 配置**：首次设置项目时，需要配置 GitHub Secrets、Variables 和分支保护规则，参考 `docs/guidelines/github-setup.md`

---

**最后更新**: 2025-12-23


