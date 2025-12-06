#!/bin/bash

# 文档清理脚本（删除版本）
# 用途：直接删除已完成的需求文档（不归档）

set -e  # 遇到错误时退出

echo "🗑️  文档清理脚本（删除模式）"
echo ""
echo "⚠️  警告：此脚本会永久删除已完成的需求文档"
echo "建议使用 reorganize-docs.sh 进行归档而非删除"
echo ""

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误：请在项目根目录运行此脚本"
    exit 1
fi

# 确认操作
read -p "确认要删除已完成的文档吗？(yes/NO) " -r
echo
if [[ ! $REPLY = "yes" ]]; then
    echo "❌ 已取消"
    exit 1
fi

echo "开始清理..."
echo ""

# 要删除的文档列表
DELETE_DOCS=(
    "dirs-crate-integration.md"
    "dirs-integration-analysis.md"
    "icloud-storage-analysis.md"
    "icloud-storage-implementation.md"
    "implementation-steps.md"
)

# 要移动的文档（转换为其他类型）
echo "步骤 1: 转换文档类型..."

# 创建目录（如果不存在）
mkdir -p docs/guides

# 转换为架构文档
if [ -f "docs/requirements/icloud-storage-decision-flow.md" ]; then
    echo "  🔄 转换: icloud-storage-decision-flow.md → ICLOUD_STORAGE_ARCHITECTURE.md"
    git mv "docs/requirements/icloud-storage-decision-flow.md" \
           "docs/architecture/lib/ICLOUD_STORAGE_ARCHITECTURE.md"
else
    echo "  ⚠️  跳过（文件不存在）: icloud-storage-decision-flow.md"
fi

# 转换为用户指南
if [ -f "docs/requirements/icloud-storage-usage-examples.md" ]; then
    echo "  🔄 转换: icloud-storage-usage-examples.md → ICLOUD_STORAGE_GUIDE.md"
    git mv "docs/requirements/icloud-storage-usage-examples.md" \
           "docs/guides/ICLOUD_STORAGE_GUIDE.md"
else
    echo "  ⚠️  跳过（文件不存在）: icloud-storage-usage-examples.md"
fi

echo "✅ 文档转换完成"
echo ""

echo "步骤 2: 删除已完成的需求文档..."

for doc in "${DELETE_DOCS[@]}"; do
    if [ -f "docs/requirements/$doc" ]; then
        echo "  🗑️  删除: $doc"
        git rm "docs/requirements/$doc"
    else
        echo "  ⚠️  跳过（文件不存在）: $doc"
    fi
done

echo "✅ 已完成文档已删除"
echo ""

echo "步骤 3: 显示当前状态..."
echo ""
echo "当前 requirements 目录内容："
ls -la docs/requirements/ | grep -v "^d" | awk '{print "  " $9}'
echo ""

echo "✅ 清理完成！"
echo ""
echo "下一步："
echo "  1. 查看更改: git status"
echo "  2. 查看差异: git diff --staged"
echo "  3. 提交更改: git commit -m 'docs: remove completed requirements documents'"
echo ""
echo "保留的活跃文档："
echo "  - dirs-optimization-analysis.md (追踪剩余优化)"
echo "  - dirs-status-summary.md (状态概览)"
echo "  - third-party-library-analysis.md (第三方库集成)"
echo "  - ui-framework-recommendations.md (UI 改进参考)"
echo "  - README.md (文档索引)"
echo ""
echo "转换的文档："
echo "  - docs/architecture/lib/ICLOUD_STORAGE_ARCHITECTURE.md"
echo "  - docs/guides/ICLOUD_STORAGE_GUIDE.md"
