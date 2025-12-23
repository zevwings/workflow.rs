#!/bin/bash
# 文档链接有效性检查脚本

set -e

echo "=========================================="
echo "文档链接有效性检查"
echo "=========================================="
echo ""

# 检查内部链接
echo "📋 检查内部链接..."
INTERNAL_LINKS=0
BROKEN_LINKS=0

# 临时文件存储断链信息
BROKEN_LINKS_FILE=$(mktemp)
trap "rm -f $BROKEN_LINKS_FILE" EXIT

find docs -name "*.md" -type f ! -path "*/templates/*" | while read -r file; do
  # 提取所有内部链接（格式：](path/to/file.md) 或 [text](path/to/file.md#anchor)）
  grep -oP '\]\([^)]+\)' "$file" 2>/dev/null | sed 's/](//;s/)//' | while read -r link; do
    # 跳过空链接
    [ -z "$link" ] && continue

    # 跳过外部链接
    [[ "$link" =~ ^https?:// ]] && continue

    # 跳过锚点链接（只检查文件存在性，不检查锚点）
    if [[ "$link" =~ ^# ]]; then
      continue
    fi

    # 解析链接路径
    if [[ "$link" =~ ^/ ]]; then
      # 绝对路径（从项目根目录开始）
      target_file="${link#/}"
    else
      # 相对路径
      file_dir=$(dirname "$file")
      target_file="$file_dir/$link"
    fi

    # 移除锚点部分（#anchor）
    target_file="${target_file%%#*}"

    # 规范化路径（移除多余的 ./ 和 ..）
    target_file=$(echo "$target_file" | sed 's|^\./||' | sed 's|/\./|/|g')

    # 检查文件是否存在
    if [ ! -f "$target_file" ]; then
      echo "❌ 断链: $file -> $link (目标文件: $target_file)" >> "$BROKEN_LINKS_FILE"
      BROKEN_LINKS=$((BROKEN_LINKS + 1))
    fi

    INTERNAL_LINKS=$((INTERNAL_LINKS + 1))
  done
done

# 显示断链信息
if [ -f "$BROKEN_LINKS_FILE" ] && [ -s "$BROKEN_LINKS_FILE" ]; then
  echo ""
  echo "发现的断链:"
  cat "$BROKEN_LINKS_FILE"
  echo ""
fi

echo ""
echo "检查了 $INTERNAL_LINKS 个内部链接"
if [ $BROKEN_LINKS -gt 0 ]; then
  echo "❌ 发现 $BROKEN_LINKS 个断链"
  exit 1
else
  echo "✅ 所有内部链接有效"
fi

# 检查外部链接（如果 lychee 可用）
if command -v lychee >/dev/null 2>&1; then
  echo ""
  echo "📋 检查外部链接..."
  # 使用 lychee 检查外部链接，但允许失败（非阻塞）
  lychee docs/**/*.md --exclude-all-private --exclude-loopback || {
    echo "⚠️  发现外部链接问题（非阻塞）"
    echo "   请检查上方的输出以了解详情"
  }
else
  echo ""
  echo "ℹ️  lychee 未安装，跳过外部链接检查"
  echo "   安装方法: cargo install lychee"
  echo "   注意: 外部链接检查在 CI/CD 中会自动运行"
fi

echo ""
echo "✅ 链接检查完成"

