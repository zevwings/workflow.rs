# 数据可视化与报告待办事项

## 📋 概述

本文档列出数据可视化与报告相关的待办功能，包括统计报告和可视化功能。

---

## ❌ 待实现功能

### 1. 统计报告

#### 1.1 `stats pr` - PR 统计
- ❌ PR 统计（创建数量、合并时间、review 时间）

**功能**：PR 统计（创建数量、合并时间、review 时间）。

**命令示例**：
```bash
workflow stats pr                                  # PR 统计
workflow stats pr --period week                    # 按周统计
workflow stats pr --author user@example.com        # 按作者统计
workflow stats pr --export report.json             # 导出报告
```

**统计指标**：
- PR 创建数量（总数、按时间分布）
- PR 合并时间（平均、中位数、最长、最短）
- Review 时间（平均、中位数）
- PR 状态分布（open、merged、closed）
- Review 状态分布（approved、changes requested、pending）

**实现建议**：
- 使用平台 API 获取 PR 数据
- 支持时间范围过滤（`--since`、`--until`）
- 支持按作者、项目、标签等过滤
- 支持导出为 JSON、CSV、HTML 格式

#### 1.2 `stats jira` - JIRA 统计
- ❌ JIRA 统计（ticket 数量、状态分布、完成时间）

**功能**：JIRA 统计（ticket 数量、状态分布、完成时间）。

**命令示例**：
```bash
workflow stats jira                                # JIRA 统计
workflow stats jira --project PROJ                 # 按项目统计
workflow stats jira --sprint "Sprint 2"            # 按 Sprint 统计
```

**统计指标**：
- Ticket 数量（总数、按类型分布、按状态分布）
- 完成时间（平均、中位数、最长、最短）
- 状态转换时间（各状态停留时间）
- 按指派人统计
- 按优先级统计

**实现建议**：
- 使用 JIRA API 获取 ticket 数据
- 支持按项目、Sprint、指派人等过滤
- 支持时间范围过滤

#### 1.3 `stats work` - 工作量统计
- ❌ 工作量统计（基于 JIRA worklog）

**功能**：工作量统计（基于 JIRA worklog）。

**命令示例**：
```bash
workflow stats work                                 # 工作量统计
workflow stats work --period month                 # 按月统计
workflow stats work --user user@example.com        # 按用户统计
```

**统计指标**：
- 总工作时间（按用户、按项目、按时间）
- 工作时间分布（按天、按周、按月）
- 平均每日工作时间
- 工作时间趋势

**实现建议**：
- 使用 JIRA API `/issue/{issueIdOrKey}/worklog` 端点
- 支持时间格式解析和统计
- 支持导出报告

---

### 2. 可视化

#### 2.1 图表输出
- ❌ ASCII 图表输出
- ❌ HTML 报告生成
- ❌ 图片导出（PNG、SVG）

**功能**：使用 ASCII 图表或生成 HTML 报告。

**实现建议**：
- 使用 `textplots` 或类似库生成 ASCII 图表
- 支持生成 HTML 报告（使用模板引擎）
- 支持导出为图片（PNG、SVG）

**图表类型**：
- 柱状图（bar chart）
- 折线图（line chart）
- 饼图（pie chart）
- 时间线图（timeline）

**命令示例**：
```bash
workflow stats pr --chart bar                      # 显示柱状图
workflow stats pr --chart line                     # 显示折线图
workflow stats pr --export report.html             # 导出 HTML 报告
workflow stats pr --export chart.png              # 导出图片
```

#### 2.2 时间线视图
- ❌ PR 时间线视图
- ❌ JIRA ticket 时间线视图

**功能**：显示 PR/JIRA ticket 的时间线。

**命令示例**：
```bash
workflow timeline pr 123                           # PR 时间线
workflow timeline jira PROJ-123                    # JIRA ticket 时间线
```

**时间线事件**：
- PR：创建、更新、评论、review、合并/关闭
- JIRA：创建、状态变更、分配、评论、解决

**实现建议**：
- 使用 ASCII 或 Unicode 字符绘制时间线
- 支持时间范围过滤
- 支持导出为文本或 HTML

---

## 📊 优先级

### 高优先级
1. **统计报告基础功能**
   - `stats pr` - PR 统计
   - `stats jira` - JIRA 统计

### 中优先级
1. **统计报告增强**
   - `stats work` - 工作量统计
   - 导出功能（JSON、CSV、HTML）

2. **可视化基础功能**
   - ASCII 图表输出
   - 时间线视图

### 低优先级
1. **可视化增强**
   - HTML 报告生成
   - 图片导出（PNG、SVG）

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：统计报告基础功能
   - `stats pr` - PR 统计
   - `stats jira` - JIRA 统计

2. **第二阶段**：统计报告增强和可视化基础
   - `stats work` - 工作量统计
   - ASCII 图表输出
   - 时间线视图

3. **第三阶段**：可视化增强
   - HTML 报告生成
   - 图片导出

### 技术考虑
1. **统计库**：使用 Rust 统计库（如 `statrs`）进行数据统计
2. **图表库**：使用 `textplots` 或类似库生成 ASCII 图表
3. **模板引擎**：使用 `handlebars` 或 `tera` 生成 HTML 报告
4. **图片生成**：使用 `plotters` 或类似库生成图片
5. **数据存储**：考虑缓存统计数据以提高性能
6. **测试**：为新功能添加单元测试和集成测试
7. **文档**：及时更新文档和示例

### 实现细节

#### 统计报告实现
```rust
pub struct PrStats {
    pub total_count: usize,
    pub merged_count: usize,
    pub open_count: usize,
    pub closed_count: usize,
    pub avg_merge_time: Duration,
    pub avg_review_time: Duration,
}

pub async fn calculate_pr_stats(
    prs: Vec<PullRequest>,
    period: Option<TimeRange>,
) -> Result<PrStats> {
    // 计算统计数据
}
```

#### 图表生成实现
```rust
use textplots::{Chart, Plot, Shape};

pub fn generate_bar_chart(data: &[(String, f64)]) -> String {
    let mut chart = Chart::new(80, 20);
    chart.lineplot(&Shape::Bars(data));
    chart.to_string()
}
```

---

## 📚 相关文档

- [JIRA 模块待办事项](./JIRA.md)
- [Git 工作流待办事项](./GIT.md)

---

**最后更新**: 2024-12-19
