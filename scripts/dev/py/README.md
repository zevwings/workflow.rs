# Python Dev 工具

Python 版本的开发工具集合，零依赖，快速启动。

## 快速开始

```bash
# 查看所有可用命令
python3 scripts/dev/py/dev.py --help

# 示例：检查 CI 是否应该跳过
python3 scripts/dev/py/dev.py ci check-skip --branch "test" --ci
```

## 前置要求

- **Python 3.13+** （必需）
- Git（用于 Git 操作）
- GitHub Token（用于 PR 操作，可选）

## 命令

- `ci check-skip` - CI 跳过检查
- `ci verify` - CI 检查验证
- `version generate` - 版本号生成
- `tag create` - 创建 Git 标签
- `tag cleanup` - 清理 Alpha 标签
- `pr create` - 创建 PR
- `pr merge` - 合并 PR
- `homebrew update` - 更新 Homebrew Formula
- `checksum calculate` - 文件哈希计算
- `tests check` - 检查测试覆盖率
- `tests metrics` - 收集测试指标
- `tests report` - 生成测试报告
- `tests trends` - 分析测试趋势
- `performance analyze` - 分析性能数据
- `docs check integrity` - 检查文档完整性
- `docs check links` - 检查文档链接
- `docs report generate` - 生成文档报告
