//! Dev 工具 CLI 定义
//!
//! 定义 `dev` 命令的所有子命令结构。

use clap::Parser;

/// Dev 工具命令主结构体
#[derive(Parser)]
#[command(name = "dev")]
#[command(about = "开发工具命令", long_about = None)]
pub struct DevCommands {
    #[command(subcommand)]
    pub command: DevSubcommand,
}

/// Dev 工具子命令枚举
#[derive(clap::Subcommand)]
pub enum DevSubcommand {
    /// 文档相关工具
    ///
    /// 提供文档检查、链接检查等功能。
    Docs {
        #[command(subcommand)]
        command: DocsSubcommand,
    },
    /// 测试相关工具（复数）
    ///
    /// 提供测试检查、覆盖率检查、指标收集、报告生成等功能。
    Tests {
        #[command(subcommand)]
        command: TestsSubcommand,
    },
    /// 性能相关工具
    ///
    /// 提供性能分析等功能。
    Performance {
        #[command(subcommand)]
        command: PerformanceSubcommand,
    },
    /// 版本号相关工具
    ///
    /// 提供版本号生成等功能。
    Version {
        #[command(subcommand)]
        command: VersionSubcommand,
    },
    /// CI 相关工具
    ///
    /// 提供 CI 跳过检查、验证等功能。
    Ci {
        #[command(subcommand)]
        command: CiSubcommand,
    },
    /// 文件哈希计算工具
    ///
    /// 提供文件 SHA256 哈希计算等功能。
    Checksum {
        #[command(subcommand)]
        command: ChecksumSubcommand,
    },
    /// Homebrew Formula 更新工具
    ///
    /// 提供 Homebrew Formula 文件更新等功能。
    Homebrew {
        #[command(subcommand)]
        command: HomebrewSubcommand,
    },
    /// Git Tag 相关工具
    ///
    /// 提供 Git tag 创建、清理等功能。
    Tag {
        #[command(subcommand)]
        command: DevTagSubcommand,
    },
    /// PR 相关工具
    ///
    /// 提供 PR 创建、合并等功能。
    Pr {
        #[command(subcommand)]
        command: PrSubcommand,
    },
}

/// 文档相关子命令
#[derive(clap::Subcommand)]
pub enum DocsSubcommand {
    /// 检查相关操作
    ///
    /// 提供文档完整性检查、链接检查等功能。
    Check {
        #[command(subcommand)]
        target: DocsCheckSubcommand,
    },
    /// 报告生成操作
    ///
    /// 生成文档检查报告。
    Report {
        #[command(subcommand)]
        command: DocsReportSubcommand,
    },
}

/// 文档检查子命令
#[derive(clap::Subcommand)]
pub enum DocsCheckSubcommand {
    /// 检查文档完整性
    ///
    /// 检查项目文档的完整性和格式：
    /// - 架构文档存在性检查
    /// - 文档时间戳格式检查
    Integrity {
        /// 只检查架构文档存在性
        #[arg(long)]
        architecture: bool,
        /// 只检查文档时间戳格式
        #[arg(long)]
        timestamps: bool,
        /// CI 模式（非阻塞退出，输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
    },
    /// 检查文档链接
    ///
    /// 检查文档中的链接有效性：
    /// - 内部链接检查
    /// - 外部链接检查（可选）
    Links {
        /// 检查外部链接（需要 lychee）
        #[arg(long)]
        external: bool,
        /// CI 模式（非阻塞退出，输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
    },
}

/// 文档报告相关子命令
#[derive(clap::Subcommand)]
pub enum DocsReportSubcommand {
    /// 生成文档检查报告
    ///
    /// 生成文档检查报告（Markdown 格式）。
    Generate {
        /// 输出文件路径
        #[arg(short, long)]
        output: Option<String>,
        /// 检查类型（默认：定期审查）
        #[arg(long)]
        check_type: Option<String>,
    },
}

/// 测试相关子命令
#[derive(clap::Subcommand)]
pub enum TestsSubcommand {
    /// 测试检查相关
    ///
    /// 提供测试覆盖率检查等功能。
    Check {
        #[command(subcommand)]
        command: TestsCheckSubcommand,
    },
    /// 测试文档相关
    ///
    /// 提供测试文档检查等功能。
    Docs {
        #[command(subcommand)]
        command: TestsDocsSubcommand,
    },
    /// 测试指标相关
    ///
    /// 提供测试指标收集等功能。
    Metrics {
        #[command(subcommand)]
        command: TestsMetricsSubcommand,
    },
    /// 报告生成操作
    ///
    /// 提供测试报告生成、PR 评论生成等功能。
    Report {
        #[command(subcommand)]
        command: TestsReportSubcommand,
    },
    /// 趋势分析操作
    ///
    /// 提供测试趋势分析等功能。
    Trends {
        #[command(subcommand)]
        command: TestsTrendsSubcommand,
    },
}

