pub mod clean;
pub mod download;
pub mod find;
pub mod info;
pub mod search;

// 重新导出所有命令，方便外部使用
// 这些导出被 bin/qk.rs 使用，但 Clippy 在库级别检查时无法检测到
#[allow(unused_imports)]
pub use clean::CleanCommand;
#[allow(unused_imports)]
pub use download::DownloadCommand;
#[allow(unused_imports)]
pub use find::FindCommand;
#[allow(unused_imports)]
pub use info::InfoCommand;
#[allow(unused_imports)]
pub use search::SearchCommand;
