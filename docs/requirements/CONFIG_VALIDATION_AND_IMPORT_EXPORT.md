# 配置验证与导入/导出需求文档

## 📋 需求概述

本文档描述配置管理模块中的两个核心功能需求：
1. **配置文件验证** - 验证配置文件的完整性和有效性
2. **配置导入/导出** - 支持配置文件的备份、恢复和迁移

---

## 1. 配置文件验证需求

### 1.1 功能描述

实现 `config validate` 命令，用于验证配置文件的完整性和有效性。该功能应能够：
- 验证配置文件格式（TOML/JSON/YAML）
- 检查必需字段是否存在
- 验证字段类型和值的有效性
- 检查引用关系的正确性（如 JIRA 项目是否存在）
- 提供自动修复功能（可选）

### 1.2 命令规范

**命令格式**：
```bash
workflow config validate [OPTIONS]
```

**选项**：
- `--fix` - 自动修复配置错误
- `--strict` - 严格模式（所有警告视为错误）

**命令示例**：
```bash
workflow config validate                           # 验证配置
workflow config validate --fix                     # 自动修复
workflow config validate --strict                  # 严格模式（所有警告视为错误）
```

### 1.3 验证项清单

#### 1.3.1 格式验证
- [ ] 配置文件格式正确（TOML/JSON/YAML）
- [ ] 文件可正常解析
- [ ] 无语法错误

#### 1.3.2 必需字段验证
- [ ] 所有必需字段存在
- [ ] 字段名称拼写正确
- [ ] 字段层级结构正确

#### 1.3.3 字段类型验证
- [ ] 字段类型匹配预期类型（字符串、数字、布尔值等）
- [ ] 数组和对象结构正确
- [ ] 嵌套结构有效

#### 1.3.4 字段值验证
- [ ] 字段值在有效范围内
- [ ] URL 格式正确
- [ ] 枚举值有效
- [ ] 数值范围合理

#### 1.3.5 引用关系验证
- [ ] JIRA 项目是否存在（如果配置了 JIRA）
- [ ] 引用的其他配置项有效
- [ ] 依赖关系正确

### 1.4 输出规范

#### 1.4.1 验证成功
```bash
$ workflow config validate
✓ Configuration file is valid
```

#### 1.4.2 验证失败（无修复）
```bash
$ workflow config validate
✗ Configuration validation failed

Errors:
  - Missing required field: 'jira.project'
  - Invalid URL format: 'jira.url' = "not-a-url"
  - Invalid value: 'pr.platform' = "invalid" (expected: github, codeup, gitlab)

Warnings:
  - Deprecated field: 'old_field' (use 'new_field' instead)

Run 'workflow config validate --fix' to attempt automatic fixes.
```

#### 1.4.3 自动修复
```bash
$ workflow config validate --fix
⚠ Found 2 issues, fixed automatically:
  - Added missing 'jira.project' field
  - Updated 'pr.platform' from 'github' to 'codeup'
✓ Configuration file is now valid
```

### 1.5 实现建议

#### 1.5.1 技术方案
- 使用 `serde` 进行配置解析和验证
- 使用 `validator` crate 进行字段验证
- 实现自定义验证逻辑处理复杂场景

#### 1.5.2 自动修复策略
- 修复常见拼写错误
- 添加缺失的默认值
- 更新已弃用的字段名
- 修正明显的格式错误

#### 1.5.3 错误处理
- 提供详细的错误信息
- 给出修复建议
- 支持错误定位（文件路径、行号）

---

## 2. 配置导入/导出需求

### 2.1 功能描述

实现配置文件的导入和导出功能，支持：
- 配置文件的备份和恢复
- 配置在不同环境间的迁移
- 选择性导出特定配置段
- 敏感信息过滤

### 2.2 导出功能需求

#### 2.2.1 命令规范

**命令格式**：
```bash
workflow config export <OUTPUT_PATH> [OPTIONS]
```

**选项**：
- `--section <SECTION>` - 只导出特定配置段（如 `jira`、`pr`）
- `--no-secrets` - 排除敏感信息
- `--format <FORMAT>` - 导出格式（toml、json、yaml，默认：toml）

**命令示例**：
```bash
workflow config export config.backup.toml                    # 导出配置
workflow config export config.backup.toml --section jira      # 只导出 JIRA 配置
workflow config export config.backup.toml --no-secrets        # 排除敏感信息
workflow config export config.backup.json --format json       # 导出为 JSON
```

#### 2.2.2 功能清单

- [ ] 导出完整配置文件
- [ ] 支持选择性导出（按配置段）
- [ ] 支持多种格式（TOML、JSON、YAML）
- [ ] 自动过滤敏感信息（API tokens、密码等）
- [ ] 保留配置文件的原始结构
- [ ] 导出前验证配置有效性

#### 2.2.3 敏感信息识别

需要自动识别并过滤的敏感信息：
- [ ] API tokens（`jira.token`、`github.token` 等）
- [ ] 密码字段（`password`、`passwd` 等）
- [ ] 密钥字段（`secret`、`key`、`api_key` 等）
- [ ] 认证信息（`auth`、`credentials` 等）

#### 2.2.4 输出示例

```bash
$ workflow config export config.backup.toml --no-secrets
✓ Configuration exported to config.backup.toml
⚠ Sensitive information has been filtered (3 fields)

$ workflow config export config.backup.toml --section jira
✓ JIRA configuration exported to config.backup.toml
```

### 2.3 导入功能需求

#### 2.3.1 命令规范

**命令格式**：
```bash
workflow config import <INPUT_PATH> [OPTIONS]
```

**选项**：
- `--overwrite` - 覆盖模式（完全替换现有配置）
- `--section <SECTION>` - 只导入特定配置段
- `--dry-run` - 试运行（不实际修改配置）

