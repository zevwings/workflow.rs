pub mod clean;
pub mod download;
pub mod find;
pub mod info;
pub mod search;

// 重新导出所有命令，方便外部使用
// 这些导出被 `bin/qk.rs` 使用（见 `src/bin/qk.rs:12-14`），
// 但 Clippy 在库级别检查时无法检测到二进制文件中的使用，因此需要 `allow` 注释
#[allow(unused_imports)] // 被 bin/qk.rs 使用
pub use clean::CleanCommand;
#[allow(unused_imports)] // 被 bin/qk.rs 使用
pub use download::DownloadCommand;
#[allow(unused_imports)] // 被 bin/qk.rs 使用
pub use find::FindCommand;
#[allow(unused_imports)] // 被 bin/qk.rs 使用
pub use info::InfoCommand;
#[allow(unused_imports)] // 被 bin/qk.rs 使用
pub use search::SearchCommand;
