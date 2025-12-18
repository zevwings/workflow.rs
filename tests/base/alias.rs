//! Base/Alias 管理器测试
//!
//! 测试别名管理器的核心业务逻辑，包括：
//! - 别名展开算法（递归展开、循环检测）
//! - 命令行参数处理
//! - 别名验证和管理
//! - 错误处理和边界条件
//!
//! 注意：这些测试主要关注算法逻辑，不依赖实际的配置文件

use std::collections::{HashMap, HashSet};

use color_eyre::Result;
use rstest::rstest;

// 由于 AliasManager 依赖 Settings，我们需要模拟别名数据进行测试
// 这里我们测试核心的展开算法逻辑

/// 模拟别名展开逻辑（不依赖配置文件）
/// 这个函数复制了 AliasManager::expand 的核心逻辑
fn mock_expand_alias(
    alias: &str,
    aliases: &HashMap<String, String>,
    visited: &mut HashSet<String>,
    depth: usize,
) -> Result<String> {
    const MAX_DEPTH: usize = 10;

    // 检查深度限制
    if depth > MAX_DEPTH {
        return Err(color_eyre::eyre::eyre!(
            "Alias expansion depth exceeded maximum: {}",
            MAX_DEPTH
        ));
    }

    // 检查循环引用
    if visited.contains(alias) {
        return Err(color_eyre::eyre::eyre!(
            "Circular alias detected: {}",
            alias
        ));
    }

    // 检查别名是否存在
    let command = aliases
        .get(alias)
        .ok_or_else(|| color_eyre::eyre::eyre!("Alias not found: {}", alias))?;

    // 标记为已访问
    visited.insert(alias.to_string());

    // 检查命令是否包含其他别名（递归展开）
    let parts: Vec<&str> = command.split_whitespace().collect();
    if let Some(first_part) = parts.first() {
        if aliases.contains_key(*first_part) {
            // 如果第一个部分等于当前别名，检查是否真的是循环
            if *first_part == alias {
                // 如果命令完全相同，这是真正的循环
                if command.trim() == alias {
                    return Err(color_eyre::eyre::eyre!(
                        "Circular alias detected: {}",
                        alias
                    ));
                }
                // 如果命令有额外参数（如 "grep --color=auto"），这不是循环
                // 直接返回原命令，不进行进一步展开
                visited.remove(alias);
                return Ok(command.clone());
            }

            // 递归展开嵌套别名
            let expanded = mock_expand_alias(first_part, aliases, visited, depth + 1)?;
            // 将展开后的命令与剩余部分组合
            let mut result: Vec<&str> = expanded.split_whitespace().collect();
            result.extend_from_slice(&parts[1..]);
            // 移除当前别名从 visited 集合，允许在不同分支中重复使用
            visited.remove(alias);
            return Ok(result.join(" "));
        }
    }

    // 移除当前别名从 visited 集合
    visited.remove(alias);
    Ok(command.clone())
}

/// 模拟命令行参数展开逻辑
fn mock_expand_args(args: Vec<String>, aliases: &HashMap<String, String>) -> Result<Vec<String>> {
    // 如果参数少于 2 个（只有程序名），直接返回
    if args.len() < 2 {
        return Ok(args);
    }

    // 获取第一个参数（命令名）
    let first_arg = &args[1];

    // 检查第一个参数是否是别名
    if aliases.contains_key(first_arg) {
        // 展开别名
        let mut visited = HashSet::new();
        let expanded = mock_expand_alias(first_arg, aliases, &mut visited, 0)?;

        // 将展开后的命令分割为参数
        let mut expanded_args: Vec<String> =
            expanded.split_whitespace().map(|s| s.to_string()).collect();

        // 保留原始参数中的程序名和剩余参数
        let mut result = vec![args[0].clone()];
        result.append(&mut expanded_args);
        result.extend_from_slice(&args[2..]);

        Ok(result)
    } else {
        // 不是别名，直接返回原参数
        Ok(args)
    }
}

