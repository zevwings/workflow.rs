//! 测试辅助模块
//!
//! 提供 Git 服务测试的通用辅助函数和性能监控工具。

use super::services::hooks::{HookContext, HookResult, HookService};
use super::GitContext;
use domain::git::GitError;
use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

// 重新导出性能监控工具，方便测试代码使用
pub use super::performance;

// ============================================================
// NoopHookService - 测试用空 Hook 服务
// ============================================================

/// 测试用空 Hook 服务
///
/// 不执行任何 hook，始终返回成功。用于测试和基准测试。
pub struct NoopHookService;

impl HookService for NoopHookService {
    fn execute_hook(
        &self,
        _hook_name: &str,
        _context: &HookContext,
    ) -> Result<HookResult, GitError> {
        Ok(HookResult::Success)
    }
}

/// 创建一个 NoopHookService 的 Arc 引用
///
/// 用于传递给需要 `Arc<dyn HookService>` 的服务。
pub fn noop_hook_service() -> Arc<dyn HookService> {
    Arc::new(NoopHookService)
}

/// 测试仓库配置
pub struct TestRepoConfig {
    /// 是否创建初始文件
    pub with_file: bool,
    /// 初始文件名
    pub file_name: &'static str,
    /// 初始文件内容
    pub file_content: &'static str,
}

impl Default for TestRepoConfig {
    fn default() -> Self {
        Self {
            with_file: false,
            file_name: "test.txt",
            file_content: "hello",
        }
    }
}

impl TestRepoConfig {
    /// 创建带文件的配置
    pub fn with_file() -> Self {
        Self {
            with_file: true,
            ..Default::default()
        }
    }

    /// 创建带多行内容文件的配置
    pub fn with_content(content: &'static str) -> Self {
        Self {
            with_file: true,
            file_content: content,
            ..Default::default()
        }
    }
}

static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

fn set_env(key: &str, value: &str, original: &mut Vec<(String, Option<String>)>) {
    original.push((key.to_string(), env::var(key).ok()));
    env::set_var(key, value);
}

fn restore_env(original: Vec<(String, Option<String>)>) {
    for (key, value) in original {
        if let Some(value) = value {
            env::set_var(key, value);
        } else {
            env::remove_var(key);
        }
    }
}

/// 在隔离的 Git 环境中运行测试逻辑
///
/// - 使用临时 HOME 目录
/// - 禁用系统级 Git 配置
/// - 使用空的全局 Git 配置文件
pub fn with_isolated_git_env<F: FnOnce()>(f: F) {
    let _lock = TEST_ENV_LOCK.lock().unwrap();
    let tmp = TempDir::new().unwrap();
    let mut original = Vec::new();

    set_env("HOME", tmp.path().to_str().unwrap(), &mut original);
    set_env(
        "XDG_CONFIG_HOME",
        tmp.path().to_str().unwrap(),
        &mut original,
    );
    set_env(
        "GIT_CONFIG_GLOBAL",
        tmp.path().join("gitconfig").to_str().unwrap(),
        &mut original,
    );
    set_env("GIT_CONFIG_NOSYSTEM", "1", &mut original);

    f();

    restore_env(original);
}

/// 创建测试仓库
///
/// 返回临时目录和 GitContext。
/// 临时目录在 drop 时会自动清理。
pub fn setup_repo() -> (TempDir, GitContext) {
    setup_repo_with_config(TestRepoConfig::default())
}

/// 创建带文件的测试仓库
pub fn setup_repo_with_file() -> (TempDir, GitContext) {
    setup_repo_with_config(TestRepoConfig::with_file())
}

/// 创建带自定义配置的测试仓库
pub fn setup_repo_with_config(config: TestRepoConfig) -> (TempDir, GitContext) {
    let tmp = TempDir::new().unwrap();
    let ctx = GitContext::init(tmp.path()).unwrap();

    // 创建初始提交
    {
        let repo = ctx.repository();

        if config.with_file {
            std::fs::write(tmp.path().join(config.file_name), config.file_content).unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(Path::new(config.file_name)).unwrap();
            index.write().unwrap();
        }

        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
    }

    (tmp, ctx)
}

