#!/usr/bin/env bash
# 安装 Linux 系统依赖
# 用于 CI/CD 和本地开发环境

set -euo pipefail

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查是否为 Linux 系统
if [[ "$(uname)" != "Linux" ]]; then
    echo -e "${YELLOW}⚠️  Warning: This script is designed for Linux systems${NC}"
    exit 0
fi

echo -e "${GREEN}📦 Installing system dependencies (Linux)...${NC}"

# 更新包列表
sudo apt-get update

# 安装 XCB 开发库（clipboard 依赖）
# xcb crate 需要 xcb-proto 来生成代码模块
# xcb-proto 需要 Python 来运行代码生成脚本
sudo apt-get install -y \
    python3 \
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
    libxkbcommon-x11-dev

# 基本验证
if ! python3 --version; then
    echo -e "${RED}❌ Error: Python3 not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ System dependencies installed successfully${NC}"