/// 模拟循环检测逻辑
fn mock_check_circular(
    name: &str,
    target: &str,
    aliases: &HashMap<String, String>,
) -> Result<bool> {
    // 创建一个临时的别名映射，包含要检查的新映射
    let mut temp_aliases = aliases.clone();
    temp_aliases.insert(name.to_string(), target.to_string());

    // 尝试展开 name，看是否会导致循环
    let mut visited = HashSet::new();
    match mock_expand_alias(name, &temp_aliases, &mut visited, 0) {
        Ok(_) => {
            // 如果展开成功，没有循环
            Ok(false)
        }
        Err(e) => {
            // 如果展开失败，检查是否是因为循环引用
            let error_msg = e.to_string();
            if error_msg.contains("Circular alias detected") {
                Ok(true)
            } else {
                // 其他错误不算循环
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 基础别名展开测试 ====================

    #[test]
    fn test_simple_alias_expansion() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());
        aliases.insert("la".to_string(), "ls -A".to_string());

        let mut visited = HashSet::new();

        // 测试简单别名展开
        let result = mock_expand_alias("ll", &aliases, &mut visited, 0)?;
        assert_eq!(result, "ls -la");

        // 重置访问集合
        visited.clear();
        let result2 = mock_expand_alias("la", &aliases, &mut visited, 0)?;
        assert_eq!(result2, "ls -A");

        Ok(())
    }

    #[test]
    fn test_nested_alias_expansion() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());
        aliases.insert("lll".to_string(), "ll -h".to_string()); // 嵌套别名

        let mut visited = HashSet::new();

        // 测试嵌套别名展开
        let result = mock_expand_alias("lll", &aliases, &mut visited, 0)?;
        assert_eq!(result, "ls -la -h");

        Ok(())
    }

    #[test]
    fn test_deep_nested_alias_expansion() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("a".to_string(), "b arg1".to_string());
        aliases.insert("b".to_string(), "c arg2".to_string());
        aliases.insert("c".to_string(), "d arg3".to_string());
        aliases.insert("d".to_string(), "echo final".to_string());

        let mut visited = HashSet::new();

        // 测试深层嵌套别名展开
        let result = mock_expand_alias("a", &aliases, &mut visited, 0)?;
        assert_eq!(result, "echo final arg3 arg2 arg1");

        Ok(())
    }

    #[test]
    fn test_alias_not_found() {
        let aliases = HashMap::new();
        let mut visited = HashSet::new();

        // 测试别名不存在的情况
        let result = mock_expand_alias("nonexistent", &aliases, &mut visited, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Alias not found"));
    }

    // ==================== 循环检测测试 ====================

    #[test]
    fn test_direct_circular_alias() {
        let mut aliases = HashMap::new();
        aliases.insert("a".to_string(), "a".to_string()); // 直接循环

        let mut visited = HashSet::new();

        // 测试直接循环检测
        let result = mock_expand_alias("a", &aliases, &mut visited, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular alias detected"));
    }

    #[test]
    fn test_indirect_circular_alias() {
        let mut aliases = HashMap::new();
        aliases.insert("a".to_string(), "b".to_string());
        aliases.insert("b".to_string(), "c".to_string());
        aliases.insert("c".to_string(), "a".to_string()); // 间接循环

        let mut visited = HashSet::new();

        // 测试间接循环检测
        let result = mock_expand_alias("a", &aliases, &mut visited, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular alias detected"));
    }

    #[test]
    fn test_circular_detection_function() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("a".to_string(), "b".to_string());
        aliases.insert("b".to_string(), "c".to_string());

        // 测试不会形成循环的情况
        let result1 = mock_check_circular("d", "a", &aliases)?;
        assert!(!result1);

        // 测试会形成直接循环的情况
        let result2 = mock_check_circular("a", "a", &aliases)?;
        assert!(result2);

        // 测试会形成间接循环的情况
        let result3 = mock_check_circular("c", "a", &aliases)?;
        assert!(result3);

        Ok(())
    }

    // ==================== 深度限制测试 ====================

    #[test]
    fn test_max_depth_limit() {
        let mut aliases = HashMap::new();

        // 创建一个很深的别名链
        for i in 0..15 {
            let current = format!("alias{}", i);
            let next = format!("alias{}", i + 1);
            aliases.insert(current, next);
        }
        aliases.insert("alias15".to_string(), "echo final".to_string());

        let mut visited = HashSet::new();

        // 测试深度限制
        let result = mock_expand_alias("alias0", &aliases, &mut visited, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth exceeded maximum"));
    }

    #[test]
    fn test_depth_within_limit() -> Result<()> {
        let mut aliases = HashMap::new();

        // 创建一个在限制内的别名链（9层）
        for i in 0..9 {
            let current = format!("alias{}", i);
            let next = format!("alias{}", i + 1);
            aliases.insert(current, next);
        }
        aliases.insert("alias9".to_string(), "echo final".to_string());

        let mut visited = HashSet::new();

        // 测试在深度限制内的展开
        let result = mock_expand_alias("alias0", &aliases, &mut visited, 0)?;
        assert_eq!(result, "echo final");

        Ok(())
    }

    // ==================== 命令行参数展开测试 ====================

    #[test]
    fn test_expand_args_with_alias() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());

        let args = vec![
            "workflow".to_string(),
            "ll".to_string(),
            "--color".to_string(),
            "/tmp".to_string(),
        ];

        let result = mock_expand_args(args, &aliases)?;

        assert_eq!(
            result,
            vec![
                "workflow".to_string(),
                "ls".to_string(),
                "-la".to_string(),
                "--color".to_string(),
                "/tmp".to_string(),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_expand_args_without_alias() -> Result<()> {
        let aliases = HashMap::new();

        let args = vec![
            "workflow".to_string(),
            "status".to_string(),
            "--verbose".to_string(),
        ];

        let result = mock_expand_args(args.clone(), &aliases)?;

        // 不是别名，应该返回原参数
        assert_eq!(result, args);

        Ok(())
    }

    #[test]
    fn test_expand_args_empty() -> Result<()> {
        let aliases = HashMap::new();

        // 测试空参数
        let empty_args = vec![];
        let result1 = mock_expand_args(empty_args.clone(), &aliases)?;
        assert_eq!(result1, empty_args);

        // 测试只有程序名的参数
        let single_arg = vec!["workflow".to_string()];
        let result2 = mock_expand_args(single_arg.clone(), &aliases)?;
        assert_eq!(result2, single_arg);

        Ok(())
    }

    #[test]
    fn test_expand_args_nested_alias() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());
        aliases.insert("lll".to_string(), "ll -h".to_string());

        let args = vec![
            "workflow".to_string(),
            "lll".to_string(),
            "/home".to_string(),
        ];

        let result = mock_expand_args(args, &aliases)?;

        assert_eq!(
            result,
            vec![
                "workflow".to_string(),
                "ls".to_string(),
                "-la".to_string(),
                "-h".to_string(),
                "/home".to_string(),
            ]
        );

        Ok(())
    }

    // ==================== 参数化测试 ====================

    #[rstest]
    #[case("ll", "ls -la", "ls -la")]
    #[case("la", "ls -A", "ls -A")]
    #[case("grep", "grep --color=auto", "grep --color=auto")]
    #[case("status", "git status --short", "git status --short")]
    fn test_simple_alias_expansion_parametrized(
        #[case] alias: &str,
        #[case] command: &str,
        #[case] expected: &str,
    ) -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert(alias.to_string(), command.to_string());

        let mut visited = HashSet::new();
        let result = mock_expand_alias(alias, &aliases, &mut visited, 0)?;

        assert_eq!(result, expected);
        Ok(())
    }

    #[rstest]
    #[case(vec!["a", "b"], vec!["b", "a"], true)] // 会循环：a->b, b->a 形成循环
    #[case(vec!["a", "a"], vec![], true)] // 直接循环
    #[case(vec!["a", "b", "c"], vec!["c", "a"], true)] // 间接循环
    fn test_circular_detection_parametrized(
        #[case] alias_chain: Vec<&str>,
        #[case] test_pairs: Vec<&str>,
        #[case] should_be_circular: bool,
    ) -> Result<()> {
        let mut aliases = HashMap::new();

        // 建立别名链
        for i in 0..alias_chain.len() - 1 {
            aliases.insert(alias_chain[i].to_string(), alias_chain[i + 1].to_string());
        }

        // 测试循环检测
        if test_pairs.len() >= 2 {
            let result = mock_check_circular(test_pairs[0], test_pairs[1], &aliases)?;
            assert_eq!(result, should_be_circular);
        }

        Ok(())
    }

    // ==================== 边界条件和特殊情况测试 ====================

    #[test]
    fn test_alias_with_special_characters() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("docker-ps".to_string(), "docker ps -a".to_string());
        aliases.insert("k8s_pods".to_string(), "kubectl get pods".to_string());
        aliases.insert(
            "log@error".to_string(),
            "grep ERROR /var/log/app.log".to_string(),
        );

        let mut visited = HashSet::new();

        // 测试包含特殊字符的别名
        let result1 = mock_expand_alias("docker-ps", &aliases, &mut visited, 0)?;
        assert_eq!(result1, "docker ps -a");

        visited.clear();
        let result2 = mock_expand_alias("k8s_pods", &aliases, &mut visited, 0)?;
        assert_eq!(result2, "kubectl get pods");

        visited.clear();
        let result3 = mock_expand_alias("log@error", &aliases, &mut visited, 0)?;
        assert_eq!(result3, "grep ERROR /var/log/app.log");

        Ok(())
    }

    #[test]
    fn test_alias_with_quotes_and_spaces() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert(
            "find-large".to_string(),
            "find . -size +100M -type f".to_string(),
        );
        aliases.insert(
            "git-log".to_string(),
            "git log --oneline --graph".to_string(),
        );

        let mut visited = HashSet::new();

        // 测试包含复杂参数的别名
        let result1 = mock_expand_alias("find-large", &aliases, &mut visited, 0)?;
        assert_eq!(result1, "find . -size +100M -type f");

        visited.clear();
        let result2 = mock_expand_alias("git-log", &aliases, &mut visited, 0)?;
        assert_eq!(result2, "git log --oneline --graph");

        Ok(())
    }

    #[test]
    fn test_empty_alias_command() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("empty".to_string(), "".to_string());

        let mut visited = HashSet::new();

        // 测试空命令的别名
        let result = mock_expand_alias("empty", &aliases, &mut visited, 0)?;
        assert_eq!(result, "");

        Ok(())
    }

    #[test]
    fn test_alias_with_single_command() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("vi".to_string(), "vim".to_string());
        aliases.insert("py".to_string(), "python3".to_string());

        let mut visited = HashSet::new();

        // 测试单个命令的别名
        let result1 = mock_expand_alias("vi", &aliases, &mut visited, 0)?;
        assert_eq!(result1, "vim");

        visited.clear();
        let result2 = mock_expand_alias("py", &aliases, &mut visited, 0)?;
        assert_eq!(result2, "python3");

        Ok(())
    }

    // ==================== 复杂场景测试 ====================

    #[test]
    fn test_mixed_alias_and_regular_commands() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());
        aliases.insert("search".to_string(), "ll | grep".to_string());

        let args = vec![
            "workflow".to_string(),
            "search".to_string(),
            "pattern".to_string(),
            "--ignore-case".to_string(),
        ];

        let result = mock_expand_args(args, &aliases)?;

        assert_eq!(
            result,
            vec![
                "workflow".to_string(),
                "ls".to_string(),
                "-la".to_string(),
                "|".to_string(),
                "grep".to_string(),
                "pattern".to_string(),
                "--ignore-case".to_string(),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_alias_expansion_preserves_argument_order() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("docker-run".to_string(), "docker run -it --rm".to_string());

        let args = vec![
            "workflow".to_string(),
            "docker-run".to_string(),
            "-v".to_string(),
            "/host:/container".to_string(),
            "ubuntu:latest".to_string(),
            "bash".to_string(),
        ];

        let result = mock_expand_args(args, &aliases)?;

        assert_eq!(
            result,
            vec![
                "workflow".to_string(),
                "docker".to_string(),
                "run".to_string(),
                "-it".to_string(),
                "--rm".to_string(),
                "-v".to_string(),
                "/host:/container".to_string(),
                "ubuntu:latest".to_string(),
                "bash".to_string(),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_performance_with_large_alias_map() -> Result<()> {
        use std::time::Instant;

        let mut aliases = HashMap::new();

        // 创建大量别名
        for i in 0..1000 {
            aliases.insert(format!("alias{}", i), format!("command{} --arg{}", i, i));
        }

        let start = Instant::now();

        // 测试查找性能
        for i in 0..100 {
            let mut visited = HashSet::new();
            let alias_name = format!("alias{}", i);
            let _result = mock_expand_alias(&alias_name, &aliases, &mut visited, 0)?;
        }

        let duration = start.elapsed();

        // 100次查找应该很快完成
        assert!(duration.as_millis() < 100);

        Ok(())
    }
}
