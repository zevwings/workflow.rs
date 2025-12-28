# 测试目录

## 概述

本目录包含项目的所有测试代码。测试使用统一的测试基础设施和辅助函数，确保测试环境隔离和跨平台兼容性。

## 📚 测试规范文档

详细的测试规范和指南请参考 [`docs/guidelines/testing/`](../docs/guidelines/testing/) 目录：

- **[测试规范索引](../docs/guidelines/testing/README.md)** - 测试规范总览和快速导航
- **[测试组织规范](../docs/guidelines/testing/organization.md)** - 测试类型、目录结构、命名约定
- **[测试编写规范](../docs/guidelines/testing/writing.md)** - AAA模式、命名规范、最佳实践
- **[测试命令参考](../docs/guidelines/testing/commands.md)** - 常用测试命令

### 快速导航

| 主题 | 文档 |
|------|------|
| **测试环境** | [测试环境工具指南](../docs/guidelines/testing/references/environments.md) |
| **路径获取** | [测试辅助工具指南 - 路径获取函数](../docs/guidelines/testing/references/helpers.md#3-路径获取函数) |
| **测试工具** | [测试工具指南](../docs/guidelines/testing/references/tools.md) |
| **Mock 服务器** | [Mock服务器使用指南](../docs/guidelines/testing/references/mock-server.md) |
| **常见问题** | [测试规范索引 - 常见问题](../docs/guidelines/testing/README.md#-常见问题) |

## 📁 目录结构

```
tests/
├── base/              # 基础模块测试
├── cli/               # CLI 命令测试
├── commands/          # 命令实现测试
├── common/            # 共享测试工具和基础设施
│   ├── environments/  # 测试环境（CliTestEnv, GitTestEnv）
│   ├── guards/        # 隔离守卫（EnvGuard, GitConfigGuard）
│   └── helpers.rs     # 测试辅助函数（包含路径获取函数）
├── integration/       # 集成测试
└── fixtures/          # 测试数据文件
```

## 🚀 快速开始

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_example

# 运行被忽略的测试
cargo test -- --ignored

# 显示详细输出
cargo test -- --nocapture
```

### 基本测试示例

```rust
use tests::common::environments::CliTestEnv;
use tests::common::helpers::test_home_dir;
use tests::common::guards::EnvGuard;

#[test]
fn test_example() -> color_eyre::Result<()> {
    // 使用测试环境隔离
    let env = CliTestEnv::new()?;

    // 使用统一的路径获取函数
    let mut guard = EnvGuard::new();
    guard.set("HOME", "/test/home");
    let home = test_home_dir()?;

    // 测试代码...

    Ok(())
}
```

## 📖 相关文档

- **[测试规范文档](../docs/guidelines/testing/README.md)** - 完整的测试规范和指南
- **[平台差异分析](../analysis/platform_differences_analysis.md)** - 平台差异分析和诊断
- **[统一使用 dirs 指南](../analysis/unify_dirs_usage_guide.md)** - 路径获取函数使用指南

---

**最后更新**: 2025-01-28

