# src/lib 模块解耦策略

本文档提供了将 `src/lib` 模块完全解耦的策略和实施方案。

## 当前耦合问题分析

### 1. 循环依赖问题

**问题**: `pr` ↔ `branch` 存在相互依赖

- `pr` 依赖 `branch::BranchType` - 用于映射分支类型到 PR 变更类型
- `branch` 依赖 `pr::llm::CreateGenerator` - 用于 LLM 生成分支名
- `branch` 依赖 `commands::pr::helpers::handle_stash_pop_result` - 用于处理 stash 恢复

### 2. 高耦合模块

- `pr` 依赖 5 个模块（base, git, jira, branch, template）
- `branch` 依赖 5 个模块（base, git, pr, repo, template）

## 解耦策略

### 策略 1: 提取共享类型到 base 模块

**目标**: 消除 `pr` 对 `branch` 的依赖

**方案**:
1. 将 `BranchType` 枚举移到 `base::types` 或创建 `base::workflow::types`
2. `branch` 和 `pr` 都依赖 `base`，而不是相互依赖

**实施步骤**:
```rust
// src/lib/base/workflow/types.rs (新建)
pub enum BranchType {
    Feature,
    Bugfix,
    Refactoring,
    Hotfix,
    Chore,
}

// src/lib/branch/types.rs - 重新导出
pub use crate::base::workflow::types::BranchType;

// src/lib/pr/platform.rs - 使用 base 的类型
use crate::base::workflow::types::BranchType;
```

**优点**:
- ✅ 完全消除 `pr` → `branch` 的依赖
- ✅ 类型定义集中管理
- ✅ 其他模块也可以使用这些类型

**缺点**:
- ⚠️ 需要重构现有代码
- ⚠️ `base` 模块会包含一些业务概念

---

### 策略 2: 抽象 LLM 生成器为 Trait

**目标**: 消除 `branch` 对 `pr::llm::CreateGenerator` 的直接依赖

**方案**:
1. 在 `base::llm` 中定义通用的 `BranchNameGenerator` trait
2. `pr::llm::CreateGenerator` 实现该 trait
3. `branch` 模块通过依赖注入使用 trait

**实施步骤**:
```rust
// src/lib/base/llm/generator.rs (新建)
pub trait BranchNameGenerator {
    fn generate_from_ticket(
        &self,
        ticket_id: &str,
        summary: &str,
    ) -> Result<String>;
}

// src/lib/pr/llm/create.rs - 实现 trait
impl BranchNameGenerator for CreateGenerator {
    fn generate_from_ticket(&self, ticket_id: &str, summary: &str) -> Result<String> {
        // 现有实现
    }
}

// src/lib/branch/naming.rs - 使用 trait
use crate::base::llm::BranchNameGenerator;

pub fn try_llm_generation(
    generator: &dyn BranchNameGenerator,  // 依赖注入
    ticket_id: &str,
    summary: &str,
) -> Result<String> {
    generator.generate_from_ticket(ticket_id, summary)
}
```

**优点**:
- ✅ 完全解耦 `branch` 和 `pr`
- ✅ 符合依赖倒置原则（DIP）
- ✅ 易于测试（可以 mock trait）
- ✅ 支持多种实现

**缺点**:
- ⚠️ 需要重构现有代码
- ⚠️ 需要依赖注入机制
- ⚠️ 可能增加代码复杂度

---

### 策略 3: 将共享功能移到 base 或 git 模块

**目标**: 消除 `branch` 对 `commands::pr::helpers` 的依赖

**方案**:
1. 将 `handle_stash_pop_result` 移到 `base::git::helpers` 或 `git::stash`
2. 这是一个通用的 Git 操作，不应该属于 `commands` 层

**实施步骤**:
```rust
// src/lib/git/stash.rs - 添加辅助函数
pub fn handle_stash_pop_result(result: Result<StashPopResult>) {
    // 从 commands::pr::helpers 移过来
}

// src/lib/branch/sync.rs - 使用 git 模块
use crate::git::stash::handle_stash_pop_result;
```

