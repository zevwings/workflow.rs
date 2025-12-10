#!/bin/bash
set -e

echo "🔍 Setting up Linux x86_64 XCB dependencies..."

# 安装 XCB 开发库
sudo apt-get update
sudo apt-get install -y \
  python3 \
  python3-pip \
  python3-xcbgen \
  libxcb1-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  xcb-proto \
  libxcb-keysyms1-dev \
  libxcb-image0-dev \
  libxcb-util-dev \
  libxcb-icccm4-dev \
  libxcb-sync-dev \
  libxcb-xinerama0-dev \
  libxcb-randr0-dev \
  libxcb-xinput-dev \
  libxcb-dri3-dev \
  libxcb-present-dev \
  libxcb-xv0-dev \
  libxcb-glx0-dev \
  libxcb-shm0-dev \
  libxcb-composite0-dev \
  libxcb-damage0-dev \
  libxcb-record0-dev \
  libxcb-screensaver0-dev \
  libxcb-res0-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  pkg-config

# 验证 xcb-proto 包
echo "🔍 Verifying xcb-proto package installation..."
if dpkg -l | grep -q "^ii.*xcb-proto"; then
  echo "✅ xcb-proto package is installed"
  if [ -d "/usr/share/xcb" ]; then
    echo "✅ xcb protocol files directory found: /usr/share/xcb"
    PROTO_COUNT=$(find /usr/share/xcb -name "*.xml" 2>/dev/null | wc -l)
    if [ "$PROTO_COUNT" -gt 0 ]; then
      echo "✅ Found $PROTO_COUNT protocol XML files"
    fi
  fi
else
  echo "❌ Error: xcb-proto package is not installed"
  exit 1
fi

# 验证 xcbgen Python 模块
echo "🔍 Checking for xcbgen Python module..."
if python3 -c "import xcbgen" 2>/dev/null; then
  echo "✅ xcbgen Python module is importable"
else
  echo "❌ Error: xcbgen Python module is not importable"
  XCBGEN_PATH=$(find /usr -name "xcbgen" -type d 2>/dev/null | head -1)
  if [ -n "$XCBGEN_PATH" ]; then
    export PYTHONPATH="$(dirname $XCBGEN_PATH):${PYTHONPATH:-}"
    if python3 -c "import xcbgen" 2>/dev/null; then
      echo "✅ xcbgen module is now importable after setting PYTHONPATH"
    else
      echo "❌ xcbgen still not importable"
      exit 1
    fi
  else
    exit 1
  fi
fi

# 验证 pkg-config
pkg-config --exists xcb && echo "✅ xcb pkg-config found" || echo "⚠️  xcb pkg-config not found"

echo "✅ Linux x86_64 XCB dependencies setup complete"
