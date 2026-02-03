//! Commit create 命令实现
//!
//! 最简单的 commit 实现，直接使用 git2 库。

use prompt::{info, success};

use storage::git::GitContext;

/// Commit Create 命令
pub struct CommitCreateCommand {
    message: String,
    all: bool,
}

impl CommitCreateCommand {
    /// 创建新的 CommitCreateCommand
    pub fn new(message: String, all: bool) -> Self {
        Self { message, all }
    }

    /// 运行 `workflow commit create` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 打开仓库
        let ctx =
            GitContext::discover().map_err(|e| format!("Failed to open repository: {}", e))?;

        info!("Repository opened successfully");

        // 先获取签名（需要临时获取锁）
        let signature =
            ctx.get_signature().map_err(|e| format!("Failed to get signature: {}", e))?;

        // 获取 repository 和 index（可以持有锁）
        let repo = ctx.repository();
        let mut index = repo.index().map_err(|e| format!("Failed to get index: {}", e))?;

        // 只在 all=true 时添加所有文件到暂存区
        if self.all {
            // 使用回调函数跳过大型目录以提高性能
            // 即使这些目录在 .gitignore 中，扫描它们仍然很慢
            index
                .add_all(
                    ["."].iter(),
                    git2::IndexAddOption::DEFAULT,
                    Some(&mut |path, _| {
                        // 显式跳过大型目录，即使它们在 .gitignore 中
                        // 这样可以避免扫描大量文件，提高性能
                        if let Some(path_str) = path.to_str() {
                            // 跳过常见的构建和缓存目录
                            if path_str.starts_with("target")
                                || path_str.starts_with(".rs")
                                || path_str.starts_with(".rs2")
                                || path_str.starts_with(".go")
                                || path_str.starts_with("coverage")
                                || path_str.starts_with("node_modules")
                                || path_str.starts_with(".git")
                            {
                                return 1; // Skip this path
                            }
                        }
                        0 // Add this path (git2 会自动处理 .gitignore)
                    }),
                )
                .map_err(|e| format!("Failed to add files to index: {}", e))?;

            index.write().map_err(|e| format!("Failed to write index: {}", e))?;

            info!("Added all files to staging area");
        }

        // 创建 tree
        let tree_id = index.write_tree().map_err(|e| format!("Failed to create tree: {}", e))?;

        let tree = repo.find_tree(tree_id).map_err(|e| format!("Failed to find tree: {}", e))?;

        // 获取 HEAD commit（如果有）
        let parent_commit = repo.head().and_then(|head| head.peel_to_commit()).ok();

        // 创建提交
        let oid = if let Some(parent) = parent_commit {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &self.message,
                &tree,
                &[&parent],
            )
            .map_err(|e| format!("Failed to create commit: {}", e))?
        } else {
            // 首次提交
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &self.message,
                &tree,
                &[],
            )
            .map_err(|e| format!("Failed to create commit: {}", e))?
        };

        success!("Created commit: {}", oid);
        Ok(())
    }
}
