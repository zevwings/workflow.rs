# iCloud 存储使用示例

## 📚 概述

本文档提供实际代码示例，展示如何在项目中使用 iCloud 存储机制。

## 🎯 核心 API

### 主要函数

```go
// 获取配置目录（自动选择 iCloud 或本地）
configDir, err := utils.GetConfigDir()

// 获取配置目录（别名，功能相同）
configDir, err := utils.GetQuickWorkflowConfigDir()

// 检查 iCloud 是否可用
isAvailable := utils.IsICLoudAvailable()

// 获取人类可读的存储位置描述
location := utils.GetConfigLocation()
```

## 💡 使用示例

### 示例 1：保存配置文件

**场景**：在 `qkflow init` 命令中保存用户配置

**代码位置**：`internal/config/config.go`

```go
func Save(cfg *Config) error {
    // 获取配置目录（自动选择 iCloud 或本地）
    configDir, err := utils.GetQuickWorkflowConfigDir()
    if err != nil {
        return fmt.Errorf("failed to get config directory: %w", err)
    }

    // 构建配置文件路径
    configFile := filepath.Join(configDir, "config.yaml")

    // 设置配置值
    viper.Set("github_token", cfg.GitHubToken)
    viper.Set("github_owner", cfg.GitHubOwner)
    // ... 其他配置项

    // 写入文件（自动保存到 iCloud 或本地）
    if err := viper.WriteConfigAs(configFile); err != nil {
        return fmt.Errorf("failed to write config file: %w", err)
    }

    return nil
}
```

**要点**：
- ✅ 使用 `GetQuickWorkflowConfigDir()` 获取路径
- ✅ 不需要关心是 iCloud 还是本地，系统自动处理
- ✅ 文件会自动保存到正确的位置

### 示例 2：读取 Jira 状态缓存

**场景**：读取或写入 Jira 项目状态映射

**代码位置**：`internal/jira/status_cache.go`

```go
func NewStatusCache() (*StatusCache, error) {
    // 获取配置目录（自动选择 iCloud 或本地）
    configDir, err := utils.GetConfigDir()
    if err != nil {
        return nil, fmt.Errorf("failed to get config directory: %w", err)
    }

    // 构建缓存文件路径
    filePath := filepath.Join(configDir, "jira-status.json")

    // 如果文件不存在，创建空文件
    if _, err := os.Stat(filePath); os.IsNotExist(err) {
        emptyData := CacheData{
            Mappings: make(map[string]StatusMapping),
        }
        data, _ := json.MarshalIndent(emptyData, "", "  ")
        if err := os.WriteFile(filePath, data, 0644); err != nil {
            return nil, fmt.Errorf("failed to create status cache file: %w", err)
        }
    }

    return &StatusCache{
        filePath: filePath,
    }, nil
}

// 读取缓存
func (sc *StatusCache) readCache() (*CacheData, error) {
    data, err := os.ReadFile(sc.filePath)
    if err != nil {
        return nil, fmt.Errorf("failed to read cache file: %w", err)
    }
    // ... 解析 JSON
}

// 写入缓存
func (sc *StatusCache) writeCache(cache *CacheData) error {
    data, err := json.MarshalIndent(cache, "", "  ")
    if err != nil {
        return fmt.Errorf("failed to marshal cache: %w", err)
    }
    if err := os.WriteFile(sc.filePath, data, 0644); err != nil {
        return fmt.Errorf("failed to write cache file: %w", err)
    }
    return nil
}
```

**要点**：
- ✅ 使用 `GetConfigDir()` 获取路径
- ✅ 文件读写操作与普通文件操作相同
- ✅ 系统自动处理 iCloud 同步

### 示例 3：显示存储位置信息

**场景**：在初始化完成后显示配置存储位置

**代码位置**：`cmd/qkflow/commands/init.go`

