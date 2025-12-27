//! 版本号生成实现

use color_eyre::{eyre::WrapErr, Result};
use regex::Regex;
use std::cmp::Ordering;

use crate::git::{GitCommand, GitCommit, GitTag};
use crate::{log_info, log_success, log_warning};

/// 版本号信息
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// 版本号（如 "1.6.0"）
    pub version: String,
    /// Tag 名称（如 "v1.6.0"）
    pub tag: String,
    /// 是否需要递增版本号
    pub needs_increment: bool,
}

/// 版本递增类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VersionIncrementType {
    Major,
    Minor,
    Patch,
}

/// 版本号生成命令
pub struct VersionGenerateCommand {
    is_master: bool,
    update_cargo: bool,
    ci_mode: bool,
}

impl VersionGenerateCommand {
    /// 创建新的版本号生成命令
    pub fn new(is_master: bool, update_cargo: bool, ci_mode: bool) -> Self {
        Self {
            is_master,
            update_cargo,
            ci_mode,
        }
    }

    /// 生成版本号
    pub fn generate(&self) -> Result<VersionInfo> {
        // 获取最新版本
        let latest_version = self.get_latest_version()?;
        log_info!("📋 Version generation inputs:");
        log_info!("   LATEST_VERSION: {}", latest_version.version);
        log_info!("   IS_MASTER: {}", self.is_master);

        let version_info = if self.is_master {
            self.generate_master_version(&latest_version)?
        } else {
            self.generate_prerelease_version(&latest_version)?
        };

        log_success!(
            "Generated version {} ({})",
            version_info.version,
            version_info.tag
        );

        if self.update_cargo {
            self.update_cargo_files(&version_info.version)?;
        }

        if self.ci_mode {
            self.output_github_actions(&version_info)?;
        }

        Ok(version_info)
    }

    /// 获取最新版本号
    fn get_latest_version(&self) -> Result<VersionInfo> {
        // 获取所有标准版本 tag（格式：vx.x.x），排除 alpha/beta 预发布版本
        let tags = GitTag::list_local_tags()?;

        // 过滤标准版本 tag（格式：vx.x.x）
        let version_regex = Regex::new(r"^v(\d+)\.(\d+)\.(\d+)$")?;
        let mut version_tags: Vec<(String, (u32, u32, u32))> = tags
            .iter()
            .filter_map(|tag| {
                if let Some(caps) = version_regex.captures(tag) {
                    let major = caps.get(1)?.as_str().parse::<u32>().ok()?;
                    let minor = caps.get(2)?.as_str().parse::<u32>().ok()?;
                    let patch = caps.get(3)?.as_str().parse::<u32>().ok()?;
                    Some((tag.clone(), (major, minor, patch)))
                } else {
                    None
                }
            })
            .collect();

        // 按版本号排序（从高到低）
        version_tags.sort_by(|a, b| match b.1 .0.cmp(&a.1 .0) {
            Ordering::Equal => match b.1 .1.cmp(&a.1 .1) {
                Ordering::Equal => b.1 .2.cmp(&a.1 .2),
                other => other,
            },
            other => other,
        });

        if let Some((latest_tag, (major, minor, patch))) = version_tags.first() {
            let version = format!("{}.{}.{}", major, minor, patch);
            log_success!(
                "Latest standard version from git tags: {} ({})",
                latest_tag,
                version
            );
            Ok(VersionInfo {
                version,
                tag: latest_tag.clone(),
                needs_increment: false,
            })
        } else {
            // 如果没有找到标准版本 tag，使用默认版本
            let version = "0.0.0".to_string();
            log_warning!("No standard version tag found, using default: {}", version);
            Ok(VersionInfo {
                version,
                tag: "v0.0.0".to_string(),
                needs_increment: false,
            })
        }
    }

