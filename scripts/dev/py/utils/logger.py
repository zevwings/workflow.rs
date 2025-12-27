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

def _safe_print(text: str, file=sys.stdout):
    """安全打印，处理编码错误"""
    try:
        print(text, file=file, flush=True)
    except UnicodeEncodeError:
        # 如果编码失败，尝试重新配置编码
        try:
            if hasattr(file, 'reconfigure'):
                file.reconfigure(encoding='utf-8', errors='replace')
                print(text, file=file, flush=True)
            else:
                # 回退：使用 ASCII 安全版本
                safe_text = text.encode('ascii', errors='replace').decode('ascii')
                print(safe_text, file=file, flush=True)
        except (UnicodeEncodeError, AttributeError):
            # 最后的回退：使用 ASCII 替代字符
            safe_text = text.encode('ascii', errors='replace').decode('ascii')
            print(safe_text, file=file, flush=True)

def log_success(message: str):
    """成功消息（绿色 ✓）"""
    symbol = '✓'
    if _use_color:
        _safe_print(f"{Colors.GREEN}{symbol} {message}{Colors.RESET}")
    else:
        _safe_print(f"{symbol} {message}")

def log_error(message: str):
    """错误消息（红色 ✗）"""
    symbol = '✗'
    if _use_color:
        _safe_print(f"{Colors.RED}{symbol} {message}{Colors.RESET}", file=sys.stderr)
    else:
        _safe_print(f"{symbol} {message}", file=sys.stderr)

def log_warning(message: str):
    """警告消息（黄色 ⚠）"""
    symbol = '⚠'
    if _use_color:
        _safe_print(f"{Colors.YELLOW}{symbol} {message}{Colors.RESET}")
    else:
        _safe_print(f"{symbol} {message}")

def log_info(message: str):
    """信息消息（蓝色 ℹ）"""
    symbol = 'ℹ'
    if _use_color:
        _safe_print(f"{Colors.BLUE}{symbol} {message}{Colors.RESET}")
    else:
        _safe_print(f"{symbol} {message}")

def log_break(char: str = '-', length: int = 80):
    """分隔线"""
    _safe_print(char * length)

