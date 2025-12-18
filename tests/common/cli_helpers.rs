//! CLI 测试辅助工具
//!
//! 提供 CLI 测试的常用辅助函数和工具。

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// CLI 测试环境
pub struct CliTestEnv {
    pub temp_dir: TempDir,
}

impl CliTestEnv {
    /// 创建新的 CLI 测试环境
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir()
            .expect(workflow::base::constants::errors::file_operations::CREATE_TEMP_DIR_FAILED);

        Self { temp_dir }
    }

    /// 初始化 Git 仓库
    pub fn init_git_repo(&self) -> &Self {
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(self.path())
            .output()
            .expect("Failed to init git repo");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(self.path())
            .output()
            .expect("Failed to set git user name");

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(self.path())
            .output()
            .expect("Failed to set git user email");

        self
    }

    /// 创建文件
    pub fn create_file(&self, path: &str, content: &str) -> &Self {
        let full_path = self.temp_dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect(
                workflow::base::constants::errors::file_operations::CREATE_PARENT_DIR_FAILED,
            );
        }
        fs::write(full_path, content)
            .expect(workflow::base::constants::errors::file_operations::WRITE_FILE_FAILED);
        self
    }

    /// 创建 Git 提交
    pub fn create_commit(&self, message: &str) -> &Self {
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(self.path())
            .output()
            .expect("Failed to add files");

        std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(self.path())
            .output()
            .expect("Failed to commit");

        self
    }

    /// 创建配置文件
    pub fn create_config(&self, content: &str) -> &Self {
        let config_dir = self.temp_dir.path().join(".workflow");
        fs::create_dir_all(&config_dir)
            .expect(workflow::base::constants::errors::file_operations::CREATE_CONFIG_DIR_FAILED);

        let config_file = config_dir.join("workflow.toml");
        fs::write(config_file, content)
            .expect(workflow::base::constants::errors::file_operations::WRITE_CONFIG_FAILED);

        self
    }

    /// 获取临时目录路径
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}

// 移除 Drop 实现，因为我们不再改变全局工作目录

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

/// 简化的测试宏
#[macro_export]
macro_rules! with_cli_env {
    ($env:ident, $block:block) => {
        let $env = CliTestEnv::new();
        $block
    };
}

/// CLI 集成测试宏
#[macro_export]
macro_rules! cli_integration_test {
    ($name:ident, $test_fn:expr) => {
        #[test]
        fn $name() {
            with_cli_env!(env, {
                $test_fn(env);
            });
        }
    };
}

/// 辅助函数：检查输出是否包含错误消息
pub fn contains_error(output: &str) -> bool {
    output.contains("❌") || output.contains("错误") || output.contains("Error")
}

/// 辅助函数：检查输出是否为 JSON 格式
pub fn is_json_format(output: &str) -> bool {
    let trimmed = output.trim();
    trimmed.starts_with('{') && trimmed.ends_with('}')
}