    /// 生成 master 分支版本号
    fn generate_master_version(&self, latest: &VersionInfo) -> Result<VersionInfo> {
        // 解析最新版本号
        let mut parts: Vec<u32> =
            latest.version.split('.').map(|s| s.parse::<u32>().unwrap_or(0)).collect();

        while parts.len() < 3 {
            parts.push(0);
        }

        let mut major = parts[0];
        let mut minor = parts[1];
        let mut patch = parts[2];

        // 检查当前 commit 是否已经有标准版本 tag 指向它
        let current_commit_sha = GitCommit::get_last_commit_sha()?;
        let tags_at_head = self.get_tags_at_commit(&current_commit_sha)?;

        // 查找标准版本 tag
        let version_regex = Regex::new(r"^v(\d+)\.(\d+)\.(\d+)$")?;
        if let Some(existing_tag) = tags_at_head.iter().find(|tag| version_regex.is_match(tag)) {
            // 当前 commit 已经有 tag，使用该 tag 的版本号
            let version = existing_tag.strip_prefix('v').unwrap_or(existing_tag).to_string();
            log_success!(
                "Found existing tag {} on current commit, reusing it",
                existing_tag
            );
            return Ok(VersionInfo {
                version,
                tag: existing_tag.clone(),
                needs_increment: false,
            });
        }

        // 当前 commit 没有 tag，需要根据 Conventional Commits 规范确定版本更新类型
        let latest_tag = &latest.tag;
        let commits = if !latest_tag.is_empty()
            && GitCommand::new(["rev-parse", latest_tag]).quiet_success()
        {
            // 从最新 tag 到当前 commit 的所有提交
            self.get_commits_between(latest_tag, "HEAD")?
        } else {
            // 如果没有找到 tag，使用最近的提交
            GitCommit::get_branch_commits(10)?
        };

        // 根据 Conventional Commits 规范确定版本更新类型
        let increment_type = self.determine_version_increment(&commits, patch)?;

        // 应用版本递增
        match increment_type {
            VersionIncrementType::Major => {
                major += 1;
                minor = 0;
                patch = 0;
                log_info!("🔴 Detected BREAKING CHANGE, incrementing MAJOR version");
            }
            VersionIncrementType::Minor => {
                minor += 1;
                patch = 0;
                log_info!("🟢 Detected feat: commit, incrementing MINOR version");
            }
            VersionIncrementType::Patch => {
                patch += 1;
                log_info!("🔵 No feat: or BREAKING CHANGE detected, incrementing PATCH version");
            }
        }

        let version = format!("{}.{}.{}", major, minor, patch);
        let tag = format!("v{}", version);

        log_success!("Version increment type: {:?}", increment_type);
        log_success!(
            "Generated version {} ({}) based on Conventional Commits",
            version,
            tag
        );

        Ok(VersionInfo {
            version,
            tag,
            needs_increment: true,
        })
    }

    /// 生成预发布版本号
    fn generate_prerelease_version(&self, latest: &VersionInfo) -> Result<VersionInfo> {
        // 解析最新版本号
        let mut parts: Vec<u32> =
            latest.version.split('.').map(|s| s.parse::<u32>().unwrap_or(0)).collect();

        while parts.len() < 3 {
            parts.push(0);
        }

        let mut major = parts[0];
        let mut minor = parts[1];
        let mut patch = parts[2];

        // 获取从最新 tag 到当前 commit 的所有 commit messages
        let latest_tag = &latest.tag;
        let commits = if !latest_tag.is_empty()
            && GitCommand::new(["rev-parse", latest_tag]).quiet_success()
        {
            self.get_commits_between(latest_tag, "HEAD")?
        } else {
            GitCommit::get_branch_commits(10)?
        };

        // 根据 Conventional Commits 规范确定版本更新类型
        let increment_type = self.determine_version_increment(&commits, patch)?;

        // 应用版本递增
        match increment_type {
            VersionIncrementType::Major => {
                major += 1;
                minor = 0;
                patch = 0;
                log_info!("🔴 Detected BREAKING CHANGE, incrementing MAJOR version");
            }
            VersionIncrementType::Minor => {
                minor += 1;
                patch = 0;
                log_info!("🟢 Detected feat: commit, incrementing MINOR version");
            }
            VersionIncrementType::Patch => {
                patch += 1;
                log_info!("🔵 No feat: or BREAKING CHANGE detected, incrementing PATCH version");
            }
        }

        let base_version = format!("{}.{}.{}", major, minor, patch);

        // 使用时间戳格式确保唯一性：YYYYMMDDHHmmssSSS
        // 格式：vx.x.x.alpha-YYYYMMDDHHmmssSSS
        let timestamp = self.get_timestamp()?;
        let version = format!("{}.alpha-{}", base_version, timestamp);
        let tag = format!("v{}", version);

        log_success!(
            "Non-master branch: Generated pre-release version {} ({})",
            version,
            tag
        );
        log_info!("   Timestamp format: YYYYMMDDHHmmssSSS");
        log_info!("   Example: v1.6.1.alpha-20251216101712000");

        Ok(VersionInfo {
            version,
            tag,
            needs_increment: false,
        })
    }

