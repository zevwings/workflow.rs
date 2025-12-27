# Python Dev 工具

## 快速开始

### 要求

- **Python 3.13+** （脚本会自动检查版本）

### 基本使用

```bash
python3 scripts/dev/dev.py <command> [options]
```

### 可用命令

- `ci check-skip` - 检查是否应该跳过 CI

更多详细信息请参考：
- [PYTHON_DEV_TOOL.md](./PYTHON_DEV_TOOL.md) - 完整使用文档
- [CI_USAGE.md](./CI_USAGE.md) - CI/CD 集成指南

## 版本检查

脚本会在启动时自动检查 Python 版本。如果版本不符合要求（< 3.13），会显示错误并退出：

```bash
$ python3 scripts/dev/dev.py --help
❌ Error: Python 3.13+ required, but found 3.12.0
   Current version: 3.12.0 (main, Oct 24 2023, 10:45:00)
```

## 为什么要求 Python 3.13？

Python 3.13 引入了许多性能改进和新特性，确保脚本能够：
- 更快的启动速度
- 更好的错误处理
- 更现代的标准库功能

## 在 CI 中使用

在 GitHub Actions 中，需要先设置 Python 3.13：

```yaml
- name: 🐍 Setup Python 3.13
  uses: actions/setup-python@v5
  with:
    python-version: '3.13'

- name: 🔍 Check CI skip
  run: python3 scripts/dev/dev.py ci check-skip --ci
```

详见 [CI_USAGE.md](./CI_USAGE.md)。

