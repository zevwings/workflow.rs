# 测试规范

> Workflow CLI 项目的测试规范和最佳实践

---

## 测试类型与组织

| 类型 | 位置 | 说明 |
|------|------|------|
| 单元测试 | 源码同文件 `#[cfg(test)]` | 测试私有逻辑，快速执行 |
| 集成测试 | 各 crate 的 `tests/` | 如 `crates/storage/tests/` |
| 测试工具 | 各 crate 的 `src/testing/` | 可执行代码放此；数据文件放 `tests/fixtures/`。本 crate 用 `#[cfg(test)]`，跨 crate 用 `#[cfg(any(test, feature = "testing"))]` |
| 性能基准 | `crates/*/benches/` | Criterion，报告在 `target/criterion/` |

---

## 编写规范

- **AAA 模式**：Arrange（准备）→ Act（执行）→ Assert（断言）
- **命名**：`test_*`，描述清晰
- **独立性**：不依赖其他测试，避免共享可变状态；需串行时用 `#[serial_test::serial]`

---

## 命令

```bash
# 测试
make test              # 运行测试
make test-all          # 含被忽略的

# 覆盖率
make coverage          # 生成 HTML 报告
make coverage-check    # 检查 ≥75%
make coverage-open     # 打开报告

# 性能
make bench             # 全部基准
make bench-cli         # CLI 启动
make bench-storage     # Storage
make bench-open        # 打开报告
```

按 crate 运行：`cargo test -p domain`、`cargo test -p storage` 等。

---

## 工具

| 工具 | 用途 |
|------|------|
| pretty_assertions | 断言彩色 diff |
| rstest | 参数化测试 |
| insta | 快照测试 |
| mockito | HTTP Mock |
| serial_test | 串行执行标记 |

---

**测试工具说明**：`tests/` 中每个 `.rs` 独立编译，无法被单元测试/其他 crate/基准测试共用，故工具放 `src/testing/`。跨 crate 时需在 Cargo.toml 添加 `testing` feature 和 `#[cfg(any(test, feature = "testing"))] pub mod testing`。

---

## 相关文档

- [开发规范](./development.md)
- [现有测试修改指南](./testing-existing-tests.md)

---

**最后更新**: 2025-02-11
