use std::path::PathBuf;
use std::process::Output;

use color_eyre::{eyre::WrapErr, Result};
use duct::cmd;

/// 轻量级 Git 命令封装
///
/// 基于 `duct::cmd` 的薄封装，统一处理工作目录、环境变量和错误上下文。
#[derive(Debug, Default, Clone)]
pub struct GitCommand {
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: Option<PathBuf>,
}

impl GitCommand {
    /// 创建新的 Git 命令
    pub fn new(args: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self {
            args: args.into_iter().map(|a| a.as_ref().to_string()).collect(),
            env: Vec::new(),
            cwd: None,
        }
    }

    /// 设置工作目录
    #[allow(dead_code)]
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// 添加单个环境变量
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// 运行命令，忽略输出
    pub fn run(&self) -> Result<()> {
        self.build()
            .run()
            .wrap_err_with(|| format!("Failed to run: {}", self.command_str()))?;
        Ok(())
    }

    /// 运行命令并读取输出（去除首尾空白）
    pub fn read(&self) -> Result<String> {
        let output = self
            .build()
            .read()
            .wrap_err_with(|| format!("Failed to run: {}", self.command_str()))?;
        Ok(output.trim().to_string())
    }

    /// 运行命令并捕获 stdout/stderr
    #[allow(dead_code)]
    pub fn capture(&self) -> Result<Output> {
        self.build()
            .stdout_capture()
            .stderr_capture()
            .run()
            .wrap_err_with(|| format!("Failed to run: {}", self.command_str()))
    }

    /// 静默运行命令，返回是否成功
    pub fn quiet_success(&self) -> bool {
        self.build().stdout_null().stderr_null().run().is_ok()
    }

    fn build(&self) -> duct::Expression {
        let args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        let mut expression = cmd("git", args);

        if let Some(cwd) = &self.cwd {
            expression = expression.dir(cwd);
        }

        for (key, value) in &self.env {
            expression = expression.env(key, value);
        }

        expression
    }

    fn command_str(&self) -> String {
        format!("git {}", self.args.join(" "))
    }
}
