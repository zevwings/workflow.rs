//! 模糊匹配过滤器
//!
//! 提供用于 select 和 multiselect 的模糊匹配功能。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use workflow::prompt::dialog::FuzzyFilter;
//!
//! let options = vec!["feature/user-auth", "feature/payment", "bugfix/login"];
//! let filter = FuzzyFilter::new();
//!
//! // 过滤选项
//! let (indices, filtered) = filter.filter(&options, "feat");
//! // indices: [0, 1] - 匹配项的原始索引
//! // filtered: [&"feature/user-auth", &"feature/payment"] - 匹配的选项
//!
//! // 检查单个选项是否匹配
//! if let Some(score) = filter.matches("feature/user-auth", "feat") {
//!     println!("匹配分数: {}", score);
//! }
//! ```

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// 模糊匹配过滤器
///
/// 用于对选项列表进行模糊匹配过滤，支持实时搜索和排序
pub struct FuzzyFilter {
    matcher: SkimMatcherV2,
}

impl FuzzyFilter {
    /// 创建新的模糊匹配过滤器
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// 过滤选项列表
    ///
    /// # 参数
    /// - `options`: 选项列表的引用
    /// - `query`: 搜索查询字符串
    ///
    /// # 返回
    /// 返回一个元组 `(原始索引列表, 过滤后的选项引用列表)`
    /// - 原始索引列表：过滤后选项对应的原始索引（按匹配分数降序排序）
    /// - 过滤后的选项引用列表：匹配的选项（按匹配分数降序排序）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::prompt::dialog::FuzzyFilter;
    ///
    /// let options = vec!["feature/user-auth", "feature/payment", "bugfix/login"];
    /// let filter = FuzzyFilter::new();
    /// let (indices, filtered) = filter.filter(&options, "feat");
    /// // indices: [0, 1] (feature/user-auth 和 feature/payment 的原始索引)
    /// // filtered: [&"feature/user-auth", &"feature/payment"]
    /// ```
    pub fn filter<'a, T: std::fmt::Display>(
        &self,
        options: &'a [T],
        query: &str,
    ) -> (Vec<usize>, Vec<&'a T>) {
        if query.is_empty() {
            // 没有搜索查询，返回所有选项
            let indices: Vec<usize> = (0..options.len()).collect();
            let filtered: Vec<&T> = options.iter().collect();
            return (indices, filtered);
        }

        // 使用模糊匹配过滤选项
        let mut scored_options: Vec<(usize, i64, &T)> = options
            .iter()
            .enumerate()
            .filter_map(|(idx, option)| {
                let option_str = option.to_string();
                self.matcher.fuzzy_match(&option_str, query).map(|score| (idx, score, option))
            })
            .collect();

        // 按分数降序排序（分数越高，匹配越好）
        scored_options.sort_by(|a, b| b.1.cmp(&a.1));

        let indices: Vec<usize> = scored_options.iter().map(|(idx, _, _)| *idx).collect();
        let filtered: Vec<&T> = scored_options.iter().map(|(_, _, option)| *option).collect();

        (indices, filtered)
    }

    /// 检查查询是否匹配选项
    ///
    /// # 参数
    /// - `option`: 选项文本
    /// - `query`: 搜索查询字符串
    ///
    /// # 返回
    /// 如果匹配返回 `Some(分数)`，否则返回 `None`
    pub fn matches(&self, option: &str, query: &str) -> Option<i64> {
        if query.is_empty() {
            return Some(1000); // 空查询匹配所有选项
        }
        self.matcher.fuzzy_match(option, query)
    }
}

impl Default for FuzzyFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_filter_empty_query() {
        let filter = FuzzyFilter::new();
        let options = vec!["option1", "option2", "option3"];
        let (indices, filtered) = filter.filter(&options, "");

        assert_eq!(indices, vec![0, 1, 2]);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_filter_with_query() {
        let filter = FuzzyFilter::new();
        let options = vec!["feature/user-auth", "feature/payment", "bugfix/login"];
        let (indices, filtered) = filter.filter(&options, "feat");

        // 应该匹配前两个选项
        assert_eq!(indices.len(), 2);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&&"feature/user-auth"));
        assert!(filtered.contains(&&"feature/payment"));
    }

    #[test]
    fn test_filter_no_match() {
        let filter = FuzzyFilter::new();
        let options = vec!["option1", "option2", "option3"];
        let (indices, filtered) = filter.filter(&options, "xyz");

        assert_eq!(indices.len(), 0);
        assert_eq!(filtered.len(), 0);
    }

    #[rstest]
    #[case("feature/user-auth", "feat", true)]
    #[case("feature/user-auth", "xyz", false)]
    #[case("feature/user-auth", "", true)]
    fn test_matches(#[case] option: &str, #[case] query: &str, #[case] should_match: bool) {
        let filter = FuzzyFilter::new();
        let result = filter.matches(option, query);
        assert_eq!(result.is_some(), should_match);
    }
}
