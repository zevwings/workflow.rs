# iCloud 存储机制分析

## 📋 概述

本文档详细分析 `qkflow` 如何实现 iCloud Drive 存储，以及如何自动区分存放到 iCloud 和本地存储的机制。

## 🎯 核心设计目标

1. **自动选择存储位置**：在 macOS 上优先使用 iCloud Drive，实现多设备自动同步
2. **优雅降级**：如果 iCloud Drive 不可用，自动回退到本地存储
3. **跨平台兼容**：非 macOS 系统使用本地存储
4. **透明使用**：用户无需手动配置，系统自动处理

## 🏗️ 架构设计

### 核心实现位置

**文件**：`internal/utils/paths.go`

这是整个存储机制的核心，提供了三个关键函数：

1. `GetConfigDir()` - 获取配置目录（自动选择 iCloud 或本地）
2. `IsICLoudAvailable()` - 检查 iCloud Drive 是否可用
3. `GetConfigLocation()` - 返回人类可读的存储位置描述

### 存储路径

#### iCloud Drive 路径（macOS，优先）
```
~/Library/Mobile Documents/com~apple~CloudDocs/.qkflow/
```

#### 本地存储路径（回退方案）
```
~/.qkflow/
```

## 🔍 详细实现分析

### 1. GetConfigDir() - 核心决策函数

```go
func GetConfigDir() (string, error) {
    homeDir, err := os.UserHomeDir()
    if err != nil {
        return "", err
    }

    // On macOS, try to use iCloud Drive first
    if runtime.GOOS == "darwin" {
        iCloudPath := filepath.Join(homeDir, "Library", "Mobile Documents",
                                   "com~apple~CloudDocs", ".qkflow")

        // Check if iCloud Drive is available
        iCloudBase := filepath.Join(homeDir, "Library", "Mobile Documents",
                                   "com~apple~CloudDocs")
        if info, err := os.Stat(iCloudBase); err == nil && info.IsDir() {
            // iCloud Drive is available, create our config dir if needed
            if err := os.MkdirAll(iCloudPath, 0755); err == nil {
                return iCloudPath, nil
            }
        }
    }

    // Fallback to local directory
    localPath := filepath.Join(homeDir, ".qkflow")
    if err := os.MkdirAll(localPath, 0755); err != nil {
        return "", err
    }

    return localPath, nil
}
```

#### 决策流程

```
开始
  ↓
检查操作系统是否为 macOS (darwin)
  ↓ 是
检查 iCloud Drive 基础目录是否存在
  ↓ 存在
尝试创建 .qkflow 目录
  ↓ 成功
返回 iCloud 路径 ✅
  ↓ 失败/不存在/非 macOS
创建本地 .qkflow 目录
  ↓
返回本地路径 ✅
```

#### 关键判断点

1. **操作系统检查**：`runtime.GOOS == "darwin"`
   - 只有 macOS 才尝试使用 iCloud Drive
   - 其他系统直接使用本地存储

2. **iCloud 可用性检查**：`os.Stat(iCloudBase)`
   - 检查 `~/Library/Mobile Documents/com~apple~CloudDocs` 是否存在
   - 如果存在且是目录，说明 iCloud Drive 已启用

3. **目录创建**：`os.MkdirAll(iCloudPath, 0755)`
   - 如果 iCloud 可用，尝试创建 `.qkflow` 子目录
   - 权限设置为 `0755`（用户可读写执行，组和其他可读执行）

4. **回退机制**：如果任何步骤失败，自动使用本地存储

### 2. IsICLoudAvailable() - 可用性检查

```go
func IsICLoudAvailable() bool {
    if runtime.GOOS != "darwin" {
        return false
    }

    homeDir, err := os.UserHomeDir()
    if err != nil {
        return false
    }

    iCloudBase := filepath.Join(homeDir, "Library", "Mobile Documents",
                               "com~apple~CloudDocs")
    info, err := os.Stat(iCloudBase)
    return err == nil && info.IsDir()
}
```

**用途**：
- 用于显示存储位置信息
- 用于 UI 提示用户当前使用的存储方式
- 不直接参与路径选择（`GetConfigDir()` 内部已处理）

### 3. GetConfigLocation() - 用户友好的描述

```go
func GetConfigLocation() string {
    if IsICLoudAvailable() {
        return "iCloud Drive (synced across devices)"
    }
    return "Local storage"
}
```

**用途**：
- 在 `qkflow init` 完成后显示存储位置
- 在 `qkflow config` 命令中显示当前配置位置

## 📁 使用场景

### 配置文件存储

**位置**：`internal/config/config.go`

```go
// Load() 函数
configDir, err := utils.GetQuickWorkflowConfigDir()
configFile := filepath.Join(configDir, "config.yaml")

// Save() 函数
configDir, err := utils.GetQuickWorkflowConfigDir()
configFile := filepath.Join(configDir, "config.yaml")
```

