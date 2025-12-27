# Dev 工具架构设计

## 当前架构

```
scripts/dev/dev.py          # 统一入口（CLI 路由）
├── ci/
│   └── check_skip.py      # 具体实现
└── utils/
    └── logger.py          # 日志工具
```

## 方案对比

### 方案 A: 保留 `dev.py`（当前方案）

**调用方式**:
```bash
python3 scripts/dev/dev.py ci check-skip --branch "xxx" --ci
```

**优点**:
- ✅ 统一的 CLI 接口，符合 Unix 哲学
- ✅ 未来扩展性好（添加新命令只需修改 `dev.py`）
- ✅ 命令结构清晰：`dev <category> <command>`
- ✅ 与 Rust 版本保持一致的命令结构

**缺点**:
- ❌ 多一层抽象，代码稍多

### 方案 B: 删除 `dev.py`，直接调用 `check_skip.py`

**调用方式**:
```bash
python3 scripts/dev/ci/check_skip.py --branch "xxx" --ci
```

**优点**:
- ✅ 更简单直接
- ✅ 减少一层抽象
- ✅ 每个命令独立，易于理解

**缺点**:
- ❌ 失去统一的 CLI 接口
- ❌ 未来添加命令时需要修改所有调用点
- ❌ 命令结构不一致（`ci/check_skip.py` vs `checksum/calculate.py`）

## 建议

### 推荐：保留 `dev.py`，但让 `check_skip.py` 也可以直接运行

**原因**:
1. **统一接口**: 保持 `dev <category> <command>` 的命令结构
2. **未来扩展**: 计划中有很多命令（ci verify, checksum, tests, docs 等）
3. **灵活性**: 让 `check_skip.py` 可以直接运行，提供两种调用方式

### 实现方式

让 `check_skip.py` 添加 CLI 入口，同时保留 `dev.py`：

```python
# scripts/dev/ci/check_skip.py
def check_skip(args):
    # ... 现有实现

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--branch')
    parser.add_argument('--pr-creator')
    parser.add_argument('--expected-user')
    parser.add_argument('--ci', action='store_true')
    args = parser.parse_args()
    check_skip(args)
```

这样既可以通过 `dev.py` 调用，也可以直接运行。

## 最终建议

**保留 `dev.py`**，原因：
1. 目前虽然只有一个命令，但计划中有很多命令
2. 统一的 CLI 接口更专业，用户体验更好
3. 与 Rust 版本保持一致，降低学习成本
4. 代码量很小（~90 行），维护成本低

如果确实想简化，可以考虑：
- 让 `check_skip.py` 也可以直接运行（添加 `if __name__ == '__main__'`）
- 但保留 `dev.py` 作为推荐方式

