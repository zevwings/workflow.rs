# 开发规范

> Workflow CLI 项目的开发规范和最佳实践

---

## 🛠️ 开发环境

```bash
make setup   # 安装 rustfmt、clippy、rust-analyzer、cargo-bloat、cargo-audit、cargo-outdated、cargo-tarpaulin、lychee，并配置 Git hooks
```

**常用命令**：`make help` 查看全部。核心：`make lint`（格式化 + Clippy + Check）、`make fix`（自动修复）、`make test`、`make bench`、`make coverage`、`make check-docs-links`。

---

## 🎨 代码风格

- **格式化**：`cargo fmt`，提交前必须运行；CI 会 `cargo fmt --check`
- **Lint**：`make lint` 或 `cargo clippy --all-targets --all-features -- -D warnings`；禁止随意使用 `#[allow(clippy::xxx)]`
- **命名**：模块/函数/变量 `snake_case`，类型/Trait `PascalCase`，常量 `SCREAMING_SNAKE_CASE`
- **导入顺序**：标准库 → 第三方 → 项目内部

---

## ⚠️ 错误处理

统一使用 `anyhow::Result<T>`，用 `Context` 添加上下文、`bail!` 快速返回、`ensure!` 断言。分层：CLI 用 clap 校验，命令层友好提示，库层详细错误。

---

## 🏷️ 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 文件 | `snake_case.rs` | `jira_client.rs` |
| 动作函数 | 动词 | `download`、`create` |
| 查询/检查 | `get_`、`is_`、`has_` | `get_status`、`is_valid` |
| CLI 字段 | `snake_case`，`value_name` 用 `SCREAMING_SNAKE_CASE` | `jira_id`、`JIRA_ID` |

共用参数提取到 `crates/app/src/commands/args.rs`，用 `#[command(flatten)]` 引入。

---

## 📁 模块组织

```
crates/
├── app/        # CLI 入口：bin/、commands/、bootstrap/、interactive/
├── domain/     # 领域模型与仓储 trait
├── storage/    # Git/GitHub/Jira 等存储实现
├── services/   # 应用服务
├── toolkit/    # 日志、路径、模板、工具
├── prompt/     # 交互与输出
├── di/         # 依赖注入
├── http/       # HTTP 客户端
└── llm/        # LLM 相关
```

**依赖规则**：app 依赖 domain、storage、services、toolkit、prompt、di；domain 不依赖具体实现。

---

## 📝 文档与提交

- **代码文档**：公共项用 `///`，含参数/返回/错误说明；`cargo doc --open` 查看
- **提交格式**：[Conventional Commits](https://www.conventionalcommits.org/)：`<type>(<scope>): <subject>`，类型：feat、fix、docs、style、refactor、test、chore、perf、ci

---

## 🔍 检查流程

**提交前**（`make lint && make test`）：

1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo check`
4. `cargo test`

**可选**：`make install-hooks` 安装 pre-commit（prek），提交时自动执行上述检查。

**版本/发布**：feat/fix 提交时检查 `Cargo.toml` 中 `[workspace.package]` 的 version 及 CHANGELOG.md。

---

## 🔧 CI/CD

- **CI**（`.github/workflows/ci.yml`）：PR 到 master/main → 格式化、Clippy、测试、多平台构建（macOS/Linux/Windows，含 ARM64）
- **Release**（`.github/workflows/release.yml`）：合并到 master → 质量检查、打 tag、多平台构建、发 Release、更新 Homebrew

---

## 📚 相关文档

- [架构设计](./architecture.md)
- [测试规范](./testing.md)
- [迁移文档](../migration/README.md)

**最后更新**: 2025-02-11