/// 测试检查相关子命令
#[derive(clap::Subcommand)]
pub enum TestsCheckSubcommand {
    /// 检查测试覆盖率
    ///
    /// 检查测试覆盖率是否达到目标阈值。
    Coverage {
        /// 覆盖率阈值（默认 80.0）
        #[arg(long)]
        threshold: Option<f64>,
        /// CI 模式（非阻塞退出，输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
        /// 输出报告文件路径（可选，Markdown 格式）
        #[arg(short, long)]
        output: Option<String>,
        /// 检查类型（默认：定期审查）
        #[arg(long)]
        check_type: Option<String>,
    },
}

/// 测试文档相关子命令
#[derive(clap::Subcommand)]
pub enum TestsDocsSubcommand {
    /// 检查测试文档
    ///
    /// 检查测试函数的文档注释完成情况。
    Check {
        /// CI 模式（非阻塞退出，输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
    },
}

/// 测试指标相关子命令
#[derive(clap::Subcommand)]
pub enum TestsMetricsSubcommand {
    /// 收集测试指标
    ///
    /// 收集测试执行指标并生成报告。
    Collect {
        /// 报告文件路径
        #[arg(long)]
        report: Option<String>,
        /// 输出文件路径
        #[arg(long)]
        output: Option<String>,
    },
}

/// 测试报告相关子命令
#[derive(clap::Subcommand)]
pub enum TestsReportSubcommand {
    /// 生成测试报告
    ///
    /// 生成测试执行报告（HTML/JSON/Markdown/JUnit/CSV）。
    Generate {
        /// 输出格式（html, json, markdown, junit, csv）
        #[arg(short, long)]
        format: Option<String>,
        /// 输出文件路径
        #[arg(short, long)]
        output: Option<String>,
        /// 生成报告并添加 PR 评论
        #[arg(long)]
        comment: bool,
        /// 报告文件路径（可多次指定，仅当使用 --comment 时有效）
        #[arg(long)]
        report: Vec<String>,
    },
}

/// 测试趋势相关子命令
#[derive(clap::Subcommand)]
pub enum TestsTrendsSubcommand {
    /// 分析测试趋势
    ///
    /// 分析测试指标的历史数据，生成趋势报告。
    Analyze {
        /// 指标数据目录
        #[arg(long)]
        metrics_dir: Option<String>,
        /// 输出文件路径
        #[arg(long)]
        output: Option<String>,
    },
}

/// 性能相关子命令
#[derive(clap::Subcommand)]
pub enum PerformanceSubcommand {
    /// 分析性能回归
    ///
    /// 对比当前性能与基准性能，检测性能回归。
    Analyze {
        /// 当前性能数据文件
        #[arg(long)]
        current: Option<String>,
        /// 基准性能数据文件
        #[arg(long)]
        baseline: Option<String>,
        /// 输出文件路径
        #[arg(long)]
        output: Option<String>,
        /// 回归阈值（默认 0.2，即 20%）
        #[arg(long)]
        threshold: Option<f64>,
    },
}

/// 版本号相关子命令
#[derive(clap::Subcommand)]
pub enum VersionSubcommand {
    /// 生成版本号
    ///
    /// 根据 Conventional Commits 规范生成版本号：
    /// - 解析 git tags 获取最新版本
    /// - 分析 commit messages（Conventional Commits）
    /// - 根据提交类型确定版本递增策略（major/minor/patch）
    /// - 生成标准版本号或预发布版本号
    Generate {
        /// 是否为 master 分支（影响版本号格式）
        #[arg(long)]
        master: bool,
        /// 更新 Cargo.toml 和 Cargo.lock
        #[arg(long)]
        update: bool,
                /// CI 模式（输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
    },
}

/// 文件哈希计算相关子命令
#[derive(clap::Subcommand)]
pub enum ChecksumSubcommand {
    /// 计算文件哈希
    ///
    /// 计算文件的 SHA256 哈希值。
    Calculate {
        /// 要计算哈希的文件路径
        #[arg(required = true)]
        file: String,
        /// 输出文件路径（可选，如果不提供则输出到标准输出）
        #[arg(short, long)]
        output: Option<String>,
    },
}

