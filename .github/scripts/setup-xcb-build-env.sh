#!/bin/bash
set -e

echo "🔍 Setting up xcb build environment..."

# 设置环境变量
export XCB_PROTO_DIR="${XCB_PROTO_DIR:-/usr/share/xcb}"
export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/pkgconfig:${PKG_CONFIG_PATH:-}"

# 确保 PYTHONPATH 包含 xcbgen 模块路径
if [ -z "$PYTHONPATH" ] || ! python3 -c "import xcbgen" 2>/dev/null; then
  XCBGEN_DIR=$(python3 -c "import xcbgen; import os; print(os.path.dirname(xcbgen.__file__))" 2>/dev/null || find /usr -name "xcbgen" -type d 2>/dev/null | head -1)
  if [ -n "$XCBGEN_DIR" ]; then
    export PYTHONPATH="$(dirname $XCBGEN_DIR):${PYTHONPATH:-}"
  fi
fi

# 验证环境
if dpkg -l | grep -q "^ii.*xcb-proto"; then
  echo "✅ xcb-proto package is installed"
  if [ -n "$XCB_PROTO_DIR" ] && [ -d "$XCB_PROTO_DIR" ]; then
    echo "✅ xcb protocol directory: $XCB_PROTO_DIR"
  fi
else
  echo "❌ Error: xcb-proto package is not installed"
  exit 1
fi

if python3 -c "import xcbgen" 2>/dev/null; then
  echo "✅ xcbgen Python module is importable"
else
  echo "❌ Error: xcbgen module not importable"
  exit 1
fi

echo "📋 Environment variables:"
echo "   XCB_PROTO_DIR: ${XCB_PROTO_DIR}"
echo "   PKG_CONFIG_PATH: ${PKG_CONFIG_PATH}"
echo "   PYTHONPATH: ${PYTHONPATH:-not set}"

echo "✅ xcb build environment setup complete"