**存储的文件**：
- `config.yaml` - 主配置文件（包含 GitHub、Jira、LLM 等配置）

### Jira 状态缓存

**位置**：`internal/jira/status_cache.go`

```go
func NewStatusCache() (*StatusCache, error) {
    configDir, err := utils.GetConfigDir()
    if err != nil {
        return nil, fmt.Errorf("failed to get config directory: %w", err)
    }
    filePath := filepath.Join(configDir, "jira-status.json")
    // ...
}
```

**存储的文件**：
- `jira-status.json` - Jira 项目状态映射缓存

### Watch 状态存储

**位置**：`internal/watcher/state.go`

```go
func NewState() (*State, error) {
    configDir, err := utils.GetConfigDir()
    if err != nil {
        return nil, fmt.Errorf("failed to get config directory: %w", err)
    }
    filePath := filepath.Join(configDir, "watch-state.json")
    // ...
}
```

**存储的文件**：
- `watch-state.json` - Watch 守护进程的状态信息

### 日志文件存储

**位置**：`internal/logger/factory.go`

```go
if logFilePath == "" {
    configDir, err := utils.GetConfigDir()
    if err == nil {
        logFilePath = filepath.Join(configDir, "qkflow.log")
    }
}
```

**存储的文件**：
- `qkflow.log` - 应用日志文件（如果配置了文件日志）

### Watch 列表存储

**位置**：`internal/watcher/watching_list.go`

```go
func NewWatchingList() (*WatchingList, error) {
    configDir, err := utils.GetConfigDir()
    if err != nil {
        return nil, fmt.Errorf("failed to get config directory: %w", err)
    }
    filePath := filepath.Join(configDir, "watching-list.json")
    // ...
}
```

**存储的文件**：
- `watching-list.json` - Watch 监控的仓库列表

## 🔄 存储位置选择逻辑

### 场景 1：macOS + iCloud Drive 已启用

```
条件检查：
  ✓ runtime.GOOS == "darwin"
  ✓ ~/Library/Mobile Documents/com~apple~CloudDocs 存在
  ✓ 成功创建 ~/Library/Mobile Documents/com~apple~CloudDocs/.qkflow

结果：
  → 使用 iCloud Drive
  → 路径：~/Library/Mobile Documents/com~apple~CloudDocs/.qkflow/
  → 配置会自动同步到所有登录同一 Apple ID 的设备
```

### 场景 2：macOS + iCloud Drive 未启用

```
条件检查：
  ✓ runtime.GOOS == "darwin"
  ✗ ~/Library/Mobile Documents/com~apple~CloudDocs 不存在

结果：
  → 使用本地存储
  → 路径：~/.qkflow/
  → 配置仅存储在本地，不会同步
```

### 场景 3：macOS + iCloud Drive 目录创建失败

```
条件检查：
  ✓ runtime.GOOS == "darwin"
  ✓ ~/Library/Mobile Documents/com~apple~CloudDocs 存在
  ✗ 创建 .qkflow 目录失败（权限问题等）

结果：
  → 回退到本地存储
  → 路径：~/.qkflow/
  → 配置仅存储在本地
```

### 场景 4：非 macOS 系统

```
条件检查：
  ✗ runtime.GOOS != "darwin"

结果：
  → 直接使用本地存储
  → 路径：~/.qkflow/
  → 配置仅存储在本地
```

## 🎨 用户体验

### 初始化时显示存储位置

在 `cmd/qkflow/commands/init.go` 中：

```go
func showStorageLocation() {
    location := utils.GetConfigLocation()
    configDir, _ := utils.GetQuickWorkflowConfigDir()
    log.Info("Storage location: %s", location)
    if configDir != "" {
        log.Info("  Config: %s/config.yaml", configDir)
    }
    log.Info("")
}
```

**输出示例**：

```
Storage location: iCloud Drive (synced across devices)
  Config: /Users/username/Library/Mobile Documents/com~apple~CloudDocs/.qkflow/config.yaml
```

或

```
Storage location: Local storage
  Config: /Users/username/.qkflow/config.yaml
```

## 🔒 安全性考虑

### 文件权限

- **目录权限**：`0755` - 用户可读写执行，组和其他可读执行
- **配置文件权限**：`0600` - 仅用户可读写（由 viper 或文件写入时设置）

### iCloud 安全特性

1. **端到端加密**：iCloud Drive 文件在传输和存储时都经过加密
2. **访问控制**：只有登录同一 Apple ID 的设备才能访问
3. **本地缓存**：文件在本地也有缓存，即使离线也能访问

## 🐛 故障排除