/// Homebrew Formula 相关子命令
#[derive(clap::Subcommand)]
pub enum HomebrewSubcommand {
    /// 更新 Formula 文件
    ///
    /// 从模板生成或更新 Formula 文件，更新版本号和下载 URL。
    Update {
        /// 版本号（如 1.6.0）
        #[arg(long, required = true)]
        version: String,
        /// Git tag（如 v1.6.0）
        #[arg(long, required = true)]
        tag: String,
        /// Formula 文件路径（默认：Formula/workflow.rb）
        #[arg(long)]
        formula_path: Option<String>,
        /// 模板文件路径（可选）
        #[arg(long)]
        template_path: Option<String>,
        /// GitHub 仓库（用于生成下载 URL）
        #[arg(long)]
        repo: Option<String>,
        /// 是否提交更改
        #[arg(long)]
        commit: bool,
        /// 是否推送到远程
        #[arg(long)]
        push: bool,
    },
}

/// Git Tag 相关子命令
#[derive(clap::Subcommand)]
pub enum DevTagSubcommand {
    /// 创建 Git tag
    ///
    /// 创建并推送 Git tag 到远程仓库。
    Create {
        /// Tag 名称
        #[arg(required = true)]
        tag: String,
        /// Commit SHA（可选，如果不提供则使用当前 HEAD）
        #[arg(long)]
        commit: Option<String>,
        /// CI 模式（输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
    },
    /// 清理 Alpha tags
    ///
    /// 清理已合并到 master 分支的 alpha tag。
    Cleanup {
        /// 合并提交的 SHA
        #[arg(long, required = true)]
        merge_commit: String,
        /// 当前版本号
        #[arg(long, required = true)]
        version: String,
        /// CI 模式（输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
    },
}

/// PR 相关子命令
#[derive(clap::Subcommand)]
pub enum PrSubcommand {
    /// 创建版本更新 PR
    ///
    /// 创建版本更新 PR，包括创建分支、提交更改、推送分支和创建 PR。
    Create {
        /// 版本号
        #[arg(required = true)]
        version: String,
        /// 分支名称（可选，默认：bump-version-{version}）
        #[arg(long)]
        branch: Option<String>,
        /// 目标分支（可选，默认：master）
        #[arg(long)]
        base: Option<String>,
        /// CI 模式（输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
    },
    /// 合并 PR
    ///
    /// 检查 PR 状态、等待 CI 完成并合并 PR。
    Merge {
        /// PR 编号
        #[arg(required = true)]
        pr_number: u64,
        /// 最大等待时间（秒，默认：300）
        #[arg(long)]
        max_wait: Option<u64>,
        /// 初始检查间隔（秒，默认：3）
        #[arg(long)]
        initial_interval: Option<u64>,
        /// 正常检查间隔（秒，默认：5）
        #[arg(long)]
        normal_interval: Option<u64>,
        /// 提交标题（可选）
        #[arg(long)]
        commit_title: Option<String>,
        /// 提交消息（可选）
        #[arg(long)]
        commit_message: Option<String>,
        /// CI 模式（输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
    },
}

/// CI 相关子命令
#[derive(clap::Subcommand)]
pub enum CiSubcommand {
    /// 检查是否应该跳过 CI
    ///
    /// 检查分支名称（是否为 `bump-version-*`）并验证 PR 创建者。
    CheckSkip {
        /// 分支名称（如果未提供，从 git 获取）
        #[arg(long)]
        branch: Option<String>,
        /// PR 创建者（用于验证）
        #[arg(long)]
        pr_creator: Option<String>,
        /// 预期的用户名称（用于验证）
        #[arg(long)]
        expected_user: Option<String>,
        /// CI 模式（输出到 GITHUB_OUTPUT）
        #[arg(long)]
        ci: bool,
    },
    /// 验证所有 CI 检查
    ///
    /// 验证所有 CI job 的状态，汇总检查结果，输出最终验证状态。
    Verify {
        /// 要验证的 job 列表（逗号分隔，如：lint,tests,doctests,build）
        #[arg(long)]
        jobs: Option<String>,
        /// 是否应该跳过 CI（从 check-skip 输出）
        #[arg(long)]
        should_skip: Option<bool>,
    },
}
