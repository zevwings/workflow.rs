# 开发工具脚本

本目录包含用于开发和维护项目的辅助脚本。

## 📁 目录结构

```
scripts/dev/
├── README.md              # 本文档（总览）
├── py/                    # Python Dev 工具（推荐使用）
│   └── README.md          # Python 工具文档
└── shell/                 # Shell 脚本工具
    └── README.md          # Shell 脚本文档
```

## 🆕 Python Dev 工具（推荐使用）

**新的 Python 版本的 dev 工具已可用！** 无需编译，直接运行，速度更快。

### 快速开始

```bash
# 查看所有可用命令
python3 scripts/dev/py/dev.py --help

# 示例：检查 CI 是否应该跳过
python3 scripts/dev/py/dev.py ci check-skip --branch "test" --ci
```

### 📚 文档

- **[Python Dev 工具文档](./py/README.md)** - 完整命令列表和使用指南（包含快速开始、CI/CD 集成、开发指南等）

### ✨ 特性

- ✅ **零依赖**: 完全使用 Python 标准库
- ✅ **快速启动**: 无需编译，直接运行
- ✅ **Python 3.13+**: 使用最新 Python 特性
- ✅ **双重调用**: 支持统一入口和直接运行

### 🎯 已实现的命令（17个）

**CI 和版本管理**:
- ✅ `ci check-skip` - CI 跳过检查
- ✅ `ci verify` - CI 检查验证
- ✅ `version generate` - 版本号生成
- ✅ `tag create` - 创建 Git 标签
- ✅ `tag cleanup` - 清理 Alpha 标签
- ✅ `pr create` - 创建 PR
- ✅ `pr merge` - 合并 PR
- ✅ `homebrew update` - 更新 Homebrew Formula
- ✅ `checksum calculate` - 文件哈希计算

**测试和性能**:
- ✅ `tests check` - 检查测试覆盖率
- ✅ `tests metrics` - 收集测试指标
- ✅ `tests report` - 生成测试报告
- ✅ `tests trends` - 分析测试趋势
- ✅ `performance analyze` - 分析性能数据

**文档检查**:
- ✅ `docs check integrity` - 检查文档完整性
- ✅ `docs check links` - 检查文档链接
- ✅ `docs report generate` - 生成文档报告

---

## 🔧 Shell 脚本工具

用于系统级操作和 CI/CD 环境的 Shell 脚本。

### 📚 文档

- **[Shell 脚本文档](./shell/README.md)** - Shell 脚本使用说明

### 主要脚本

- ✅ `shell/dependencies/install-basic.sh` - 安装 Linux 基本系统依赖
- ✅ `shell/dependencies/install-build.sh` - 安装 Linux 构建依赖

---

## 🎯 迁移状态

### ✅ Python Dev 工具迁移完成

所有命令已迁移完成！所有 workflow 文件已更新为使用 Python dev 工具。

**完成度**: 17/17 命令 (100%) 🎉

### 🔑 关键特性

1. **零依赖**: 完全使用 Python 标准库
2. **GitHub API**: 使用 `urllib.request` 实现，无需第三方库
3. **Git 操作**: 使用 `subprocess` 执行 Git 命令
4. **版本检查**: 要求 Python 3.13+
5. **双重调用**: 支持统一入口和直接运行两种方式

### 📊 性能提升

- **CI 流程**: 从 ~3 分钟编译时间 → 即时运行
- **Release 流程**: 从 ~3 分钟编译时间 → 即时运行
- **总性能提升**: 所有 workflows 节省 ~12 分钟

---

## 📝 注意事项

1. **Python 版本**: Python Dev 工具要求 Python 3.13+
2. **兼容性**: 所有脚本应在 macOS 和 Linux 上正常工作
3. **路径**: 脚本应在项目根目录运行
4. **错误处理**: Shell 脚本使用 `set -e` 确保错误时退出

---

## 🤝 贡献

如需添加新脚本或改进现有脚本，请：

1. 遵循代码风格规范
2. 添加适当的文档注释
3. 测试脚本功能
4. 更新本文档

---

## 📚 相关文档

- [Python Dev 工具文档](./py/README.md) - 完整使用指南
- [Shell 脚本工具文档](./shell/README.md) - Shell 脚本使用说明
- [脚本迁移分析报告](../../docs/requirements/scripts-migration-analysis.md) - 迁移分析
- [脚本迁移实施计划](../../docs/requirements/scripts-migration-plan.md) - 迁移计划
