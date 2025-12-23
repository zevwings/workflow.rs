# 代码审查指南

> 专为 AI 助手设计的代码检查指南，提供系统化的代码质量和复用性检查方法，识别重复代码和优化机会。

## 🎯 核心原则

**检查重点**：识别代码重复、工具复用和优化机会。

**关键目标**：
- ✅ 消除重复：识别并抽取重复代码为公共方法
- ✅ 工具复用：使用已封装的工具函数替换重复实现
- ✅ 依赖优化：识别可用第三方库替换的自实现代码
- ✅ 规范遵循：确保代码风格和导入顺序符合项目规范
- ✅ **避免过度设计**：保持代码简洁，避免不必要的抽象和复杂性

---

## 📋 目录

- [检查目标](#-检查目标)
- [检查流程](#-检查流程)
- [重复代码检查](#-重复代码检查)
- [已封装工具检查](#-已封装工具检查)
- [第三方工具检查](#-第三方工具检查)
- [检查清单](#-检查清单)
- [示例分析](#-示例分析)

---

## 🎯 检查目标

代码检查的主要目标：

1. **消除重复代码**：识别并抽取重复的代码模式为公共方法
2. **复用已有工具**：识别可以使用已封装工具函数替换的代码
3. **优化依赖使用**：识别可以使用第三方库替换的自实现代码
4. **规范代码风格**：确保代码遵循项目规范，包括导入顺序等
5. **避免过度设计**：保持代码简洁实用，避免不必要的抽象层和复杂性

---

## 🔄 检查流程

### 步骤 1：扫描代码库

1. **全量扫描**：使用 `grep` 或语义搜索工具扫描 `src/` 目录
2. **分类统计**：按模块、功能分类统计代码模式
3. **模式识别**：识别常见的代码模式和重复片段

### 步骤 2：重复代码分析

1. **相似度检测**：查找结构相似、逻辑重复的代码块
2. **参数化分析**：分析重复代码的差异点，确定可参数化的部分
3. **抽取建议**：提出公共方法抽取方案

### 步骤 3：工具函数检查

1. **工具函数清单**：列出所有已封装的工具函数
2. **使用情况分析**：检查是否有代码可以直接使用这些工具
3. **替换建议**：提出使用工具函数替换的方案

### 步骤 4：第三方库检查

1. **功能识别**：识别自实现的功能
2. **库匹配**：查找是否有成熟的第三方库可以替换
3. **替换评估**：评估替换的可行性和收益

### 步骤 5：过度设计检查

1. **抽象层评估**：检查是否有不必要的抽象层或中间层
2. **复杂度评估**：评估代码复杂度是否与实际需求匹配
3. **实用性评估**：确保设计满足当前需求，避免为未来可能的需求过度设计

### 步骤 6：生成报告

1. **问题汇总**：汇总所有发现的问题
2. **优先级排序**：按影响范围和收益排序
3. **改进建议**：提供具体的改进方案

---

## 🔍 重复代码检查

### 检查方法

#### 1. 文件操作模式

**检查目标**：查找重复的文件读写操作

**搜索模式**：
```bash
# 查找文件读取操作
grep -r "fs::read-_to-_string" src/
grep -r "File::open" src/
grep -r "BufReader::new" src/

# 查找文件写入操作
grep -r "fs::write" src/
grep -r "fs::create-_dir" src/
grep -r "fs::create-_dir-_all" src/
```

**常见重复模式**：
- 文件读取 + 错误处理
- 文件写入 + 目录创建
- 文件打开 + BufReader 创建

**已封装工具**：
- `lib/base/util/file.rs` - `FileReader::open()` - 文件打开和 BufReader 创建
- 建议封装：`read-_file-_with-_context()`, `write-_file-_with-_context()`

**检查清单**：
- [ ] 是否有多个地方使用相同的文件读取模式？
- [ ] 错误处理是否一致？
- [ ] 是否可以抽取为公共函数？

#### 2. Git 命令执行模式

**检查目标**：查找重复的 Git 命令执行代码

**搜索模式**：
```bash
# 查找 Git 命令执行
grep -r "cmd(\"git\"" src/
grep -r "duct::cmd" src/
```

**常见重复模式**：
- Git 命令执行 + 输出读取
- Git 命令执行 + 错误处理
- Git 命令执行 + 静默模式

**已封装工具**：
- `lib/git/helpers.rs`：
  - `cmd-_read()` - 执行 Git 命令并读取输出
  - `cmd-_run()` - 执行 Git 命令（不读取输出）
  - `check-_success()` - 静默执行并检查成功
  - `check-_ref-_exists()` - 检查 Git 引用是否存在
  - `switch-_or-_checkout()` - 尝试 switch，失败时回退到 checkout

**检查清单**：
- [ ] 是否直接使用 `cmd("git", ...)` 而不是 `cmd-_read()` 或 `cmd-_run()`？
- [ ] 是否有重复的错误处理逻辑？
- [ ] 是否可以复用 `git/helpers.rs` 中的函数？

#### 3. 错误处理模式

**检查目标**：查找重复的错误处理代码

**搜索模式**：
```bash
# 查找错误处理模式
grep -r "\.wrap-_err" src/
grep -r "\.context" src/
grep -r "\.with-_context" src/
grep -r "bail!" src/
grep -r "ensure!" src/
```

**常见重复模式**：
- 相同的错误消息格式
- 相同的上下文添加方式
- 相同的错误转换逻辑

**已封装工具**：
- `anyhow::Context` - 错误上下文添加
- `anyhow::bail!` - 快速返回错误
- `anyhow::ensure!` - 断言式错误

**检查清单**：
- [ ] 是否有重复的错误消息？
- [ ] 错误上下文是否一致？
- [ ] 是否可以统一错误处理函数？

#### 4. 字符串处理模式

**检查目标**：查找重复的字符串处理代码

**搜索模式**：
```bash
# 查找字符串处理
grep -r "\.trim()" src/
grep -r "\.to-_string()" src/
grep -r "\.to-_lowercase()" src/
grep -r "format!" src/
```

**常见重复模式**：
- 字符串清理和规范化
- 敏感值隐藏
- 字符串格式化

**已封装工具**：
- `lib/base/util/string.rs`：
  - `mask-_sensitive-_value()` - 隐藏敏感值

**检查清单**：
- [ ] 是否有重复的字符串处理逻辑？
- [ ] 是否可以使用 `mask-_sensitive-_value()` 隐藏敏感信息？
- [ ] 是否可以抽取为公共函数？

#### 5. 路径处理模式

**检查目标**：查找重复的路径处理代码

**搜索模式**：
```bash
# 查找路径处理
grep -r "Path::new" src/
grep -r "\.parent()" src/
grep -r "\.join(" src/
grep -r "\.exists()" src/
```

**常见重复模式**：
- 路径拼接和规范化
- 父目录获取和创建
- 路径存在性检查

**检查清单**：
- [ ] 是否有重复的路径处理逻辑？
- [ ] 是否可以抽取为公共函数？

#### 6. HTTP 请求模式

**检查目标**：查找重复的 HTTP 请求代码

**搜索模式**：
```bash
# 查找 HTTP 请求
grep -r "HttpClient::new" src/
grep -r "\.get(" src/
grep -r "\.post(" src/
grep -r "reqwest" src/
```

**常见重复模式**：
- HTTP 客户端创建
- 请求构建和发送
- 响应处理和错误处理

**已封装工具**：
- `lib/base/http/client.rs` - `HttpClient` - 统一的 HTTP 客户端
- `lib/base/http/retry.rs` - `HttpRetry` - HTTP 重试机制

**检查清单**：
- [ ] 是否直接使用 `reqwest` 而不是 `HttpClient`？
- [ ] 是否有重复的请求构建逻辑？
- [ ] 是否可以使用 `HttpRetry` 处理重试？

#### 7. 配置读取模式

**检查目标**：查找重复的配置读取代码

**搜索模式**：
```bash
# 查找配置读取
grep -r "Settings::get" src/
grep -r "Settings::load" src/
grep -r "toml::from-_str" src/
```

**常见重复模式**：
- 配置文件读取和解析
- 配置验证
- 默认值处理

**已封装工具**：
- `lib/base/settings/settings.rs` - `Settings` - 统一配置管理

**检查清单**：
- [ ] 是否直接读取配置文件而不是使用 `Settings`？
- [ ] 是否有重复的配置解析逻辑？
- [ ] 是否可以使用 `Settings` API？

#### 8. 日志输出模式

**检查目标**：查找重复的日志输出代码

**搜索模式**：
```bash
# 查找日志输出
grep -r "println!" src/
grep -r "eprintln!" src/
grep -r "info!" src/
grep -r "error!" src/
```

**常见重复模式**：
- 直接使用 `println!` 而不是日志宏
- 重复的日志格式
- 不一致的日志级别

**已封装工具**：
- `lib/base/logger/console.rs`：
  - `info!()` - 信息日志
  - `error!()` - 错误日志
  - `warning!()` - 警告日志
  - `success!()` - 成功日志
  - `debug!()` - 调试日志
  - `separator!()` - 分隔符
  - `separator-_with-_text!()` - 带文本的分隔符

**检查清单**：
- [ ] 是否使用 `println!` 而不是日志宏？
- [ ] 日志格式是否一致？
- [ ] 是否可以使用统一的日志宏？

#### 9. 用户交互模式

**检查目标**：查找重复的用户交互代码

**搜索模式**：
```bash
# 查找用户交互
grep -r "Input::new" src/
grep -r "Select::new" src/
grep -r "Confirm::new" src/
grep -r "dialoguer" src/
```

**常见重复模式**：
- 重复的对话框配置
- 重复的用户输入验证
- 重复的选项处理

**已封装工具**：
- `lib/base/dialog/`：
  - `InputDialog` - 输入对话框
  - `SelectDialog` - 选择对话框
  - `MultiSelectDialog` - 多选对话框
  - `ConfirmDialog` - 确认对话框

**检查清单**：
- [ ] 是否直接使用 `dialoguer` 而不是封装的 Dialog？
- [ ] 是否有重复的对话框配置？
- [ ] 是否可以使用封装的 Dialog 类型？

#### 10. 进度指示器模式

**检查目标**：查找重复的进度指示器代码

**搜索模式**：
```bash
# 查找进度指示器
grep -r "Spinner::new" src/
grep -r "Progress::new" src/
grep -r "indicatif" src/
```

**常见重复模式**：
- 重复的 Spinner 配置
- 重复的 Progress 配置
- 重复的样式设置

**已封装工具**：
- `lib/base/indicator/`：
  - `Spinner` - 旋转指示器
  - `Progress` - 进度条

**检查清单**：
- [ ] 是否直接使用 `indicatif` 而不是封装的 Spinner/Progress？
- [ ] 是否有重复的样式配置？
- [ ] 是否可以使用封装的指示器？

#### 11. 导入顺序模式

**检查目标**：检查导入语句是否遵循从顶部导入的规范

**导入顺序规范**：
1. **标准库导入**：`std::` 和 `core::` 相关导入
2. **第三方库导入**：外部 crate 的导入
3. **项目内部导入**：`crate::` 或相对路径导入
4. **每个分组之间用空行分隔**

**搜索模式**：
```bash
# 查找文件中的导入语句
grep -r "^use " src/ -A 0 -B 0

# 查找可能违反导入顺序的文件（导入语句不在文件顶部）
# 需要手动检查或使用代码分析工具
```

**常见问题模式**：
- 导入语句分散在代码中间
- 导入顺序不符合规范（标准库、第三方库、项目内部）
- 导入分组之间没有空行分隔
- 在函数或模块中间使用 `use` 语句

**检查方法**：
1. **手动检查**：查看每个文件的导入部分
2. **自动化检查**：使用 `rustfmt` 或 `cargo clippy` 检查导入顺序
3. **代码审查**：在代码审查时重点关注导入顺序

**正确的导入顺序示例**：
```rust
// 标准库导入
use std::fs;
use std::path::Path;
use std::io::BufRead;

// 第三方库导入
use color-_eyre::Result;
use serde::{Deserialize, Serialize};

// 项目内部导入
use crate::base::util::file::FileReader;
use crate::base::settings::Settings;
```

**错误的导入顺序示例**：
```rust
// ❌ 不好的做法：导入分散在代码中
fn some-_function() {
    use std::fs;  // 不应该在函数内部导入
    // ...
}

// ❌ 不好的做法：导入顺序混乱
use crate::base::util::file::FileReader;  // 项目内部导入
use std::fs;  // 标准库导入应该在前面
use color-_eyre::Result;  // 第三方库导入
```

**检查清单**：
- [ ] 所有导入语句是否都在文件顶部？
- [ ] 导入顺序是否符合规范（标准库 → 第三方库 → 项目内部）？
- [ ] 导入分组之间是否有空行分隔？
- [ ] 是否在函数或模块中间使用了 `use` 语句？
- [ ] 是否使用了 `rustfmt` 格式化导入顺序？

#### 12. 过度设计模式

**检查目标**：识别不必要的抽象和过度设计

**检查原则**：
1. **YAGNI（You Aren't Gonna Need It）**：不要为未来可能的需求添加功能
2. **KISS（Keep It Simple, Stupid）**：保持简单，避免不必要的复杂性
3. **实用主义**：优先满足当前需求，避免过度抽象

**搜索模式**：
```bash
# 查找可能过度设计的模式
# - 过多的 trait 定义
grep -r "trait " src/ | wc -l

# - 过多的抽象层
grep -r "impl.*for" src/ | wc -l

# - 不必要的泛型
grep -r "<T>" src/ | grep -v "test"

# - 过多的中间层或包装器
grep -r "Wrapper\|Adapter\|Facade" src/
```

**常见过度设计模式**：

1. **不必要的抽象层**：
   - 为单一实现创建 trait
   - 创建多层抽象但只有一层实现
   - 过度使用设计模式（如为简单场景使用策略模式）

2. **过早优化**：
   - 为未来可能的需求添加功能
   - 创建过于灵活的配置系统但实际只需要简单配置
   - 添加不必要的泛型参数

3. **过度封装**：
   - 将简单函数包装成复杂的结构体
   - 创建不必要的中间层
   - 过度使用 Builder 模式

4. **不必要的复杂性**：
   - 使用复杂的数据结构处理简单数据
   - 创建复杂的错误处理系统处理简单错误
   - 过度使用宏或代码生成

**检查清单**：
- [ ] 是否有为单一实现创建的 trait？是否可以简化为直接实现？
- [ ] 是否有不必要的抽象层？是否可以减少中间层？
- [ ] 是否有为未来需求添加的功能？是否可以移除？
- [ ] 是否有过度封装的简单函数？是否可以简化为直接函数调用？
- [ ] 代码复杂度是否与实际需求匹配？
- [ ] 是否使用了不必要的设计模式？
- [ ] 是否有不必要的泛型参数？

**示例：过度设计 vs 简洁设计**

**过度设计示例**：
```rust
// ❌ 过度设计：为单一实现创建 trait
trait FileReader {
    fn read(&self, path: &Path) -> Result<String>;
}

struct SimpleFileReader;

impl FileReader for SimpleFileReader {
    fn read(&self, path: &Path) -> Result<String> {
        fs::read-_to-_string(path)
            .context(format!("Failed to read file: {:?}", path))
    }
}

// 使用
let reader = SimpleFileReader;
let content = reader.read(&path)?;
```

**简洁设计示例**：
```rust
// ✅ 简洁设计：直接使用函数
pub fn read-_file(path: &Path) -> Result<String> {
    fs::read-_to-_string(path)
        .context(format!("Failed to read file: {:?}", path))
}

// 使用
let content = read-_file(&path)?;
```

**过度设计示例**：
```rust
// ❌ 过度设计：为简单场景使用复杂的 Builder 模式
struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
}

impl ConfigBuilder {
    fn new() -> Self {
        Self {
            host: None,
            port: None,
        }
    }

    fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn build(self) -> Result<Config> {
        Ok(Config {
            host: self.host.unwrap-_or-_else(|| "localhost".to-_string()),
            port: self.port.unwrap-_or(8080),
        })
    }
}
```

**简洁设计示例**：
```rust
// ✅ 简洁设计：直接使用结构体初始化
pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub fn default() -> Self {
        Self {
            host: "localhost".to-_string(),
            port: 8080,
        }
    }
}
```

---

## 🛠️ 已封装工具检查

### 工具函数清单

#### 文件操作工具

**位置**：`lib/base/util/file.rs`

- `FileReader::open(path)` - 打开文件并返回 BufReader

**检查方法**：
```bash
# 查找直接使用 File::open 的地方
grep -r "File::open" src/ | grep -v "lib/base/util/file.rs"
```

#### 字符串处理工具

**位置**：`lib/base/util/string.rs`

- `mask-_sensitive-_value(value)` - 隐藏敏感值（如密码、token）

**检查方法**：
```bash
# 查找可能包含敏感信息的字符串处理
grep -r "password\|token\|secret\|key" src/ -i
```

#### 格式化工具

**位置**：`lib/base/util/format.rs`

- `format-_size(bytes)` - 格式化文件大小（B, KB, MB, GB）

**检查方法**：
```bash
# 查找文件大小格式化
grep -r "1024\|bytes\|KB\|MB\|GB" src/ -i
```

#### 平台检测工具

**位置**：`lib/base/util/platform.rs`

- `detect-_release-_platform()` - 检测发布平台（操作系统和架构）

**检查方法**：
```bash
# 查找平台检测代码
grep -r "target-_os\|target-_arch\|cfg!" src/
```

#### 浏览器和剪贴板工具

**位置**：`lib/base/util/browser.rs`, `lib/base/util/clipboard.rs`

- `Browser::open(url)` - 打开浏览器
- `Clipboard::copy(text)` - 复制到剪贴板

**检查方法**：
```bash
# 查找浏览器和剪贴板操作
grep -r "open::that\|arboard\|clipboard" src/
```

#### 解压和校验和工具

**位置**：`lib/base/util/unzip.rs`, `lib/base/util/checksum.rs`

- `Unzip::extract()` - 解压 tar.gz 文件
- `Checksum::verify()` - 验证 SHA256 校验和

**检查方法**：
```bash
# 查找解压和校验和操作
grep -r "tar::\|sha2\|sha256" src/
```

#### 日期格式化工具

**位置**：`lib/base/util/date.rs`

- `format-_document-_timestamp()` - 格式化文档时间戳
- `format-_last-_updated()` - 格式化最后更新时间
- `format-_last-_updated-_with-_time()` - 格式化最后更新时间（带时间）

**检查方法**：
```bash
# 查找日期格式化
grep -r "SystemTime\|UNIX_EPOCH\|format.*date" src/ -i
```

#### 表格输出工具

**位置**：`lib/base/util/table.rs`

- `TableBuilder` - 表格构建器
- `TableStyle` - 表格样式

**检查方法**：
```bash
# 查找表格输出
grep -r "tabled\|Table" src/
```

#### Git 操作工具

**位置**：`lib/git/helpers.rs`

- `cmd-_read(args)` - 执行 Git 命令并读取输出
- `cmd-_run(args)` - 执行 Git 命令（不读取输出）
- `check-_success(args)` - 静默执行并检查成功
- `check-_ref-_exists(ref-_path)` - 检查 Git 引用是否存在
- `switch-_or-_checkout(switch-_args, checkout-_args, error-_msg)` - 尝试 switch，失败时回退到 checkout
- `remove-_branch-_prefix(branch)` - 移除分支名称前缀

**检查方法**：
```bash
# 查找直接使用 cmd("git", ...) 的地方
grep -r 'cmd("git"' src/ | grep -v "lib/git/helpers.rs"
```

#### HTTP 客户端工具

**位置**：`lib/base/http/`

- `HttpClient` - 统一的 HTTP 客户端
- `HttpRetry` - HTTP 重试机制
- `HttpResponse` - HTTP 响应处理

**检查方法**：
```bash
# 查找直接使用 reqwest 的地方
grep -r "reqwest::\|Client::new" src/ | grep -v "lib/base/http"
```

#### 日志工具

**位置**：`lib/base/logger/console.rs`

- `info!(...)` - 信息日志
- `error!(...)` - 错误日志
- `warning!(...)` - 警告日志
- `success!(...)` - 成功日志
- `debug!(...)` - 调试日志
- `separator!()` - 分隔符
- `separator-_with-_text!(text)` - 带文本的分隔符

**检查方法**：
```bash
# 查找直接使用 println! 的地方
grep -r "println!\|eprintln!" src/
```

#### 对话框工具

**位置**：`lib/base/dialog/`

- `InputDialog` - 输入对话框
- `SelectDialog` - 选择对话框
- `MultiSelectDialog` - 多选对话框
- `ConfirmDialog` - 确认对话框

**检查方法**：
```bash
# 查找直接使用 dialoguer 的地方
grep -r "dialoguer::" src/ | grep -v "lib/base/dialog"
```

#### 进度指示器工具

**位置**：`lib/base/indicator/`

- `Spinner` - 旋转指示器
- `Progress` - 进度条

**检查方法**：
```bash
# 查找直接使用 indicatif 的地方
grep -r "indicatif::" src/ | grep -v "lib/base/indicator"
```

#### 配置管理工具

**位置**：`lib/base/settings/`

- `Settings` - 统一配置管理
- `Paths` - 路径管理
- `LLMSettings` - LLM 配置

**检查方法**：
```bash
# 查找直接读取配置文件的地方
grep -r "fs::read-_to-_string.*toml\|toml::from-_str" src/ | grep -v "lib/base/settings"
```

---

## 📦 第三方工具检查

### 检查方法

#### 1. 正则表达式处理

**检查目标**：查找自实现的正则表达式处理

**搜索模式**：
```bash
# 查找正则表达式使用
grep -r "Regex::new\|regex::" src/
```

**第三方工具**：
- `regex` - Rust 标准正则表达式库（已使用）
- 检查是否有重复的正则表达式编译

**检查清单**：
- [ ] 是否重复编译相同的正则表达式？
- [ ] 是否可以缓存编译后的正则表达式？

#### 2. JSON 处理

**检查目标**：查找 JSON 序列化/反序列化代码

**搜索模式**：
```bash
# 查找 JSON 处理
grep -r "serde-_json\|json::" src/
grep -r "\.to-_string()\|from-_str" src/
```

**第三方工具**：
- `serde-_json` - JSON 序列化/反序列化（已使用）
- `serde` - 序列化框架（已使用）

**检查清单**：
- [ ] 是否使用统一的序列化方式？
- [ ] 是否有重复的 JSON 解析逻辑？

#### 3. TOML 处理

**检查目标**：查找 TOML 配置文件处理

**搜索模式**：
```bash
# 查找 TOML 处理
grep -r "toml::\|toml-_edit" src/
```

**第三方工具**：
- `toml` - TOML 解析（已使用）
- `toml-_edit` - TOML 编辑（如果需要修改 TOML）

**检查清单**：
- [ ] 是否使用统一的 TOML 解析方式？
- [ ] 是否需要 TOML 编辑功能？

#### 4. 命令行参数解析

**检查目标**：查找命令行参数处理

**搜索模式**：
```bash
# 查找命令行参数
grep -r "clap\|Arg\|Command" src/
```

**第三方工具**：
- `clap` - 命令行参数解析（已使用）

**检查清单**：
- [ ] 是否使用 `clap` 进行参数解析？
- [ ] 是否有重复的参数定义？

#### 5. 异步处理

**检查目标**：查找异步代码

**搜索模式**：
```bash
# 查找异步代码
grep -r "async\|await\|tokio\|futures" src/
```

**第三方工具**：
- `tokio` - 异步运行时（如果需要）
- `futures` - 异步工具（如果需要）

**检查清单**：
- [ ] 是否需要异步处理？
- [ ] 是否可以使用异步提高性能？

#### 6. 并发处理

**检查目标**：查找并发处理代码

**搜索模式**：
```bash
# 查找并发处理
grep -r "thread\|std::thread\|rayon" src/
```

**已封装工具**：
- `lib/base/concurrent/executor.rs` - `ConcurrentExecutor` - 并发执行器

**第三方工具**：
- `rayon` - 数据并行处理（如果需要）
- `tokio::task` - 异步任务（如果需要）

**检查清单**：
- [ ] 是否可以使用 `ConcurrentExecutor`？
- [ ] 是否需要更高级的并发工具？

#### 7. 时间处理

**检查目标**：查找时间处理代码

**搜索模式**：
```bash
# 查找时间处理
grep -r "SystemTime\|UNIX_EPOCH\|chrono\|time::" src/
```

**第三方工具**：
- `chrono` - 日期和时间处理（如果需要更复杂的时间操作）
- `time` - 时间处理（Rust 标准库已足够）

**检查清单**：
- [ ] 是否需要更复杂的时间操作？
- [ ] 是否可以使用 `chrono` 简化代码？

#### 8. 路径处理

**检查目标**：查找路径处理代码

**搜索模式**：
```bash
# 查找路径处理
grep -r "PathBuf\|Path::\|\.join\|\.parent" src/
```

**第三方工具**：
- `pathdiff` - 路径差异计算（如果需要）
- `walkdir` - 目录遍历（如果需要）

**检查清单**：
- [ ] 是否需要路径差异计算？
- [ ] 是否需要目录遍历？

#### 9. 环境变量处理

**检查目标**：查找环境变量处理

**搜索模式**：
```bash
# 查找环境变量
grep -r "std::env\|env::var\|env::set-_var" src/
```

**第三方工具**：
- `dotenv` - .env 文件支持（如果需要）
- `envy` - 环境变量到结构体（如果需要）

**检查清单**：
- [ ] 是否需要 .env 文件支持？
- [ ] 是否需要环境变量到结构体的转换？

#### 10. 错误处理

**检查目标**：查找错误处理代码

**搜索模式**：
```bash
# 查找错误处理
grep -r "Result\|Error\|anyhow\|color-_eyre" src/
```

**第三方工具**：
- `anyhow` / `color-_eyre` - 错误处理（已使用）

**检查清单**：
- [ ] 是否统一使用 `anyhow::Result`？
- [ ] 是否使用 `Context` 添加上下文？

---

## ✅ 检查清单

### 重复代码检查清单

- [ ] **文件操作**：检查是否有重复的文件读写模式
- [ ] **Git 命令**：检查是否直接使用 `cmd("git", ...)` 而不是 `cmd-_read()` 或 `cmd-_run()`
- [ ] **错误处理**：检查是否有重复的错误处理逻辑
- [ ] **字符串处理**：检查是否有重复的字符串处理逻辑
- [ ] **路径处理**：检查是否有重复的路径处理逻辑
- [ ] **HTTP 请求**：检查是否直接使用 `reqwest` 而不是 `HttpClient`
- [ ] **配置读取**：检查是否直接读取配置文件而不是使用 `Settings`
- [ ] **日志输出**：检查是否使用 `println!` 而不是日志宏
- [ ] **用户交互**：检查是否直接使用 `dialoguer` 而不是封装的 Dialog
- [ ] **进度指示器**：检查是否直接使用 `indicatif` 而不是封装的 Spinner/Progress
- [ ] **导入顺序**：检查导入语句是否遵循从顶部导入的规范（标准库 → 第三方库 → 项目内部）
- [ ] **过度设计**：检查是否有不必要的抽象层、过度封装或过早优化

### 已封装工具检查清单

- [ ] **文件操作**：是否可以使用 `FileReader::open()`？
- [ ] **字符串处理**：是否可以使用 `mask-_sensitive-_value()`？
- [ ] **格式化**：是否可以使用 `format-_size()`？
- [ ] **平台检测**：是否可以使用 `detect-_release-_platform()`？
- [ ] **浏览器/剪贴板**：是否可以使用 `Browser::open()` 或 `Clipboard::copy()`？
- [ ] **解压/校验和**：是否可以使用 `Unzip::extract()` 或 `Checksum::verify()`？
- [ ] **日期格式化**：是否可以使用日期格式化函数？
- [ ] **表格输出**：是否可以使用 `TableBuilder`？
- [ ] **Git 操作**：是否可以使用 `git/helpers.rs` 中的函数？
- [ ] **HTTP 客户端**：是否可以使用 `HttpClient` 和 `HttpRetry`？
- [ ] **日志**：是否可以使用日志宏？
- [ ] **对话框**：是否可以使用封装的 Dialog 类型？
- [ ] **进度指示器**：是否可以使用封装的 Spinner/Progress？
- [ ] **配置管理**：是否可以使用 `Settings` API？

### 第三方工具检查清单

- [ ] **正则表达式**：是否重复编译正则表达式？是否可以缓存？
- [ ] **JSON 处理**：是否使用统一的序列化方式？
- [ ] **TOML 处理**：是否使用统一的 TOML 解析方式？
- [ ] **命令行参数**：是否使用 `clap` 进行参数解析？
- [ ] **异步处理**：是否需要异步处理？是否可以使用异步提高性能？
- [ ] **并发处理**：是否可以使用 `ConcurrentExecutor`？
- [ ] **时间处理**：是否需要更复杂的时间操作？是否可以使用 `chrono`？
- [ ] **路径处理**：是否需要路径差异计算或目录遍历？
- [ ] **环境变量**：是否需要 .env 文件支持或环境变量到结构体的转换？
- [ ] **错误处理**：是否统一使用 `anyhow::Result`？是否使用 `Context` 添加上下文？

---

## 📝 示例分析

### 示例 1：文件读取重复代码

**问题代码**：
```rust
// 文件 A
let content = fs::read-_to-_string(&config-_path)
    .wrap-_err(format!("Failed to read config file: {:?}", config-_path))?;

// 文件 B
let content = fs::read-_to-_string(&input-_path)
    .wrap-_err(format!("Failed to read file: {:?}", input-_path))?;
```

**改进方案**：
```rust
// 抽取为公共函数
pub fn read-_file-_with-_context(path: &Path, context: &str) -> Result<String> {
    fs::read-_to-_string(path)
        .wrap-_err(format!("{}: {:?}", context, path))
}

// 使用
let content = read-_file-_with-_context(&config-_path, "Failed to read config file")?;
```

### 示例 2：Git 命令执行重复代码

**问题代码**：
```rust
// 直接使用 cmd("git", ...)
let output = cmd("git", &["rev-parse", "--abbrev-ref", "HEAD"])
    .read()
    .wrap-_err("Failed to get current branch")?;
```

**改进方案**：
```rust
// 使用已封装的工具函数
use crate::git::helpers::cmd-_read;

let output = cmd-_read(&["rev-parse", "--abbrev-ref", "HEAD"])?;
```

### 示例 3：日志输出重复代码

**问题代码**：
```rust
// 直接使用 println!
println!("✓ Successfully created branch: {}", branch-_name);
println!("✗ Failed to create branch: {}", error);
```

**改进方案**：
```rust
// 使用日志宏
use crate::base::util::{success, error};

success!("Successfully created branch: {}", branch-_name);
error!("Failed to create branch: {}", error);
```

### 示例 4：HTTP 请求重复代码

**问题代码**：
```rust
// 直接使用 reqwest
let client = reqwest::Client::new();
let response = client
    .get(url)
    .header("Authorization", format!("Bearer {}", token))
    .send()
    .await?;
```

**改进方案**：
```rust
// 使用封装的 HttpClient
use crate::base::http::HttpClient;

let client = HttpClient::new()?;
let response = client
    .get(url)?
    .auth(Authorization::Bearer(token))
    .send()?;
```

### 示例 5：用户交互重复代码

**问题代码**：
```rust
// 直接使用 dialoguer
use dialoguer::Input;

let input: String = Input::new()
    .with-_prompt("Enter branch name")
    .interact()?;
```

**改进方案**：
```rust
// 使用封装的 Dialog
use crate::base::dialog::InputDialog;

let input = InputDialog::new("Enter branch name")
    .interact()?;
```

### 示例 6：导入顺序不规范

**问题代码**：
```rust
// ❌ 不好的做法：导入顺序混乱，导入分散
use crate::base::util::file::FileReader;  // 项目内部导入应该在最后
use std::fs;  // 标准库导入应该在前面
use color-_eyre::Result;  // 第三方库导入

fn some-_function() {
    use std::path::Path;  // 不应该在函数内部导入
    // ...
}
```

**改进方案**：
```rust
// ✅ 好的做法：导入顺序规范，都在文件顶部
// 标准库导入
use std::fs;
use std::path::Path;

// 第三方库导入
use color-_eyre::Result;

// 项目内部导入
use crate::base::util::file::FileReader;

fn some-_function() {
    // 使用已导入的类型
    // ...
}
```

---

## 🔧 检查工具推荐

### 代码搜索工具

1. **ripgrep (rg)**：快速文本搜索
   ```bash
   rg "pattern" src/
   ```

2. **语义搜索**：使用 AI 工具进行语义搜索
   - 查找相似功能的代码
   - 识别重复模式

### 代码分析工具

1. **cargo clippy**：Rust 代码检查
   ```bash
   cargo clippy -- -D warnings
   ```

2. **cargo fmt**：代码格式化检查
   ```bash
   cargo fmt --check
   ```

### 重复代码检测

1. **人工审查**：逐文件检查
2. **模式匹配**：使用 grep 查找常见模式
3. **语义分析**：使用 AI 工具进行语义分析

---

## 📚 参考文档

### 项目文档

- [开发规范](../development.md) - 代码风格和开发规范
- [架构文档](../architecture/) - 各模块架构文档
- [工具函数架构](../architecture/tools.md) - 工具函数模块架构

### 工具函数位置

- `lib/base/util/` - 通用工具函数
- `lib/git/helpers.rs` - Git 操作辅助函数
- `lib/base/http/` - HTTP 客户端工具
- `lib/base/dialog/` - 对话框工具
- `lib/base/indicator/` - 进度指示器工具
- `lib/base/logger/` - 日志工具
- `lib/base/settings/` - 配置管理工具

---

## 📝 检查报告模板

检查完成后，应生成检查报告，包含：

1. **问题汇总**：列出所有发现的问题
2. **优先级排序**：按影响范围和收益排序
3. **改进建议**：提供具体的改进方案
4. **代码示例**：提供改进前后的代码对比

**报告格式**：
```markdown
# 代码检查报告

## 问题汇总

### 1. 重复代码问题
- [问题描述]
- [位置]
- [改进方案]

### 2. 工具函数替换
- [问题描述]
- [位置]
- [改进方案]

### 3. 第三方工具替换
- [问题描述]
- [位置]
- [改进方案]

## 优先级排序

1. [高优先级问题]
2. [中优先级问题]
3. [低优先级问题]

## 改进建议

[具体的改进建议和代码示例]
```

---

**最后更新**: 2025-12-23