/// 创建包含多个文件的测试仓库
///
/// # 参数
/// - `file_count`: 要创建的文件数量
///
/// # 返回
/// 返回临时目录和 GitContext
pub fn setup_repo_with_files(file_count: usize) -> (TempDir, GitContext) {
    let tmp = TempDir::new().unwrap();
    let ctx = GitContext::init(tmp.path()).unwrap();

    {
        let repo = ctx.repository();

        // 创建多个文件
        for i in 0..file_count {
            let file_name = format!("file_{}.txt", i);
            let file_content = format!("content of file {}", i);
            std::fs::write(tmp.path().join(&file_name), file_content).unwrap();

            let mut index = repo.index().unwrap();
            index.add_path(Path::new(&file_name)).unwrap();
            index.write().unwrap();
        }

        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial commit with multiple files",
            &tree,
            &[],
        )
        .unwrap();
    }

    (tmp, ctx)
}

/// 创建包含多个分支的测试仓库
///
/// # 参数
/// - `branch_count`: 要创建的分支数量
///
/// # 返回
/// 返回临时目录和 GitContext
pub fn setup_repo_with_branches(branch_count: usize) -> (TempDir, GitContext) {
    let (tmp, ctx) = setup_repo_with_file();

    {
        let repo = ctx.repository();
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();

        // 创建多个分支
        for i in 0..branch_count {
            let branch_name = format!("branch_{}", i);
            repo.branch(&branch_name, &commit, false).unwrap();
        }
    }

    (tmp, ctx)
}

/// 创建包含大文件的测试仓库
///
/// # 参数
/// - `line_count`: 文件行数
///
/// # 返回
/// 返回临时目录和 GitContext
pub fn setup_repo_with_large_file(line_count: usize) -> (TempDir, GitContext) {
    let tmp = TempDir::new().unwrap();
    let ctx = GitContext::init(tmp.path()).unwrap();

    {
        let repo = ctx.repository();

        // 创建大文件
        let mut content = String::new();
        for i in 0..line_count {
            content.push_str(&format!("This is line {} with some content\n", i));
        }

        std::fs::write(tmp.path().join("large_file.txt"), content).unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("large_file.txt")).unwrap();
        index.write().unwrap();

        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial commit with large file",
            &tree,
            &[],
        )
        .unwrap();
    }

    (tmp, ctx)
}

/// 创建包含多次提交的测试仓库
///
/// # 参数
/// - `commit_count`: 提交次数
///
/// # 返回
/// 返回临时目录和 GitContext
pub fn setup_repo_with_commits(commit_count: usize) -> (TempDir, GitContext) {
    let tmp = TempDir::new().unwrap();
    let ctx = GitContext::init(tmp.path()).unwrap();

    {
        let repo = ctx.repository();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();

        let mut parent_commit = None;

        for i in 0..commit_count {
            let file_name = format!("file_{}.txt", i);
            let file_content = format!("commit {}", i);
            std::fs::write(tmp.path().join(&file_name), file_content).unwrap();

            let mut index = repo.index().unwrap();
            index.add_path(Path::new(&file_name)).unwrap();
            index.write().unwrap();

            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();

            let message = format!("Commit {}", i);
            let parents = if let Some(ref parent) = parent_commit {
                vec![parent]
            } else {
                vec![]
            };

            let oid = repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents).unwrap();

            parent_commit = Some(repo.find_commit(oid).unwrap());
        }
    }

    (tmp, ctx)
}

/// 创建包含未暂存更改的测试仓库
///
/// # 参数
/// - `modified_count`: 修改文件的数量
/// - `untracked_count`: 未跟踪文件的数量
///
/// # 返回
/// 返回临时目录和 GitContext
pub fn setup_repo_with_changes(
    modified_count: usize,
    untracked_count: usize,
) -> (TempDir, GitContext) {
    let (tmp, ctx) = setup_repo_with_files(modified_count);

    // 修改已跟踪的文件
    for i in 0..modified_count {
        let file_name = format!("file_{}.txt", i);
        let file_content = format!("modified content {}", i);
        std::fs::write(tmp.path().join(&file_name), file_content).unwrap();
    }

    // 创建未跟踪的文件
    for i in 0..untracked_count {
        let file_name = format!("untracked_{}.txt", i);
        let file_content = format!("untracked content {}", i);
        std::fs::write(tmp.path().join(&file_name), file_content).unwrap();
    }

    (tmp, ctx)
}