**命令示例**：
```bash
workflow config import config.backup.toml                    # 导入配置（合并模式）
workflow config import config.backup.toml --overwrite         # 覆盖模式
workflow config import config.backup.toml --section jira      # 只导入 JIRA 配置
workflow config import config.backup.toml --dry-run           # 试运行
```

#### 2.3.2 功能清单

- [ ] 导入配置文件
- [ ] 支持多种格式（TOML、JSON、YAML）
- [ ] 合并模式（保留现有配置，只更新导入的部分）
- [ ] 覆盖模式（完全替换现有配置）
- [ ] 导入前自动验证配置有效性
- [ ] 导入前创建备份
- [ ] 支持选择性导入（按配置段）
- [ ] 试运行模式（预览变更）

#### 2.3.3 导入流程

1. **验证阶段**
   - [ ] 检查输入文件是否存在
   - [ ] 验证文件格式
   - [ ] 解析配置文件
   - [ ] 验证配置有效性

2. **备份阶段**
   - [ ] 创建当前配置的备份
   - [ ] 备份文件命名：`config.backup.<timestamp>.toml`

3. **导入阶段**
   - [ ] 根据模式（合并/覆盖）执行导入
   - [ ] 合并模式：深度合并配置
   - [ ] 覆盖模式：完全替换配置

4. **验证阶段**
   - [ ] 验证导入后的配置
   - [ ] 如有错误，自动恢复备份

#### 2.3.4 输出示例

**合并模式**：
```bash
$ workflow config import config.backup.toml
✓ Configuration backup created: config.backup.20250127_143022.toml
✓ Configuration imported successfully (merge mode)
  - Updated: jira.project
  - Added: pr.template_path
  - Preserved: github.token
```

**覆盖模式**：
```bash
$ workflow config import config.backup.toml --overwrite
⚠ This will replace your current configuration
✓ Configuration backup created: config.backup.20250127_143022.toml
✓ Configuration imported successfully (overwrite mode)
```

**验证失败**：
```bash
$ workflow config import config.backup.toml
✗ Configuration validation failed
  - Missing required field: 'jira.url'
  - Invalid value: 'pr.platform' = "invalid"
✗ Import cancelled. Original configuration preserved.
```

---

## 3. 实现优先级

### 高优先级
1. **配置文件验证** (`config validate`)
   - 基础验证功能
   - 错误报告

### 中优先级
2. **配置导出** (`config export`)
   - 基础导出功能
   - 敏感信息过滤

3. **配置导入** (`config import`)
   - 基础导入功能（合并模式）
   - 配置验证

### 低优先级
4. **高级功能**
   - 自动修复功能
   - 选择性导入/导出
   - 试运行模式

---

## 4. 技术实现建议

### 4.1 配置验证实现

```rust
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct Config {
    #[validate]
    pub jira: Option<JiraConfig>,
    #[validate]
    pub pr: Option<PrConfig>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct JiraConfig {
    #[validate(url)]
    pub url: String,
    #[validate(length(min = 1))]
    pub project: String,
}

pub fn validate_config(config: &Config) -> Result<(), Vec<ValidationError>> {
    config.validate()
}
```

### 4.2 配置导入/导出实现

```rust
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn export(&self, output_path: &Path, options: ExportOptions) -> Result<()> {
        let config = self.load_config()?;
        let exported = if options.no_secrets {
            self.filter_secrets(config)
        } else {
            config
        };

        if let Some(section) = options.section {
            let section_config = self.extract_section(&exported, section)?;
            self.save_config(&section_config, output_path)?;
        } else {
            self.save_config(&exported, output_path)?;
        }

        Ok(())
    }

    pub fn import(&self, input_path: &Path, options: ImportOptions) -> Result<()> {
        let imported = self.load_config_from(input_path)?;
        self.validate_config(&imported)?;

        // 创建备份
        let backup_path = self.create_backup()?;

        if options.overwrite {
            self.save_config(&imported, &self.config_path)?;
        } else {
            let current = self.load_config()?;
            let merged = self.merge_configs(current, imported);
            self.save_config(&merged, &self.config_path)?;
        }

        Ok(())
    }
}
```

---

## 5. 验收标准

### 5.1 配置验证验收标准

- [ ] `config validate` 命令可以正确识别配置错误
- [ ] 错误信息清晰、可操作
- [ ] `--fix` 选项可以自动修复常见错误
- [ ] `--strict` 选项将所有警告视为错误
- [ ] 验证覆盖所有必需字段
- [ ] 验证覆盖字段类型和值范围

### 5.2 配置导出验收标准

- [ ] `config export` 可以导出完整配置
- [ ] 支持导出为 TOML、JSON、YAML 格式
- [ ] `--section` 选项可以只导出特定配置段
- [ ] `--no-secrets` 选项正确过滤敏感信息
- [ ] 导出的配置文件可以正常导入

### 5.3 配置导入验收标准

- [ ] `config import` 可以正确导入配置
- [ ] 合并模式保留现有配置并更新导入部分
- [ ] 覆盖模式完全替换现有配置
- [ ] 导入前自动验证配置有效性
- [ ] 导入前自动创建备份
- [ ] 验证失败时自动恢复备份
- [ ] `--section` 选项可以只导入特定配置段
- [ ] `--dry-run` 选项可以预览变更

---

## 6. 相关文档

- [配置管理待办事项](../todo/CONFIG_TODO.md)
- [配置命令架构文档](../architecture/commands/CONFIG_COMMAND_ARCHITECTURE.md)

---

**创建日期**: 2025-01-27
**状态**: 📋 需求分析中
**优先级**: 高优先级（配置验证）、中优先级（导入/导出）
