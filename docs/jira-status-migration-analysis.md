# Jira Status JSON 迁移可行性分析

## 当前状态

### 当前实现
- **文件格式**: 文本格式 (`jira-status.txt`)
- **存储位置**: 项目根目录（当前工作目录）
- **文件格式**:
  ```
  WEW:
  created-pr,WEW,In Progress
  merged-pr,WEW,Done
  ```

### 使用位置
1. `src/lib/jira/status.rs` - 核心读写逻辑
2. `src/commands/jira/status.rs` - 配置和读取接口
3. `src/commands/pr/create.rs` - 读取 PR 创建时的状态
4. `src/commands/pr/merge.rs` - 读取 PR 合并时的状态

## 目标方案

### 新格式
- **文件格式**: JSON (`jira-status.json`)
- **存储位置**: `${HOME}/.workflow/jira-status.json`
- **JSON 格式**（推荐对象格式）:
  ```json
  {
    "WEW": {
      "created-pr": "In Progress",
      "merged-pr": "Done"
    },
    "PROJ": {
      "created-pr": "In Review",
      "merged-pr": "Done"
    }
  }
  ```

### 数组格式（用户原始需求）
```json
[
  {
    "WEW": {
      "created-pr": "In Progress",
      "merged-pr": "Done"
    }
  },
  {
    "PROJ": {
      "created-pr": "In Review",
      "merged-pr": "Done"
    }
  }
]
```

## 可行性分析

### ✅ 优势

1. **依赖已具备**
   - ✅ `serde_json = "1.0"` 已在 `Cargo.toml` 中
   - ✅ `JiraStatusConfig` 已有 `Serialize/Deserialize` derive

2. **代码结构清晰**
   - ✅ 读写逻辑集中在 `src/lib/jira/status.rs`
   - ✅ 外部调用仅通过 `read_jira_status()` 和 `write_jira_status()`
   - ✅ 接口稳定，内部实现可替换

3. **实现简单**
   - ✅ JSON 序列化/反序列化比文本解析更简单
   - ✅ 无需手动处理文本格式和换行
   - ✅ 类型安全，编译时检查

### ⚠️ 需要考虑的问题

1. **数据结构选择**
   - **对象格式**（推荐）: `{ "WEW": {...}, "PROJ": {...} }`
     - ✅ 查找更快（O(1)）
     - ✅ 更新更简单
     - ✅ 更符合 JSON 常见用法
   - **数组格式**: `[{ "WEW": {...} }, { "PROJ": {...} }]`
     - ⚠️ 需要遍历查找（O(n)）
     - ⚠️ 更新时需要找到并替换元素
     - ⚠️ 格式冗余

2. **目录创建**
   - 需要确保 `${HOME}/.workflow` 目录存在
   - 如果不存在，需要自动创建

3. **向后兼容**
   - 可选：迁移现有 `jira-status.txt` 到新格式
   - 如果不存在 JSON 文件，可以忽略或提示

4. **文件路径**
   - 当前使用 `ConfigPaths::new()` 获取路径
   - 需要修改为使用 `$HOME/.workflow` 目录

## 实现建议

### 推荐方案

1. **使用对象格式**（而非数组）
   ```json
   {
     "WEW": {
       "created-pr": "In Progress",
       "merged-pr": "Done"
     }
   }
   ```

2. **修改点**
   - `ConfigPaths::new()` - 改为使用 `${HOME}/.workflow/jira-status.json`
   - `read_jira_status()` - 使用 `serde_json::from_str` 读取
   - `write_jira_status()` - 使用 `serde_json::to_string_pretty` 写入
   - 确保目录存在：`fs::create_dir_all()`

3. **数据结构**
   ```rust
   use std::collections::HashMap;

   type JiraStatusMap = HashMap<String, ProjectStatusConfig>;

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ProjectStatusConfig {
       #[serde(rename = "created-pr")]
       pub created_pr_status: Option<String>,
       #[serde(rename = "merged-pr")]
       pub merged_pr_status: Option<String>,
   }
   ```

### 如果必须使用数组格式

可以使用，但需要额外处理：
- 查找时需要遍历数组
- 更新时需要找到对应元素并替换
- 代码复杂度稍高

## 总结

**可行性**: ✅ **完全可行**

**建议**:
1. 使用对象格式而非数组格式（更简洁高效）
2. 实现自动目录创建
3. 可选：实现旧格式迁移（向后兼容）

**工作量**: 小（主要修改 `src/lib/jira/status.rs` 文件）

**风险**: 低（接口保持不变，仅内部实现改变）

