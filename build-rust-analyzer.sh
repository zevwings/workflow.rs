#!/bin/bash
# 从源代码构建并安装 rust-analyzer 的脚本
# 使用方法: ./build-rust-analyzer.sh

set -e

echo "正在克隆 rust-analyzer 仓库..."
cd /tmp

if [ -d "rust-analyzer" ]; then
    echo "检测到已存在的 rust-analyzer 目录，正在更新..."
    cd rust-analyzer
    git pull
else
    git clone --depth 1 https://github.com/rust-lang/rust-analyzer.git
    cd rust-analyzer
fi

echo "正在构建并安装 rust-analyzer..."
cargo xtask install --server

echo "✅ rust-analyzer 安装完成！"
echo "安装位置: $(which rust-analyzer)"
rust-analyzer --version

