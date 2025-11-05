# Homebrew 安装指南

本项目支持通过 Homebrew 安装。有两种方式：

## 方式一：从源码构建（当前推荐）

如果项目还没有发布 GitHub Releases，可以使用从源码构建的方式。

### 1. 创建 Homebrew Tap

首先，在 GitHub 上创建一个新的仓库用于存放 Homebrew formula，命名为 `homebrew-workflow`：

**重要**：创建仓库时，请确保：
- 仓库名称：`homebrew-workflow`
- 默认分支：选择 `main`（推荐）或 `master`
- 不要初始化 README、.gitignore 或 license（创建空仓库）

在 GitHub 上创建新仓库：https://github.com/new

或者使用 GitHub CLI：

```bash
gh repo create zevwings/homebrew-workflow --public --clone
cd homebrew-workflow
```

### 2. 设置 Formula 文件

将 `Formula/workflow.rb` 文件推送到 tap 仓库：

```bash
# 如果还未克隆，先克隆 tap 仓库
git clone git@github.com:zevwings/homebrew-workflow.git
cd homebrew-workflow

# 创建 Formula 目录
mkdir -p Formula

# 复制 formula 文件（从当前项目目录）
cp /path/to/workflow.rs/Formula/workflow.rb Formula/workflow.rb

# 提交并推送
git add Formula/workflow.rb
git commit -m "Add workflow formula"

# 根据仓库的默认分支推送（如果使用 main）
git push origin main

# 或者如果使用 master
# git push origin master
```

**注意**：如果遇到 "Cannot determine remote HEAD" 错误，通常是因为：
1. Tap 仓库尚未创建
2. Tap 仓库是空的（没有任何提交）
3. 默认分支名称不匹配（Homebrew 期望 `main`，但仓库使用 `master`）

如果使用 `master` 作为默认分支，可以将其重命名为 `main`：

```bash
git branch -m master main
git push origin main
git push origin --delete master
```

### 3. 安装

用户可以通过以下命令安装：

```bash
brew tap zevwings/workflow
brew install workflow
```

## 方式二：使用 GitHub Releases（推荐用于生产环境）

如果项目已经发布了 GitHub Releases，可以使用预编译的二进制文件，这样安装更快。

### 自动化流程

**重要**：本项目已配置 GitHub Actions 自动化流程，当推送 tag（如 `v0.1.0`）时会自动：

1. **自动构建**：并行构建所有平台的二进制文件（macOS Intel/ARM、Linux x86_64/ARM）
2. **自动创建 Release**：创建 GitHub Release 并上传所有平台的二进制文件
3. **自动更新 Formula**：自动计算 SHA256 并更新 Homebrew Formula，提交到 `homebrew-workflow` 仓库

因此，**无需手动操作**，只需：

```bash
# 1. 创建并推送 tag
git tag v0.1.0
git push origin v0.1.0

# 2. GitHub Actions 会自动完成所有工作
# 3. 用户可以直接安装
brew tap zevwings/workflow
brew install workflow
```

### 手动创建 Release（不推荐）

如果需要手动创建 Release，可以参考 `.github/workflows/release.yml` 中的构建流程。

### 安装

```bash
brew tap zevwings/workflow
brew install workflow
```

## 方式三：本地安装（开发测试）

如果只是想本地测试，可以直接从本地路径安装：

```bash
brew install --build-from-source /path/to/workflow/Formula/workflow.rb
```

或者使用 `brew` 的 `--HEAD` 选项从 Git 仓库安装最新版本：

```bash
brew install --HEAD https://raw.githubusercontent.com/zevwings/workflow.rs/master/Formula/workflow.rb
```

## 验证安装

安装完成后，验证是否安装成功：

```bash
workflow --help
pr --help
qk --help
```

## 更新 Formula

当项目有新版本时，**无需手动更新**。GitHub Actions 会自动：

1. 构建新版本的二进制文件
2. 创建 GitHub Release
3. 自动计算 SHA256
4. 自动更新 Formula 文件
5. 自动提交到 `homebrew-workflow` 仓库

只需推送新的 tag 即可：

```bash
git tag v0.2.0
git push origin v0.2.0
```

## 故障排除

### 错误：Cannot determine remote HEAD / ambiguous argument 'refs/remotes/origin/main'

**原因**：Tap 仓库尚未创建，或者仓库是空的，或者默认分支名称不匹配。

**解决方案**：

1. **确保 tap 仓库已创建**：
   ```bash
   # 检查仓库是否存在
   gh repo view zevwings/homebrew-workflow
   # 或者访问 https://github.com/zevwings/homebrew-workflow
   ```

2. **如果仓库不存在，创建它**：
   ```bash
   gh repo create zevwings/homebrew-workflow --public
   cd /tmp
   git clone git@github.com:zevwings/homebrew-workflow.git
   cd homebrew-workflow
   mkdir -p Formula
   cp /path/to/workflow.rs/Formula/workflow.rb Formula/workflow.rb
   git add Formula/workflow.rb
   git commit -m "Add workflow formula"
   git push origin main  # 或 master
   ```

3. **如果仓库使用 `master` 但 Homebrew 期望 `main`**：
   - 在 GitHub 上：Settings → Branches → Default branch → 重命名为 `main`
   - 或者在本地：
     ```bash
     git branch -m master main
     git push origin main
     git push origin --delete master
     ```

### 错误：Repository not found

**原因**：Tap 仓库不存在或没有访问权限。

**解决方案**：确保已在 GitHub 上创建 `homebrew-workflow` 仓库，并且仓库是公开的（public）。

## 注意事项

1. **Tap 仓库命名**：Homebrew tap 仓库必须遵循 `homebrew-<name>` 的命名规范
2. **Formula 文件位置**：在 tap 仓库中，formula 文件应该放在 `Formula/` 目录下
3. **版本号**：确保 Formula 中的版本号与 `Cargo.toml` 中的版本号一致
4. **SHA256**：如果使用预编译二进制，必须提供正确的 SHA256 校验和
5. **二进制文件**：确保所有三个二进制文件（`workflow`、`pr`、`qk`）都被正确安装
6. **默认分支**：推荐使用 `main` 作为默认分支（符合现代 Git 规范）

## 参考资源

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Tap 创建指南](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Rust 项目 Homebrew Formula 示例](https://github.com/Homebrew/homebrew-core/blob/master/Formula/ripgrep.rb)

