# Shell 脚本工具

用于系统级操作和 CI/CD 环境的 Shell 脚本。

## 依赖安装脚本

### install-basic.sh

安装 Linux 基本系统依赖（XCB 开发库、Python3）。

```bash
./scripts/dev/shell/deps/install-basic.sh
```

### install-build.sh

安装 Linux 构建依赖（基本依赖 + 构建工具）。

```bash
./scripts/dev/shell/deps/install-build.sh
```

## Git Hooks

- `hooks/install-hooks.sh` - 安装 Git pre-commit hook（Bash）
- `hooks/install-hooks.ps1` - 安装 Git pre-commit hook（PowerShell）
