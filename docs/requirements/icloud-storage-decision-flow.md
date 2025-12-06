# iCloud 存储决策流程图

## 🔄 存储位置选择决策树

```
                    ┌─────────────────┐
                    │  GetConfigDir() │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ 获取用户主目录    │
                    │ os.UserHomeDir() │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ 检查操作系统类型  │
                    │ runtime.GOOS     │
                    └────────┬────────┘
                             │
                ┌────────────┴────────────┐
                │                         │
                ▼                         ▼
        ┌───────────────┐        ┌───────────────┐
        │ 是 macOS?     │        │ 非 macOS      │
        │ (darwin)      │        │               │
        └───────┬───────┘        └───────┬───────┘
                │                         │
                │                         │
                ▼                         ▼
    ┌───────────────────────┐    ┌──────────────────┐
    │ 检查 iCloud 基础目录   │    │ 使用本地存储      │
    │ ~/Library/Mobile     │    │ ~/.qkflow/       │
    │ Documents/           │    │                  │
    │ com~apple~CloudDocs  │    │ ✅ 返回路径       │
    └───────────┬───────────┘    └──────────────────┘
                │
        ┌───────┴───────┐
        │               │
        ▼               ▼
┌───────────────┐ ┌───────────────┐
│ 目录存在?      │ │ 目录不存在     │
│ 且是目录?      │ │               │
└───────┬───────┘ └───────┬───────┘
        │                 │
        │                 │
        ▼                 ▼
┌───────────────────┐ ┌──────────────────┐
│ 尝试创建 .qkflow  │ │ 使用本地存储      │
│ 目录              │ │ ~/.qkflow/       │
└───────┬───────────┘ └───────┬──────────┘
        │                     │
        │                     │
        ▼                     │
┌───────────────────┐         │
│ 创建成功?          │         │
└───────┬───────────┘         │
        │                     │
    ┌───┴───┐                 │
    │       │                 │
    ▼       ▼                 ▼
┌──────┐ ┌──────┐      ┌──────────────┐
│ 成功 │ │ 失败 │      │ 使用本地存储  │
└──┬───┘ └──┬───┘      │ ~/.qkflow/   │
   │        │          └──────┬────────┘
   │        │                 │
   │        └────────┬────────┘
   │                 │
   ▼                 ▼
┌──────────────────────────┐
│ 使用 iCloud Drive        │
│ ~/Library/Mobile         │
│ Documents/.../.qkflow/   │
│                          │
│ ✅ 返回 iCloud 路径      │
└──────────────────────────┘
```

## 📊 状态转换图

```
┌─────────────┐
│  应用启动    │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│ 调用 GetConfigDir()  │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐      ┌──────────────────┐
│ 检查: macOS?        │ NO   │ 状态: 本地存储    │
└──────┬──────────────┘──────▶│ 路径: ~/.qkflow/ │
       │ YES                  └──────────────────┘
       ▼
┌─────────────────────┐      ┌──────────────────┐
│ 检查: iCloud 可用?   │ NO   │ 状态: 本地存储    │
└──────┬──────────────┘──────▶│ 路径: ~/.qkflow/ │
       │ YES                  └──────────────────┘
       ▼
┌─────────────────────┐      ┌──────────────────┐
│ 创建 .qkflow 目录   │ FAIL │ 状态: 本地存储    │
└──────┬──────────────┘──────▶│ 路径: ~/.qkflow/ │
       │ SUCCESS              └──────────────────┘
       ▼
┌──────────────────────────────────────────┐
│ 状态: iCloud Drive                       │
│ 路径: ~/Library/Mobile Documents/.../    │
│      com~apple~CloudDocs/.qkflow/        │
│                                          │
│ ✅ 配置自动同步到所有设备                 │
└──────────────────────────────────────────┘
```

## 🎯 实际执行示例

### 场景 A：macOS + iCloud 已启用

