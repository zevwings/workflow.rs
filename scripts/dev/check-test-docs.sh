#!/bin/bash
# 检查测试文件文档注释完成情况

set -e

echo "=== 测试文档注释统计 ==="
echo ""

TOTAL_TESTS=0
TOTAL_DOCUMENTED=0
TOTAL_FILES=0
FILES_100=0
FILES_PARTIAL=0
FILES_0=0

# 高优先级文件（≥20个测试）
HIGH_PRIORITY_0=0
HIGH_PRIORITY_PARTIAL=0
HIGH_PRIORITY_MISSING=0

# 中优先级文件（10-19个测试）
MEDIUM_PRIORITY_0=0
MEDIUM_PRIORITY_PARTIAL=0
MEDIUM_PRIORITY_MISSING=0

# 低优先级文件（<10个测试）
LOW_PRIORITY_0=0
LOW_PRIORITY_PARTIAL=0
LOW_PRIORITY_MISSING=0

# 临时文件存储结果
TEMP_FILE=$(mktemp)
HIGH_PARTIAL_FILE=$(mktemp)
MEDIUM_PARTIAL_FILE=$(mktemp)
LOW_0_FILE=$(mktemp)
LOW_PARTIAL_FILE=$(mktemp)

# 检查每个测试文件
for file in $(find tests -name "*.rs" -type f | sort); do
    # 跳过工具和辅助文件
    if [[ "$file" == *"/common/"* ]] || [[ "$file" == *"/utils/"* ]] || [[ "$file" == *"/fixtures/"* ]]; then
        continue
    fi

    # 统计测试函数数量
    TEST_COUNT=$(grep -c "^    #\[test\]" "$file" 2>/dev/null || echo 0)

    if [ "$TEST_COUNT" -eq 0 ]; then
        continue
    fi

    TOTAL_TESTS=$((TOTAL_TESTS + TEST_COUNT))
    TOTAL_FILES=$((TOTAL_FILES + 1))

    # 统计有文档注释的测试函数
    # 查找测试函数前是否有文档注释（包含"测试目的"、"测试场景"或"预期结果"）
    DOCUMENTED=0

    # 读取文件内容，逐行分析
    while IFS= read -r line; do
        if [[ "$line" =~ ^[[:space:]]*#\[test\] ]]; then
            # 找到测试函数，检查前面是否有文档注释
            # 向前查找最多20行，看是否有包含"测试目的"、"测试场景"或"预期结果"的注释
            # 这里简化处理：检查测试函数前5行是否有"///"
            # 实际应该更复杂，但为了快速统计，先这样
            :
        fi
    done < "$file"

    # 更准确的方法：统计每个测试函数前是否有文档注释
    # 使用awk来更准确地统计
    DOCUMENTED=$(awk '
    BEGIN {
        in_doc=0
        has_doc=0
        test_count=0
        documented_count=0
    }
    /^[[:space:]]*\/\/\/.*(测试目的|测试场景|预期结果|## 测试目的|## 测试场景|## 预期结果)/ {
        in_doc=1
        has_doc=1
    }
    /^[[:space:]]*#\[test\]/ {
        if (has_doc) {
            documented_count++
        }
        test_count++
        has_doc=0
        in_doc=0
    }
    /^[[:space:]]*fn test_/ {
        if (has_doc) {
            documented_count++
        }
        test_count++
        has_doc=0
        in_doc=0
    }
    END {
        print documented_count " " test_count
    }' "$file" | awk '{print $1}')

    TEST_COUNT=$(awk '
    BEGIN { test_count=0 }
    /^[[:space:]]*#\[test\]/ { test_count++ }
    END { print test_count }
    ' "$file")

    # 更简单的方法：统计测试函数前有"///"注释的数量
    # 查找模式：测试函数前有包含"测试目的"、"测试场景"或"预期结果"的注释
    DOCUMENTED=$(python3 << EOF
import re
import sys

with open('$file', 'r') as f:
    content = f.read()

# 查找所有测试函数
test_pattern = r'^\s*#\[test\].*?\n\s*fn\s+(test_\w+)'
tests = re.finditer(test_pattern, content, re.MULTILINE)

test_count = 0
documented_count = 0

for test_match in tests:
    test_count += 1
    test_start = test_match.start()

    # 向前查找最多50行，看是否有文档注释
    lines_before = content[:test_start].split('\n')
    found_doc = False

    # 检查前50行
    for i in range(max(0, len(lines_before) - 50), len(lines_before)):
        line = lines_before[i]
        if re.search(r'///.*(测试目的|测试场景|预期结果|## 测试目的|## 测试场景|## 预期结果)', line):
            found_doc = True
            break

    if found_doc:
        documented_count += 1

print(f"{documented_count} {test_count}")
EOF
)

    DOCUMENTED=$(echo "$DOCUMENTED" | awk '{print $1}')
    TEST_COUNT=$(echo "$DOCUMENTED" | awk '{print $2}')

    # 如果Python方法失败，使用简单统计
    if [ -z "$DOCUMENTED" ] || [ -z "$TEST_COUNT" ]; then
        # 简单统计：测试函数数量
        TEST_COUNT=$(grep -c "^    #\[test\]" "$file" 2>/dev/null || echo 0)
        # 统计有文档注释的测试（测试函数前有包含"测试目的"的行）
        DOCUMENTED=$(grep -B 10 "^    #\[test\]" "$file" 2>/dev/null | grep -c "测试目的\|测试场景\|预期结果" || echo 0)
    fi

    TOTAL_DOCUMENTED=$((TOTAL_DOCUMENTED + DOCUMENTED))

    # 计算完成度
    if [ "$TEST_COUNT" -eq 0 ]; then
        PERCENT=0
    else
        PERCENT=$((DOCUMENTED * 100 / TEST_COUNT))
    fi

    # 分类统计
    if [ "$PERCENT" -eq 100 ]; then
        FILES_100=$((FILES_100 + 1))
    elif [ "$PERCENT" -eq 0 ]; then
        FILES_0=$((FILES_0 + 1))
        if [ "$TEST_COUNT" -ge 20 ]; then
            HIGH_PRIORITY_0=$((HIGH_PRIORITY_0 + 1))
            echo "$file - 0/$TEST_COUNT (0%)" >> "$LOW_0_FILE"
        elif [ "$TEST_COUNT" -ge 10 ]; then
            MEDIUM_PRIORITY_0=$((MEDIUM_PRIORITY_0 + 1))
            echo "$file - 0/$TEST_COUNT (0%)" >> "$LOW_0_FILE"
        else
            LOW_PRIORITY_0=$((LOW_PRIORITY_0 + 1))
            echo "$file - 0/$TEST_COUNT (0%)" >> "$LOW_0_FILE"
        fi
    else
        FILES_PARTIAL=$((FILES_PARTIAL + 1))
        MISSING=$((TEST_COUNT - DOCUMENTED))
        if [ "$TEST_COUNT" -ge 20 ]; then
            HIGH_PRIORITY_PARTIAL=$((HIGH_PRIORITY_PARTIAL + 1))
            HIGH_PRIORITY_MISSING=$((HIGH_PRIORITY_MISSING + MISSING))
            echo "$file - $DOCUMENTED/$TEST_COUNT ($PERCENT%)，缺少$MISSING个" >> "$HIGH_PARTIAL_FILE"
        elif [ "$TEST_COUNT" -ge 10 ]; then
            MEDIUM_PRIORITY_PARTIAL=$((MEDIUM_PRIORITY_PARTIAL + 1))
            MEDIUM_PRIORITY_MISSING=$((MEDIUM_PRIORITY_MISSING + MISSING))
            echo "$file - $DOCUMENTED/$TEST_COUNT ($PERCENT%)，缺少$MISSING个" >> "$MEDIUM_PARTIAL_FILE"
        else
            LOW_PRIORITY_PARTIAL=$((LOW_PRIORITY_PARTIAL + 1))
            LOW_PRIORITY_MISSING=$((LOW_PRIORITY_MISSING + MISSING))
            echo "$file - $DOCUMENTED/$TEST_COUNT ($PERCENT%)，缺少$MISSING个" >> "$LOW_PARTIAL_FILE"
        fi
    fi

    echo "$file: $DOCUMENTED/$TEST_COUNT ($PERCENT%)" >> "$TEMP_FILE"
done

# 计算总体完成度
if [ "$TOTAL_TESTS" -gt 0 ]; then
    TOTAL_PERCENT=$(echo "scale=1; $TOTAL_DOCUMENTED * 100 / $TOTAL_TESTS" | bc)
else
    TOTAL_PERCENT=0
fi

echo "**总体统计**:"
echo "- ✅ 100%完成：$FILES_100 个文件"
echo "- 🚧 部分完成：$FILES_PARTIAL 个文件"
echo "- ❌ 0%完成：$FILES_0 个文件"
echo "- **总计**: $TOTAL_FILES 个文件，$TOTAL_TESTS 个测试函数"
echo "- **已添加文档注释**: $TOTAL_DOCUMENTED 个测试函数"
echo "- **缺少文档注释**: $((TOTAL_TESTS - TOTAL_DOCUMENTED)) 个测试函数"
echo "- **覆盖率**: $TOTAL_PERCENT%"
echo ""

echo "**高优先级文件（≥20个测试）**:"
echo "- 0%完成：$HIGH_PRIORITY_0 个文件"
echo "- 部分完成：$HIGH_PRIORITY_PARTIAL 个文件，缺少 $HIGH_PRIORITY_MISSING 个文档注释"
if [ -s "$HIGH_PARTIAL_FILE" ]; then
    echo ""
    echo "部分完成的文件："
    cat "$HIGH_PARTIAL_FILE" | head -20
fi
echo ""

echo "**中优先级文件（10-19个测试）**:"
echo "- 0%完成：$MEDIUM_PRIORITY_0 个文件"
echo "- 部分完成：$MEDIUM_PRIORITY_PARTIAL 个文件，缺少 $MEDIUM_PRIORITY_MISSING 个文档注释"
if [ -s "$MEDIUM_PARTIAL_FILE" ]; then
    echo ""
    echo "部分完成的文件："
    cat "$MEDIUM_PARTIAL_FILE"
fi
echo ""

echo "**低优先级文件（<10个测试）**:"
echo "- 0%完成：$LOW_PRIORITY_0 个文件"
echo "- 部分完成：$LOW_PRIORITY_PARTIAL 个文件，缺少 $LOW_PRIORITY_MISSING 个文档注释"
if [ "$LOW_PRIORITY_0" -le 20 ]; then
    echo ""
    echo "0%完成的文件："
    cat "$LOW_0_FILE" | head -20
fi

# 清理临时文件
rm -f "$TEMP_FILE" "$HIGH_PARTIAL_FILE" "$MEDIUM_PARTIAL_FILE" "$LOW_0_FILE" "$LOW_PARTIAL_FILE"

