# 配置管理待办事项

## 📋 概述

本文档列出配置管理相关的待办功能，包括配置文件验证、配置导入/导出和多环境支持。

---

## ❌ 待实现功能

### 1. 配置文件验证

#### 1.1 `config validate` - 配置文件验证
- ❌ 验证配置文件完整性
- ❌ 验证配置项的有效性
- ❌ 自动修复配置错误（可选）

**功能**：验证配置文件的完整性和有效性。

**命令示例**：
```bash
workflow config validate                           # 验证配置
workflow config validate --fix                     # 自动修复
workflow config validate --strict                  # 严格模式（所有警告视为错误）
```

**验证项**：
- 配置文件格式（TOML/JSON/YAML）
- 必需字段是否存在
- 字段类型是否正确
- 字段值是否在有效范围内
- 引用关系是否正确（如 JIRA 项目是否存在）

**实现建议**：
- 使用 `serde` 进行配置验证
- 提供详细的错误信息和建议
- 支持自动修复常见错误

**输出示例**：
```bash
$ workflow config validate
✓ Configuration file is valid

$ workflow config validate --fix
⚠ Found 2 issues, fixed automatically:
  - Added missing 'jira.project' field
  - Updated 'pr.platform' from 'github' to 'codeup'
✓ Configuration file is now valid
```

---

### 2. 配置导入/导出

#### 2.1 `config export` - 导出配置
- ❌ 导出配置文件
- ❌ 支持选择性导出（只导出特定部分）
- ❌ 支持敏感信息过滤

**功能**：备份配置文件。

**命令示例**：
```bash
workflow config export config.backup.toml          # 导出配置
workflow config export config.backup.toml --section jira  # 只导出 JIRA 配置
workflow config export config.backup.toml --no-secrets  # 排除敏感信息
```

**实现建议**：
- 支持导出为 TOML、JSON、YAML 格式
- 支持选择性导出（只导出特定配置段）
- 自动过滤敏感信息（如 API tokens、密码等）

#### 2.2 `config import` - 导入配置
- ❌ 导入配置文件
- ❌ 支持合并模式（合并到现有配置）
- ❌ 支持覆盖模式（完全替换）

**功能**：恢复或合并配置文件。

**命令示例**：
```bash
workflow config import config.backup.toml          # 导入配置（合并模式）
workflow config import config.backup.toml --overwrite  # 覆盖模式
workflow config import config.backup.toml --section jira  # 只导入 JIRA 配置
```

**实现建议**：
- 支持导入 TOML、JSON、YAML 格式
- 支持合并模式（保留现有配置，只更新导入的部分）
- 支持覆盖模式（完全替换现有配置）
- 导入前自动验证配置有效性
- 导入前创建备份

---

### 3. 多环境支持

#### 3.1 多环境配置
- ❌ 开发/测试/生产环境配置
- ❌ 环境变量覆盖
- ❌ 配置文件继承

**功能**：支持不同环境的配置管理。

**实现建议**：
- 支持环境变量覆盖配置
- 支持配置文件继承（base 配置 + 环境特定配置）
- 支持环境切换命令

**配置结构示例**：
```toml
# config.toml (base config)
[jira]
url = "https://jira.example.com"
project = "PROJ"

# config.dev.toml (development)
[jira]
url = "https://jira-dev.example.com"

# config.prod.toml (production)
[jira]
url = "https://jira.example.com"
```

**命令示例**：
```bash
workflow config env set dev                       # 切换到开发环境
workflow config env set prod                      # 切换到生产环境
workflow config env show                           # 显示当前环境
```

**环境变量覆盖示例**：
```bash
# 使用环境变量覆盖配置
export WORKFLOW_JIRA_URL="https://jira-custom.example.com"
export WORKFLOW_JIRA_PROJECT="CUSTOM"
workflow jira info PROJ-123
```

---

## 📊 优先级

### 高优先级
1. **配置文件验证**
   - `config validate` - 验证配置文件

### 中优先级
1. **配置导入/导出**
   - `config export` - 导出配置
   - `config import` - 导入配置

2. **多环境支持**
   - 环境变量覆盖
   - 配置文件继承

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：配置文件验证
   - `config validate` - 验证配置文件
   - 提供详细的错误信息和建议

2. **第二阶段**：配置导入/导出
   - `config export` - 导出配置
   - `config import` - 导入配置

3. **第三阶段**：多环境支持
   - 环境变量覆盖
   - 配置文件继承
   - 环境切换命令

### 技术考虑
1. **配置验证**：使用 `serde` 和自定义验证逻辑
2. **配置格式**：支持 TOML、JSON、YAML
3. **敏感信息**：自动识别和过滤敏感信息
4. **错误处理**：提供清晰的错误信息和建议
5. **测试**：为新功能添加单元测试和集成测试
6. **文档**：及时更新文档和示例

### 实现细节

#### 配置验证实现
```rust
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct Config {
    #[validate]
    pub jira: JiraConfig,
    #[validate]
    pub pr: PrConfig,
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

#### 配置导入/导出实现
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
        self.save_config(&exported, output_path)?;
        Ok(())
    }

    pub fn import(&self, input_path: &Path, options: ImportOptions) -> Result<()> {
        let imported = self.load_config_from(input_path)?;
        self.validate_config(&imported)?;

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

## 📚 相关文档

- [功能拓展分析文档](./FEATURE_EXTENSIONS.md)

---

**最后更新**: 2024-12-19
