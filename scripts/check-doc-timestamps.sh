#!/bin/bash

# 文档更新时间检查脚本
# 用于验证所有文档都有统一格式的更新时间标记

echo "=== 文档更新时间标准化检查 ==="

# 检查是否有使用中文冒号的格式（排除文档示例和命令示例）
echo "1. 检查格式一致性（中文冒号，应该为0）："
# 检查实际的更新时间标记（排除代码块中的示例）
# 通过检查文件末尾几行来避免匹配代码块中的示例
chinese_colon_count=0
for file in $(find . -name "*.md" | grep -v ".git" | grep -v "report/"); do
    # 检查文件的最后5行是否包含中文冒号的更新时间格式
    if tail -5 "$file" | grep -q "^\*\*最后更新：\*\*"; then
        chinese_colon_count=$((chinese_colon_count + 1))
    fi
done
echo "   发现中文冒号格式: $chinese_colon_count"

# 检查总的更新时间标记数量
echo "2. 统计更新时间标记数量："
total_count=$(grep -r "最后更新.*:" . --include="*.md" | grep -v ".git" | grep -v "report/" | grep -E "20[0-9]{2}-[0-9]{2}-[0-9]{2}" | wc -l)
echo "   包含更新时间的文档: $total_count"

# 检查日期格式是否正确 (YYYY-MM-DD)
echo "3. 验证日期格式 (YYYY-MM-DD)："
valid_format_count=$(grep -r "最后更新.*:" . --include="*.md" | grep -v ".git" | grep -v "report/" | grep -E "20[0-9]{2}-[0-9]{2}-[0-9]{2}" | wc -l)
echo "   正确格式的文档: $valid_format_count"

# 检查是否有缺少更新时间的重要文档
echo "4. 检查重要文档的更新时间："
important_docs=("README.md" "CHANGELOG.md" "docs/README.md")
for doc in "${important_docs[@]}"; do
    if [ -f "$doc" ]; then
        if grep -q "最后更新.*:" "$doc"; then
            echo "   ✅ $doc - 有更新时间"
        else
            echo "   ❌ $doc - 缺少更新时间"
        fi
    fi
done

echo ""
if [ "$chinese_colon_count" -eq 0 ]; then
    echo "✅ 格式检查通过 - 所有文档使用统一的英文冒号格式"
else
    echo "❌ 格式检查失败 - 发现 $chinese_colon_count 个中文冒号格式"
fi

echo "📊 总结: $total_count 个文档包含更新时间标记"
