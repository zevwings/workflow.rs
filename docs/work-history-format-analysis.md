# Work History 格式优化分析

## 当前格式问题

### 当前实现
- **格式**: CSV 文本文件 (`work-history.txt`)
- **存储位置**: 项目根目录（每个项目独立）
- **格式**: `PROJ-123,456` (Jira ticket, PR ID)
- **查询方式**: 线性扫描（O(n)）

### 存在的问题

1. **查询效率低**
   - 每次查询都需要读取整个文件并逐行扫描
   - O(n) 时间复杂度，随着记录增多性能下降

2. **信息量不足**
   - 只存储 Jira ticket 和 PR ID
   - 缺少时间戳、PR URL、仓库信息等
   - 无法扩展更多元数据

3. **存储位置不合理**
   - 每个项目都有独立的文件
   - 无法全局管理和查询
   - 与 `jira-status.json` 的存储策略不一致

4. **格式扩展性差**
   - CSV 格式难以添加新字段
   - 容易出错（逗号、换行等问题）
   - 无法进行结构化查询

5. **缺少双向查询**
   - 当前只能通过 PR ID 查找 Jira ticket
   - 无法通过 Jira ticket 查找所有关联的 PR

---

## 优化方案

### 方案 1: JSON 格式（推荐）⭐

#### 格式设计
```json
{
  "456": {
    "jira_ticket": "PROJ-123",
    "pr_url": "https://github.com/xxx/pull/456",
    "created_at": "2024-01-15T10:30:00Z",
    "merged_at": null,
    "repository": "github.com/xxx/yyy",
    "branch": "feature/PROJ-123-add-feature"
  },
  "457": {
    "jira_ticket": "PROJ-124",
    "pr_url": "https://codeup.aliyun.com/xxx/pull/457",
    "created_at": "2024-01-16T14:20:00Z",
    "merged_at": "2024-01-17T09:15:00Z",
    "repository": "codeup.aliyun.com/xxx/yyy",
    "branch": "feature/PROJ-124-fix-bug"
  }
}
```

#### 数据结构
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHistoryEntry {
    pub jira_ticket: String,
    pub pr_url: Option<String>,
    pub created_at: Option<String>,  // ISO 8601 格式
    pub merged_at: Option<String>,
    pub repository: Option<String>,
    pub branch: Option<String>,
}

type WorkHistoryMap = HashMap<String, WorkHistoryEntry>;  // PR ID -> Entry
```

#### 优势
- ✅ **快速查询**: O(1) 时间复杂度（HashMap 查找）
- ✅ **信息丰富**: 支持扩展多个字段
- ✅ **双向查询**: 可以建立反向索引（Jira ticket -> PR IDs）
- ✅ **类型安全**: 使用 serde 自动序列化/反序列化
- ✅ **易于扩展**: 添加新字段只需修改结构体
- ✅ **统一格式**: 与 `jira-status.json` 保持一致

#### 存储位置
- `${HOME}/.workflow/work-history.json`（全局管理）

---

### 方案 2: 双向索引 JSON 格式

如果需要支持双向查询（PR ID → Jira ticket 和 Jira ticket → PR IDs）：

```json
{
  "by_pr_id": {
    "456": {
      "jira_ticket": "PROJ-123",
      "pr_url": "https://github.com/xxx/pull/456",
      "created_at": "2024-01-15T10:30:00Z",
      "merged_at": null
    }
  },
  "by_jira_ticket": {
    "PROJ-123": ["456", "789"],
    "PROJ-124": ["457"]
  }
}
```

#### 优势
- ✅ 支持双向查询
- ✅ 可以查找一个 Jira ticket 关联的所有 PR

#### 劣势
- ⚠️ 数据结构更复杂
- ⚠️ 写入时需要同时更新两个索引

---

### 方案 3: 时间序列 JSON 格式

如果需要按时间排序和查询：

```json
{
  "entries": [
    {
      "pr_id": "456",
      "jira_ticket": "PROJ-123",
      "created_at": "2024-01-15T10:30:00Z",
      "merged_at": null
    },
    {
      "pr_id": "457",
      "jira_ticket": "PROJ-124",
      "created_at": "2024-01-16T14:20:00Z",
      "merged_at": "2024-01-17T09:15:00Z"
    }
  ]
}
```

#### 优势
- ✅ 保持时间顺序
- ✅ 便于生成时间序列报告

#### 劣势
- ⚠️ 查询需要遍历数组（O(n)）
- ⚠️ 可以结合 HashMap 索引优化

---

## 推荐方案：方案 1（JSON HashMap）

### 实现建议

1. **数据结构**
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct WorkHistoryEntry {
       pub jira_ticket: String,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub pr_url: Option<String>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub created_at: Option<String>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub merged_at: Option<String>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub repository: Option<String>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub branch: Option<String>,
   }

   type WorkHistoryMap = HashMap<String, WorkHistoryEntry>;
   ```

2. **存储位置**
   - `${HOME}/.workflow/work-history.json`

3. **API 设计**
   ```rust
   // 写入（创建 PR 时）
   pub fn write_work_history(
       pr_id: &str,
       jira_ticket: &str,
       pr_url: Option<&str>,
       repository: Option<&str>,
       branch: Option<&str>,
   ) -> Result<()>

   // 读取（通过 PR ID 查找 Jira ticket）
   pub fn read_work_history(pr_id: &str) -> Result<Option<WorkHistoryEntry>>

   // 更新（合并 PR 时）
   pub fn update_work_history_merged(pr_id: &str, merged_at: &str) -> Result<()>

   // 扩展：通过 Jira ticket 查找所有 PR（可选）
   pub fn find_prs_by_jira_ticket(jira_ticket: &str) -> Result<Vec<String>>
   ```

---

## 格式对比

| 特性 | 当前格式（CSV） | 推荐格式（JSON HashMap） |
|------|----------------|-------------------------|
| **查询效率** | O(n) 线性扫描 | O(1) HashMap 查找 |
| **信息量** | 2 个字段 | 可扩展多个字段 |
| **存储位置** | 项目根目录 | `${HOME}/.workflow/` |
| **扩展性** | 差 | 优秀 |
| **类型安全** | 无 | 有（serde） |
| **双向查询** | 不支持 | 支持（可选实现） |
| **格式统一** | 否 | 是（与 jira-status.json 一致） |

---

## 迁移建议

1. **向后兼容**
   - 读取时先检查新格式文件
   - 如果不存在，尝试读取旧格式并转换
   - 自动迁移到新格式

2. **渐进式扩展**
   - 第一阶段：保持基本字段（jira_ticket, pr_id）
   - 第二阶段：添加时间戳和 URL
   - 第三阶段：添加更多元数据

3. **性能优化**
   - 使用 HashMap 实现 O(1) 查询
   - 可选：添加反向索引支持双向查询

---

## 总结

**推荐使用 JSON HashMap 格式**，原因：
1. ✅ 查询效率高（O(1)）
2. ✅ 信息丰富，易于扩展
3. ✅ 与现有 `jira-status.json` 格式统一
4. ✅ 存储位置统一（`${HOME}/.workflow/`）
5. ✅ 类型安全，减少错误

这是最平衡的方案，兼顾了性能、可扩展性和实现复杂度。