**优点**:
- ✅ 消除对 `commands` 层的依赖（违反分层架构）
- ✅ 功能归属更合理
- ✅ 其他模块也可以复用

**缺点**:
- ⚠️ 需要移动代码
- ⚠️ 可能影响现有调用

---

### 策略 4: 事件驱动架构（Event-Driven Architecture）

**目标**: 通过事件解耦模块间的直接调用

**方案**:
1. 在 `base` 中定义事件系统
2. 模块之间通过事件通信，而不是直接调用

**实施步骤**:
```rust
// src/lib/base/events/mod.rs (新建)
pub enum WorkflowEvent {
    BranchCreated { name: String },
    PRCreated { id: String },
    // ...
}

pub trait EventHandler {
    fn handle(&self, event: &WorkflowEvent);
}

pub struct EventBus {
    handlers: Vec<Box<dyn EventHandler>>,
}

// 使用示例
// branch 模块发布事件
event_bus.publish(WorkflowEvent::BranchCreated { name: branch_name });

// pr 模块订阅事件
event_bus.subscribe(Box::new(PRBranchHandler));
```

**优点**:
- ✅ 完全解耦模块
- ✅ 支持异步处理
- ✅ 易于扩展新功能
- ✅ 符合观察者模式

**缺点**:
- ⚠️ 架构变更较大
- ⚠️ 需要重新设计很多功能
- ⚠️ 可能过度设计（对于当前规模）

---

### 策略 5: 依赖注入容器（Dependency Injection Container）

**目标**: 通过 DI 容器管理模块依赖

**方案**:
1. 创建 DI 容器管理所有依赖
2. 模块通过接口（trait）交互
3. 容器负责创建和注入依赖

**实施步骤**:
```rust
// src/lib/base/di/mod.rs (新建)
pub struct Container {
    branch_name_generator: Box<dyn BranchNameGenerator>,
    // ...
}

impl Container {
    pub fn new() -> Self {
        // 创建所有依赖
    }
    
    pub fn get_branch_name_generator(&self) -> &dyn BranchNameGenerator {
        &*self.branch_name_generator
    }
}

// 使用
let container = Container::new();
let generator = container.get_branch_name_generator();
```

**优点**:
- ✅ 完全解耦
- ✅ 易于测试
- ✅ 支持多种实现切换
- ✅ 符合 SOLID 原则

**缺点**:
- ⚠️ 需要引入 DI 框架或自己实现
- ⚠️ 增加代码复杂度
- ⚠️ 可能过度设计

---

## 推荐方案：组合策略

基于当前代码规模和复杂度，推荐采用**组合策略**：

### 阶段 1: 快速解耦（低风险）

1. **提取 `BranchType` 到 `base::workflow::types`**
   - 消除 `pr` → `branch` 的依赖
   - 影响范围小，风险低

2. **移动 `handle_stash_pop_result` 到 `git::stash`**
   - 消除 `branch` 对 `commands` 的依赖
   - 功能归属更合理

### 阶段 2: 接口抽象（中等风险）

3. **抽象 `BranchNameGenerator` trait**
   - 消除 `branch` → `pr::llm` 的依赖
   - 通过依赖注入使用

### 阶段 3: 架构优化（可选，高风险）

4. **考虑事件驱动或 DI 容器**
   - 仅在模块数量继续增长时考虑
   - 当前规模可能过度设计

## 实施优先级

### 高优先级（立即实施）

1. ✅ **提取 `BranchType` 到 base**
   - 收益：消除 `pr` → `branch` 依赖
   - 成本：低（主要是移动代码）
   - 风险：低

2. ✅ **移动 `handle_stash_pop_result` 到 git**
   - 收益：修复架构分层问题
   - 成本：低
   - 风险：低

### 中优先级（计划实施）