```
步骤 1: GetConfigDir() 被调用
  ↓
步骤 2: 检查 runtime.GOOS == "darwin"
  → 结果: true ✅
  ↓
步骤 3: 检查 ~/Library/Mobile Documents/com~apple~CloudDocs
  → os.Stat() 成功，目录存在 ✅
  ↓
步骤 4: 创建 ~/Library/Mobile Documents/com~apple~CloudDocs/.qkflow
  → os.MkdirAll() 成功 ✅
  ↓
步骤 5: 返回 iCloud 路径
  → ~/Library/Mobile Documents/com~apple~CloudDocs/.qkflow ✅
```

### 场景 B：macOS + iCloud 未启用

```
步骤 1: GetConfigDir() 被调用
  ↓
步骤 2: 检查 runtime.GOOS == "darwin"
  → 结果: true ✅
  ↓
步骤 3: 检查 ~/Library/Mobile Documents/com~apple~CloudDocs
  → os.Stat() 失败，目录不存在 ❌
  ↓
步骤 4: 跳过 iCloud，直接使用本地存储
  → 创建 ~/.qkflow ✅
  ↓
步骤 5: 返回本地路径
  → ~/.qkflow ✅
```

### 场景 C：Linux/Windows

```
步骤 1: GetConfigDir() 被调用
  ↓
步骤 2: 检查 runtime.GOOS == "darwin"
  → 结果: false ❌
  ↓
步骤 3: 跳过 iCloud 检查，直接使用本地存储
  → 创建 ~/.qkflow ✅
  ↓
步骤 4: 返回本地路径
  → ~/.qkflow ✅
```

## 🔍 代码执行路径追踪

### 示例：保存配置文件

```go
// 1. 用户运行: qkflow init
// 2. 调用链:
config.Save(cfg)
  ↓
utils.GetQuickWorkflowConfigDir()
  ↓
utils.GetConfigDir()  // 核心决策函数
  ↓
[决策逻辑]
  ├─ macOS? → 检查 iCloud
  │   ├─ iCloud 可用? → 使用 iCloud 路径
  │   └─ iCloud 不可用? → 使用本地路径
  └─ 非 macOS? → 使用本地路径
  ↓
返回路径字符串
  ↓
viper.WriteConfigAs(configFile)  // 写入文件
```

### 示例：读取 Jira 状态缓存

```go
// 1. 用户运行: qkflow pr create PROJ-123
// 2. 调用链:
jira.NewStatusCache()
  ↓
utils.GetConfigDir()  // 核心决策函数
  ↓
[决策逻辑 - 同上]
  ↓
返回路径字符串
  ↓
filepath.Join(configDir, "jira-status.json")
  ↓
os.ReadFile(filePath)  // 读取文件
```

## 📈 性能考虑

### 缓存机制

`GetConfigDir()` 每次调用都会执行检查，但：

1. **文件系统检查很快**：`os.Stat()` 是本地操作，通常 < 1ms
2. **目录创建是幂等的**：`os.MkdirAll()` 如果目录已存在会直接返回成功
3. **路径计算简单**：`filepath.Join()` 只是字符串拼接

### 优化建议

如果需要进一步优化，可以考虑：

```go
var configDirCache struct {
    sync.Once
    path string
    err  error
}

func GetConfigDir() (string, error) {
    configDirCache.Do(func() {
        // 执行实际的路径获取逻辑
        configDirCache.path, configDirCache.err = getConfigDirImpl()
    })
    return configDirCache.path, configDirCache.err
}
```

**注意**：当前实现已经足够高效，因为：
- 配置文件操作不频繁
- 文件系统检查开销很小
- 简单的实现更容易维护

## 🧪 测试场景

### 单元测试应该覆盖的场景

1. ✅ macOS + iCloud 可用 → 返回 iCloud 路径
2. ✅ macOS + iCloud 不可用 → 返回本地路径
3. ✅ macOS + iCloud 目录创建失败 → 返回本地路径
4. ✅ Linux → 返回本地路径
5. ✅ Windows → 返回本地路径
6. ✅ 无法获取用户主目录 → 返回错误

### 集成测试场景

1. ✅ 在真实 macOS 环境中测试 iCloud 路径
2. ✅ 测试配置文件的读写
3. ✅ 测试多文件（config.yaml, jira-status.json 等）的存储
4. ✅ 测试从 iCloud 迁移到本地（或反之）

---

**最后更新**：2025-01-XX