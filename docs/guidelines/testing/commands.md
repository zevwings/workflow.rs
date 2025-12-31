# 测试命令参考

> 本文档提供常用测试命令的快速参考。

---

## 📋 目录

- [基本测试命令](#-基本测试命令)
- [测试类型命令](#-测试类型命令)
- [Makefile测试命令](#makefile测试命令)
- [测试调试](#-测试调试)

---

## 🚀 基本测试命令

### 运行测试

**运行所有测试**：
```bash
# 使用 Cargo
cargo test

# 使用 Makefile
make test
```

**运行特定测试**：
```bash
# 运行特定模块的测试
cargo test --lib 模块名

# 运行特定测试文件
cargo test --test 测试文件名

# 运行匹配模式的测试
cargo test test_parse_url

# 运行被忽略的测试
cargo test -- --ignored

# 运行所有测试（包括被忽略的）
make test-all
```

**测试输出选项**：
```bash
# 显示详细输出
cargo test -- --nocapture

# 显示测试执行时间
cargo test -- --nocapture --test-threads=1

# 只运行失败的测试（需要先运行一次）
cargo test -- --failed
```

---

## 🎯 测试类型命令

### 单元测试

```bash
# 运行所有单元测试
cargo test --lib

# 运行特定模块的单元测试
cargo test --lib 模块名::函数名
```

### 集成测试

```bash
# 运行所有集成测试
cargo test --test '*'

# 运行特定集成测试
cargo test --test integration_test
```

### 模块级集成测试

```bash
# 运行模块级集成测试（module_test）
# 这些测试运行较快，适合频繁运行
cargo test --test module_test

# 运行特定模块的测试
cargo test --test module_test base::format
```

### 端到端集成测试

```bash
# 运行端到端集成测试（e2e_test）
# 这些测试运行较慢，需要 Mock 服务器、Git 仓库等完整环境
cargo test --test e2e_test

# 运行特定的端到端测试
cargo test --test e2e_test e2e::end_to_end
```

### 文档测试

```bash
# 运行文档中的代码示例（doctest）
cargo test --doc

# 运行特定模块的文档测试
cargo test --doc 模块名
```

---

## Makefile测试命令

项目提供了便捷的 Makefile 命令：

```bash
# 运行所有测试
make test

# 运行所有测试（包括被忽略的）
make test-all

# 生成覆盖率报告
make coverage

# 打开覆盖率报告
make coverage-open

# CI 环境覆盖率检查
make coverage-ci

# 查看覆盖率趋势
make coverage-trend
```

---

## 🐛 测试调试

### 运行单个测试

```bash
# 运行单个测试函数
cargo test test_parse_url -- --nocapture

# 运行单个测试并显示详细输出
cargo test test_parse_url -- --nocapture --test-threads=1
```

### 测试失败时调试

```bash
# 显示失败的测试输出
cargo test -- --nocapture

# 只运行失败的测试
cargo test -- --failed
```

---

## 📊 常用命令组合

### 开发时常用

```bash
# 快速测试（只运行单元测试）
cargo test --lib

# 详细测试输出
cargo test -- --nocapture

# 测试并显示覆盖率
make coverage && make coverage-open
```

### CI 环境常用

```bash
# 运行所有测试
cargo test

# 运行所有测试（包括被忽略的）
cargo test -- --ignored

# 生成覆盖率报告
make coverage-ci
```

---

## 相关文档

- [测试组织规范](./organization.md) - 测试组织结构
- [测试编写规范](./writing.md) - 测试编写规范
- [覆盖率测试指南](./references/coverage.md) - 覆盖率工具详细使用

---

**最后更新**: 2025-12-25

