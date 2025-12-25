//! CLI 测试辅助工具
//!
//! 提供 CLI 测试的常用辅助函数和工具。
//!
//! 注意：`CliTestEnv` 已迁移到 `tests/common/environments/cli_test_env.rs`。
//! 本文件仅保留 `CliCommandBuilder` 和 `TestDataGenerator` 等辅助工具。

use assert_cmd::Command;
use std::path::Path;

/// CLI 命令构建器
pub struct CliCommandBuilder {
    cmd: Command,
}

impl CliCommandBuilder {
    /// 创建新的命令构建器
    pub fn new() -> Self {
        Self {
            cmd: Command::new(assert_cmd::cargo::cargo_bin!("workflow")),
        }
    }

    /// 添加参数
    pub fn arg<S: AsRef<std::ffi::OsStr>>(mut self, arg: S) -> Self {
        self.cmd.arg(arg);
        self
    }

    /// 添加多个参数
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.cmd.args(args);
        self
    }

    /// 设置环境变量
    pub fn env<K, V>(mut self, key: K, val: V) -> Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.cmd.env(key, val);
        self
    }

    /// 设置工作目录
    pub fn current_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.cmd.current_dir(dir);
        self
    }

    /// 执行命令并断言成功
    pub fn assert_success(mut self) -> assert_cmd::assert::Assert {
        self.cmd.assert().success()
    }

    /// 执行命令并断言失败
    pub fn assert_failure(mut self) -> assert_cmd::assert::Assert {
        self.cmd.assert().failure()
    }

    /// 执行命令并返回断言对象
    pub fn assert(mut self) -> assert_cmd::assert::Assert {
        self.cmd.assert()
    }
}

/// 测试数据生成器
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// 生成测试用的配置内容
    pub fn config_content() -> String {
        r#"
[jira]
url = "https://test.atlassian.net"
username = "test@example.com"

[github]
token = "test_token"
"#
        .to_string()
    }
}

// 注意：以下宏已废弃，请使用新版 environments::CliTestEnv
// #[macro_export]
// macro_rules! with_cli_env { ... }
// #[macro_export]
// macro_rules! cli_integration_test { ... }

/// 辅助函数：检查输出是否包含错误消息
pub fn contains_error(output: &str) -> bool {
    output.contains("❌") || output.contains("错误") || output.contains("Error")
}

/// 辅助函数：检查输出是否为 JSON 格式
pub fn is_json_format(output: &str) -> bool {
    let trimmed = output.trim();
    trimmed.starts_with('{') && trimmed.ends_with('}')
}
