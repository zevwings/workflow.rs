//! 仓库相关实体

use serde::{Deserialize, Serialize};

/// 代码托管平台类型
///
/// 用于标识远程仓库的类型，以便使用不同的 API 或处理逻辑。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodePlatform {
    /// GitHub 仓库
    GitHub,
    /// CNB (Code & Build) 仓库
    CNB,
    /// Codeup 仓库（检测支持，但 PR 功能不支持）
    Codeup,
    /// 未知类型的仓库
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCodePlatformError(String);

impl std::fmt::Display for ParseCodePlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid code platform: {}", self.0)
    }
}

impl std::error::Error for ParseCodePlatformError {}

impl CodePlatform {
    /// 获取平台的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            CodePlatform::GitHub => "GitHub",
            CodePlatform::CNB => "CNB (Code & Build)",
            CodePlatform::Codeup => "Codeup",
            CodePlatform::Unknown => "Unknown",
        }
    }

    /// 获取平台的标识符（用于配置文件）
    pub fn identifier(&self) -> &'static str {
        match self {
            CodePlatform::GitHub => "github",
            CodePlatform::CNB => "cnb",
            CodePlatform::Codeup => "codeup",
            CodePlatform::Unknown => "unknown",
        }
    }

    /// 获取所有已实现的平台（不包括 Unknown）
    pub fn implemented() -> Vec<CodePlatform> {
        vec![CodePlatform::GitHub, CodePlatform::CNB]
    }

    /// 检查平台是否已完全实现（支持完整的 PR 功能）
    pub fn is_fully_implemented(&self) -> bool {
        matches!(self, CodePlatform::GitHub)
    }

    /// 从字符串解析平台类型（不包括 Unknown）
    fn parse_identifier(s: &str) -> Option<CodePlatform> {
        match s.to_lowercase().as_str() {
            "github" => Some(CodePlatform::GitHub),
            "cnb" | "code & build" => Some(CodePlatform::CNB),
            "codeup" => Some(CodePlatform::Codeup),
            _ => None,
        }
    }
}

impl std::fmt::Display for CodePlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for CodePlatform {
    type Err = ParseCodePlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CodePlatform::parse_identifier(s).ok_or_else(|| ParseCodePlatformError(s.to_owned()))
    }
}

/// Git 仓库信息
///
/// 包含仓库的所有基本信息，通过 `get_repo_info()` 方法一次性获取。
#[derive(Debug, Clone)]
pub struct RepoInfo {
    /// 是否为 Git 仓库
    pub is_valid: bool,
    /// 仓库类型（如果无法检测则为 None）
    pub kind: Option<CodePlatform>,
    /// origin 远程仓库 URL（如果无法获取则为 None）
    pub origin_url: Option<String>,
    /// Git 目录路径（.git 目录的绝对路径，如果无法获取则为 None）
    pub directory: Option<String>,
    /// 仓库名称（owner/repo 格式，如果无法提取则为 None）
    pub name: Option<String>,
    /// 仓库所有者（owner，如果无法提取则为 None）
    pub owner: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_code_platform_display_name() {
        assert_eq!(CodePlatform::GitHub.display_name(), "GitHub");
        assert_eq!(CodePlatform::CNB.display_name(), "CNB (Code & Build)");
        assert_eq!(CodePlatform::Codeup.display_name(), "Codeup");
        assert_eq!(CodePlatform::Unknown.display_name(), "Unknown");
    }

    #[test]
    fn test_code_platform_identifier() {
        assert_eq!(CodePlatform::GitHub.identifier(), "github");
        assert_eq!(CodePlatform::CNB.identifier(), "cnb");
        assert_eq!(CodePlatform::Codeup.identifier(), "codeup");
        assert_eq!(CodePlatform::Unknown.identifier(), "unknown");
    }

    #[test]
    fn test_code_platform_is_fully_implemented() {
        assert!(CodePlatform::GitHub.is_fully_implemented());
        assert!(!CodePlatform::CNB.is_fully_implemented());
        assert!(!CodePlatform::Codeup.is_fully_implemented());
        assert!(!CodePlatform::Unknown.is_fully_implemented());
    }

    #[test]
    fn test_code_platform_from_str() {
        assert_eq!(
            "github".parse::<CodePlatform>().ok(),
            Some(CodePlatform::GitHub)
        );
        assert_eq!(
            "GitHub".parse::<CodePlatform>().ok(),
            Some(CodePlatform::GitHub)
        );
        assert_eq!("cnb".parse::<CodePlatform>().ok(), Some(CodePlatform::CNB));
        assert_eq!("CNB".parse::<CodePlatform>().ok(), Some(CodePlatform::CNB));
        assert_eq!(
            "code & build".parse::<CodePlatform>().ok(),
            Some(CodePlatform::CNB)
        );
        assert_eq!(
            "codeup".parse::<CodePlatform>().ok(),
            Some(CodePlatform::Codeup)
        );
        assert!("invalid".parse::<CodePlatform>().is_err());
    }

    #[test]
    fn test_code_platform_implemented() {
        let implemented = CodePlatform::implemented();
        assert_eq!(implemented.len(), 2);
        assert!(implemented.contains(&CodePlatform::GitHub));
        assert!(implemented.contains(&CodePlatform::CNB));
    }

    #[test]
    fn test_code_platform_display() {
        assert_eq!(format!("{}", CodePlatform::GitHub), "GitHub");
        assert_eq!(format!("{}", CodePlatform::CNB), "CNB (Code & Build)");
        assert_eq!(format!("{}", CodePlatform::Codeup), "Codeup");
    }
}
