# 开发规范与指南

> **⚠️ 同步提示**：此文件必须与其英文版本 `.cursor/rules/development.md` 保持同步。修改此文件时，必须立即更新对应的英文版本。

---

## 开发规范与指南

**重要**：所有开发规范和指南请严格遵循 `docs/guidelines/development/` 目录中的详细说明。

### 开发规范引用

**必须遵守的开发规范文档**：`docs/guidelines/development/README.md`（开发规范索引）

该目录包含完整的开发规范，包括：
- **核心规范**：代码风格、错误处理、命名规范、模块组织
- **流程规范**：Git 工作流、提交规范、代码审查
- **参考文档**：日志、文档、配置管理、安全性、依赖管理、性能优化、开发工具等

### 快速参考

以下是最常用的规范快速参考（**详细说明和完整规范请查看 `docs/guidelines/development/README.md`**）：

- **代码格式化**：`cargo fmt`（提交前必须运行）
- **代码检查**：`cargo clippy -- -D warnings`（所有警告必须修复）
- **命名约定**：模块/函数/变量 `snake_case`，类型/Trait `PascalCase`，常量 `SCREAMING_SNAKE_CASE`
- **导入顺序**：标准库 → 第三方库 → 项目内部
- **错误处理**：使用 `color_eyre::Result<T>` 并为错误消息添加上下文
- **文档注释**：所有公共 API 必须有 `///` 文档注释，包含参数、返回值、错误、示例

### 代码生成规则

**重要**：只有在用户明确要求生成代码时，才执行代码生成操作。

生成代码时必须严格遵循 `docs/guidelines/development/` 目录中的所有开发规范，包括：

- **代码风格**：遵循项目的命名约定和代码风格，使用 `rustfmt` 格式化，通过 `clippy` 检查
- **错误处理**：使用 `color_eyre::Result<T>`，为错误消息添加上下文，考虑所有错误情况
- **文档注释**：为新功能添加完整的文档注释（包含参数、返回值、错误、示例）
- **文档编写**：如果生成文档，必须参考 `docs/guidelines/document.md` 选择合适的模板，并在文档末尾添加"最后更新"时间戳（参考 `docs/guidelines/document-timestamp.md`）
- **API 设计**：遵循公共 API 设计原则，保持向后兼容性，使用 `#[deprecated]` 标记废弃的 API，详细规范请参考 `docs/guidelines/development/references/documentation.md` 中的文档规范章节
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

**必须遵循**：`docs/guidelines/development/git-workflow.md` 和 `docs/guidelines/development/commit.md` 中的 Git 工作流和提交规范。

- **分支策略**：使用 `feature/*`、`fix/*`、`hotfix/*` 分支
- **提交格式**：使用 Conventional Commits 格式（`<type>(<scope>): <subject>`）
- **提交类型**：`feat`、`fix`、`docs`、`style`、`refactor`、`test`、`chore`、`perf`、`ci`
- **提交信息要求**：主题行不超过 50 个字符，使用祈使语气

### 测试规范

**详细规范**：请参考 `docs/guidelines/development/README.md` 和 `docs/guidelines/testing/README.md`。

- 单元测试放在对应模块的 `#[cfg(test)]` 模块中
- 集成测试放在 `tests/` 目录
- 使用 `cargo test` 运行测试
- 目标覆盖率：> 80%，关键业务逻辑：> 90%

### 代码审查

**审查清单**：请参考 `docs/guidelines/development/code-review.md` 中的代码审查规范。

**代码审查工作流**：
- **提交前检查**（5-15分钟）：参考 `docs/guidelines/development/workflows/pre-commit.md`
- **综合深入检查**（2-4小时）：参考 `docs/guidelines/development/workflows/review.md`（功能完成后、定期审查、重大重构前）

提交 PR 前，确保：
- [ ] 代码已格式化（`cargo fmt`）
- [ ] 通过 Clippy 检查（`cargo clippy`）
- [ ] 所有测试通过（`cargo test`）
- [ ] 添加了必要的文档注释
- [ ] 遵循了错误处理规范
- [ ] 提交信息符合规范

### 依赖管理

**详细原则**：请参考 `docs/guidelines/development/references/dependency-management.md` 中的依赖管理规范。

- 使用 `cargo add <package-name>` 添加依赖
- 优先使用稳定版本的 crate
- 避免不必要的依赖
- 添加新依赖前，考虑是否真的需要、是否有更轻量的替代方案

### 开发工具

**必需工具和常用命令**：请参考 `docs/guidelines/development/references/development-tools.md` 中的开发工具规范（如已创建）。

**CI/CD 配置**：
- **CI 工作流配置**：参考 `docs/guidelines/ci-workflow.md`
- **GitHub 配置**：参考 `docs/guidelines/github-setup.md`（Secrets、Variables、分支保护规则等）

**性能分析工具**：
- **二进制大小分析**：参考 `docs/guidelines/cargo-bloat.md`（使用 `cargo-bloat` 分析二进制文件大小）

安装开发工具：`make setup`

---

**最后更新**: 2025-12-25