```go
func showStorageLocation() {
    // 获取人类可读的存储位置描述
    location := utils.GetConfigLocation()

    // 获取实际配置目录路径
    configDir, _ := utils.GetQuickWorkflowConfigDir()

    // 显示给用户
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

**要点**：
- ✅ 使用 `GetConfigLocation()` 获取用户友好的描述
- ✅ 使用 `GetQuickWorkflowConfigDir()` 获取实际路径
- ✅ 向用户清晰展示存储位置

### 示例 4：保存 Watch 状态

**场景**：保存 Watch 守护进程的状态信息

**代码位置**：`internal/watcher/state.go`

```go
func NewState() (*State, error) {
    // 获取配置目录（自动选择 iCloud 或本地）
    configDir, err := utils.GetConfigDir()
    if err != nil {
        return nil, fmt.Errorf("failed to get config directory: %w", err)
    }

    // 构建状态文件路径
    filePath := filepath.Join(configDir, "watch-state.json")

    // 如果文件存在，加载现有状态
    if _, err := os.Stat(filePath); err == nil {
        data, err := os.ReadFile(filePath)
        if err != nil {
            return nil, fmt.Errorf("failed to read state file: %w", err)
        }

        var state State
        if err := json.Unmarshal(data, &state); err != nil {
            return nil, fmt.Errorf("failed to parse state file: %w", err)
        }

        state.filePath = filePath
        return &state, nil
    }

    // 创建新状态
    state := &State{
        ProcessedPRs: make([]ProcessedPR, 0),
        Stats:        Statistics{},
        filePath:     filePath,
    }

    // 保存初始状态
    if err := state.Save(); err != nil {
        return nil, err
    }

    return state, nil
}