### 问题 1：iCloud Drive 不可用

**症状**：配置存储在本地而不是 iCloud

**检查步骤**：
```bash
# 1. 检查 iCloud Drive 是否启用
ls -la ~/Library/Mobile\ Documents/com~apple~CloudDocs/

# 2. 如果目录不存在，启用 iCloud Drive
# 系统设置 → Apple ID → iCloud → iCloud Drive

# 3. 检查当前存储位置
qkflow config
```

### 问题 2：配置不同步

**症状**：在一台设备上修改配置，另一台设备看不到

**可能原因**：
1. iCloud Drive 同步延迟（通常几秒到几分钟）
2. 网络连接问题
3. 另一台设备未登录同一 Apple ID

**解决方案**：
```bash
# 1. 检查 iCloud 同步状态
# Finder → iCloud Drive → 检查文件是否有云图标

# 2. 强制同步
# 右键点击文件 → "从 iCloud 下载"

# 3. 检查网络
ping icloud.com
```

### 问题 3：权限错误

**症状**：无法创建或写入配置文件

**检查步骤**：
```bash
# 1. 检查目录权限
ls -la ~/Library/Mobile\ Documents/com~apple~CloudDocs/

# 2. 检查 .qkflow 目录权限
ls -la ~/Library/Mobile\ Documents/com~apple~CloudDocs/.qkflow/

# 3. 手动创建目录（如果需要）
mkdir -p ~/Library/Mobile\ Documents/com~apple~CloudDocs/.qkflow
chmod 755 ~/Library/Mobile\ Documents/com~apple~CloudDocs/.qkflow
```

## 📊 代码调用关系图

```
应用层
  │
  ├─ cmd/qkflow/commands/init.go
  │   └─ utils.GetQuickWorkflowConfigDir()
  │
  ├─ internal/config/config.go
  │   ├─ Load() → utils.GetQuickWorkflowConfigDir()
  │   └─ Save() → utils.GetQuickWorkflowConfigDir()
  │
  ├─ internal/jira/status_cache.go
  │   └─ NewStatusCache() → utils.GetConfigDir()
  │
  ├─ internal/watcher/state.go
  │   └─ NewState() → utils.GetConfigDir()
  │
  └─ internal/watcher/watching_list.go
      └─ NewWatchingList() → utils.GetConfigDir()
          │
          └─ 核心实现层
              └─ internal/utils/paths.go
                  ├─ GetConfigDir() [核心决策逻辑]
                  ├─ GetQuickWorkflowConfigDir() [包装函数]
                  ├─ IsICLoudAvailable() [可用性检查]
                  └─ GetConfigLocation() [用户友好描述]
```

## 🔄 迁移场景

### 从本地存储迁移到 iCloud

如果用户之前使用本地存储，现在想迁移到 iCloud：

```bash
# 1. 确保 iCloud Drive 已启用
# 系统设置 → Apple ID → iCloud → iCloud Drive

# 2. 迁移配置文件
if [ -f ~/.qkflow/config.yaml ]; then
  cp ~/.qkflow/config.yaml \
     ~/Library/Mobile\ Documents/com~apple~CloudDocs/.qkflow/config.yaml
fi

if [ -f ~/.qkflow/jira-status.json ]; then
  cp ~/.qkflow/jira-status.json \
     ~/Library/Mobile\ Documents/com~apple~CloudDocs/.qkflow/jira-status.json
fi

# 3. 验证迁移
qkflow config
```

### 从 iCloud 迁移到本地存储

如果用户想禁用 iCloud 同步：

```bash
# 1. 禁用 iCloud Drive（系统设置）
# 或手动移动配置回本地

# 2. 移动配置文件
cp ~/Library/Mobile\ Documents/com~apple~CloudDocs/.qkflow/config.yaml \
   ~/.qkflow/config.yaml

# 3. 验证迁移
qkflow config
```

## 📝 总结

### 核心设计原则

1. **自动选择**：系统自动选择最佳存储位置，用户无需配置
2. **优雅降级**：iCloud 不可用时自动回退到本地存储
3. **透明使用**：所有使用 `GetConfigDir()` 的代码都自动获得正确的路径
4. **跨平台兼容**：非 macOS 系统使用本地存储

### 关键优势

1. **多设备同步**：macOS 用户配置自动同步到所有设备
2. **零配置**：用户无需手动选择存储位置
3. **可靠性**：有完善的回退机制
4. **安全性**：利用 iCloud 的加密和访问控制

### 实现要点

1. **单一入口**：所有配置相关文件都通过 `GetConfigDir()` 获取路径
2. **统一逻辑**：存储位置选择逻辑集中在一个函数中
3. **易于维护**：修改存储逻辑只需修改 `paths.go` 文件

---

**最后更新**：2025-01-XX