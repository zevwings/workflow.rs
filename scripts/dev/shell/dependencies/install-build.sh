#!/usr/bin/env bash
# 安装 Linux 构建依赖
# 包含基本依赖 + 构建工具和验证

set -euo pipefail

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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

echo -e "${GREEN}📦 Installing build dependencies (Linux)...${NC}"

# 先安装基本依赖
echo -e "${GREEN}Step 1: Installing basic dependencies...${NC}"
bash "$SCRIPT_DIR/install-basic.sh"

# 更新包列表
sudo apt-get update

# 安装构建工具
echo -e "${GREEN}Step 2: Installing build tools...${NC}"
sudo apt-get install -y \
    python3-pip \
    python3-xcbgen \
    pkg-config

# 验证构建依赖
echo -e "${GREEN}🔍 Verifying build dependencies...${NC}"

# 验证 xcbgen 模块
if ! python3 -c "import xcbgen" 2>/dev/null; then
    echo -e "${RED}❌ Error: xcbgen module not available${NC}"
    exit 1
fi

# 验证 pkg-config
if ! pkg-config --exists xcb; then
    echo -e "${RED}❌ Error: xcb pkg-config not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build dependencies installed and verified successfully${NC}"