3. ⚠️ **抽象 `BranchNameGenerator` trait**
   - 收益：完全解耦 `branch` 和 `pr`
   - 成本：中等（需要重构）
   - 风险：中等

### 低优先级（未来考虑）

4. 💡 **事件驱动架构**
   - 收益：完全解耦，支持扩展
   - 成本：高（需要大量重构）
   - 风险：高（可能过度设计）

## 解耦后的依赖关系

### 目标架构

```
base (基础设施)
  ├── workflow::types (共享类型)
  ├── llm::generator (抽象 trait)
  └── git::stash (共享功能)

git (依赖 base)
jira (依赖 base)
template (依赖 base)

branch (依赖 base, git, repo, template)
  └── 通过 trait 使用 LLM 生成器（不直接依赖 pr）

pr (依赖 base, git, jira, template)
  └── 使用 base::workflow::types（不依赖 branch）
```

### 依赖矩阵（解耦后）

| 模块 | base | git | jira | pr | branch | commit | completion | proxy | repo | template | rollback | cli |
|------|------|-----|------|----|--------|--------|------------|-------|------|----------|----------|-----|
| **base** | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **git** | ✅ | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **jira** | ✅ | ❌ | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **pr** | ✅ | ✅ | ✅ | - | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| **branch** | ✅ | ✅ | ❌ | ❌ | - | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ |
| **commit** | ✅ | ✅ | ❌ | ❌ | ❌ | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **completion** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ❌ | ❌ | ❌ | ❌ | ❌ |
| **proxy** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ❌ | ❌ | ❌ | ❌ |
| **repo** | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ❌ | ❌ | ❌ |
| **template** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ❌ | ❌ |
| **rollback** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | - | ❌ |
| **cli** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - |

**关键变化**:
- ✅ `pr` 不再依赖 `branch`
- ✅ `branch` 不再依赖 `pr`
- ✅ 所有模块都只依赖 `base` 和必要的业务模块

## 实施检查清单

### 阶段 1: 快速解耦

- [ ] 创建 `src/lib/base/workflow/types.rs`
- [ ] 移动 `BranchType` 到 `base::workflow::types`
- [ ] 更新 `branch::types` 重新导出
- [ ] 更新 `pr::platform` 使用 `base::workflow::types::BranchType`
- [ ] 移动 `handle_stash_pop_result` 到 `git::stash`
- [ ] 更新 `branch::sync` 使用 `git::stash::handle_stash_pop_result`
- [ ] 运行测试确保功能正常

### 阶段 2: 接口抽象

- [ ] 创建 `src/lib/base/llm/generator.rs`
- [ ] 定义 `BranchNameGenerator` trait
- [ ] 实现 `CreateGenerator` 实现 trait
- [ ] 重构 `branch::naming` 使用 trait
- [ ] 添加依赖注入机制（简单实现）
- [ ] 运行测试确保功能正常

### 阶段 3: 架构优化（可选）

- [ ] 评估是否需要事件驱动架构
- [ ] 评估是否需要 DI 容器
- [ ] 如果决定实施，制定详细计划

## 注意事项

1. **保持向后兼容**: 在重构时尽量保持公共 API 不变
2. **渐进式重构**: 分阶段实施，每个阶段都确保功能正常
3. **充分测试**: 每次重构后运行完整测试套件
4. **文档更新**: 及时更新相关文档
5. **代码审查**: 重要重构需要代码审查

## 总结

完全解耦是**可行的**，但需要权衡成本和收益：

- **快速解耦**（阶段 1）: ✅ 强烈推荐，成本低，收益高
- **接口抽象**（阶段 2）: ⚠️ 推荐，但需要评估成本
- **架构优化**（阶段 3）: 💡 可选，仅在规模继续增长时考虑

当前建议：**先实施阶段 1**，可以立即消除循环依赖，改善架构。阶段 2 和 3 可以根据实际需求决定是否实施。
