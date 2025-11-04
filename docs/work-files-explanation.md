# Work History 和 Work Report 文件说明

## work-history.txt

### 📋 作用
存储 **PR ID 到 Jira ticket 的映射关系**，用于在合并 PR 时自动查找对应的 Jira ticket。

### 📍 存储位置
项目根目录（当前工作目录）：`{current_dir}/work-history.txt`

### 📝 文件格式
```
PROJ-123,456
PROJ-124,457
```
- 格式：`{JIRA_TICKET},{PR_ID}`（每行一条记录）
- 示例：`PROJ-123,456` 表示 PR #456 关联了 Jira ticket PROJ-123

### 🔄 使用流程

#### 1. **写入时机**（`src/commands/pr/create.rs`）
当创建 PR 并关联了 Jira ticket 时：
```rust
// 写入历史记录
let pr_id = extract_pr_id_from_url(&pr_url)?;
crate::jira::status::write_work_history(ticket, &pr_id)?;
```

**场景**：
- 用户执行 `workflow pr create PROJ-123`
- PR 创建成功后，自动记录映射关系
- 记录格式：`PROJ-123,456`（假设 PR ID 是 456）

#### 2. **读取时机**（`src/commands/pr/merge.rs`）
当合并 PR 时，需要更新 Jira 状态：
```rust
// 尝试从历史记录读取
let mut jira_ticket = crate::jira::status::read_work_history(&pr_id)?;

// 如果历史记录中没有，尝试从 PR 标题提取
if jira_ticket.is_none() {
    // 从 PR 标题中提取 Jira ticket ID
    jira_ticket = extract_jira_ticket_id(&title);
}
```

**场景**：
- 用户执行 `workflow pr merge 456`
- 系统首先从 `work-history.txt` 中查找 PR #456 对应的 Jira ticket
- 如果找到，自动更新该 ticket 的状态为 "合并后的状态"（如 "Done"）
- 如果找不到，尝试从 PR 标题中提取（作为后备方案）

### ✅ 优势
1. **自动关联**：无需手动记住 PR 和 Jira ticket 的对应关系
2. **自动更新**：合并 PR 时自动更新 Jira 状态
3. **容错机制**：如果历史记录中没有，会尝试从 PR 标题提取

### 📊 数据结构
```rust
// 读取函数
pub fn read_work_history(pr_id: &str) -> Result<Option<String>>

// 写入函数
pub fn write_work_history(jira_ticket: &str, pr_id: &str) -> Result<()>
```

---

## work-report.txt

### 📋 状态
**预留功能，当前未实现**

### 📍 存储位置
项目根目录（当前工作目录）：`{current_dir}/work-report.txt`

### 🔮 可能的用途
根据命名推测，可能是为了：
1. **生成工作报告**：汇总一段时间内的 PR 和 Jira ticket 信息
2. **工作统计**：统计工作量和完成情况
3. **导出数据**：将工作历史导出为报告格式

### 💻 代码状态
```rust
pub struct ConfigPaths {
    pub jira_status: PathBuf,
    pub work_history: PathBuf,
    #[allow(dead_code)]  // 标记为未使用
    pub work_report: PathBuf,
}
```

**注意**：目前没有实际的读写函数，只有路径定义。

---

## 文件对比

| 特性 | work-history.txt | work-report.txt |
|------|-----------------|-----------------|
| **状态** | ✅ 已实现并正在使用 | ⚠️ 预留，未实现 |
| **存储位置** | 项目根目录 | 项目根目录 |
| **格式** | CSV（逗号分隔） | 未知 |
| **主要用途** | PR ↔ Jira ticket 映射 | 工作报告（推测） |
| **读写函数** | ✅ `read_work_history()`<br>✅ `write_work_history()` | ❌ 无 |
| **使用场景** | PR 创建和合并流程 | 未使用 |

---

## 使用示例

### 实际使用流程

```bash
# 1. 创建 PR（会自动写入 work-history.txt）
workflow pr create PROJ-123
# 输出：PR created: https://github.com/xxx/pull/456
# 自动记录：PROJ-123,456 → work-history.txt

# 2. 合并 PR（会自动读取 work-history.txt）
workflow pr merge 456
# 自动读取：从 work-history.txt 查找 PR #456 → 找到 PROJ-123
# 自动更新：更新 PROJ-123 的状态为 "Done"
```

### 文件内容示例

**work-history.txt**:
```
PROJ-123,456
PROJ-124,457
WEW-100,458
```

---

## 总结

- **work-history.txt** 是核心功能文件，用于维护 PR 和 Jira ticket 的映射关系，确保工作流程的自动化
- **work-report.txt** 是预留功能，可能用于将来的工作报告生成功能

