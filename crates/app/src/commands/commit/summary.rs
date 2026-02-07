//! workflow commit summary：三阶段提交分析（阶段一文件分类 → 阶段二分类分析 → 阶段三全局总结）
//!
//! 对当前分支相对基准分支的变更执行完整分析，输出结构化的 commit 总结。

use std::collections::HashMap;

use domain::git::entity::{CommitChangeType, CommitFileChange};
use domain::llm::entity::{
    CommitBatchAnalysis, CommitConfigAnalysis, CommitFileClassification, CommitLogicAnalysis,
    CommitTestAnalysis,
};
use prompt::info;
use serde_json::json;

use crate::registry::{get_git_repository, get_llm_repository};

/// 三阶段提交分析命令（阶段一分类 → 阶段二分析 → 阶段三总结）
pub struct CommitSummaryCommand;

impl Default for CommitSummaryCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitSummaryCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("开始三阶段提交分析（当前分支相对基准分支）...");

        let git_repo = get_git_repository();
        let llm_repo = get_llm_repository();

        let current_branch = git_repo.get_current_branch()?;
        let base_branch = git_repo
            .infer_target_branch(&current_branch)
            .map_err(|e| format!("Failed to infer base branch: {}", e))?;
        let source_branch = base_branch.as_deref().unwrap_or("master").to_string();

        info!(
            "当前分支: {}，基准分支: {}，获取变更文件列表...",
            current_branch, source_branch
        );

        let files = git_repo.get_merge_changed_files(&current_branch, &source_branch)?;
        if files.is_empty() {
            info!("当前分支相对 {} 无变更文件，无需分析。", source_branch);
            return Ok(());
        }

        let commit_info = git_repo.get_commit_info("HEAD")?;
        let files_json: Vec<serde_json::Value> = files
            .iter()
            .map(|f| {
                json!({
                    "path": f.path,
                    "status": status_to_str(f.change_type),
                    "additions": f.additions.unwrap_or(0),
                    "deletions": f.deletions.unwrap_or(0),
                    "old_path": f.old_path
                })
            })
            .collect();
        let input_json = json!({
            "commit_id": commit_info.sha,
            "author": commit_info.author_email,
            "timestamp": commit_info.author_time,
            "files": files_json
        });

        // ---------- 阶段一：文件分类 ----------
        info!("阶段一：对 {} 个变更文件进行智能分类...", files.len());
        let stage1 = llm_repo.classify_commit_files(&input_json.to_string())?;
        let stage1_json =
            serde_json::to_string(&stage1).map_err(|e| format!("stage1 serialize: {}", e))?;
        info!("阶段一完成。");

        // 获取完整 merge diff 并解析为按文件 diff，供阶段二使用
        let full_diff =
            git_repo.get_merge_diff(&current_branch, &source_branch)?.unwrap_or_default();
        let file_diffs = parse_diff_per_file(&full_diff);

        // 统计信息（阶段三用）
        let (added_count, deleted_count, modified_count, renamed_count) = count_by_status(&files);
        let total_additions: u32 = files.iter().filter_map(|f| f.additions).sum();
        let total_deletions: u32 = files.iter().filter_map(|f| f.deletions).sum();

        // ---------- 阶段二：分类分析 ----------
        let batch_json = run_stage2_batch(llm_repo.as_ref(), &stage1, &file_diffs, &files)?;
        let logic_json = run_stage2_logic(llm_repo.as_ref(), &stage1, &file_diffs, &files)?;
        let config_json = run_stage2_config(llm_repo.as_ref(), &stage1, &file_diffs, &files)?;
        let test_json = run_stage2_tests(llm_repo.as_ref(), &stage1, &file_diffs, &files)?;

        // ---------- 阶段三：全局总结 ----------
        info!("阶段三：生成全局 commit 总结...");
        let total_files = files.len() as u32;
        let summary = llm_repo.summarize_commit_analysis(
            &stage1_json,
            &batch_json,
            &logic_json,
            &config_json,
            &test_json,
            total_files,
            added_count,
            deleted_count,
            modified_count,
            renamed_count,
            total_additions,
            total_deletions,
        )?;

        info!("三阶段分析完成。");
        println!("{}", serde_json::to_string_pretty(&summary).unwrap());
        Ok(())
    }
}

