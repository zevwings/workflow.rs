# GitHub Actions 自动化打包配置指南

本文档说明如何配置和使用 GitHub Actions 进行自动化打包和发布（方案二：完整自动化）。

## 📋 功能概览

本自动化流程实现了以下功能：

1. ✅ **多平台构建**：自动构建 macOS Intel/ARM 和 Linux x86_64/ARM 四个平台
2. ✅ **自动创建 Release**：Push tag 后自动创建 GitHub Release
3. ✅ **自动计算 SHA256**：自动计算所有二进制文件的 SHA256 校验和
4. ✅ **自动更新 Homebrew Formula**：自动更新 Formula 文件中的版本号和 SHA256
5. ✅ **自动提交到 Tap 仓库**：自动提交更新到 `homebrew-workflow` 仓库

## 🔧 前置条件

### 1. 创建 Homebrew Tap 仓库

首先需要在 GitHub 上创建 Homebrew tap 仓库：

```bash
# 使用 GitHub CLI 创建
gh repo create zevwings/homebrew-workflow --public --clone

# 或者手动在 GitHub 上创建
# 仓库名称必须是：homebrew-workflow
# 默认分支：main 或 master
```

### 2. 初始化 Tap 仓库

```bash
cd homebrew-workflow
mkdir -p Formula

# 从主仓库复制 Formula 文件
cp /path/to/workflow.rs/Formula/workflow.rb Formula/workflow.rb

# 提交并推送
git add Formula/workflow.rb
git commit -m "Add workflow formula"
git push origin main  # 或 master
```

### 3. 配置 GitHub Secrets

在 GitHub 仓库设置中添加以下 Secret：

1. 进入仓库：`Settings` → `Secrets and variables` → `Actions`
2. 添加 Secret：`HOMEBREW_TAP_TOKEN`

**如何获取 Token：**

```bash
# 使用 GitHub CLI 创建 Personal Access Token
gh auth token

# 或者手动创建：
# 1. 访问 https://github.com/settings/tokens
# 2. 点击 "Generate new token (classic)"
# 3. 选择权限：repo（完整权限）
# 4. 复制生成的 token
```

**重要提示：**
- Token 需要有对 `zevwings/homebrew-workflow` 仓库的写权限
- 如果 tap 仓库是公开的，可以使用 `GITHUB_TOKEN`（但需要额外配置）

## 🚀 使用方法

### 方式一：通过 Git Tag 触发（推荐）

```bash
# 1. 更新版本号（如果需要）
# 编辑 Cargo.toml 中的 version

# 2. 提交更改
git add .
git commit -m "Release v0.1.0"

# 3. 创建并推送 tag
git tag v0.1.0
git push origin v0.1.0
```

推送 tag 后，GitHub Actions 会自动：
1. 构建所有平台的二进制文件
2. 创建 GitHub Release
3. 上传二进制文件
4. 更新 Homebrew Formula
5. 提交到 tap 仓库

### 方式二：手动触发

1. 进入 GitHub 仓库
2. 点击 `Actions` 标签
3. 选择 `Release` 工作流
4. 点击 `Run workflow`
5. 输入版本号（如 `v0.1.0`）
6. 点击 `Run workflow`

## 📦 工作流程详解

### 1. 构建阶段（Build Job）

- **触发**：Push tag 或手动触发
- **平台**：
  - macOS Intel (x86_64-apple-darwin)
  - macOS ARM (aarch64-apple-darwin)
  - Linux x86_64 (x86_64-unknown-linux-gnu)
  - Linux ARM (aarch64-unknown-linux-gnu)
- **输出**：
  - 二进制文件压缩包（`.tar.gz`）
  - SHA256 校验和文件

### 2. 发布阶段（Release Job）

- **依赖**：等待所有构建完成
- **功能**：
  - 下载所有平台的构建产物
  - 创建 GitHub Release
  - 上传所有二进制文件
  - 生成 Release 说明

