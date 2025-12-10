#!/bin/bash
set -e

TARGET="$1"

if [ -z "$TARGET" ]; then
  echo "❌ Error: Missing target argument"
  echo "Usage: $0 <target>"
  exit 1
fi

# 为 Linux x86_64 设置 xcb 构建环境
if [[ "$TARGET" == "x86_64-unknown-linux-gnu" ]]; then
  echo "🔍 Setting up xcb build environment..."
  ./.github/scripts/setup-xcb-build-env.sh

  # 清理之前的构建缓存
  echo "🧹 Cleaning xcb build artifacts..."
  cargo clean -p xcb 2>/dev/null || true
  rm -rf target/$TARGET/release/build/xcb-* 2>/dev/null || true
  rm -rf target/$TARGET/release/deps/libxcb-* 2>/dev/null || true

  # 构建（启用详细输出以便调试）
  echo "🔨 Building with verbose output for xcb debugging..."
  if cargo build --release --target $TARGET --bin workflow --bin install -vv 2>&1 | tee build.log; then
    BUILD_SUCCESS=true
  else
    BUILD_SUCCESS=false
    echo "❌ Build failed. Checking build log for xcb-related errors..."
    grep -i "xcb\|xproto\|big_requests\|xcbgen\|xc_misc\|render\|shape\|xfixes" build.log || echo "No xcb-related errors found in log"
    grep -i "error\|warning\|failed" build.log | head -20 || true
  fi
elif [[ "$TARGET" == "aarch64-unknown-linux-gnu" ]]; then
  # 设置交叉编译环境变量
  export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
  echo "🔧 Cross-compiling for Linux ARM64"
  echo "ℹ️  Note: clipboard feature is disabled for Linux ARM64"

  if cargo build --release --target $TARGET --bin workflow --bin install; then
    BUILD_SUCCESS=true
  else
    BUILD_SUCCESS=false
  fi
else
  # 其他平台的标准构建
  if cargo build --release --target $TARGET --bin workflow --bin install; then
    BUILD_SUCCESS=true
  else
    BUILD_SUCCESS=false
  fi
fi

if [ "$BUILD_SUCCESS" != "true" ]; then
  echo "❌ Build failed"
  exit 1
fi

echo "✅ Build successful for $TARGET"