/// 将完整 diff 按文件拆分为 path -> diff 内容
fn parse_diff_per_file(full_diff: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if full_diff.trim().is_empty() {
        return map;
    }
    // 统一在开头加换行，再按 "\ndiff --git " 分割，则每段首行为 "a/old b/new"
    let normalized = format!("\n{}", full_diff.trim_start());
    let segments: Vec<&str> = normalized.split("\ndiff --git ").collect();
    for seg in segments {
        if seg.is_empty() {
            continue;
        }
        // 第一行格式: "a/oldpath b/newpath" 或 "a/path b/path"
        let first_line_end = seg.find('\n').unwrap_or(seg.len());
        let first_line = seg[..first_line_end].trim();
        let path = first_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.strip_prefix("b/"))
            .map(String::from)
            .unwrap_or_default();
        if path.is_empty() {
            continue;
        }
        let diff_content = seg[first_line_end..].trim();
        if !diff_content.is_empty() {
            let full_block = format!("diff --git a/{} b/{}\n{}", path, path, diff_content);
            map.insert(path, full_block);
        }
    }
    map
}

fn count_by_status(files: &[CommitFileChange]) -> (u32, u32, u32, u32) {
    let mut added = 0u32;
    let mut deleted = 0u32;
    let mut modified = 0u32;
    let mut renamed = 0u32;
    for f in files {
        match f.change_type {
            CommitChangeType::Added => added += 1,
            CommitChangeType::Deleted => deleted += 1,
            CommitChangeType::Modified
            | CommitChangeType::Copied
            | CommitChangeType::TypeChanged => modified += 1,
            CommitChangeType::Renamed => renamed += 1,
        }
    }
    (added, deleted, modified, renamed)
}

fn run_stage2_batch(
    llm_repo: &dyn domain::llm::repository::LLMRepository,
    stage1: &CommitFileClassification,
    file_diffs: &HashMap<String, String>,
    files: &[CommitFileChange],
) -> Result<String, Box<dyn std::error::Error>> {
    let batch_group = &stage1.analysis_strategy.batch_group;
    if batch_group.is_empty() {
        return Ok("{}".to_string());
    }
    let pattern_type = if stage1.patterns.mass_rename.detected {
        "批量重命名"
    } else if stage1.patterns.formatting.detected {
        "批量格式化"
    } else if stage1.patterns.config_update.detected {
        "统一配置更新"
    } else if stage1.patterns.dependency_upgrade.detected {
        "依赖版本升级"
    } else if stage1.patterns.import_path_change.detected {
        "导入路径调整"
    } else {
        "批量操作"
    };
    let pattern_desc = format!(
        "{} {}",
        stage1.patterns.mass_rename.pattern, stage1.patterns.formatting.description
    );
    let sample_paths: Vec<&String> = batch_group.iter().take(3).collect();
    let mut sample_diffs = String::new();
    for (i, path) in sample_paths.iter().enumerate() {
        let additions =
            files.iter().find(|f| f.path == **path).and_then(|f| f.additions).unwrap_or(0);
        let deletions =
            files.iter().find(|f| f.path == **path).and_then(|f| f.deletions).unwrap_or(0);
        let diff = file_diffs.get(*path).map(String::as_str).unwrap_or("");
        sample_diffs.push_str(&format!(
            "\n### 文件{}: {}\n变更：+{} -{}\n```diff\n{}\n```\n",
            i + 1,
            path,
            additions,
            deletions,
            diff
        ));
    }
    let user_prompt = format!(
        r##"## 批量操作信息
- 操作类型：{}
- 涉及文件数：{}
- 操作模式：{}

## 样本文件Diff（前{}个代表性文件）
{}
"##,
        pattern_type,
        batch_group.len(),
        pattern_desc,
        sample_paths.len(),
        sample_diffs
    );
    let result: CommitBatchAnalysis = llm_repo.analyze_commit_batch(&user_prompt)?;
    serde_json::to_string(&result).map_err(|e| e.into())
}

