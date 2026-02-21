# 开发规范

## 核心规则

- **格式化**：提交前 `cargo fmt`
- **Lint**：`cargo clippy -- -D warnings`，禁止随意 `#[allow]`
- **命名**：模块/函数/变量 `snake_case`，类型/Trait `PascalCase`，常量 `SCREAMING_SNAKE_CASE`
- **导入顺序**：标准库 → 第三方 → 项目内部
- **错误处理**：`anyhow::Result<T>` + `Context` 添加上下文；分层：CLI 校验、命令层友好提示、库层详细错误
- **文档注释**：公共项用 `///`，含参数/返回/错误
- **提交格式**：Conventional Commits（`<type>(<scope>): <subject>`）

---

## 模块组织

- **app**：bin/、commands/、bootstrap/、interactive/
- **domain**：不依赖具体实现
- **共用参数**：`commands/args.rs`，`#[command(flatten)]` 引入

---

## 检查流程

提交前：格式化 → Clippy → 测试通过

---

**最后更新**: 2025-02-20
