"""简单的日志工具，使用标准库实现，零依赖"""
import sys
import os
from typing import Optional

# 检测是否支持颜色输出
def _supports_color() -> bool:
    """检测终端是否支持颜色"""
    # GitHub Actions 支持颜色
    if os.environ.get('GITHUB_ACTIONS') == 'true':
        return True
    # 检查 TERM 环境变量
    term = os.environ.get('TERM', '')
    return term and term != 'dumb'

# ANSI 颜色码
class Colors:
    GREEN = '\033[32m'
    RED = '\033[31m'
    YELLOW = '\033[33m'
    BLUE = '\033[34m'
    RESET = '\033[0m'

_use_color = _supports_color()

def log_success(message: str):
    """成功消息（绿色 ✓）"""
    if _use_color:
        print(f"{Colors.GREEN}✓ {message}{Colors.RESET}")
    else:
        print(f"✓ {message}")

def log_error(message: str):
    """错误消息（红色 ✗）"""
    if _use_color:
        print(f"{Colors.RED}✗ {message}{Colors.RESET}", file=sys.stderr)
    else:
        print(f"✗ {message}", file=sys.stderr)

def log_warning(message: str):
    """警告消息（黄色 ⚠）"""
    if _use_color:
        print(f"{Colors.YELLOW}⚠ {message}{Colors.RESET}")
    else:
        print(f"⚠ {message}")

def log_info(message: str):
    """信息消息（蓝色 ℹ）"""
    if _use_color:
        print(f"{Colors.BLUE}ℹ {message}{Colors.RESET}")
    else:
        print(f"ℹ {message}")

def log_break(char: str = '-', length: int = 80):
    """分隔线"""
    print(char * length)

