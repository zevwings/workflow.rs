# 基于大模型的代码提交分析系统设计方案

## 目录
- [1. 方案概述](#1-方案概述)
- [2. 系统架构](#2-系统架构)
- [3. 阶段一：文件分类](#3-阶段一文件分类)
- [4. 阶段二：分类分析](#4-阶段二分类分析)
- [5. 阶段三：全局总结](#5-阶段三全局总结)
- [6. 技术实现](#6-技术实现)
- [7. 优化策略](#7-优化策略)
- [8. 性能评估](#8-性能评估)

---

## 1. 方案概述

### 1.1 核心思路

采用**三阶段渐进式分析**方法，通过分类、细化、总结的流程，高效准确地分析代码提交内容。

```
阶段1：文件分类（轻量级分类）
   ↓
阶段2：分类分析（针对性深入）
   ↓
阶段3：全局总结（结构化输出）
```

### 1.2 方案优势

✅ **高效性**：第一阶段只需文件元信息，不需要完整diff内容
✅ **精准性**：针对不同文件类型使用不同分析策略
✅ **可控性**：每个阶段输出明确，易于调试和优化
✅ **可扩展**：支持针对特定场景自定义分析规则
✅ **成本优化**：避免对所有文件进行深度分析，节省Token

### 1.3 适用场景

- ✓ 大型提交（100+ 文件）
- ✓ 混合类型修改（功能+重构+配置）
- ✓ 批量操作（重命名、格式化、配置更新）
- ✓ 需要结构化commit message的团队
- ✓ 需要自动生成变更报告的CI/CD流程

---

## 2. 系统架构

### 2.1 数据流程图

```
Git Commit
    ↓
提取文件元信息（status, additions, deletions, path）
    ↓
┌─────────────────────────────────────┐
│   阶段1: 智能分类                    │
│   - 按变更类型分类                   │
│   - 按文件性质分类                   │
│   - 识别批量操作模式                 │
│   - 确定分析策略                     │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│   阶段2: 分类深入分析                │
│   ├─ 批量操作：抽样分析              │
│   ├─ 核心逻辑：完整分析              │
│   ├─ 配置文档：简要总结              │
│   └─ 测试文件：关联性分析            │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│   阶段3: 全局总结                    │
│   - 生成commit标题和描述             │
│   - 影响分析和风险评估               │
│   - 生成结构化输出                   │
└─────────────────────────────────────┘
    ↓
输出结果（JSON / Markdown / Git Message）
```

### 2.2 技术栈

| 组件 | 技术选型 | 说明 |
|------|---------|------|
| LLM模型 | Claude Sonnet 4 / GPT-4 | 支持长上下文，分析能力强 |
| Git解析 | GitPython / PyGit2 | 提取commit信息和diff |
| 并发处理 | asyncio / concurrent.futures | 阶段2并行分析 |
| 结构化输出 | JSON Schema验证 | 确保输出格式一致 |
| 缓存 | Redis / 本地文件 | 缓存分类结果和模式 |

---

## 3. 阶段一：文件分类

### 3.1 目标

通过**轻量级分析**对所有修改文件进行智能分类，为后续深入分析提供指导。

### 3.2 输入数据格式

```json
{
  "commit_id": "abc123def456",
  "author": "developer@example.com",
  "timestamp": "2024-02-07T10:30:00Z",
  "files": [
    {
      "path": "src/components/Header.tsx",
      "status": "modified",
      "additions": 15,
      "deletions": 3,
      "old_path": null
    },
    {
      "path": "src/utils/newHelper.ts",
      "status": "added",
      "additions": 50,
      "deletions": 0
    },
    {
      "path": "config/old_settings.json",
      "status": "deleted",
      "additions": 0,
      "deletions": 20
    },
    {
      "path": "src/services/api.ts",
      "status": "renamed",
      "old_path": "src/services/apiService.ts",
      "additions": 2,
      "deletions": 2
    }
  ]
}
```

### 3.3 Prompt模板

```markdown
你是一个代码提交分析专家。请分析以下文件变更列表，进行智能分类。

## 文件变更信息
{file_list_json}

## 分析任务

请按以下维度对文件进行分类：

### 1. 按变更类型分类
- 新增文件（added）
- 删除文件（deleted）
- 重命名/移动文件（renamed）
- 修改文件（modified）

### 2. 按文件性质分类
- 核心业务逻辑（business_logic）：如 service、controller、model 等
- 配置文件（configuration）：如 config、env、settings 等
- 测试文件（tests）：如 test、spec 文件
- 文档文件（documentation）：如 README、docs、注释
- 依赖/构建文件（dependencies）：如 package.json、requirements.txt
- UI/样式文件（ui_style）：如 css、scss、styled-components
- 基础设施（infrastructure）：如 Dockerfile、CI配置

### 3. 按修改规模分类
- 大改动（large）：单文件修改 >100 行
- 中等改动（medium）：单文件修改 20-100 行
- 小改动（small）：单文件修改 <20 行

### 4. 识别批量操作模式
分析是否存在以下模式：
- 批量重命名（mass_rename）
- 批量格式化（formatting）
- 统一配置更新（config_update）
- 依赖版本升级（dependency_upgrade）
- 导入路径调整（import_path_change）

### 5. 确定分析策略
根据以上分类，将文件分为：
- 批量处理组：可以合并分析的相似文件
- 重点分析组：需要详细分析的核心文件
- 可跳过组：不需要深入分析的文件（如自动生成文件）

## 输出格式

请严格按照以下JSON格式输出：

```json
{
  "categories": {
    "by_status": {
      "added": ["文件路径列表"],
      "deleted": ["文件路径列表"],
      "renamed": [
        {
          "old": "旧路径",
          "new": "新路径",
          "changes": 行数
        }
      ],
      "modified": ["文件路径列表"]
    },
    "by_nature": {
      "business_logic": ["文件列表"],
      "configuration": ["文件列表"],
      "tests": ["文件列表"],
      "documentation": ["文件列表"],
      "dependencies": ["文件列表"],
      "ui_style": ["文件列表"],
      "infrastructure": ["文件列表"]
    },
    "by_scale": {
      "large": ["文件列表"],
      "medium": ["文件列表"],
      "small": ["文件列表"]
    }
  },
  "patterns": {
    "mass_rename": {
      "detected": true/false,
      "pattern": "描述重命名模式，如：将 .js 统一改为 .ts",
      "affected_files": 10
    },
    "formatting": {
      "detected": true/false,
      "description": "代码格式化工具运行，如 Prettier"
    },
    "config_update": {
      "detected": true/false,
      "type": "环境变量更新 / API地址变更 / 特性开关等"
    },
    "dependency_upgrade": {
      "detected": true/false,
      "packages": ["升级的包名列表"]
    },
    "import_path_change": {
      "detected": true/false,
      "pattern": "导入路径调整模式"
    }
  },
  "analysis_strategy": {
    "批量处理组": ["可以合并分析的文件列表"],
    "重点分析组": ["需要详细分析的核心文件列表"],
    "可跳过组": ["自动生成或无需深入分析的文件"]
  },
  "summary": {
    "total_files": 25,
    "primary_change_type": "功能开发 / 重构 / 修复 / 配置更新",
    "complexity": "simple / moderate / complex"
  }
}
```
```

### 3.4 输出示例

```json
{
  "categories": {
    "by_status": {
      "added": ["src/utils/newHelper.ts", "tests/newHelper.test.ts"],
      "deleted": ["config/old_settings.json"],
      "renamed": [
        {
          "old": "src/services/apiService.ts",
          "new": "src/services/api.ts",
          "changes": 4
        }
      ],
      "modified": ["src/components/Header.tsx", "src/App.tsx"]
    },
    "by_nature": {
      "business_logic": ["src/components/Header.tsx", "src/utils/newHelper.ts"],
      "configuration": ["config/old_settings.json"],
      "tests": ["tests/newHelper.test.ts"],
      "documentation": [],
      "dependencies": [],
      "ui_style": [],
      "infrastructure": []
    },
    "by_scale": {
      "large": ["src/utils/newHelper.ts"],
      "medium": ["src/components/Header.tsx"],
      "small": ["src/services/api.ts", "src/App.tsx"]
    }
  },
  "patterns": {
    "mass_rename": {
      "detected": false,
      "pattern": "",
      "affected_files": 0
    },
    "formatting": {
      "detected": false,
      "description": ""
    },
    "config_update": {
      "detected": true,
      "type": "删除旧配置文件"
    }
  },
  "analysis_strategy": {
    "批量处理组": [],
    "重点分析组": ["src/components/Header.tsx", "src/utils/newHelper.ts"],
    "可跳过组": ["tests/newHelper.test.ts"]
  },
  "summary": {
    "total_files": 5,
    "primary_change_type": "功能开发",
    "complexity": "moderate"
  }
}
```

---

## 4. 阶段二：分类分析

### 4.1 目标

根据阶段一的分类结果，针对不同类型文件采用**差异化分析策略**。

### 4.2 分析策略矩阵

| 文件类型 | 分析深度 | Diff需求 | 批量处理 | 输出详细度 |
|---------|---------|---------|---------|-----------|
| 批量操作 | 浅层 | 抽样3-5个 | ✓ | 简要 |
| 核心逻辑 | 深度 | 完整diff | ✗ | 详细 |
| 配置文件 | 中等 | 完整diff | ✓ | 中等 |
| 测试文件 | 浅层 | 可选 | ✓ | 简要 |
| 文档 | 浅层 | 可选 | ✓ | 极简 |

### 4.3 Prompt 2.1：批量操作分析

**适用场景**：当阶段一检测到批量重命名、格式化、配置更新等模式

```markdown
你是代码提交分析专家。检测到以下批量操作，请进行分析。

## 批量操作信息
- 操作类型：{pattern_type}
- 涉及文件数：{count}
- 操作模式：{pattern_description}

## 样本文件Diff（前3个代表性文件）

### 文件1: {file_path_1}
变更：+{additions} -{deletions}
```diff
{diff_content_1}
```

### 文件2: {file_path_2}
变更：+{additions} -{deletions}
```diff
{diff_content_2}
```

### 文件3: {file_path_3}
变更：+{additions} -{deletions}
```diff
{diff_content_3}
```

## 分析任务

请分析：
1. 批量操作的统一目的是什么？
2. 所有文件的变更是否遵循一致模式？
3. 是否有例外文件需要特别说明？
4. 这个批量操作的影响和风险？

## 输出格式

```json
{
  "batch_summary": "一句话总结批量操作的目的",
  "common_changes": "所有文件共同的变更内容描述",
  "pattern_consistency": "high / medium / low",
  "exceptions": [
    {
      "file": "有特殊情况的文件路径",
      "reason": "为什么这个文件特殊"
    }
  ],
  "impact": {
    "scope": "影响范围：全局 / 模块级 / 局部",
    "breaking": false,
    "description": "具体影响说明"
  },
  "total_affected": 25
}
```

### 4.4 Prompt 2.2：核心逻辑分析

**适用场景**：业务代码、服务层、核心功能模块

```markdown
你是资深代码审查专家。请深入分析以下核心业务逻辑的修改。

## 修改文件列表

{for each file in business_logic_files}

### 文件：{file_path}
修改规模：+{additions} -{deletions}
文件类型：{file_type}

#### Diff内容：
```diff
{full_diff_content}
```

---
{end for}

## 分析任务

对每个文件分别进行分析：

1. **修改目的**：这个文件改动是为了实现什么功能或解决什么问题？
2. **关键变更点**：列出3-5个最重要的代码变更
3. **技术实现**：使用了什么技术方案或设计模式？
4. **影响范围**：
   - 是否影响API接口？
   - 是否影响数据库？
   - 是否影响其他模块？
5. **关联性**：与其他修改的文件有什么关联？
6. **风险评估**：潜在的bug风险或性能影响

## 输出格式

```json
{
  "files": [
    {
      "file": "文件路径",
      "purpose": "修改目的的简要描述",
      "key_changes": [
        "变更点1：具体描述",
        "变更点2：具体描述",
        "变更点3：具体描述"
      ],
      "technical_approach": "使用的技术方案",
      "impact_scope": {
        "api_changes": true/false,
        "database_changes": true/false,
        "module_dependencies": ["受影响的其他模块"],
        "description": "影响范围详细说明"
      },
      "related_files": [
        {
          "file": "关联文件路径",
          "relationship": "关系说明"
        }
      ],
      "risk_assessment": {
        "level": "low / medium / high",
        "concerns": ["潜在问题1", "潜在问题2"],
        "recommendations": ["建议1", "建议2"]
      }
    }
  ]
}
```


### 4.5 Prompt 2.3：配置文档分析

**适用场景**：配置文件、环境变量、文档更新

```markdown
你是配置管理专家。请分析以下配置或文档类文件的修改。

## 修改文件

{for each file in config_or_doc_files}

### {file_path}
变更：+{additions} -{deletions}

```diff
{diff_content}
```

---
{end for}

## 分析任务

这类文件通常改动目的明确，请简要总结：

1. 配置项变更的具体内容
2. 变更的原因和影响
3. 是否需要配套的代码或环境调整

## 输出格式

```json
{
  "config_changes": [
    {
      "file": "文件路径",
      "change_type": "新增配置 / 修改配置 / 删除配置",
      "items": [
        {
          "key": "配置项名称",
          "old_value": "旧值",
          "new_value": "新值",
          "purpose": "变更原因"
        }
      ]
    }
  ],
  "doc_updates": [
    {
      "file": "文档路径",
      "update_type": "新增章节 / 更新内容 / 修正错误",
      "summary": "更新内容概要"
    }
  ],
  "deployment_notes": "部署时需要注意的事项（如果有）"
}
```

### 4.6 Prompt 2.4：测试文件分析

**适用场景**：单元测试、集成测试文件

```markdown
你是测试专家。请分析测试文件的变更。

## 测试文件变更

{test_files_diff}

## 分析任务

1. 新增了哪些测试用例？
2. 修改或删除了哪些测试？
3. 测试覆盖的功能模块是什么？
4. 测试变更与业务代码变更的对应关系

## 输出格式

```json
{
  "test_summary": {
    "new_tests": ["新测试用例描述"],
    "modified_tests": ["修改的测试描述"],
    "deleted_tests": ["删除的测试描述"],
    "coverage_modules": ["覆盖的功能模块"]
  },
  "alignment_with_code": "测试变更与代码变更的匹配度：good / partial / poor"
}
```


---

## 5. 阶段三：全局总结

### 5.1 目标

综合前两个阶段的分析结果，生成**结构化的commit总结**，包括标题、详细描述、影响分析等。

### 5.2 Prompt模板

```markdown
你是Git提交信息专家。请基于前两个阶段的分析结果，生成完整的commit总结。

## 输入信息

### 阶段一：文件分类结果
```json
{stage1_classification}
```

### 阶段二：详细分析结果

#### 批量操作分析
```json
{stage2_batch_analysis}
```

#### 核心逻辑分析
```json
{stage2_logic_analysis}
```

#### 配置文档分析
```json
{stage2_config_analysis}
```

#### 测试分析
```json
{stage2_test_analysis}
```

### 统计信息
- 总文件数：{total_files}
- 新增：{added_count} 个
- 删除：{deleted_count} 个
- 修改：{modified_count} 个
- 重命名：{renamed_count} 个
- 代码行变化：+{total_additions} -{total_deletions}

## 输出要求

请生成符合以下规范的commit总结：

### 1. Commit标题
- 遵循 Conventional Commits 规范
- 格式：`<type>(<scope>): <subject>`
- 长度：不超过50个字符
- type可选：feat, fix, refactor, docs, style, test, chore, perf
- subject使用动词开头，首字母小写

### 2. Commit描述
- 简洁说明修改的主要目的（Why）
- 列出关键变更点（What）
- 说明技术方案或实现方式（How）

### 3. 影响分析
- Breaking Changes（如果有）
- 受影响的模块
- 风险评估
- 测试建议

## 输出格式

```json
{
  "commit_message": {
    "title": "feat(user-auth): add OAuth2.0 login support",
    "body": "完整的commit message主体内容，包含多行描述",
    "footer": "BREAKING CHANGE: 描述（如果有）\nCloses #123"
  },

  "structured_summary": {
    "type": "feat",
    "scope": "user-auth",
    "subject": "add OAuth2.0 login support",

    "main_purpose": "本次提交的核心目的（1-2句话）",

    "key_changes": [
      "关键变更1",
      "关键变更2",
      "关键变更3"
    ],

    "details_by_category": {
      "features": ["新增的功能列表"],
      "fixes": ["修复的问题列表"],
      "refactors": ["重构内容"],
      "config": ["配置变更"],
      "docs": ["文档更新"],
      "tests": ["测试变更"],
      "others": ["其他变更"]
    }
  },

  "impact_analysis": {
    "breaking_changes": {
      "has_breaking": true/false,
      "description": "破坏性变更的详细说明",
      "migration_guide": "迁移指南（如果需要）"
    },

    "affected_modules": [
      {
        "module": "模块名称",
        "impact": "影响描述",
        "severity": "low / medium / high"
      }
    ],

    "risk_assessment": {
      "overall_risk": "low / medium / high",
      "risk_factors": ["风险因素1", "风险因素2"],
      "mitigation": ["缓解措施1", "缓解措施2"]
    },

    "testing_suggestions": [
      "建议的测试重点1",
      "建议的测试重点2"
    ]
  },

  "statistics": {
    "total_files": 25,
    "additions": 450,
    "deletions": 120,
    "net_change": 330,
    "file_breakdown": {
      "added": 5,
      "modified": 18,
      "deleted": 2,
      "renamed": 0
    }
  },

  "metadata": {
    "complexity": "simple / moderate / complex",
    "review_priority": "low / medium / high",
    "estimated_review_time": "15 minutes",
    "tags": ["feature", "authentication", "breaking-change"]
  }
}
```


### 5.3 输出示例

```json
{
  "commit_message": {
    "title": "feat(auth): implement OAuth2.0 authentication flow",
    "body": "实现了完整的OAuth2.0认证流程，支持Google和GitHub登录。\n\n主要变更：\n- 新增OAuth2.0服务层，处理授权码流程\n- 重构用户认证中间件，支持多种登录方式\n- 更新API路由，添加OAuth回调端点\n- 配置OAuth客户端ID和密钥管理\n\n技术实现：\n- 使用passport.js作为认证框架\n- JWT token用于会话管理\n- Redis缓存OAuth状态参数",
    "footer": "Closes #234\nReviewed-by: @tech-lead"
  },

  "structured_summary": {
    "type": "feat",
    "scope": "auth",
    "subject": "implement OAuth2.0 authentication flow",
    "main_purpose": "为应用添加社交账号登录功能，提升用户注册和登录体验",
    "key_changes": [
      "新增OAuth2.0服务层，支持Google和GitHub第三方登录",
      "重构认证中间件，统一多种登录方式的处理逻辑",
      "添加OAuth回调路由和状态管理机制",
      "更新用户数据模型，支持关联多个登录方式"
    ],
    "details_by_category": {
      "features": [
        "OAuth2.0 Google登录",
        "OAuth2.0 GitHub登录",
        "多账号关联功能"
      ],
      "refactors": [
        "认证中间件重构为策略模式",
        "用户服务层解耦"
      ],
      "config": [
        "添加OAuth客户端配置",
        "更新环境变量模板"
      ],
      "tests": [
        "OAuth流程集成测试",
        "认证中间件单元测试"
      ]
    }
  },

  "impact_analysis": {
    "breaking_changes": {
      "has_breaking": false,
      "description": "",
      "migration_guide": ""
    },
    "affected_modules": [
      {
        "module": "用户认证模块",
        "impact": "新增OAuth登录方式，原有登录方式保持兼容",
        "severity": "medium"
      },
      {
        "module": "用户数据模型",
        "impact": "扩展字段以支持OAuth provider信息",
        "severity": "low"
      }
    ],
    "risk_assessment": {
      "overall_risk": "medium",
      "risk_factors": [
        "OAuth第三方服务依赖性",
        "用户数据模型变更需要数据库迁移"
      ],
      "mitigation": [
        "添加OAuth服务降级机制",
        "提供详细的数据库迁移脚本和回滚方案"
      ]
    },
    "testing_suggestions": [
      "测试OAuth授权码流程的完整性",
      "验证多账号关联和解绑功能",
      "测试原有登录方式的兼容性",
      "进行安全性测试，防止CSRF和状态劫持"
    ]
  },

  "statistics": {
    "total_files": 18,
    "additions": 856,
    "deletions": 123,
    "net_change": 733,
    "file_breakdown": {
      "added": 6,
      "modified": 12,
      "deleted": 0,
      "renamed": 0
    }
  },

  "metadata": {
    "complexity": "complex",
    "review_priority": "high",
    "estimated_review_time": "45 minutes",
    "tags": ["feature", "authentication", "oauth", "security"]
  }
}
```

---

## 6. 优化策略

### 6.1 Token优化

| 优化点 | 方法 | 节省比例 |
|--------|------|---------|
| 阶段1输入 | 只传元数据，不传diff | ~90% |
| 批量操作 | 抽样3-5个文件代替全部 | ~80% |
| 小改动文件 | 合并到一次调用 | ~70% |
| 测试文件 | 简化分析或跳过 | ~50% |
| 自动生成文件 | 直接跳过 | 100% |

### 6.2 并发优化

### 6.3 缓存策略

**适用场景**：
- 重复分析同一commit
- CI/CD中多次触发
- 团队成员查看相同提交

### 6.4 成本控制

### 6.5 渐进式展示

---

## 7. 最佳实践

### 7.1 Prompt工程技巧

1. **结构化输出**：始终要求JSON格式，便于解析
2. **明确边界**：清晰定义输入输出格式
3. **提供示例**：在prompt中包含好的输出示例
4. **分层指令**：用标题和编号组织复杂指令
5. **错误处理**：处理JSON解析失败的情况

### 7.2 质量保证

### 9.3 人工审核集成

### 9.4 CI/CD集成示例

---