func (s *State) Save() error {
    data, err := json.MarshalIndent(s, "", "  ")
    if err != nil {
        return fmt.Errorf("failed to marshal state: %w", err)
    }

    if err := os.WriteFile(s.filePath, data, 0644); err != nil {
        return fmt.Errorf("failed to write state file: %w", err)
    }

    return nil
}
```

**要点**：
- ✅ 使用 `GetConfigDir()` 获取路径
- ✅ 状态文件会自动同步到 iCloud（如果可用）
- ✅ 多设备可以共享 Watch 状态

### 示例 5：保存日志文件

**场景**：如果用户配置了文件日志，保存到配置目录

**代码位置**：`internal/logger/factory.go`

```go
func createFileHandler(logFilePath string) (logger.Handler, error) {
    // 如果未指定路径，使用默认路径（在配置目录中）
    if logFilePath == "" {
        configDir, err := utils.GetConfigDir()
        if err == nil {
            logFilePath = filepath.Join(configDir, "qkflow.log")
        } else {
            // 如果无法获取配置目录，使用临时目录
            logFilePath = filepath.Join(os.TempDir(), "qkflow.log")
        }
    }

    // 确保目录存在
    dir := filepath.Dir(logFilePath)
    if err := os.MkdirAll(dir, 0755); err != nil {
        return nil, fmt.Errorf("failed to create log directory: %w", err)
    }

    // 打开日志文件
    file, err := os.OpenFile(logFilePath, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
    if err != nil {
        return nil, fmt.Errorf("failed to open log file: %w", err)
    }

    return logger.NewFileHandler(file), nil
}
```

**要点**：
- ✅ 使用 `GetConfigDir()` 获取默认日志路径
- ✅ 日志文件可以存储在 iCloud 中（如果用户需要）
- ✅ 有回退机制（使用临时目录）

## 🔄 常见模式

### 模式 1：获取路径并创建文件

```go
// 1. 获取配置目录
configDir, err := utils.GetConfigDir()
if err != nil {
    return fmt.Errorf("failed to get config directory: %w", err)
}

// 2. 构建文件路径
filePath := filepath.Join(configDir, "my-file.json")

// 3. 确保目录存在（通常不需要，GetConfigDir 已创建）
// os.MkdirAll(filepath.Dir(filePath), 0755)

// 4. 读写文件
data, err := os.ReadFile(filePath)
// 或
err := os.WriteFile(filePath, data, 0644)
```

### 模式 2：检查文件是否存在

```go
configDir, err := utils.GetConfigDir()
if err != nil {
    return err
}

filePath := filepath.Join(configDir, "my-file.json")
if _, err := os.Stat(filePath); os.IsNotExist(err) {
    // 文件不存在，创建默认内容
    defaultData := []byte("{}")
    if err := os.WriteFile(filePath, defaultData, 0644); err != nil {
        return err
    }
}
```

### 模式 3：显示存储位置

```go
location := utils.GetConfigLocation()
configDir, _ := utils.GetConfigDir()

fmt.Printf("Storage: %s\n", location)
fmt.Printf("Path: %s\n", configDir)
```

## ⚠️ 注意事项

### 1. 错误处理

**总是检查错误**：

```go
// ❌ 错误：忽略错误
configDir, _ := utils.GetConfigDir()

// ✅ 正确：处理错误
configDir, err := utils.GetConfigDir()
if err != nil {
    return fmt.Errorf("failed to get config directory: %w", err)
}
```

### 2. 文件权限

**设置适当的文件权限**：

```go
// 配置文件：仅用户可读写
os.WriteFile(filePath, data, 0600)

// 缓存文件：用户可读写，其他可读
os.WriteFile(filePath, data, 0644)
```

### 3. 目录创建

**通常不需要手动创建目录**：

```go
// GetConfigDir() 已经创建了目录
configDir, err := utils.GetConfigDir()  // 目录已存在

// 通常不需要再次创建
// os.MkdirAll(configDir, 0755)  // 不需要
```

### 4. iCloud 同步延迟

**注意 iCloud 同步可能有延迟**：

```go
// 写入文件后，立即读取可能获取旧数据（如果从另一台设备读取）
os.WriteFile(filePath, data, 0644)

// iCloud 同步可能需要几秒到几分钟
// 在同一台设备上，通常可以立即读取
```

## 🧪 测试示例

### 单元测试

```go
func TestGetConfigDir(t *testing.T) {
    // 测试获取配置目录
    configDir, err := utils.GetConfigDir()
    if err != nil {
        t.Fatalf("GetConfigDir() failed: %v", err)
    }

    // 验证目录存在
    if info, err := os.Stat(configDir); err != nil {
        t.Fatalf("Config directory does not exist: %v", err)
    } else if !info.IsDir() {
        t.Fatalf("Config path is not a directory: %s", configDir)
    }
}

func TestIsICLoudAvailable(t *testing.T) {
    // 测试 iCloud 可用性检查
    isAvailable := utils.IsICLoudAvailable()

    // 在 macOS 上，结果取决于 iCloud Drive 是否启用
    // 在非 macOS 上，应该总是返回 false
    if runtime.GOOS != "darwin" && isAvailable {
        t.Error("IsICLoudAvailable() should return false on non-macOS")
    }
}
```

### 集成测试

```go
func TestConfigSaveAndLoad(t *testing.T) {
    // 创建测试配置
    cfg := &config.Config{
        Email:              "test@example.com",
        JiraServiceAddress: "https://test.atlassian.net",
        // ...
    }

    // 保存配置
    if err := config.Save(cfg); err != nil {
        t.Fatalf("Save() failed: %v", err)
    }

    // 重置缓存
    config.Reset()

    // 加载配置
    loadedCfg, err := config.Load()
    if err != nil {
        t.Fatalf("Load() failed: %v", err)
    }

    // 验证配置
    if loadedCfg.Email != cfg.Email {
        t.Errorf("Email mismatch: got %s, want %s", loadedCfg.Email, cfg.Email)
    }
}
```

## 📝 总结

### 最佳实践

1. ✅ **总是使用 `GetConfigDir()` 或 `GetQuickWorkflowConfigDir()`**
   - 不要硬编码路径
   - 让系统自动选择存储位置

2. ✅ **正确处理错误**
   - 检查所有错误返回值
   - 提供有意义的错误信息

3. ✅ **设置适当的文件权限**
   - 配置文件：`0600`（仅用户可读写）
   - 缓存文件：`0644`（用户可读写，其他可读）

4. ✅ **向用户显示存储位置**
   - 使用 `GetConfigLocation()` 获取用户友好的描述
   - 在初始化或配置命令中显示

5. ✅ **考虑 iCloud 同步延迟**
   - 在同一台设备上，文件操作是立即的
   - 跨设备同步可能需要时间

### 避免的常见错误

1. ❌ **硬编码路径**
   ```go
   // ❌ 错误
   configPath := "~/.qkflow/config.yaml"

   // ✅ 正确
   configDir, _ := utils.GetConfigDir()
   configPath := filepath.Join(configDir, "config.yaml")
   ```

2. ❌ **忽略错误**
   ```go
   // ❌ 错误
   configDir, _ := utils.GetConfigDir()

   // ✅ 正确
   configDir, err := utils.GetConfigDir()
   if err != nil {
       return err
   }
   ```

3. ❌ **手动检查 iCloud**
   ```go
   // ❌ 错误：不需要手动检查
   if utils.IsICLoudAvailable() {
       path := "~/Library/Mobile Documents/..."
   } else {
       path := "~/.qkflow"
   }

   // ✅ 正确：让系统自动选择
   configDir, _ := utils.GetConfigDir()
   ```

---

**最后更新**：2025-01-XX