    /// 确定版本递增类型
    fn determine_version_increment(
        &self,
        commits: &[crate::git::CommitInfo],
        current_patch: u32,
    ) -> Result<VersionIncrementType> {
        // 优先级：BREAKING CHANGE > patch >= 9 > feat: > 其他
        let mut has_breaking = false;
        let mut has_feat = false;

        for commit in commits {
            let message = &commit.message;

            // 检查 BREAKING CHANGE 或 BREAKING:
            if message.contains("BREAKING CHANGE") || message.contains("BREAKING:") {
                has_breaking = true;
            }

            // 检查 ! 标记（BREAKING CHANGE 的简写）
            if message.contains('!') && message.matches(':').count() > 0 {
                // 检查格式：type! 或 type(scope)!:
                if let Some(colon_pos) = message.find(':') {
                    let before_colon = &message[..colon_pos];
                    if before_colon.ends_with('!') {
                        has_breaking = true;
                    }
                }
            }

            // 检查 feat: 或 feature:
            if message.starts_with("feat:") || message.starts_with("feature:") {
                has_feat = true;
            }
        }

        if has_breaking {
            return Ok(VersionIncrementType::Major);
        }

        // 规则：如果 patch 版本达到 9，自动递增 minor 版本（如 v1.5.9 → v1.6.0）
        if current_patch >= 9 {
            log_warning!("Patch version reached 9, incrementing MINOR version");
            return Ok(VersionIncrementType::Minor);
        }

        if has_feat {
            return Ok(VersionIncrementType::Minor);
        }

        Ok(VersionIncrementType::Patch)
    }

    /// 获取指定 commit 的所有 tag
    fn get_tags_at_commit(&self, commit_sha: &str) -> Result<Vec<String>> {
        let output = GitCommand::new(["tag", "--points-at", commit_sha]).read()?;
        if output.trim().is_empty() {
            return Ok(Vec::new());
        }
        Ok(output.lines().map(|s| s.trim().to_string()).collect())
    }

    /// 获取两个 commit 之间的所有 commits
    fn get_commits_between(&self, from: &str, to: &str) -> Result<Vec<crate::git::CommitInfo>> {
        let output = GitCommand::new([
            "log",
            &format!("{}..{}", from, to),
            "--format=%H|%s|%an <%ae>|%ai",
            "--no-merges",
        ])
        .read()?;

        if output.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut commits = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 4 {
                commits.push(crate::git::CommitInfo {
                    sha: parts[0].trim().to_string(),
                    message: parts[1].trim().to_string(),
                    author: parts[2].trim().to_string(),
                    date: parts[3].trim().to_string(),
                });
            }
        }

        Ok(commits)
    }

    /// 获取时间戳（YYYYMMDDHHmmssSSS）
    fn get_timestamp(&self) -> Result<String> {
        use chrono::{Datelike, Timelike, Utc};

        let now = Utc::now();

        // 格式：YYYYMMDDHHmmssSSS
        let formatted = format!(
            "{:04}{:02}{:02}{:02}{:02}{:02}{:03}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_subsec_millis()
        );

        Ok(formatted)
    }

    /// 更新 Cargo.toml 和 Cargo.lock
    fn update_cargo_files(&self, version: &str) -> Result<()> {
        use std::fs;
        use std::path::Path;

        let cargo_toml_path = Path::new("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err(color_eyre::eyre::eyre!("Cargo.toml not found"));
        }

        let content = fs::read_to_string(cargo_toml_path).wrap_err("Failed to read Cargo.toml")?;

        // 更新版本号（简单替换，可能需要更复杂的解析）
        let version_regex = Regex::new(r#"version\s*=\s*"[^"]+""#)?;
        let updated = version_regex.replace(&content, &format!(r#"version = "{}""#, version));

        fs::write(cargo_toml_path, updated.as_ref()).wrap_err("Failed to write Cargo.toml")?;

        log_success!("Updated Cargo.toml to version {}", version);

        // 运行 cargo update 更新 Cargo.lock
        GitCommand::new(["cargo", "update", "--workspace"])
            .run()
            .wrap_err("Failed to update Cargo.lock")?;

        log_success!("Updated Cargo.lock");

        Ok(())
    }

    /// 输出到 GitHub Actions GITHUB_OUTPUT
    fn output_github_actions(&self, info: &VersionInfo) -> Result<()> {
        use std::env;
        use std::fs::OpenOptions;
        use std::io::Write;

        let output_file = env::var("GITHUB_OUTPUT")
            .ok()
            .ok_or_else(|| color_eyre::eyre::eyre!("GITHUB_OUTPUT not set"))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file)
            .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

        writeln!(file, "version={}", info.version)
            .wrap_err("Failed to write version to GITHUB_OUTPUT")?;
        writeln!(file, "tag={}", info.tag).wrap_err("Failed to write tag to GITHUB_OUTPUT")?;
        writeln!(file, "needs_increment={}", info.needs_increment)
            .wrap_err("Failed to write needs_increment to GITHUB_OUTPUT")?;

        log_success!("Output version info to GITHUB_OUTPUT");

        Ok(())
    }
}
