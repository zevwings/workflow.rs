# 别名系统需求文档

## 📋 需求概述

本文档描述别名系统的需求，包括别名配置、别名展开和别名管理命令。

**状态**: 📋 需求分析中
**分类**: 用户体验优化
**优先级**: 中优先级
**来源**: 从 `docs/todo/ALIAS_TODO.md` 迁移

---

## 🎯 需求目标

实现命令别名系统，以：
1. 简化常用命令输入，提高命令输入效率
2. 支持自定义别名，满足个人使用习惯
3. 支持别名嵌套和参数传递，提供灵活的扩展能力

---

## 📝 详细需求

### 1. 别名配置

#### 1.1 功能描述
支持在配置文件中定义别名，将简短别名映射到完整命令。

#### 1.2 功能要求
- 在配置文件中定义别名
- 支持命令参数传递
- 支持别名嵌套（别名引用别名）
- 防止循环别名（无限递归）

#### 1.3 配置格式
```toml
[aliases]
ci = "pr create"
cm = "pr merge"
js = "jira search"
ji = "jira info"

# 支持嵌套别名
prc = "ci"  # prc -> ci -> pr create
```

#### 1.4 使用示例
```bash
workflow ci                                        # 等同于 workflow pr create
workflow cm                                        # 等同于 workflow pr merge
workflow js "project = PROJ"                       # 等同于 workflow jira search "project = PROJ"
workflow ji PROJ-123                               # 等同于 workflow jira info PROJ-123
```

---

### 2. 别名展开

#### 2.1 功能描述
在主入口处自动展开别名，将别名替换为完整命令。

#### 2.2 功能要求
- 自动检测第一个参数是否是别名
- 如果是别名，展开为完整命令
- 保留原始命令的剩余参数
- 支持嵌套别名展开
- 防止循环别名导致的无限递归

#### 2.3 展开逻辑
1. **基本展开**：
   - 检查第一个参数是否是别名
   - 如果是，替换为别名值
   - 保留剩余参数

2. **嵌套别名处理**：
   - 使用 `HashSet` 跟踪已访问的别名（防止循环）
   - 递归展开嵌套别名
   - 最多展开深度限制（可选）

3. **参数传递**：
   - 别名展开后，将原始命令的剩余参数追加到展开后的命令
   - 例如：`workflow ci --title "test"` → `workflow pr create --title "test"`

---

### 3. 别名管理命令

#### 3.1 `alias list` - 列出所有别名

##### 功能描述
列出所有已定义的别名。

##### 命令示例
```bash
workflow alias list
```

##### 输出示例
```
Defined aliases:

  ci = pr create
  cm = pr merge
  js = jira search
  ji = jira info
```

---

#### 3.2 `alias add` - 添加别名

##### 功能描述
添加新的别名。

##### 命令示例
```bash
workflow alias add ci "pr create"
workflow alias add cm "pr merge"
```

##### 功能要求
- 检查别名是否已存在
- 如果已存在，提示用户
- 保存别名到配置文件

---

#### 3.3 `alias remove` - 删除别名

##### 功能描述
删除指定的别名。

##### 命令示例
```bash
workflow alias remove ci
```

##### 功能要求
- 检查别名是否存在
- 如果不存在，提示用户
- 从配置文件中删除别名

---

### 4. 别名自动补全

#### 4.1 功能描述
在 Shell 自动补全中支持别名，用户输入 `workflow ` 后按 Tab 键时，应显示所有子命令和已定义的别名。

#### 4.2 功能要求
- 别名应作为候选项出现在自动补全列表中
- 别名与子命令一起显示，按字母顺序排序
- 支持动态加载别名（从配置文件读取）
- 别名添加/删除后，自动补全应自动更新（需要重新生成补全脚本）

#### 4.3 使用示例
```bash
# 用户输入 workflow 后按 Tab
$ workflow <TAB>
alias    branch   ci       cm       config   github   ji       js       jira     log      pr       ...

# 别名 ci, cm, ji, js 出现在补全列表中
```

#### 4.4 技术实现要求
- 在生成 completion 脚本时，从配置文件读取所有别名
- 将别名作为额外的候选项添加到补全脚本中
- 支持所有 shell 类型（zsh, bash, fish, powershell, elvish）
- 别名与子命令在补全列表中应能区分（可选：添加标记或描述）

