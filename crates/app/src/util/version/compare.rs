// ============================================================================
// 版本比较
// ============================================================================

/// 版本比较结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    /// 当前版本已是最新
    UpToDate,
    /// 需要更新
    NeedsUpdate,
    /// 当前版本更新（降级）
    Downgrade,
}

/// 比较两个版本号
///
/// 返回版本比较结果。
pub fn compare_versions(current: impl AsRef<str>, target: impl AsRef<str>) -> VersionComparison {
    let current_parts: Vec<u32> =
        current.as_ref().split('.').filter_map(|s| s.parse().ok()).collect();
    let target_parts: Vec<u32> =
        target.as_ref().split('.').filter_map(|s| s.parse().ok()).collect();

    // 补齐到相同长度
    let max_len = current_parts.len().max(target_parts.len());
    let mut current_padded = current_parts;
    let mut target_padded = target_parts;
    current_padded.resize(max_len, 0);
    target_padded.resize(max_len, 0);

    // 逐级比较
    for (c, t) in current_padded.iter().zip(target_padded.iter()) {
        if c < t {
            return VersionComparison::NeedsUpdate;
        } else if c > t {
            return VersionComparison::Downgrade;
        }
    }

    VersionComparison::UpToDate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions_equal() {
        assert_eq!(
            compare_versions("1.0.0", "1.0.0"),
            VersionComparison::UpToDate
        );
        assert_eq!(
            compare_versions("1.2.3", "1.2.3"),
            VersionComparison::UpToDate
        );
    }

    #[test]
    fn test_compare_versions_needs_update() {
        assert_eq!(
            compare_versions("1.0.0", "1.0.1"),
            VersionComparison::NeedsUpdate
        );
        assert_eq!(
            compare_versions("1.0.0", "2.0.0"),
            VersionComparison::NeedsUpdate
        );
        assert_eq!(
            compare_versions("1.2.3", "1.3.0"),
            VersionComparison::NeedsUpdate
        );
    }

    #[test]
    fn test_compare_versions_downgrade() {
        assert_eq!(
            compare_versions("1.0.1", "1.0.0"),
            VersionComparison::Downgrade
        );
        assert_eq!(
            compare_versions("2.0.0", "1.0.0"),
            VersionComparison::Downgrade
        );
    }

    #[test]
    fn test_compare_versions_different_lengths() {
        assert_eq!(
            compare_versions("1.0", "1.0.0"),
            VersionComparison::UpToDate
        );
        assert_eq!(
            compare_versions("1.0.0", "1.0"),
            VersionComparison::UpToDate
        );
        assert_eq!(
            compare_versions("1.0", "1.0.1"),
            VersionComparison::NeedsUpdate
        );
    }
}
