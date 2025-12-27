"""文件哈希计算实现"""
import hashlib
import sys
from pathlib import Path

# 添加父目录到路径，以便导入 utils
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_info, log_success, log_error


def calculate(args) -> str:
    """计算文件 SHA256 哈希"""
    file_path = Path(args.file)

    if not file_path.exists():
        log_error(f"File not found: {args.file}")
        sys.exit(1)

    # 读取文件并计算哈希（使用标准库 hashlib）
    sha256_hash = hashlib.sha256()
    try:
        with open(file_path, 'rb') as f:
            # 分块读取，避免大文件占用过多内存
            for chunk in iter(lambda: f.read(4096), b''):
                sha256_hash.update(chunk)
    except IOError as e:
        log_error(f"Failed to read file: {e}")
        sys.exit(1)

    hash_hex = sha256_hash.hexdigest()

    log_info(f"File: {args.file}")
    log_info(f"SHA256: {hash_hex}")

    # 输出到文件或 stdout
    if args.output:
        output_path = Path(args.output)
        try:
            output_path.write_text(hash_hex)
            log_success(f"Hash written to: {args.output}")
        except IOError as e:
            log_error(f"Failed to write hash to {args.output}: {e}")
            sys.exit(1)
    else:
        # 直接输出哈希值（用于管道）
        print(hash_hex)

    return hash_hex


def main():
    """CLI 入口（可以直接运行此脚本）"""
    import argparse

    parser = argparse.ArgumentParser(
        prog='calculate',
        description='Calculate file SHA256 checksum'
    )
    parser.add_argument('--file', required=True, help='File path')
    parser.add_argument('--output', help='Output file path (optional)')

    args = parser.parse_args()
    calculate(args)


if __name__ == '__main__':
    main()

