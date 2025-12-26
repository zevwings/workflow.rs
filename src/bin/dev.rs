//! Dev 工具命令入口
//!
//! 提供文档检查、测试检查、报告生成等开发工具功能。

use clap::Parser;
use color_eyre::Result;

use workflow::cli::{
    DevCommands, DevSubcommand, DocsSubcommand, DocsCheckSubcommand,
    TestsSubcommand, TestsCheckSubcommand, TestsDocsSubcommand, TestsMetricsSubcommand,
    TestsReportSubcommand, TestsTrendsSubcommand, PerformanceSubcommand,
    VersionSubcommand, CiSubcommand, ChecksumSubcommand, HomebrewSubcommand,
    TagSubcommand, PrSubcommand,
};
use workflow::commands::dev::checksum;
use workflow::commands::dev::ci;
use workflow::commands::dev::docs;
use workflow::commands::dev::homebrew;
use workflow::commands::dev::performance;
use workflow::commands::dev::pr;
use workflow::commands::dev::tag;
use workflow::commands::dev::tests;
use workflow::commands::dev::version;

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = DevCommands::parse();

    match cli.command {
        DevSubcommand::Docs { command } => {
            match command {
                DocsSubcommand::Check { target } => {
                    match target {
                        DocsCheckSubcommand::Integrity { architecture, timestamps, ci } => {
                            let cmd = docs::DocsIntegrityCheckCommand::new(architecture, timestamps, ci);
                            cmd.check()?;
                        }
                        DocsCheckSubcommand::Links { external, ci } => {
                            let cmd = docs::DocsLinksCheckCommand::new(external, ci);
                            cmd.check()?;
                        }
                    }
                }
                DocsSubcommand::Report { command } => {
                    match command {
                        workflow::cli::DocsReportSubcommand::Generate { output, check_type } => {
                            let cmd = docs::DocsReportGenerateCommand::new(output, check_type);
                            cmd.generate()?;
                        }
                    }
                }
            }
        }
        DevSubcommand::Tests { command } => {
            match command {
                TestsSubcommand::Check { command } => {
                    match command {
                        TestsCheckSubcommand::Coverage { threshold, ci, output, check_type } => {
                            let cmd = tests::TestsCoverageCheckCommand::new(threshold, ci, output, check_type);
                            cmd.check()?;
                        }
                    }
                }
                TestsSubcommand::Docs { command } => {
                    match command {
                        TestsDocsSubcommand::Check { ci } => {
                            let cmd = tests::TestsDocsCheckCommand::new(ci);
                            cmd.check()?;
                        }
                    }
                }
                TestsSubcommand::Metrics { command } => {
                    match command {
                        TestsMetricsSubcommand::Collect { report, output } => {
                            let cmd = tests::TestsMetricsCollectCommand::new(report, output);
                            cmd.collect()?;
                        }
                    }
                }
                TestsSubcommand::Report { command } => {
                    match command {
                        TestsReportSubcommand::Generate { format, output, comment, report: reports } => {
                            let cmd = tests::TestsReportGenerateCommand::new(format, output, comment, reports);
                            cmd.generate()?;
                        }
                    }
                }
                TestsSubcommand::Trends { command } => {
                    match command {
                        TestsTrendsSubcommand::Analyze { metrics_dir, output } => {
                            let cmd = tests::TestsTrendsAnalyzeCommand::new(metrics_dir, output);
                            cmd.analyze()?;
                        }
                    }
                }
            }
        }
        DevSubcommand::Performance { command } => {
            match command {
                PerformanceSubcommand::Analyze { current, baseline, output, threshold } => {
                    let cmd = performance::PerformanceAnalyzeCommand::new(current, baseline, output, threshold);
                    cmd.analyze()?;
                }
            }
        }
        DevSubcommand::Version { command } => {
            match command {
                VersionSubcommand::Generate { master, update, ci } => {
                    let command = version::VersionGenerateCommand::new(master, update, ci);
                    command.generate()?;
                }
            }
        }
        DevSubcommand::Ci { command } => {
            match command {
                CiSubcommand::CheckSkip { branch, pr_creator, expected_user, ci } => {
                    let cmd = ci::CiSkipCommand::new(branch, pr_creator, expected_user, ci);
                    cmd.check()?;
                }
                CiSubcommand::Verify { jobs, should_skip } => {
                    let cmd = ci::CiVerifyCommand::new(jobs, should_skip);
                    cmd.verify()?;
                }
            }
        }
        DevSubcommand::Checksum { command } => {
            match command {
                ChecksumSubcommand::Calculate { file, output } => {
                    let cmd = checksum::ChecksumCalculateCommand::new(file, output);
                    cmd.calculate()?;
                }
            }
        }
        DevSubcommand::Homebrew { command } => {
            match command {
                HomebrewSubcommand::Update {
                    version,
                    tag,
                    formula_path,
                    template_path,
                    repo,
                    commit,
                    push,
                } => {
                    let cmd = homebrew::HomebrewUpdateCommand::new(
                        version, tag, formula_path, template_path, repo, commit, push,
                    );
                    cmd.update()?;
                }
            }
        }
        DevSubcommand::Tag { command } => {
            match command {
                TagSubcommand::Create { tag, commit, ci } => {
                    let cmd = tag::TagCreateCommand::new(tag, commit, ci);
                    cmd.create()?;
                }
                TagSubcommand::Cleanup { merge_commit, version, ci } => {
                    let cmd = tag::TagCleanupCommand::new(merge_commit, version, ci);
                    cmd.cleanup()?;
                }
            }
        }
        DevSubcommand::Pr { command } => {
            match command {
                PrSubcommand::Create { version, branch, base, ci } => {
                    let cmd = pr::PrCreateCommand::new(version, branch, base, ci);
                    cmd.create()?;
                }
                PrSubcommand::Merge {
                    pr_number,
                    max_wait,
                    initial_interval,
                    normal_interval,
                    commit_title,
                    commit_message,
                    ci,
                } => {
                    let cmd = pr::PrMergeCommand::new(
                        pr_number,
                        max_wait,
                        initial_interval,
                        normal_interval,
                        commit_title,
                        commit_message,
                        ci,
                    );
                    cmd.merge()?;
                }
            }
        }
    }

    Ok(())
}

