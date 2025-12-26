#!/bin/bash
# 安全地批量替换测试函数名中的 _return_result 为 _return_ok
#
# 使用方法:
#   1. 预览替换: ./scripts/dev/rename-test-return-result.sh --dry-run
#   2. 执行替换: ./scripts/dev/rename-test-return-result.sh
#
# 安全特性:
#   - 只替换以 "fn test" 开头的函数定义
#   - 不会修改注释、字符串或其他内容
#   - 支持 dry-run 模式预览

set -e

DRY_RUN=false
BACKUP=true

if [ "$1" == "--dry-run" ]; then
    DRY_RUN=true
    echo "🔍 DRY RUN MODE - 只预览，不实际修改"
elif [ "$1" == "--no-backup" ]; then
    BACKUP=false
    echo "⚠️  NO BACKUP MODE - 不会创建备份文件"
fi

# 获取所有包含 _return_result 的测试文件
FILES=$(find tests/ -name "*.rs" -exec grep -l "fn test.*_return_result" {} \; | sort)

TOTAL_FILES=0
TOTAL_REPLACEMENTS=0

for file in $FILES; do
    # 检查文件中是否有 _return_result
    if grep -q "fn test.*_return_result" "$file"; then
        TOTAL_FILES=$((TOTAL_FILES + 1))

        # 统计需要替换的数量
        COUNT=$(grep -c "fn test.*_return_result" "$file" || true)
        TOTAL_REPLACEMENTS=$((TOTAL_REPLACEMENTS + COUNT))

        if [ "$DRY_RUN" = true ]; then
            echo ""
            echo "📄 $file ($COUNT 处需要替换)"
            grep "fn test.*_return_result" "$file" | head -3 | sed 's/^/   /'
            if [ "$COUNT" -gt 3 ]; then
                echo "   ... 还有 $((COUNT - 3)) 处"
            fi
        else
            echo "🔄 处理: $file ($COUNT 处)"

            # 创建备份（如果需要）
            if [ "$BACKUP" = true ]; then
                cp "$file" "$file.bak"
            fi

            # 使用 sed 只替换函数定义中的 _return_result
            # 匹配模式: fn test..._return_result( 或 fn test..._return_result ->
            # 使用更精确的模式：确保 _return_result 在函数名中，后面跟着 ( 或空格+->
            # 先处理单行函数定义: fn test..._return_result(...) -> Result
            sed -i '' 's/\(fn test[^(]*\)_return_result\(([^)]*)\)/\1_return_ok\2/g' "$file"
            # 处理多行函数定义的情况（参数在下一行）: fn test..._return_result\n    -> Result
            sed -i '' 's/\(fn test[^(]*\)_return_result\([[:space:]]*->\)/\1_return_ok\2/g' "$file"

            # 验证替换结果
            REMAINING=$(grep -c "fn test.*_return_result" "$file" 2>/dev/null || echo "0")
            if [ -n "$REMAINING" ] && [ "$REMAINING" -gt 0 ] 2>/dev/null; then
                echo "   ⚠️  警告: 仍有 $REMAINING 处未替换（可能是多行定义或其他格式）"
            fi
        fi
    fi
done

if [ "$DRY_RUN" = true ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📊 统计:"
    echo "   文件数: $TOTAL_FILES"
    echo "   替换数: $TOTAL_REPLACEMENTS"
    echo ""
    echo "💡 要执行替换，请运行: $0"
else
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ 完成!"
    echo "   处理文件: $TOTAL_FILES"
    echo "   替换数量: $TOTAL_REPLACEMENTS"

    if [ "$BACKUP" = true ]; then
        BACKUP_COUNT=$(find tests/ -name "*.bak" | wc -l | tr -d ' ')
        echo "   备份文件: $BACKUP_COUNT (.bak 文件)"
        echo ""
        echo "💡 如需恢复备份，运行: find tests/ -name '*.bak' -exec sh -c 'mv \"\$1\" \"\${1%.bak}\"' _ {} \\;"
    fi

    echo ""
    echo "💡 建议运行测试验证: cargo test"
    echo "💡 验证替换结果: grep -r '_return_result' tests/ | grep 'fn test'"
fi

