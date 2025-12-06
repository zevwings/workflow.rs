#!/bin/bash

# 文档重组脚本
# 用途：将已完成的需求文档归档，重组文档结构

set -e  # 遇到错误时退出

echo "📚 开始重组文档..."
echo ""

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误：请在项目根目录运行此脚本"
    exit 1
fi

# 检查是否有未提交的更改
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  警告：有未提交的更改"
    echo "建议先提交或暂存当前更改"
    read -p "是否继续？(y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ 已取消"
        exit 1
    fi
fi

echo "步骤 1: 创建目录结构..."

# 创建归档目录
mkdir -p docs/archive/requirements
mkdir -p docs/guides

echo "✅ 目录结构已创建"
echo ""

echo "步骤 2: 归档已完成的需求文档..."

# 要归档的文档列表
ARCHIVE_DOCS=(
    "dirs-crate-integration.md"
    "dirs-integration-analysis.md"
    "icloud-storage-analysis.md"
    "icloud-storage-implementation.md"
    "implementation-steps.md"
)

for doc in "${ARCHIVE_DOCS[@]}"; do
    if [ -f "docs/requirements/$doc" ]; then
        echo "  📦 归档: $doc"
        git mv "docs/requirements/$doc" "docs/archive/requirements/"
    else
        echo "  ⚠️  跳过（文件不存在）: $doc"
    fi
done

echo "✅ 已完成文档已归档"
echo ""

echo "步骤 3: 转换文档类型..."

# 转换为架构文档
if [ -f "docs/requirements/icloud-storage-decision-flow.md" ]; then
    echo "  🔄 转换为架构文档: icloud-storage-decision-flow.md → ICLOUD_STORAGE_ARCHITECTURE.md"
    git mv "docs/requirements/icloud-storage-decision-flow.md" \
           "docs/architecture/lib/ICLOUD_STORAGE_ARCHITECTURE.md"
else
    echo "  ⚠️  跳过（文件不存在）: icloud-storage-decision-flow.md"
fi

# 转换为用户指南
if [ -f "docs/requirements/icloud-storage-usage-examples.md" ]; then
    echo "  🔄 转换为用户指南: icloud-storage-usage-examples.md → ICLOUD_STORAGE_GUIDE.md"
    git mv "docs/requirements/icloud-storage-usage-examples.md" \
           "docs/guides/ICLOUD_STORAGE_GUIDE.md"
else
    echo "  ⚠️  跳过（文件不存在）: icloud-storage-usage-examples.md"
fi

echo "✅ 文档类型转换完成"
echo ""

echo "步骤 4: 创建归档目录 README..."

# 创建归档 README
cat > docs/archive/requirements/README.md << 'EOF'
# 已归档的需求文档

本目录包含已完成实施的需求文档，作为历史记录保留。

## 📦 归档文档列表

### dirs Crate 集成（已完成）

- `dirs-crate-integration.md` - dirs crate 集成方案
- `dirs-integration-analysis.md` - dirs 集成影响分析

**实施状态**: ✅ 已完成 80%（核心功能）
**归档时间**: 2025-12-06
**参考**: 详见 `docs/requirements/dirs-optimization-analysis.md` 了解剩余优化点

### iCloud 存储支持（已完成）

- `icloud-storage-analysis.md` - iCloud 存储机制分析
- `icloud-storage-implementation.md` - iCloud 实施步骤指南

**实施状态**: ✅ 已完成 95%
**归档时间**: 2025-12-06
**参考**:
- 架构文档: `docs/architecture/lib/ICLOUD_STORAGE_ARCHITECTURE.md`
- 用户指南: `docs/guides/ICLOUD_STORAGE_GUIDE.md`

### 综合实施指南（已过时）

- `implementation-steps.md` - iCloud 存储功能实施步骤指南

**状态**: 已完成，文档已过时
**归档时间**: 2025-12-06

---

## 📝 说明

这些文档已完成其作为需求/实施指南的使命，归档保留作为历史记录。

如需了解当前活跃的需求，请查看：
- `docs/requirements/` - 活跃需求目录
- `docs/requirements/README.md` - 需求文档索引

---

**最后更新**: 2025-12-06
EOF

echo "✅ 归档 README 已创建"
echo ""

echo "步骤 5: 显示当前状态..."
echo ""
echo "当前 requirements 目录内容："
ls -la docs/requirements/ | grep -v "^d" | awk '{print "  " $9}'
echo ""

echo "✅ 文档重组完成！"
echo ""
echo "下一步："
echo "  1. 查看更改: git status"
echo "  2. 查看差异: git diff --staged"
echo "  3. 提交更改: git commit -m 'docs: reorganize requirements documents'"
echo ""
echo "保留的活跃文档："
echo "  - dirs-optimization-analysis.md (追踪剩余优化)"
echo "  - dirs-status-summary.md (状态概览)"
echo "  - third-party-library-analysis.md (第三方库集成)"
echo "  - ui-framework-recommendations.md (UI 改进参考)"
echo "  - README.md (文档索引)"
