#!/bin/bash
# Workflow CLI 安装脚本
#
# 使用方法:
#   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.sh)"
#
# 指定版本:
#   VERSION=v1.4.8 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/install.sh)"

set -euo pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

# 错误处理
error_exit() {
    log_error "$1"
    exit 1
}

# 清理函数
cleanup() {
    if [ -n "${TMPDIR:-}" ] && [ -d "$TMPDIR" ]; then
        log_info "Cleaning up temporary files..."
        rm -rf "$TMPDIR"
    fi
}

# 设置清理 trap
trap cleanup EXIT

# 检测操作系统和架构
detect_platform() {
    log_info "Detecting platform..."

    local os=""
    local arch=""
    local platform=""

    # 检测操作系统
    if [[ "$OSTYPE" == "darwin"* ]]; then
        os="macOS"
        # 检测 macOS 架构
        if [[ $(uname -m) == "arm64" ]]; then
            arch="ARM64"
            platform="macOS-AppleSilicon"
        else
            arch="x86_64"
            platform="macOS-Intel"
        fi
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        os="Linux"
        # 检测 Linux 架构
        local machine=$(uname -m)
        if [[ "$machine" == "aarch64" || "$machine" == "arm64" ]]; then
            arch="ARM64"
            platform="Linux-ARM64"
        elif [[ "$machine" == "x86_64" ]]; then
            arch="x86_64"
            platform="Linux-x86_64"
        else
            error_exit "Unsupported architecture: $machine"
        fi
    else
        error_exit "Unsupported operating system: $OSTYPE"
    fi

    echo "$platform"
}

# 检查必要的工具
check_requirements() {
    log_info "Checking requirements..."

    local missing_tools=()

    if ! command -v curl &> /dev/null; then
        missing_tools+=("curl")
    fi

    if ! command -v tar &> /dev/null; then
        missing_tools+=("tar")
    fi

    if [ ${#missing_tools[@]} -gt 0 ]; then
        error_exit "Missing required tools: ${missing_tools[*]}. Please install them first."
    fi

    log_success "All requirements met"
}

# 获取最新版本
get_latest_version() {
    log_info "Fetching latest version..."

    local api_url="https://api.github.com/repos/zevwings/workflow.rs/releases/latest"
    local version=$(curl -fsSL "$api_url" | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4 || echo "")

    if [ -z "$version" ]; then
        error_exit "Failed to fetch latest version from GitHub"
    fi

    # 移除 'v' 前缀（如果有）
    version=${version#v}

    echo "$version"
}

# 获取版本号
get_version() {
    if [ -n "${VERSION:-}" ]; then
        # 移除 'v' 前缀（如果有）
        echo "${VERSION#v}"
    else
        get_latest_version
    fi
}

# 下载文件
download_file() {
    local url=$1
    local output=$2
    local max_retries=3
    local retry_count=0

    while [ $retry_count -lt $max_retries ]; do
        if curl -fsSL -o "$output" "$url"; then
            return 0
        fi

        retry_count=$((retry_count + 1))
        if [ $retry_count -lt $max_retries ]; then
            log_warning "Download failed, retrying... ($retry_count/$max_retries)"
            sleep 2
        fi
    done

    return 1
}

# 验证 SHA256
verify_sha256() {
    local file=$1
    local expected_hash=$2

    log_info "Verifying SHA256 checksum..."

    local calculated_hash=""
    if [[ "$OSTYPE" == "darwin"* ]]; then
        calculated_hash=$(shasum -a 256 "$file" | cut -d' ' -f1)
    else
        calculated_hash=$(sha256sum "$file" | cut -d' ' -f1)
    fi

    if [ "$calculated_hash" != "$expected_hash" ]; then
        log_error "SHA256 verification failed!"
        log_error "Expected: $expected_hash"
        log_error "Got:      $calculated_hash"
        return 1
    fi

    log_success "SHA256 verification passed"
    return 0
}

# 主安装函数
main() {
    echo ""
    log_info "=========================================="
    log_info "Workflow CLI Installation"
    log_info "=========================================="
    echo ""

    # 检测平台
    local platform=$(detect_platform)
    log_success "Detected platform: $platform"

    # 检查必要工具
    check_requirements

    # 获取版本
    local version=$(get_version)
    local tag="v$version"
    log_success "Installing version: $version"

    # 创建临时目录
    TMPDIR=$(mktemp -d)
    log_info "Using temporary directory: $TMPDIR"

    # 构建下载 URL
    local archive_name="workflow-${version}-${platform}.tar.gz"
    local download_url="https://github.com/zevwings/workflow.rs/releases/download/${tag}/${archive_name}"
    local sha256_url="${download_url}.sha256"

    log_info "Downloading from: $download_url"

    # 下载二进制包
    local archive_path="$TMPDIR/$archive_name"
    if ! download_file "$download_url" "$archive_path"; then
        error_exit "Failed to download binary package"
    fi
    log_success "Downloaded binary package"

    # 下载 SHA256 文件
    local sha256_path="$TMPDIR/${archive_name}.sha256"
    if ! download_file "$sha256_url" "$sha256_path"; then
        log_warning "Failed to download SHA256 file, skipping verification"
    else
        # 读取期望的 SHA256
        local expected_hash=$(cat "$sha256_path" | head -n 1 | tr -d '[:space:]')

        if [ -n "$expected_hash" ]; then
            # 验证 SHA256
            if ! verify_sha256 "$archive_path" "$expected_hash"; then
                error_exit "SHA256 verification failed. The downloaded file may be corrupted."
            fi
        else
            log_warning "Could not read SHA256 from file, skipping verification"
        fi
    fi

    # 解压
    log_info "Extracting archive..."
    local extract_dir="$TMPDIR/extract"
    mkdir -p "$extract_dir"

    if ! tar -xzf "$archive_path" -C "$extract_dir"; then
        error_exit "Failed to extract archive"
    fi
    log_success "Extracted archive"

    # 检查 install 二进制是否存在
    local install_binary="$extract_dir/install"
    if [ ! -f "$install_binary" ]; then
        error_exit "install binary not found in archive"
    fi

    # 设置执行权限
    chmod +x "$install_binary"

    # 运行安装
    log_info "Running installation..."
    echo ""

    if "$install_binary"; then
        echo ""
        log_success "Installation completed successfully!"
        log_info "You can now use 'workflow' command."
        log_info "Run 'workflow --help' to get started."
    else
        error_exit "Installation failed"
    fi
}

# 运行主函数
main "$@"
