#!/bin/bash
# Workflow CLI 卸载脚本
#
# 使用方法:
#   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/zevwings/workflow.rs/master/scripts/uninstall.sh)"

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

# 检测 workflow 命令是否存在
check_workflow_installed() {
    if command -v workflow &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# 使用 workflow uninstall 命令卸载
uninstall_with_command() {
    log_info "Using 'workflow uninstall' command..."

    if workflow uninstall; then
        log_success "Uninstallation completed successfully!"
        return 0
    else
        log_warning "Uninstallation command failed, trying manual removal..."
        return 1
    fi
}

# 手动卸载（当 workflow 命令不可用时）
manual_uninstall() {
    log_info "Performing manual uninstallation..."

    local removed_count=0

    # 检测平台
    local install_dir=""
    if [[ "$OSTYPE" == "darwin"* ]] || [[ "$OSTYPE" == "linux-gnu"* ]]; then
        install_dir="/usr/local/bin"
    else
        error_exit "Unsupported operating system: $OSTYPE"
    fi

    # 删除二进制文件
    local binaries=("workflow" "install")
    for binary in "${binaries[@]}"; do
        local binary_path="$install_dir/$binary"
        if [ -f "$binary_path" ]; then
            log_info "Removing $binary_path..."
            if sudo rm -f "$binary_path" 2>/dev/null; then
                log_success "Removed $binary_path"
                removed_count=$((removed_count + 1))
            else
                log_warning "Failed to remove $binary_path (may require manual removal)"
            fi
        fi
    done

    # 删除配置文件（可选）
    local config_dir=""
    if [[ "$OSTYPE" == "darwin"* ]]; then
        config_dir="$HOME/.workflow"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        config_dir="$HOME/.workflow"
    fi

    if [ -d "$config_dir" ]; then
        log_info "Configuration directory found: $config_dir"
        echo ""
        read -p "Do you want to remove configuration files? (y/N): " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if rm -rf "$config_dir"; then
                log_success "Removed configuration directory: $config_dir"
            else
                log_warning "Failed to remove configuration directory: $config_dir"
            fi
        else
            log_info "Keeping configuration files"
        fi
    fi

    # 删除 completion 脚本
    local completion_dir="$config_dir/completions"
    if [ -d "$completion_dir" ]; then
        log_info "Removing completion scripts..."
        if rm -rf "$completion_dir"; then
            log_success "Removed completion scripts"
        else
            log_warning "Failed to remove completion scripts"
        fi
    fi

    # 从 shell 配置文件中移除 completion 配置
    remove_completion_config() {
        local shell_config=$1
        local config_line=$2

        if [ -f "$shell_config" ]; then
            if grep -q "$config_line" "$shell_config" 2>/dev/null; then
                log_info "Removing completion config from $shell_config..."
                # 使用 sed 删除包含配置的行
                if [[ "$OSTYPE" == "darwin"* ]]; then
                    sed -i '' "/$config_line/d" "$shell_config"
                else
                    sed -i "/$config_line/d" "$shell_config"
                fi
                log_success "Removed completion config from $shell_config"
            fi
        fi
    }

    # 检测 shell 并移除配置
    local shell_name=$(basename "$SHELL")
    case "$shell_name" in
        zsh)
            remove_completion_config "$HOME/.zshrc" "source.*\.workflow.*completions"
            ;;
        bash)
            remove_completion_config "$HOME/.bashrc" "source.*\.workflow.*completions"
            remove_completion_config "$HOME/.bash_profile" "source.*\.workflow.*completions"
            ;;
    esac

    if [ $removed_count -gt 0 ]; then
        log_success "Uninstallation completed! ($removed_count binary file(s) removed)"
    else
        log_warning "No binary files found to remove"
    fi
}

# 主卸载函数
main() {
    echo ""
    log_info "=========================================="
    log_info "Workflow CLI Uninstallation"
    log_info "=========================================="
    echo ""

    # 检查是否已安装
    if ! check_workflow_installed; then
        log_warning "Workflow CLI is not installed or not in PATH"
        log_info "Checking for manual installation..."

        # 尝试手动卸载
        manual_uninstall
        return 0
    fi

    # 显示已安装的版本
    local installed_version=$(workflow --version 2>/dev/null || echo "unknown")
    log_info "Found installed version: $installed_version"
    echo ""

    # 确认卸载
    read -p "Are you sure you want to uninstall Workflow CLI? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Uninstallation cancelled"
        return 0
    fi

    echo ""

    # 尝试使用 workflow uninstall 命令
    if ! uninstall_with_command; then
        # 如果命令失败，尝试手动卸载
        log_info "Falling back to manual uninstallation..."
        manual_uninstall
    fi

    echo ""
    log_success "Uninstallation process completed!"
    log_info "You may need to reload your shell configuration:"
    log_info "  source ~/.zshrc   # for zsh"
    log_info "  source ~/.bashrc  # for bash"
}

# 运行主函数
main "$@"
