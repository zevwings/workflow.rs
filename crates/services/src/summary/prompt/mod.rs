/// 提交文件分类 prompt（阶段一）
pub const fn classify_files() -> &'static str {
    include_str!("classify_files.md")
}

/// 阶段二 2.1：批量操作分析 prompt
pub const fn analyze_batch() -> &'static str {
    include_str!("analyze_batch.md")
}

/// 阶段二 2.2：核心逻辑分析 prompt
pub const fn analyze_logic() -> &'static str {
    include_str!("analyze_logic.md")
}

/// 阶段二 2.3：配置/文档分析 prompt
pub const fn analyze_config() -> &'static str {
    include_str!("analyze_config.md")
}

/// 阶段二 2.4：测试文件分析 prompt
pub const fn analyze_tests() -> &'static str {
    include_str!("analyze_tests.md")
}

/// 阶段三：全局总结 prompt
pub const fn summary() -> &'static str {
    include_str!("summary.md")
}