fn run_stage2_logic(
    llm_repo: &dyn domain::llm::repository::LLMRepository,
    stage1: &CommitFileClassification,
    file_diffs: &HashMap<String, String>,
    files: &[CommitFileChange],
) -> Result<String, Box<dyn std::error::Error>> {
    let focus_group = &stage1.analysis_strategy.focus_group;
    if focus_group.is_empty() {
        return Ok("{}".to_string());
    }
    let mut parts = String::new();
    for path in focus_group {
        let additions =
            files.iter().find(|f| f.path == *path).and_then(|f| f.additions).unwrap_or(0);
        let deletions =
            files.iter().find(|f| f.path == *path).and_then(|f| f.deletions).unwrap_or(0);
        let diff = file_diffs.get(path).map(String::as_str).unwrap_or("");
        parts.push_str(&format!(
            r##"
### 文件：{}
修改规模：+{} -{}

#### Diff内容：
```diff
{}
```

---
"##,
            path, additions, deletions, diff
        ));
    }
    let user_prompt = format!(
        r##"## 修改文件列表
{}
"##,
        parts
    );
    let result: CommitLogicAnalysis = llm_repo.analyze_commit_logic(&user_prompt)?;
    serde_json::to_string(&result).map_err(|e| e.into())
}

fn run_stage2_config(
    llm_repo: &dyn domain::llm::repository::LLMRepository,
    stage1: &CommitFileClassification,
    file_diffs: &HashMap<String, String>,
    files: &[CommitFileChange],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut config_paths: Vec<&String> = stage1.categories.by_nature.configuration.iter().collect();
    config_paths.extend(stage1.categories.by_nature.documentation.iter());
    if config_paths.is_empty() {
        return Ok("{}".to_string());
    }
    let mut parts = String::new();
    for path in config_paths {
        let additions =
            files.iter().find(|f| f.path == *path).and_then(|f| f.additions).unwrap_or(0);
        let deletions =
            files.iter().find(|f| f.path == *path).and_then(|f| f.deletions).unwrap_or(0);
        let diff = file_diffs.get(path).map(String::as_str).unwrap_or("");
        parts.push_str(&format!(
            "\n### {}\n变更：+{} -{}\n\n```diff\n{}\n```\n\n---\n",
            path, additions, deletions, diff
        ));
    }
    let user_prompt = format!("## 修改文件\n{}\n", parts);
    let result: CommitConfigAnalysis = llm_repo.analyze_commit_config(&user_prompt)?;
    serde_json::to_string(&result).map_err(|e| e.into())
}

fn run_stage2_tests(
    llm_repo: &dyn domain::llm::repository::LLMRepository,
    stage1: &CommitFileClassification,
    file_diffs: &HashMap<String, String>,
    _files: &[CommitFileChange],
) -> Result<String, Box<dyn std::error::Error>> {
    let test_paths = &stage1.categories.by_nature.tests;
    if test_paths.is_empty() {
        return Ok("{}".to_string());
    }
    let mut combined = String::new();
    for path in test_paths {
        let diff = file_diffs.get(path).map(String::as_str).unwrap_or("");
        combined.push_str(&format!("\n### {}\n\n```diff\n{}\n```\n\n", path, diff));
    }
    let user_prompt = format!("## 测试文件变更\n{}\n", combined);
    let result: CommitTestAnalysis = llm_repo.analyze_commit_tests(&user_prompt)?;
    serde_json::to_string(&result).map_err(|e| e.into())
}

fn status_to_str(t: CommitChangeType) -> &'static str {
    match t {
        CommitChangeType::Added => "added",
        CommitChangeType::Modified => "modified",
        CommitChangeType::Deleted => "deleted",
        CommitChangeType::Renamed => "renamed",
        CommitChangeType::Copied => "copied",
        CommitChangeType::TypeChanged => "type_changed",
    }
}