### 3. 更新 Homebrew 阶段（Update Homebrew Job）

- **依赖**：等待构建和发布完成
- **功能**：
  - 读取 SHA256 值
  - 更新 Formula 文件
  - 验证更新
  - 提交到 tap 仓库

## 🔍 验证和调试

### 检查工作流状态

1. 进入 GitHub 仓库
2. 点击 `Actions` 标签
3. 查看工作流运行状态
4. 点击具体的运行查看详细日志

### 常见问题排查

#### 1. 构建失败

**问题**：某个平台的构建失败

**排查步骤**：
- 检查构建日志中的错误信息
- 确认 Rust 工具链是否正确安装
- 检查是否有平台特定的依赖问题

#### 2. Release 创建失败

**问题**：Release 创建失败

**排查步骤**：
- 检查 `GITHUB_TOKEN` 权限
- 确认 tag 是否已存在
- 检查 Release 名称是否冲突

#### 3. Homebrew Formula 更新失败

**问题**：Formula 更新或提交失败

**排查步骤**：
- 检查 `HOMEBREW_TAP_TOKEN` 是否正确配置
- 确认 tap 仓库是否存在且有写权限
- 检查 Formula 文件格式是否正确
- 查看工作流日志中的错误信息

#### 4. SHA256 提取失败

**问题**：无法提取 SHA256 值

**排查步骤**：
- 检查构建阶段是否成功生成了 SHA256 文件
- 确认文件路径和格式是否正确
- 查看工作流日志中的 SHA256 提取步骤

## 📝 Formula 文件结构

更新后的 Formula 文件结构如下：

```ruby
class Workflow < Formula
  desc "Workflow CLI tool for PR management, Jira integration, and log processing"
  homepage "https://github.com/zevwings/workflow.rs"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/zevwings/workflow.rs/releases/download/v0.1.0/workflow-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "实际的SHA256值"
    end
    if Hardware::CPU.arm?
      url "https://github.com/zevwings/workflow.rs/releases/download/v0.1.0/workflow-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "实际的SHA256值"
    end
  end

  def install
    bin.install "workflow"
    bin.install "pr"
    bin.install "qk"
  end

  test do
    system "#{bin}/workflow", "--help"
    system "#{bin}/pr", "--help"
    system "#{bin}/qk", "--help"
  end
end
```

## 🔐 安全注意事项

1. **Token 安全**：
   - 不要在代码中硬编码 token
   - 使用 GitHub Secrets 存储敏感信息
   - 定期轮换 token

2. **权限最小化**：
   - Token 只授予必要的权限
   - 如果可能，使用只读权限的 token

3. **仓库访问**：
   - 确保 tap 仓库的访问控制正确配置
   - 如果使用私有仓库，确保 token 有相应权限

## 📊 工作流文件位置

工作流文件位于：
```
.github/workflows/release.yml
```

## 🎯 最佳实践

1. **版本管理**：
   - 使用语义化版本（Semantic Versioning）
   - Tag 命名格式：`v0.1.0`、`v1.0.0` 等

2. **测试**：
   - 在发布前本地测试构建
   - 验证 Formula 文件格式

3. **监控**：
   - 定期检查工作流运行状态
   - 设置通知以便及时发现问题

4. **文档**：
   - 保持文档与代码同步
   - 记录重要的配置变更

## 🆘 获取帮助

如果遇到问题：

1. 查看工作流日志
2. 检查 GitHub Actions 文档
3. 参考 Homebrew Formula 文档
4. 查看项目 Issues

## ✅ 检查清单

在首次使用前，请确认：

- [ ] Homebrew tap 仓库已创建
- [ ] Formula 文件已推送到 tap 仓库
- [ ] `HOMEBREW_TAP_TOKEN` 已配置
- [ ] 工作流文件 `.github/workflows/release.yml` 存在
- [ ] 本地测试构建成功
- [ ] 版本号已更新（如需要）

完成以上检查后，就可以开始使用自动化打包流程了！

