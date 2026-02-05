//! 工具检测器
//!
//! 负责检测项目中使用的 hooks 管理工具（prek/pre-commit）。

use std::path::PathBuf;

/// 检测到的工具类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookTool {
    /// prek (Rust，pre-commit 的高性能替代)
    Prek,
    /// pre-commit (Python)
    PreCommit,
    /// 标准 Git hooks
    Standard,
}

/// 工具检测结果
#[derive(Debug, Clone)]
pub struct ToolDetectionResult {
    /// 工具类型
    pub tool: HookTool,
    /// 配置文件路径（如果适用）
    pub config_path: Option<PathBuf>,
    /// 可执行文件路径（如果适用）
    pub executable_path: Option<PathBuf>,
}

/// 工具检测器
///
/// 负责检测项目中使用的 hooks 管理工具。
pub struct HookToolDetector {
    repo_path: PathBuf,
}

impl HookToolDetector {
    /// 创建新的工具检测器
    ///
    /// # 参数
    /// - `repo_path`: 仓库根目录路径
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    /// 检测项目中使用的 hooks 工具
    ///
    /// 按优先级返回检测结果（优先级高的在前）
    ///
    /// # 参数
    /// - `hook_name`: Hook 名称（用于特定 hook 的检测）
    ///
    /// # 返回
    /// 检测结果列表，按优先级排序
    pub fn detect_tools(&self, hook_name: &str) -> Vec<ToolDetectionResult> {
        let mut results = Vec::new();

        // 1. 检测 prek/pre-commit
        // hook_name 参数保留以便将来按 hook 类型过滤
        let _ = hook_name;
        if let Some(result) = self.detect_hook_tools() {
            results.push(result);
        }

        // 2. 标准 Git hooks（总是检查，作为兜底）
        // 注意：HookDiscoverer 会自动检查 core.hooksPath 配置
        // 如果 core.hooksPath 已设置，会使用该路径；否则使用 .git/hooks/
        results.push(ToolDetectionResult {
            tool: HookTool::Standard,
            config_path: None,
            executable_path: None,
        });

        results
    }

    /// 检测 prek/pre-commit
    ///
    /// 优先使用 prek（性能更好，快 7 倍），如果不可用则回退到 pre-commit。
    ///
    /// # 参数
    /// - `hook_name`: Hook 名称
    ///
    /// # 返回
    /// - `Some(ToolDetectionResult)`: 检测到工具
    /// - `None`: 未检测到工具或工具不可用
    fn detect_hook_tools(&self) -> Option<ToolDetectionResult> {
        let config_path = self.repo_path.join(".pre-commit-config.yaml");

        if !config_path.exists() {
            return None;
        }

        // 优先使用 prek（性能更好，快 7 倍）
        if let Ok(path) = which::which("prek") {
            return Some(ToolDetectionResult {
                tool: HookTool::Prek,
                config_path: Some(config_path),
                executable_path: Some(path),
            });
        }

        // 回退到 pre-commit
        if let Ok(path) = which::which("pre-commit") {
            return Some(ToolDetectionResult {
                tool: HookTool::PreCommit,
                config_path: Some(config_path),
                executable_path: Some(path),
            });
        }

        // 配置文件存在但工具不可用，记录警告但返回 None
        // 实际使用时会在日志中记录警告，但不阻止操作
        None
    }
}