#### 4.5 补全脚本更新
- 当用户执行 `workflow alias add` 或 `workflow alias remove` 时，提示用户重新生成补全脚本
- 提供 `workflow completion generate` 命令来更新补全脚本
- 可选：自动检测别名变化并提示更新补全脚本

---

## 🔧 技术实现

### 核心模块结构
```
src/lib/base/alias/
├── mod.rs          # 模块声明和导出
└── manager.rs      # AliasManager 实现
```

### 核心数据结构
- `AliasConfig`: 别名配置结构体
- `AliasManager`: 别名管理器，提供加载、展开、添加、删除等功能

### 主入口集成
- 在 `src/bin/workflow.rs` 中集成别名展开逻辑
- 在命令解析前先进行别名展开
- 重新构建命令行参数并解析

### 配置文件存储
- 别名配置存储在 `workflow.toml` 中
- 使用现有的配置管理系统

### 自动补全集成
- 在 `src/lib/completion/generate.rs` 中集成别名补全
- 生成补全脚本时，从配置文件读取别名列表
- 将别名添加到 clap Command 的补全候选项中
- 支持动态别名补全（运行时从配置文件读取）

---

## ✅ 验收标准

### 功能验收
- [ ] 能够在配置文件中定义别名
- [ ] 别名能够正确展开为完整命令
- [ ] 支持命令参数传递（`workflow ci --title "test"`）
- [ ] 支持别名嵌套（别名引用别名）
- [ ] 能够添加新别名（`workflow alias add <name> <command>`）
- [ ] 能够删除别名（`workflow alias remove <name>`）
- [ ] 能够列出所有别名（`workflow alias list`）
- [ ] 别名出现在自动补全候选项中
- [ ] 支持所有 shell 类型的别名补全（zsh, bash, fish, powershell, elvish）
- [ ] 别名添加/删除后，补全脚本能够更新

### 边界情况
- [ ] 处理循环别名（防止无限递归）
- [ ] 处理不存在的别名（返回原命令，不报错）
- [ ] 处理空别名配置
- [ ] 处理别名名称冲突（与现有命令冲突时给出提示）

### 用户体验
- [ ] 别名展开对用户透明
- [ ] 错误信息清晰友好
- [ ] 命令帮助信息完整
- [ ] 配置文件不存在时使用默认空配置
- [ ] 别名在补全列表中清晰可见
- [ ] 添加/删除别名后，提示用户更新补全脚本（可选）

---

## 📊 优先级说明

### 优先级：中优先级

### 原因
- 提高命令输入效率
- 简化常用命令
- 提升用户体验

### 依赖
- 配置文件管理系统（已实现）
- CLI 命令解析系统（已实现）
- Shell Completion 模块（已实现）

---

## 🔗 依赖关系

### 实现顺序建议
1. **第一阶段**：核心功能
   - 创建别名管理模块
   - 实现别名配置加载和保存
   - 实现别名展开逻辑

2. **第二阶段**：主入口集成
   - 在主入口集成别名展开
   - 测试别名展开功能
   - 测试参数传递和嵌套别名

3. **第三阶段**：管理命令
   - 实现 `alias list` 命令
   - 实现 `alias add` 命令
   - 实现 `alias remove` 命令

4. **第四阶段**：自动补全集成
   - 在 completion 生成器中集成别名读取
   - 将别名添加到补全候选项
   - 测试各 shell 类型的别名补全
   - 实现补全脚本更新机制

5. **第五阶段**：测试和文档
   - 编写集成测试
   - 测试边界情况
   - 更新文档

---

## 📚 相关文档

- [配置架构文档](../architecture/lib/SETTINGS_ARCHITECTURE.md) - 配置文件管理
- [CLI 架构文档](../architecture/lib/CLI_ARCHITECTURE.md) - 命令解析
- [Completion 架构文档](../architecture/lib/COMPLETION_ARCHITECTURE.md) - Shell 自动补全实现

---

**创建日期**: 2025-01-27
**最后更新**: 2025-01-27
**更新内容**: 添加别名自动补全需求说明
