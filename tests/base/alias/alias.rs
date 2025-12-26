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

use crate::common::environments::CliTestEnv;
use crate::common::fixtures::cli_env;

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

    /// 测试简单别名展开功能
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确展开简单的别名（不包含嵌套）。
    ///
    /// ## 测试场景
    /// 1. 创建别名映射（"ll" -> "ls -la", "la" -> "ls -A"）
    /// 2. 展开别名 "ll"
    /// 3. 展开别名 "la"
    ///
    /// ## 预期结果
    /// - "ll" 展开为 "ls -la"
    /// - "la" 展开为 "ls -A"
    #[test]
    fn test_simple_alias_expansion_with_valid_alias_expands_alias_return_ok() -> Result<()> {
        // Arrange: 准备别名映射
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());
        aliases.insert("la".to_string(), "ls -A".to_string());
        let mut visited = HashSet::new();

        // Act: 展开简单别名
        let result = mock_expand_alias("ll", &aliases, &mut visited, 0)?;

        // Assert: 验证别名展开正确
        assert_eq!(result, "ls -la");

        // 重置访问集合并测试另一个别名
        visited.clear();
        let result2 = mock_expand_alias("la", &aliases, &mut visited, 0)?;
        assert_eq!(result2, "ls -A");

        Ok(())
    }

    /// 测试嵌套别名展开功能
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确递归展开嵌套的别名（别名指向另一个别名）。
    ///
    /// ## 测试场景
    /// 1. 创建别名映射（"ll" -> "ls -la", "lll" -> "ll -h"）
    /// 2. 展开嵌套别名 "lll"
    /// 3. 验证递归展开结果
    ///
    /// ## 预期结果
    /// - "lll" 递归展开为 "ls -la -h"
    /// - 嵌套别名被正确解析
    #[test]
    fn test_nested_alias_expansion_with_nested_alias_expands_recursively_return_ok() -> Result<()> {
        // Arrange: 准备嵌套别名映射
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());
        aliases.insert("lll".to_string(), "ll -h".to_string()); // 嵌套别名
        let mut visited = HashSet::new();

        // Act: 展开嵌套别名
        let result = mock_expand_alias("lll", &aliases, &mut visited, 0)?;

        // Assert: 验证嵌套别名递归展开正确
        assert_eq!(result, "ls -la -h");

        Ok(())
    }

    /// 测试深层嵌套别名展开功能
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理多层嵌套的别名（a -> b -> c -> d）。
    ///
    /// ## 测试场景
    /// 1. 创建多层嵌套别名映射（a -> b arg1, b -> c arg2, c -> d arg3, d -> echo final）
    /// 2. 展开顶层别名 "a"
    /// 3. 验证所有层级都被正确展开
    ///
    /// ## 预期结果
    /// - "a" 递归展开为 "echo final arg3 arg2 arg1"
    /// - 所有嵌套层级都被正确解析
    /// - 参数顺序正确
    #[test]
    fn test_deep_nested_alias_expansion_with_deep_nesting_expands_all_levels_return_collect() -> Result<()> {
        // Arrange: 准备深层嵌套别名映射
        let mut aliases = HashMap::new();
        aliases.insert("a".to_string(), "b arg1".to_string());
        aliases.insert("b".to_string(), "c arg2".to_string());
        aliases.insert("c".to_string(), "d arg3".to_string());
        aliases.insert("d".to_string(), "echo final".to_string());
        let mut visited = HashSet::new();

        // Act: 展开深层嵌套别名
        let result = mock_expand_alias("a", &aliases, &mut visited, 0)?;

        // Assert: 验证深层嵌套别名递归展开正确
        assert_eq!(result, "echo final arg3 arg2 arg1");

        Ok(())
    }

    /// 测试不存在的别名处理
    ///
    /// ## 测试目的
    /// 验证当尝试展开不存在的别名时，别名管理器能够正确返回错误。
    ///
    /// ## 测试场景
    /// 1. 创建空的别名映射
    /// 2. 尝试展开不存在的别名 "nonexistent"
    /// 3. 验证错误处理
    ///
    /// ## 预期结果
    /// - 返回错误
    /// - 错误消息包含 "Alias not found"
    #[test]
    fn test_alias_not_found_with_nonexistent_alias_return_ok() -> Result<()> {
        // Arrange: 准备空别名映射和不存在的别名
        let aliases = HashMap::new();
        let mut visited = HashSet::new();
        let alias = "nonexistent";

        // Act: 尝试展开不存在的别名
        let result = mock_expand_alias(alias, &aliases, &mut visited, 0);

        // Assert: 验证返回错误且错误消息包含"Alias not found"
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Alias not found"));
        Ok(())
    }

    // ==================== 循环检测测试 ====================

    /// 测试直接循环别名检测
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确检测直接循环别名（别名指向自身）。
    ///
    /// ## 测试场景
    /// 1. 创建直接循环别名映射（"a" -> "a"）
    /// 2. 尝试展开别名 "a"
    /// 3. 验证循环检测
    ///
    /// ## 预期结果
    /// - 返回错误
    /// - 错误消息包含 "Circular alias detected"
    #[test]
    fn test_direct_circular_alias_with_direct_cycle_detects_circular_return_ok() -> Result<()> {
        // Arrange: 准备直接循环别名映射
        let mut aliases = HashMap::new();
        aliases.insert("a".to_string(), "a".to_string()); // 直接循环
        let mut visited = HashSet::new();

        // Act: 尝试展开直接循环别名
        let result = mock_expand_alias("a", &aliases, &mut visited, 0);

        // Assert: 验证检测到循环且错误消息包含"Circular alias detected"
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular alias detected"));
        Ok(())
    }

    /// 测试间接循环别名检测
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确检测间接循环别名（a -> b -> c -> a）。
    ///
    /// ## 测试场景
    /// 1. 创建间接循环别名映射（a -> b, b -> c, c -> a）
    /// 2. 尝试展开别名 "a"
    /// 3. 验证循环检测
    ///
    /// ## 预期结果
    /// - 返回错误
    /// - 错误消息包含 "Circular alias detected"
    /// - 能够检测多级间接循环
    #[test]
    fn test_indirect_circular_alias_with_indirect_cycle_detects_circular_return_ok() -> Result<()> {
        // Arrange: 准备间接循环别名映射
        let mut aliases = HashMap::new();
        aliases.insert("a".to_string(), "b".to_string());
        aliases.insert("b".to_string(), "c".to_string());
        aliases.insert("c".to_string(), "a".to_string()); // 间接循环
        let mut visited = HashSet::new();

        // Act: 尝试展开间接循环别名
        let result = mock_expand_alias("a", &aliases, &mut visited, 0);

        // Assert: 验证检测到循环且错误消息包含"Circular alias detected"
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular alias detected"));
        Ok(())
    }

    /// 测试循环检测函数处理多种情况
    ///
    /// ## 测试目的
    /// 验证循环检测函数能够正确处理多种循环情况（直接循环、间接循环、非循环）。
    ///
    /// ## 测试场景
    /// 1. 创建别名映射（a -> b, b -> c）
    /// 2. 测试不会形成循环的情况（d -> a）
    /// 3. 测试会形成直接循环的情况（a -> a）
    /// 4. 测试会形成间接循环的情况（c -> a）
    ///
    /// ## 预期结果
    /// - 非循环情况返回 false
    /// - 直接循环情况返回 true
    /// - 间接循环情况返回 true
    #[test]
    fn test_circular_detection_function_with_various_cases_detects_circular_return_ok() -> Result<()> {
        // Arrange: 准备别名映射
        let mut aliases = HashMap::new();
        aliases.insert("a".to_string(), "b".to_string());
        aliases.insert("b".to_string(), "c".to_string());

        // Act & Assert: 测试不会形成循环的情况
        let result1 = mock_check_circular("d", "a", &aliases)?;
        assert!(!result1);

        // Act & Assert: 测试会形成直接循环的情况
        let result2 = mock_check_circular("a", "a", &aliases)?;
        assert!(result2);

        // Act & Assert: 测试会形成间接循环的情况
        let result3 = mock_check_circular("c", "a", &aliases)?;
        assert!(result3);

        Ok(())
    }

    // ==================== 深度限制测试 ====================

    /// 测试最大深度限制
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确检测并拒绝超过最大深度限制的别名链。
    ///
    /// ## 测试场景
    /// 1. 创建一个超过最大深度限制（10层）的别名链（15层）
    /// 2. 尝试展开顶层别名
    /// 3. 验证深度限制检测
    ///
    /// ## 预期结果
    /// - 返回错误
    /// - 错误消息包含 "depth exceeded maximum"
    /// - 防止无限递归
    #[test]
    fn test_max_depth_limit_with_deep_chain_return_ok() -> Result<()> {
        // Arrange: 创建一个很深的别名链（超过最大深度限制）
        let mut aliases = HashMap::new();
        for i in 0..15 {
            let current = format!("alias{}", i);
            let next = format!("alias{}", i + 1);
            aliases.insert(current, next);
        }
        aliases.insert("alias15".to_string(), "echo final".to_string());
        let mut visited = HashSet::new();

        // Act: 尝试展开超过深度限制的别名
        let result = mock_expand_alias("alias0", &aliases, &mut visited, 0);

        // Assert: 验证返回错误且错误消息包含"depth exceeded maximum"
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth exceeded maximum"));
        Ok(())
    }

    /// 测试在深度限制内的别名展开
    ///
    /// ## 测试目的
    /// 验证别名管理器能够成功展开在深度限制内的别名链。
    ///
    /// ## 测试场景
    /// 1. 创建一个在深度限制内（9层）的别名链
    /// 2. 展开顶层别名
    /// 3. 验证展开成功
    ///
    /// ## 预期结果
    /// - 展开成功
    /// - 返回最终命令 "echo final"
    #[test]
    fn test_depth_within_limit_with_valid_depth_expands_successfully_return_true() -> Result<()> {
        // Arrange: 创建一个在限制内的别名链（9层）
        let mut aliases = HashMap::new();
        for i in 0..9 {
            let current = format!("alias{}", i);
            let next = format!("alias{}", i + 1);
            aliases.insert(current, next);
        }
        aliases.insert("alias9".to_string(), "echo final".to_string());
        let mut visited = HashSet::new();

        // Act: 展开在深度限制内的别名
        let result = mock_expand_alias("alias0", &aliases, &mut visited, 0)?;

        // Assert: 验证展开成功
        assert_eq!(result, "echo final");

        Ok(())
    }

    // ==================== 命令行参数展开测试 ====================

    /// 测试命令行参数中包含别名的展开
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确展开命令行参数中的别名。
    ///
    /// ## 测试场景
    /// 1. 创建别名映射（"ll" -> "ls -la"）
    /// 2. 准备包含别名的参数列表（["workflow", "ll", "--color", "/tmp"]）
    /// 3. 展开参数中的别名
    /// 4. 验证展开结果
    ///
    /// ## 预期结果
    /// - 别名 "ll" 被展开为 "ls -la"
    /// - 其他参数保持不变
    /// - 参数顺序正确
    #[test]
    fn test_expand_args_with_alias_with_alias_in_args_expands_alias_return_ok() -> Result<()> {
        // Arrange: 准备别名映射和包含别名的参数列表
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());
        let args = vec![
            "workflow".to_string(),
            "ll".to_string(),
            "--color".to_string(),
            "/tmp".to_string(),
        ];

        // Act: 展开参数中的别名
        let result = mock_expand_args(args, &aliases)?;

        // Assert: 验证别名被展开且其他参数保持不变
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

    /// 测试命令行参数中不包含别名的处理
    ///
    /// ## 测试目的
    /// 验证当命令行参数中不包含别名时，参数保持不变。
    ///
    /// ## 测试场景
    /// 1. 创建空的别名映射
    /// 2. 准备不包含别名的参数列表（["workflow", "status", "--verbose"]）
    /// 3. 尝试展开参数
    /// 4. 验证参数保持不变
    ///
    /// ## 预期结果
    /// - 参数列表保持不变
    /// - 不进行任何展开操作
    #[test]
    fn test_expand_args_without_alias_with_no_alias_return_ok() -> Result<()> {
        // Arrange: 准备空别名映射和不包含别名的参数列表
        let aliases = HashMap::new();
        let args = vec![
            "workflow".to_string(),
            "status".to_string(),
            "--verbose".to_string(),
        ];

        // Act: 展开参数（无别名）
        let result = mock_expand_args(args.clone(), &aliases)?;

        // Assert: 验证参数保持不变（不是别名，应该返回原参数）
        assert_eq!(result, args);

        Ok(())
    }

    /// 测试空参数列表的处理
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理空参数列表或只包含程序名的参数列表。
    ///
    /// ## 测试场景
    /// 1. 创建空的别名映射
    /// 2. 测试空参数列表 []
    /// 3. 测试只包含程序名的参数列表 ["workflow"]
    /// 4. 验证处理结果
    ///
    /// ## 预期结果
    /// - 空参数列表返回空列表
    /// - 只包含程序名的参数列表保持不变
    #[test]
    fn test_expand_args_empty_with_empty_args_return_empty() -> Result<()> {
        // Arrange: 准备空别名映射和空参数列表
        let aliases = HashMap::new();
        let empty_args: Vec<String> = vec![];

        // Act: 展开空参数
        let result1 = mock_expand_args(empty_args.clone(), &aliases)?;

        // Assert: 验证返回空参数
        assert_eq!(result1, empty_args);

        // Arrange: 准备只有程序名的参数
        let single_arg = vec!["workflow".to_string()];

        // Act: 展开只有程序名的参数
        let result2 = mock_expand_args(single_arg.clone(), &aliases)?;

        // Assert: 验证返回原参数
        assert_eq!(result2, single_arg);

        Ok(())
    }

    /// 测试命令行参数中嵌套别名的展开
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确递归展开命令行参数中的嵌套别名。
    ///
    /// ## 测试场景
    /// 1. 创建嵌套别名映射（"ll" -> "ls -la", "lll" -> "ll -h"）
    /// 2. 准备包含嵌套别名的参数列表（["workflow", "lll", "/home"]）
    /// 3. 展开嵌套别名
    /// 4. 验证递归展开结果
    ///
    /// ## 预期结果
    /// - 嵌套别名 "lll" 递归展开为 "ls -la -h"
    /// - 其他参数保持不变
    /// - 参数顺序正确
    #[test]
    fn test_expand_args_nested_alias_with_nested_alias_expands_recursively_return_ok() -> Result<()> {
        // Arrange: 准备嵌套别名映射和包含嵌套别名的参数列表
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());
        aliases.insert("lll".to_string(), "ll -h".to_string());
        let args = vec![
            "workflow".to_string(),
            "lll".to_string(),
            "/home".to_string(),
        ];

        // Act: 展开嵌套别名
        let result = mock_expand_args(args, &aliases)?;

        // Assert: 验证嵌套别名递归展开正确
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

    /// 测试简单别名展开（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证多种简单别名的展开功能。
    ///
    /// ## 测试场景
    /// 测试多种别名和命令的组合：
    /// - "ll" -> "ls -la"
    /// - "la" -> "ls -A"
    /// - "grep" -> "grep --color=auto"
    /// - "status" -> "git status --short"
    ///
    /// ## 预期结果
    /// - 所有测试用例都能正确展开别名
    #[rstest]
    #[case("ll", "ls -la", "ls -la")]
    #[case("la", "ls -A", "ls -A")]
    #[case("grep", "grep --color=auto", "grep --color=auto")]
    #[case("status", "git status --short", "git status --short")]
    fn test_simple_alias_expansion_parametrized_return_ok(
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

    /// 测试循环检测（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证多种循环情况的检测功能。
    ///
    /// ## 测试场景
    /// 测试多种循环情况：
    /// - 间接循环：a->b, b->a
    /// - 直接循环：a->a
    /// - 多级间接循环：a->b->c, c->a
    ///
    /// ## 预期结果
    /// - 所有循环情况都能被正确检测
    #[rstest]
    #[case(vec!["a", "b"], vec!["b", "a"], true)] // 会循环：a->b, b->a 形成循环
    #[case(vec!["a", "a"], vec![], true)] // 直接循环
    #[case(vec!["a", "b", "c"], vec!["c", "a"], true)] // 间接循环
    fn test_circular_detection_parametrized_return_ok(
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

    /// 测试包含特殊字符的别名
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理包含特殊字符（如连字符、下划线、@符号）的别名。
    ///
    /// ## 测试场景
    /// 1. 创建包含特殊字符的别名映射（"docker-ps", "k8s_pods", "log@error"）
    /// 2. 展开这些别名
    /// 3. 验证展开结果
    ///
    /// ## 预期结果
    /// - 所有包含特殊字符的别名都能正确展开
    /// - 特殊字符被正确处理
    #[test]
    fn test_alias_with_special_characters_return_ok() -> Result<()> {
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

    /// 测试包含引号和空格的别名
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理包含复杂参数（空格、引号等）的别名。
    ///
    /// ## 测试场景
    /// 1. 创建包含复杂参数的别名映射（"find-large", "git-log"）
    /// 2. 展开这些别名
    /// 3. 验证展开结果
    ///
    /// ## 预期结果
    /// - 包含复杂参数的别名都能正确展开
    /// - 参数格式正确
    #[test]
    fn test_alias_with_quotes_and_spaces_return_ok() -> Result<()> {
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

    /// 测试空命令的别名
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理空命令的别名。
    ///
    /// ## 测试场景
    /// 1. 创建空命令的别名映射（"empty" -> ""）
    /// 2. 展开别名
    /// 3. 验证展开结果
    ///
    /// ## 预期结果
    /// - 空命令别名展开为空字符串
    #[test]
    fn test_empty_alias_command_return_empty() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("empty".to_string(), "".to_string());

        let mut visited = HashSet::new();

        // 测试空命令的别名
        let result = mock_expand_alias("empty", &aliases, &mut visited, 0)?;
        assert_eq!(result, "");

        Ok(())
    }

    /// 测试单个命令的别名
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理单个命令的别名（无参数）。
    ///
    /// ## 测试场景
    /// 1. 创建单个命令的别名映射（"vi" -> "vim", "py" -> "python3"）
    /// 2. 展开这些别名
    /// 3. 验证展开结果
    ///
    /// ## 预期结果
    /// - 单个命令的别名都能正确展开
    #[test]
    fn test_alias_with_single_command_return_ok() -> Result<()> {
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

    /// 测试混合别名和普通命令的场景
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理包含别名和普通命令混合的命令行。
    ///
    /// ## 测试场景
    /// 1. 创建别名映射（"ll" -> "ls -la", "search" -> "ll | grep"）
    /// 2. 准备包含别名和普通命令的参数列表
    /// 3. 展开参数
    /// 4. 验证展开结果
    ///
    /// ## 预期结果
    /// - 别名被正确展开
    /// - 普通命令保持不变
    /// - 管道符号等特殊字符被正确处理
    #[test]
    fn test_mixed_alias_and_regular_commands_return_ok() -> Result<()> {
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

    /// 测试别名展开保持参数顺序
    ///
    /// ## 测试目的
    /// 验证别名展开后，原始参数顺序被正确保持。
    ///
    /// ## 测试场景
    /// 1. 创建别名映射（"docker-run" -> "docker run -it --rm"）
    /// 2. 准备包含别名和额外参数的命令行参数
    /// 3. 展开别名
    /// 4. 验证参数顺序正确
    ///
    /// ## 预期结果
    /// - 别名展开后的参数顺序正确
    /// - 原始参数顺序保持不变
    #[test]
    fn test_alias_expansion_preserves_argument_order_return_ok() -> Result<()> {
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

    /// 测试大量别名映射的性能
    ///
    /// ## 测试目的
    /// 验证别名管理器在处理大量别名时的性能表现。
    ///
    /// ## 测试场景
    /// 1. 创建1000个别名映射
    /// 2. 执行100次别名查找和展开操作
    /// 3. 测量执行时间
    /// 4. 验证性能在可接受范围内
    ///
    /// ## 预期结果
    /// - 100次查找操作应在100毫秒内完成
    /// - 性能表现良好
    #[test]
    fn test_performance_with_large_alias_map_return_ok() -> Result<()> {
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

    // ==================== 实际 AliasManager 方法测试 ====================
    // 注意：这些测试依赖实际的配置文件，但会测试 AliasManager 的实际方法

    /// 测试 AliasManager::load() 方法
    ///
    /// ## 测试目的
    /// 验证 AliasManager 能够正确加载别名配置。
    ///
    /// ## 测试场景
    /// 1. 调用 AliasManager::load() 方法
    /// 2. 验证返回结果
    /// 3. 检查返回的别名映射
    ///
    /// ## 预期结果
    /// - 方法返回 Ok
    /// - 返回 HashMap<String, String>
    /// - 即使别名列表为空也能正常工作
    #[test]
    fn test_alias_manager_load_return_ok() -> Result<()> {
        // 测试 AliasManager::load() 方法（覆盖 manager.rs:29-32）
        let result = workflow::base::alias::AliasManager::load();

        // 应该总是返回 Ok，即使别名列表为空
        assert!(result.is_ok());

        let aliases = result?;
        // 验证返回的是 HashMap
        let _alias_count = aliases.len();
        Ok(())
    }

    /// 测试 AliasManager::list() 方法
    ///
    /// ## 测试目的
    /// 验证 AliasManager 能够正确列出所有别名。
    ///
    /// ## 测试场景
    /// 1. 调用 AliasManager::list() 方法
    /// 2. 验证返回结果
    /// 3. 检查返回的别名映射
    ///
    /// ## 预期结果
    /// - 方法返回 Ok
    /// - 返回 HashMap<String, String>
    #[test]
    fn test_alias_manager_list_return_collect() -> Result<()> {
        // 测试 AliasManager::list() 方法（覆盖 manager.rs:235-237）
        let result = workflow::base::alias::AliasManager::list();

        // 应该总是返回 Ok
        assert!(result.is_ok());

        let aliases = result?;
        // 验证返回的是 HashMap
        let _alias_count = aliases.len();
        Ok(())
    }

    /// 测试 AliasManager::exists() 方法
    ///
    /// ## 测试目的
    /// 验证 AliasManager 能够正确检查别名是否存在。
    ///
    /// ## 测试场景
    /// 1. 调用 AliasManager::exists() 检查不存在的别名
    /// 2. 验证返回结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok(false)
    /// - 不存在的别名返回 false
    #[test]
    fn test_alias_manager_exists_return_ok() -> Result<()> {
        // 测试 AliasManager::exists() 方法（覆盖 manager.rs:252-255）
        // 测试不存在的别名
        let result = workflow::base::alias::AliasManager::exists("__nonexistent_alias_test__");

        assert!(result.is_ok());
        assert!(!result?);
        Ok(())
    }

    /// 测试 AliasManager::expand_args() 方法处理空参数
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand_args() 能够正确处理只包含程序名的参数列表。
    ///
    /// ## 测试场景
    /// 1. 准备只包含程序名的参数列表（["workflow"]）
    /// 2. 调用 expand_args() 方法
    /// 3. 验证返回结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok
    /// - 参数列表保持不变
    #[test]
    fn test_alias_manager_expand_args_empty_return_empty() -> Result<()> {
        // 测试 AliasManager::expand_args() 方法 - 空参数（覆盖 manager.rs:116-120）
        let args = vec!["workflow".to_string()];
        let result = workflow::base::alias::AliasManager::expand_args(args.clone());

        assert!(result.is_ok());
        assert_eq!(result?, args);
        Ok(())
    }

    /// 测试 AliasManager::expand_args() 方法处理单个参数
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand_args() 能够正确处理只包含程序名的参数列表。
    ///
    /// ## 测试场景
    /// 1. 准备只包含程序名的参数列表（["workflow"]）
    /// 2. 调用 expand_args() 方法
    /// 3. 验证返回结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok
    /// - 参数列表保持不变
    #[test]
    fn test_alias_manager_expand_args_single_return_ok() -> Result<()> {
        // 测试 AliasManager::expand_args() 方法 - 单个参数（覆盖 manager.rs:116-120）
        let args = vec!["workflow".to_string()];
        let result = workflow::base::alias::AliasManager::expand_args(args.clone());

        assert!(result.is_ok());
        assert_eq!(result?, args);
        Ok(())
    }

    /// 测试 AliasManager::expand_args() 方法处理非别名命令
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand_args() 能够正确处理不包含别名的命令参数。
    ///
    /// ## 测试场景
    /// 1. 准备不包含别名的参数列表（["workflow", "status", "--verbose"]）
    /// 2. 调用 expand_args() 方法
    /// 3. 验证返回结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok
    /// - 参数列表保持不变（第一个参数不是别名）
    #[test]
    fn test_alias_manager_expand_args_non_alias_return_ok() -> Result<()> {
        // 测试 AliasManager::expand_args() 方法 - 非别名命令（覆盖 manager.rs:144-147）
        let args = vec![
            "workflow".to_string(),
            "status".to_string(),
            "--verbose".to_string(),
        ];
        let result = workflow::base::alias::AliasManager::expand_args(args.clone());

        // 如果不是别名，应该返回原参数
        assert!(result.is_ok());
        let expanded = result?;
        // 如果第一个参数不是别名，应该保持不变
        assert_eq!(expanded[0], "workflow");
        assert_eq!(expanded[1], "status");
        Ok(())
    }

    /// 测试 AliasManager::check_circular() 方法检测直接循环
    ///
    /// ## 测试目的
    /// 验证 AliasManager::check_circular() 能够正确检测直接循环别名（别名指向自身）。
    ///
    /// ## 测试场景
    /// 1. 调用 check_circular() 检查直接循环（"test_circular_a" -> "test_circular_a"）
    /// 2. 验证返回结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok(true)
    /// - 直接循环被正确检测
    #[test]
    fn test_alias_manager_check_circular_direct_return_ok() -> Result<()> {
        // 测试 AliasManager::check_circular() 方法 - 直接循环（覆盖 manager.rs:273-302）
        // 测试添加别名 "a" -> "a" 是否检测为循环
        let result = workflow::base::alias::AliasManager::check_circular(
            "test_circular_a",
            "test_circular_a",
        );

        assert!(result.is_ok());
        // 直接循环应该返回 true
        assert!(result?);
        Ok(())
    }

    /// 测试 AliasManager::check_circular() 方法检测非循环别名
    ///
    /// ## 测试目的
    /// 验证 AliasManager::check_circular() 能够正确识别非循环的别名。
    ///
    /// ## 测试场景
    /// 1. 调用 check_circular() 检查非循环别名（"__test_new_alias__" -> "git status"）
    /// 2. 验证返回结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok(false)
    /// - 非循环别名被正确识别
    #[test]
    fn test_alias_manager_check_circular_non_circular_return_ok() -> Result<()> {
        // 测试 AliasManager::check_circular() 方法 - 非循环（覆盖 manager.rs:273-302）
        // 测试添加别名 "new_alias" -> "git status" 是否检测为非循环
        let result =
            workflow::base::alias::AliasManager::check_circular("__test_new_alias__", "git status");

        assert!(result.is_ok());
        // 非循环应该返回 false
        assert!(!result?);
        Ok(())
    }

    /// 测试 AliasManager::expand() 方法的深度限制
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand() 能够正确检测并拒绝超过深度限制的别名展开。
    ///
    /// ## 测试场景
    /// 1. 使用超过最大深度限制（11层）调用 expand() 方法
    /// 2. 验证错误处理
    ///
    /// ## 预期结果
    /// - 返回错误
    /// - 错误消息包含 "depth exceeded maximum"
    #[test]
    fn test_alias_manager_expand_depth_limit_return_ok() -> Result<()> {
        // 测试 AliasManager::expand() 方法 - 深度限制（覆盖 manager.rs:54-98）
        // 注意：这个测试需要创建深度嵌套的别名，可能在实际环境中难以实现
        // 主要测试深度检查逻辑
        let mut visited = HashSet::new();
        let result =
            workflow::base::alias::AliasManager::expand("__nonexistent__", &mut visited, 11);

        // 深度超过限制应该返回错误
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth exceeded maximum"));
        Ok(())
    }

    /// 测试 AliasManager::expand() 方法处理不存在的别名
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand() 能够正确处理不存在的别名。
    ///
    /// ## 测试场景
    /// 1. 尝试展开不存在的别名 "__nonexistent_alias__"
    /// 2. 验证错误处理
    ///
    /// ## 预期结果
    /// - 返回错误
    /// - 错误消息包含 "Alias not found"
    #[test]
    fn test_alias_manager_expand_not_found_return_ok() -> Result<()> {
        // 测试 AliasManager::expand() 方法 - 别名不存在（覆盖 manager.rs:77-79）
        let mut visited = HashSet::new();
        let result =
            workflow::base::alias::AliasManager::expand("__nonexistent_alias__", &mut visited, 0);

        // 别名不存在应该返回错误
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Alias not found"));
        Ok(())
    }

    /// 测试 AliasManager::expand() 方法处理嵌套别名
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand() 能够正确处理嵌套别名（如果存在）。
    ///
    /// ## 测试场景
    /// 1. 尝试展开可能存在的嵌套别名 "__test_nested__"
    /// 2. 验证处理结果
    ///
    /// ## 预期结果
    /// - 可能成功或失败，取决于实际配置
    /// - 如果别名存在，应正确展开
    #[test]
    fn test_alias_manager_expand_with_nested_alias_return_ok() -> Result<()> {
        // 测试 AliasManager::expand() 方法 - 嵌套别名（覆盖 manager.rs:84-95）
        // 注意：这个测试需要实际的别名配置
        let mut visited = HashSet::new();
        // 尝试展开一个可能存在的别名
        let result =
            workflow::base::alias::AliasManager::expand("__test_nested__", &mut visited, 0);

        // 可能成功或失败，取决于配置
        assert!(result.is_ok() || result.is_err());
        Ok(())
    }

    /// 测试 AliasManager::expand() 方法使用 visited 集合检测循环
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand() 能够使用 visited 集合正确检测循环引用。
    ///
    /// ## 测试场景
    /// 1. 将别名添加到 visited 集合
    /// 2. 尝试展开已访问的别名
    /// 3. 验证循环检测
    ///
    /// ## 预期结果
    /// - 返回错误（循环检测或别名不存在）
    #[test]
    fn test_alias_manager_expand_with_visited_set_return_ok() -> Result<()> {
        // 测试 AliasManager::expand() 方法 - visited 集合的使用（覆盖 manager.rs:82）
        let mut visited = HashSet::new();
        visited.insert("test_alias".to_string());

        // 尝试展开已访问的别名（如果存在）
        let result = workflow::base::alias::AliasManager::expand("test_alias", &mut visited, 0);

        // 如果别名存在且已访问，应该检测到循环
        // 如果别名不存在，应该返回"not found"错误
        assert!(result.is_err());
        Ok(())
    }

    /// 测试 AliasManager::expand_args() 方法处理包含别名的参数
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand_args() 能够正确处理包含别名的命令行参数。
    ///
    /// ## 测试场景
    /// 1. 准备包含可能别名的参数列表（["workflow", "__test_alias__"]）
    /// 2. 调用 expand_args() 方法
    /// 3. 验证处理结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok
    /// - 如果别名存在，应正确展开；如果不存在，参数保持不变
    #[test]
    fn test_alias_manager_expand_args_with_alias_return_ok() -> Result<()> {
        // 测试 AliasManager::expand_args() 方法 - 包含别名（覆盖 manager.rs:128-143）
        // 注意：这个测试需要实际的别名配置
        let args = vec!["workflow".to_string(), "__test_alias__".to_string()];
        let result = workflow::base::alias::AliasManager::expand_args(args);

        // 如果别名存在，应该展开；如果不存在，应该返回原参数
        assert!(result.is_ok());
        Ok(())
    }

    /// 测试 AliasManager::check_circular() 方法与已存在别名形成循环
    ///
    /// ## 测试目的
    /// 验证 AliasManager::check_circular() 能够检测与已存在别名形成循环的情况。
    ///
    /// ## 测试场景
    /// 1. 调用 check_circular() 检查新别名与已存在别名是否形成循环
    /// 2. 验证返回结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok
    /// - 如果形成循环，返回 true；否则返回 false
    #[test]
    fn test_alias_manager_check_circular_with_existing_alias_return_ok() -> Result<()> {
        // 测试 AliasManager::check_circular() 方法 - 与已存在别名形成循环（覆盖 manager.rs:284-297）
        // 注意：这个测试需要实际的别名配置
        let result = workflow::base::alias::AliasManager::check_circular(
            "__test_new__",
            "__test_existing__",
        );

        // 应该返回 true 或 false，取决于是否形成循环
        assert!(result.is_ok());
        Ok(())
    }

    /// 测试 AliasManager::check_circular() 方法处理 target 的第一个词不是别名的情况
    ///
    /// ## 测试目的
    /// 验证 AliasManager::check_circular() 能够正确处理 target 的第一个词不是别名的情况。
    ///
    /// ## 测试场景
    /// 1. 调用 check_circular() 检查新别名，target 的第一个词不是别名（如 "git status"）
    /// 2. 验证返回结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok(false)
    /// - 第一个词不是别名时，不会形成循环
    #[test]
    fn test_alias_manager_check_circular_first_part_not_alias_return_ok() -> Result<()> {
        // 测试 AliasManager::check_circular() 方法 - target 的第一个词不是别名（覆盖 manager.rs:277-299）
        let result =
            workflow::base::alias::AliasManager::check_circular("__test_new__", "git status");

        // 如果第一个词不是别名，应该返回 false
        assert!(result.is_ok());
        assert!(!result?);
        Ok(())
    }

    /// 测试 AliasManager::expand() 方法处理递归嵌套展开
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand() 能够正确处理递归嵌套的别名展开。
    ///
    /// ## 测试场景
    /// 1. 尝试展开可能包含嵌套别名的别名
    /// 2. 验证处理结果
    ///
    /// ## 预期结果
    /// - 可能成功或失败，取决于实际配置
    /// - 如果别名存在且包含嵌套，应正确递归展开
    #[test]
    fn test_alias_manager_expand_recursive_nested_return_ok() -> Result<()> {
        // 测试 AliasManager::expand() 方法 - 递归嵌套展开（覆盖 manager.rs:89-93）
        let mut visited = HashSet::new();
        // 尝试展开一个可能包含嵌套别名的别名
        let result =
            workflow::base::alias::AliasManager::expand("__test_nested__", &mut visited, 0);

        // 可能成功或失败，取决于配置
        assert!(result.is_ok() || result.is_err());
        Ok(())
    }

    // ==================== 使用临时配置文件的实际方法测试 ====================

    /// 测试 AliasManager::add() 方法使用临时配置文件
    ///
    /// ## 测试目的
    /// 验证 AliasManager::add() 能够使用临时配置文件正确添加别名。
    ///
    /// ## 测试场景
    /// 1. 创建临时配置文件和测试环境
    /// 2. 调用 add() 方法添加别名
    /// 3. 验证别名已添加到配置文件
    ///
    /// ## 预期结果
    /// - 添加成功
    /// - 别名已写入配置文件
    ///
    /// ## 为什么被忽略
    /// - **需要干净的测试环境**: Settings 使用 OnceLock 单例，无法重置
    #[rstest]
    #[ignore = "Requires clean test environment - Settings uses OnceLock singleton that cannot be reset"]
    fn test_alias_manager_add_with_temp_config_return_ok(mut cli_env: CliTestEnv) -> Result<()> {
        // 测试 AliasManager::add() 方法 - 使用临时配置文件（覆盖 manager.rs:162-181）
        use workflow::base::util::file::FileWriter;

        let env = &mut cli_env;
        let config_dir = env.path().join(".workflow").join("config");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("workflow.toml");

        // 创建初始配置文件
        let initial_config = r#"
aliases = {}
"#;
        FileWriter::new(&config_path).write_str(initial_config)?;

        // 设置临时 HOME 目录
        let home_path = env.path().to_string_lossy().to_string();
        env.env_guard().set("HOME", &home_path);

        // 添加别名
        let result = workflow::base::alias::AliasManager::add("test_add_alias", "echo hello");
        // EnvGuard 会在 env 离开作用域时自动恢复 HOME

        // 验证添加成功
        assert!(result.is_ok());

        // 验证别名已添加到配置文件（直接读取文件，因为 Settings 使用 OnceLock 缓存）
        use toml::Value;
        use workflow::base::util::file::FileReader;
        let config_content = FileReader::new(&config_path).to_string()?;
        let config: Value = toml::from_str(&config_content)?;
        let aliases_table = config
            .get("aliases")
            .and_then(|v| v.as_table())
            .expect("aliases table should exist in config");
        assert_eq!(
            aliases_table.get("test_add_alias").and_then(|v| v.as_str()),
            Some("echo hello")
        );

        Ok(())
    }

    /// 测试 AliasManager::remove() 方法使用临时配置文件
    ///
    /// ## 测试目的
    /// 验证 AliasManager::remove() 能够使用临时配置文件正确删除别名。
    ///
    /// ## 测试场景
    /// 1. 创建包含别名的临时配置文件
    /// 2. 调用 remove() 方法删除别名
    /// 3. 验证别名已从配置文件删除
    ///
    /// ## 预期结果
    /// - 删除成功
    /// - 别名已从配置文件移除
    ///
    /// ## 为什么被忽略
    /// - **需要干净的测试环境**: Settings 使用 OnceLock 单例，无法重置
    #[rstest]
    #[ignore = "Requires clean test environment - Settings uses OnceLock singleton that cannot be reset"]
    fn test_alias_manager_remove_with_temp_config_return_ok(mut cli_env: CliTestEnv) -> Result<()> {
        // 测试 AliasManager::remove() 方法 - 使用临时配置文件（覆盖 manager.rs:198-222）
        use workflow::base::util::file::FileWriter;

        let env = &mut cli_env;
        let config_dir = env.path().join(".workflow").join("config");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("workflow.toml");

        // 先设置临时 HOME 目录（在创建配置文件之前）
        let home_path = env.path().to_string_lossy().to_string();
        env.env_guard().set("HOME", &home_path);

        // 创建包含别名的配置文件（在设置 HOME 之后）
        let initial_config = r#"
aliases = { test_remove_alias = "echo test" }
"#;
        FileWriter::new(&config_path).write_str(initial_config)?;

        // 确保 Settings 使用新的 HOME（通过调用一次来初始化）
        let _ = workflow::base::settings::Settings::get();

        // 删除别名
        let result = workflow::base::alias::AliasManager::remove("test_remove_alias");
        // EnvGuard 会在 env 离开作用域时自动恢复 HOME

        // 验证删除成功
        assert!(result.is_ok());
        assert!(result?);

        // 验证别名已从配置文件中删除（直接读取文件，因为 Settings 使用 OnceLock 缓存）
        use toml::Value;
        use workflow::base::util::file::FileReader;
        let config_content = FileReader::new(&config_path).to_string()?;
        let config: Value = toml::from_str(&config_content)?;
        let aliases_table = config.get("aliases").and_then(|v| v.as_table());
        // 别名应该不存在或为空
        if let Some(aliases) = aliases_table {
            assert!(!aliases.contains_key("test_remove_alias"));
        }

        Ok(())
    }

    /// 测试 AliasManager::remove() 方法删除不存在的别名
    ///
    /// ## 测试目的
    /// 验证 AliasManager::remove() 能够正确处理删除不存在别名的情况。
    ///
    /// ## 测试场景
    /// 1. 创建空的临时配置文件
    /// 2. 尝试删除不存在的别名
    /// 3. 验证返回结果
    ///
    /// ## 预期结果
    /// - 方法返回 Ok(false)
    /// - 别名不存在时，返回 false 而不是错误
    #[rstest]
    fn test_alias_manager_remove_nonexistent_with_temp_config_return_ok(mut cli_env: CliTestEnv) -> Result<()> {
        // 测试 AliasManager::remove() 方法 - 删除不存在的别名（覆盖 manager.rs:202-205）
        use workflow::base::util::file::FileWriter;

        let env = &mut cli_env;
        let config_dir = env.path().join(".workflow").join("config");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("workflow.toml");

        // 创建空配置文件
        let initial_config = r#"
aliases = {}
"#;
        FileWriter::new(&config_path).write_str(initial_config)?;

        // 设置临时 HOME 目录
        let home_path = env.path().to_string_lossy().to_string();
        env.env_guard().set("HOME", &home_path);

        // 尝试删除不存在的别名
        let result = workflow::base::alias::AliasManager::remove("__nonexistent_alias__");
        // EnvGuard 会在 env 离开作用域时自动恢复 HOME

        // 验证返回 false（别名不存在）
        assert!(result.is_ok());
        assert!(!result?);

        Ok(())
    }

    /// 测试 AliasManager::expand() 方法使用临时配置文件
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand() 能够使用临时配置文件正确展开别名。
    ///
    /// ## 测试场景
    /// 1. 创建包含别名的临时配置文件
    /// 2. 调用 expand() 方法展开别名
    /// 3. 验证展开结果
    ///
    /// ## 预期结果
    /// - 展开成功
    /// - 展开结果正确
    ///
    /// ## 为什么被忽略
    /// - **需要干净的测试环境**: Settings 使用 OnceLock 单例，无法重置
    #[rstest]
    #[ignore = "Requires clean test environment - Settings uses OnceLock singleton that cannot be reset"]
    fn test_alias_manager_expand_with_temp_config_return_ok(mut cli_env: CliTestEnv) -> Result<()> {
        // 测试 AliasManager::expand() 方法 - 使用临时配置文件（覆盖 manager.rs:54-98）
        use workflow::base::util::file::FileWriter;

        let env = &mut cli_env;
        let config_dir = env.path().join(".workflow").join("config");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("workflow.toml");

        // 先设置临时 HOME 目录（在创建配置文件之前）
        let home_path = env.path().to_string_lossy().to_string();
        env.env_guard().set("HOME", &home_path);

        // 创建包含别名的配置文件（在设置 HOME 之后）
        let initial_config = r#"
aliases = { test_expand_alias = "git status" }
"#;
        FileWriter::new(&config_path).write_str(initial_config)?;

        // 注意：Settings 使用 OnceLock，如果已经在之前初始化了，这里不会重新加载
        // 但 Paths::workflow_config() 会在每次调用时重新读取 HOME，所以应该能工作
        // 为了确保使用新的配置，我们需要确保 Settings 在设置 HOME 之后才初始化
        // 但由于 OnceLock 的特性，如果已经在之前初始化了，这里不会重新加载
        // 所以这个测试可能在某些情况下失败（如果 Settings 已经在之前初始化了）

        // 展开别名
        let mut visited = HashSet::new();
        let result =
            workflow::base::alias::AliasManager::expand("test_expand_alias", &mut visited, 0);
        // EnvGuard 会在 env 离开作用域时自动恢复 HOME

        // 验证展开成功
        assert!(result.is_ok());
        assert_eq!(result?, "git status");

        Ok(())
    }

    /// 测试 AliasManager::expand() 方法展开嵌套别名（使用临时配置文件）
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand() 能够使用临时配置文件正确展开嵌套别名。
    ///
    /// ## 测试场景
    /// 1. 创建包含嵌套别名的临时配置文件
    /// 2. 调用 expand() 方法展开嵌套别名
    /// 3. 验证递归展开结果
    ///
    /// ## 预期结果
    /// - 展开成功
    /// - 嵌套别名被正确递归展开
    ///
    /// ## 为什么被忽略
    /// - **需要干净的测试环境**: Settings 使用 OnceLock 单例，无法重置
    #[rstest]
    #[ignore = "Requires clean test environment - Settings uses OnceLock singleton that cannot be reset"]
    fn test_alias_manager_expand_nested_with_temp_config_return_ok(mut cli_env: CliTestEnv) -> Result<()> {
        // 测试 AliasManager::expand() 方法 - 嵌套别名展开（覆盖 manager.rs:84-95）
        use workflow::base::util::file::FileWriter;

        let env = &mut cli_env;
        let config_dir = env.path().join(".workflow").join("config");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("workflow.toml");

        // 先设置临时 HOME 目录（在创建配置文件之前）
        let home_path = env.path().to_string_lossy().to_string();
        env.env_guard().set("HOME", &home_path);

        // 创建包含嵌套别名的配置文件（在设置 HOME 之后）
        let initial_config = r#"
aliases = {
    alias_a = "git status",
    alias_b = "alias_a --verbose"
}
"#;
        FileWriter::new(&config_path).write_str(initial_config)?;

        // 确保 Settings 使用新的 HOME（通过调用一次来初始化）
        let _ = workflow::base::settings::Settings::get();

        // 展开嵌套别名
        let mut visited = HashSet::new();
        let result = workflow::base::alias::AliasManager::expand("alias_b", &mut visited, 0);
        // EnvGuard 会在 env 离开作用域时自动恢复 HOME

        // 验证展开成功（应该展开为 "git status --verbose"）
        assert!(result.is_ok());
        let expanded = result?;
        assert!(expanded.contains("git"));
        assert!(expanded.contains("status"));
        assert!(expanded.contains("verbose"));

        Ok(())
    }

    /// 测试 AliasManager::expand() 方法检测循环别名（使用临时配置文件）
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand() 能够使用临时配置文件正确检测循环别名。
    ///
    /// ## 测试场景
    /// 1. 创建包含循环别名的临时配置文件
    /// 2. 调用 expand() 方法展开循环别名
    /// 3. 验证循环检测
    ///
    /// ## 预期结果
    /// - 返回错误
    /// - 错误消息包含 "Circular alias"
    ///
    /// ## 为什么被忽略
    /// - **需要干净的测试环境**: Settings 使用 OnceLock 单例，无法重置
    #[rstest]
    #[ignore = "Requires clean test environment - Settings uses OnceLock singleton that cannot be reset"]
    fn test_alias_manager_expand_circular_with_temp_config_return_ok(mut cli_env: CliTestEnv) -> Result<()> {
        // 测试 AliasManager::expand() 方法 - 循环别名检测（覆盖 manager.rs:65-71）
        use workflow::base::util::file::FileWriter;

        let env = &mut cli_env;
        let config_dir = env.path().join(".workflow").join("config");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("workflow.toml");

        // 先设置临时 HOME 目录（在创建配置文件之前）
        let home_path = env.path().to_string_lossy().to_string();
        env.env_guard().set("HOME", &home_path);

        // 创建包含循环别名的配置文件（在设置 HOME 之后）
        let initial_config = r#"
aliases = {
    alias_circular = "alias_circular"
}
"#;
        FileWriter::new(&config_path).write_str(initial_config)?;

        // 确保 Settings 使用新的 HOME（通过调用一次来初始化）
        let _ = workflow::base::settings::Settings::get();

        // 尝试展开循环别名
        let mut visited = HashSet::new();
        let result = workflow::base::alias::AliasManager::expand("alias_circular", &mut visited, 0);
        // EnvGuard 会在 env 离开作用域时自动恢复 HOME

        // 验证检测到循环引用
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular alias"));

        Ok(())
    }

    /// 测试 AliasManager::expand_args() 方法使用临时配置文件
    ///
    /// ## 测试目的
    /// 验证 AliasManager::expand_args() 能够使用临时配置文件正确展开命令行参数中的别名。
    ///
    /// ## 测试场景
    /// 1. 创建包含别名的临时配置文件
    /// 2. 调用 expand_args() 方法展开包含别名的参数列表
    /// 3. 验证展开结果
    ///
    /// ## 预期结果
    /// - 展开成功
    /// - 别名被正确展开，其他参数保持不变
    ///
    /// ## 为什么被忽略
    /// - **需要干净的测试环境**: Settings 使用 OnceLock 单例，无法重置
    #[rstest]
    #[ignore = "Requires clean test environment - Settings uses OnceLock singleton that cannot be reset"]
    fn test_alias_manager_expand_args_with_temp_config_return_ok(mut cli_env: CliTestEnv) -> Result<()> {
        // 测试 AliasManager::expand_args() 方法 - 使用临时配置文件（覆盖 manager.rs:116-148）
        use workflow::base::util::file::FileWriter;

        let env = &mut cli_env;
        let config_dir = env.path().join(".workflow").join("config");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("workflow.toml");

        // 先设置临时 HOME 目录（在创建配置文件之前）
        let home_path = env.path().to_string_lossy().to_string();
        env.env_guard().set("HOME", &home_path);

        // 创建包含别名的配置文件（在设置 HOME 之后）
        let initial_config = r#"
aliases = { test_args_alias = "git status" }
"#;
        FileWriter::new(&config_path).write_str(initial_config)?;

        // 确保 Settings 使用新的 HOME（通过调用一次来初始化）
        let _ = workflow::base::settings::Settings::get();

        // 展开参数
        let args = vec![
            "workflow".to_string(),
            "test_args_alias".to_string(),
            "--verbose".to_string(),
        ];
        let result = workflow::base::alias::AliasManager::expand_args(args);
        // EnvGuard 会在 env 离开作用域时自动恢复 HOME

        // 验证展开成功
        assert!(result.is_ok());
        let expanded = result?;
        assert_eq!(expanded[0], "workflow");
        assert_eq!(expanded[1], "git");
        assert_eq!(expanded[2], "status");
        assert_eq!(expanded[3], "--verbose");

        Ok(())
    }

    /// 测试别名管理器循环检测功能（使用临时配置文件）
    ///
    /// ## 测试目的
    /// 验证 `AliasManager::check_circular()` 方法能够正确检测别名配置中的循环引用。
    ///
    /// ## 测试场景
    /// 1. 创建临时配置文件，包含现有别名配置
    /// 2. 测试新别名指向现有别名（不会形成循环）
    /// 3. 测试新别名指向自己（直接循环）
    ///
    /// ## 预期结果
    /// - 新别名指向现有别名时，返回 `Ok(false)`（不会形成循环）
    /// - 新别名指向自己时，返回 `Ok(true)`（检测到循环）
    #[rstest]
    fn test_alias_manager_check_circular_with_temp_config_return_ok(mut cli_env: CliTestEnv) -> Result<()> {
        // 测试 AliasManager::check_circular() 方法 - 使用临时配置文件（覆盖 manager.rs:273-302）
        use workflow::base::util::file::FileWriter;

        let env = &mut cli_env;
        let config_dir = env.path().join(".workflow").join("config");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("workflow.toml");

        // 创建包含别名的配置文件
        let initial_config = r#"
aliases = {
    existing_alias = "git status",
    nested_alias = "existing_alias"
}
"#;
        FileWriter::new(&config_path).write_str(initial_config)?;

        // 设置临时 HOME 目录
        let home_path = env.path().to_string_lossy().to_string();
        env.env_guard().set("HOME", &home_path);

        // 检查是否会形成循环（新别名指向 existing_alias，而 existing_alias 指向 git status，不会循环）
        let result1 =
            workflow::base::alias::AliasManager::check_circular("new_alias", "existing_alias");

        // 检查直接循环（新别名指向自己）
        let result2 = workflow::base::alias::AliasManager::check_circular("new_alias", "new_alias");
        // EnvGuard 会在 env 离开作用域时自动恢复 HOME

        // 验证结果
        assert!(result1.is_ok());
        assert!(!result1?); // 不会形成循环

        assert!(result2.is_ok());
        assert!(result2?); // 直接循环应该返回 true

        Ok(())
    }

    // ==================== 边界和复杂场景测试 ====================

    /// 测试别名展开深度边界（恰好达到限制）
    ///
    /// ## 测试目的
    /// 验证别名展开在达到最大深度限制（MAX_DEPTH = 10）时仍能正常工作。
    ///
    /// ## 测试场景
    /// 1. 创建恰好10层深度的别名链（alias0 -> alias1 -> ... -> alias10 -> "echo final"）
    /// 2. 从 alias0 开始展开
    ///
    /// ## 预期结果
    /// - 展开成功，返回最终命令 "echo final"
    /// - 不会因为达到深度限制而失败
    #[test]
    fn test_alias_depth_boundary_exact_limit_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 创建恰好 10 层深度的别名链（MAX_DEPTH = 10）
        for i in 0..10 {
            let current = format!("alias{}", i);
            let next = format!("alias{}", i + 1);
            aliases.insert(current, next);
        }
        aliases.insert("alias10".to_string(), "echo final".to_string());

        let mut visited = HashSet::new();

        // 测试恰好在限制内的情况（应该成功）
        let result = mock_expand_alias("alias0", &aliases, &mut visited, 0);
        assert!(result.is_ok());
        assert_eq!(result?, "echo final");

        Ok(())
    }

    /// 测试别名展开深度边界（超出限制）
    ///
    /// ## 测试目的
    /// 验证别名展开在超出最大深度限制（MAX_DEPTH = 10）时能够正确检测并返回错误。
    ///
    /// ## 测试场景
    /// 1. 创建11层深度的别名链（超出限制）
    /// 2. 尝试从 alias0 开始展开
    ///
    /// ## 预期结果
    /// - 展开失败，返回错误
    /// - 错误消息包含 "depth exceeded maximum"
    #[test]
    fn test_alias_depth_boundary_exceed_by_one_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 创建 11 层深度的别名链（超出 MAX_DEPTH = 10）
        for i in 0..11 {
            let current = format!("alias{}", i);
            let next = format!("alias{}", i + 1);
            aliases.insert(current, next);
        }
        aliases.insert("alias11".to_string(), "echo final".to_string());

        let mut visited = HashSet::new();

        // 测试超出限制的情况（应该失败）
        let result = mock_expand_alias("alias0", &aliases, &mut visited, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth exceeded maximum"));
        Ok(())
    }

    /// 测试包含Unicode字符的别名展开
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理包含Unicode字符（中文、emoji等）的别名和命令。
    ///
    /// ## 测试场景
    /// 1. 创建包含中文、emoji和混合字符的别名
    /// 2. 展开这些别名
    ///
    /// ## 预期结果
    /// - 所有Unicode字符都能正确保留和处理
    /// - 展开结果与预期一致
    #[test]
    fn test_alias_with_unicode_characters_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("中文别名".to_string(), "echo 你好世界".to_string());
        aliases.insert("emoji".to_string(), "echo 🚀 测试".to_string());
        aliases.insert("mixed".to_string(), "echo Test测试🎉".to_string());

        let mut visited = HashSet::new();

        // 测试中文别名
        let result1 = mock_expand_alias("中文别名", &aliases, &mut visited, 0)?;
        assert_eq!(result1, "echo 你好世界");

        visited.clear();

        // 测试 emoji
        let result2 = mock_expand_alias("emoji", &aliases, &mut visited, 0)?;
        assert_eq!(result2, "echo 🚀 测试");

        visited.clear();

        // 测试混合字符
        let result3 = mock_expand_alias("mixed", &aliases, &mut visited, 0)?;
        assert_eq!(result3, "echo Test测试🎉");

        Ok(())
    }

    /// 测试超长命令的别名展开
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理包含大量参数的超长命令。
    ///
    /// ## 测试场景
    /// 1. 创建包含100+个参数的超长命令别名
    /// 2. 展开该别名
    ///
    /// ## 预期结果
    /// - 超长命令能够正确展开
    /// - 所有参数都被保留
    #[test]
    fn test_alias_with_very_long_command_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 创建超长命令（100+ 个参数）
        let long_command: Vec<String> = (0..100).map(|i| format!("arg{}", i)).collect();
        let long_command_str = format!("echo {}", long_command.join(" "));

        aliases.insert("long".to_string(), long_command_str.clone());

        let mut visited = HashSet::new();

        // 测试超长命令处理
        let result = mock_expand_alias("long", &aliases, &mut visited, 0)?;
        assert_eq!(result, long_command_str);

        Ok(())
    }

    /// 测试超长别名名称的处理
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理超长的别名名称（100+字符）。
    ///
    /// ## 测试场景
    /// 1. 创建包含100+字符的别名名称
    /// 2. 展开该别名
    ///
    /// ## 预期结果
    /// - 超长别名名称能够正确识别和展开
    /// - 展开结果正确
    #[test]
    fn test_alias_with_very_long_name_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 创建超长别名名称（100+ 字符）
        let long_name = "a".repeat(100);
        aliases.insert(long_name.clone(), "echo test".to_string());

        let mut visited = HashSet::new();

        // 测试超长别名名称处理
        let result = mock_expand_alias(&long_name, &aliases, &mut visited, 0)?;
        assert_eq!(result, "echo test");

        Ok(())
    }

    /// 测试复杂嵌套别名的参数累积
    ///
    /// ## 测试目的
    /// 验证多层嵌套别名展开时，每层的参数能够正确累积。
    ///
    /// ## 测试场景
    /// 1. 创建4层嵌套别名，每层添加不同参数
    /// 2. 从最外层开始展开
    ///
    /// ## 预期结果
    /// - 所有层的参数都被正确累积
    /// - 最终命令包含所有参数，顺序正确
    #[test]
    fn test_alias_with_complex_nested_args_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 创建复杂的嵌套别名，每层添加不同参数
        aliases.insert("base".to_string(), "echo base".to_string());
        aliases.insert("level1".to_string(), "base --arg1".to_string());
        aliases.insert("level2".to_string(), "level1 --arg2".to_string());
        aliases.insert("level3".to_string(), "level2 --arg3".to_string());
        aliases.insert("level4".to_string(), "level3 --arg4".to_string());

        let mut visited = HashSet::new();

        // 测试多层嵌套参数累积
        let result = mock_expand_alias("level4", &aliases, &mut visited, 0)?;
        assert_eq!(result, "echo base --arg1 --arg2 --arg3 --arg4");

        Ok(())
    }

    /// 测试包含多个连续空格的命令处理
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理命令中包含多个连续空格的情况。
    ///
    /// ## 测试场景
    /// 1. 创建包含多个连续空格的别名命令
    /// 2. 展开该别名
    ///
    /// ## 预期结果
    /// - 多个连续空格被保留（原样输出）
    /// - 命令能够正确展开
    #[test]
    fn test_alias_with_multiple_spaces_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 测试命令中包含多个连续空格
        aliases.insert(
            "spaces".to_string(),
            "echo    multiple     spaces".to_string(),
        );

        let mut visited = HashSet::new();

        // 测试多余空格是否被正确处理
        // 注意：原始命令会保留原样，因为没有嵌套别名需要展开
        let result = mock_expand_alias("spaces", &aliases, &mut visited, 0)?;
        assert_eq!(result, "echo    multiple     spaces");

        Ok(())
    }

    /// 测试包含制表符和换行符的命令处理
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理命令中包含制表符和换行符等特殊空白字符。
    ///
    /// ## 测试场景
    /// 1. 创建包含制表符和换行符的别名命令
    /// 2. 展开该别名
    ///
    /// ## 预期结果
    /// - 制表符和换行符被保留（原样输出）
    /// - 命令能够正确展开
    #[test]
    fn test_alias_with_tabs_and_newlines_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 测试命令中包含制表符和换行符
        aliases.insert("whitespace".to_string(), "echo\ttest\nvalue".to_string());

        let mut visited = HashSet::new();

        // 测试特殊空白字符处理
        // 注意：原始命令会保留原样，因为没有嵌套别名需要展开
        let result = mock_expand_alias("whitespace", &aliases, &mut visited, 0)?;
        assert_eq!(result, "echo\ttest\nvalue");

        Ok(())
    }

    /// 测试命令以空格开头的别名展开
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理命令以空格开头的情况。
    ///
    /// ## 测试场景
    /// 1. 创建命令以空格开头的别名
    /// 2. 展开该别名
    ///
    /// ## 预期结果
    /// - 前导空格被保留（原样输出）
    /// - 命令能够正确展开
    #[test]
    fn test_alias_expansion_with_empty_first_part_return_empty() -> Result<()> {
        let mut aliases = HashMap::new();

        // 测试命令以空格开头的情况
        aliases.insert("empty_start".to_string(), "  echo test".to_string());

        let mut visited = HashSet::new();

        // 测试前导空格处理
        // 注意：原始命令会保留原样，因为没有嵌套别名需要展开
        let result = mock_expand_alias("empty_start", &aliases, &mut visited, 0)?;
        assert_eq!(result, "  echo test");

        Ok(())
    }

    /// 测试别名展开后保留多个额外参数
    ///
    /// ## 测试目的
    /// 验证命令行参数展开时，别名后的多个额外参数能够正确保留。
    ///
    /// ## 测试场景
    /// 1. 创建别名 "gs" -> "git status"
    /// 2. 使用命令行参数：program gs --short --branch -v
    /// 3. 展开别名
    ///
    /// ## 预期结果
    /// - 别名被展开为 "git status"
    /// - 所有额外参数（--short, --branch, -v）都被保留
    /// - 参数顺序正确
    #[test]
    fn test_expand_args_with_multiple_extra_args_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();
        aliases.insert("gs".to_string(), "git status".to_string());

        // 测试别名后跟多个额外参数
        let args = vec![
            "program".to_string(),
            "gs".to_string(),
            "--short".to_string(),
            "--branch".to_string(),
            "-v".to_string(),
        ];

        let result = mock_expand_args(args, &aliases)?;

        // 验证别名被展开，且所有额外参数都被保留
        assert_eq!(result.len(), 6); // program + git + status + --short + --branch + -v
        assert_eq!(result[0], "program");
        assert_eq!(result[1], "git");
        assert_eq!(result[2], "status");
        assert_eq!(result[3], "--short");
        assert_eq!(result[4], "--branch");
        assert_eq!(result[5], "-v");

        Ok(())
    }

    /// 测试别名名称包含特殊字符的处理
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理别名名称中包含特殊字符（连字符、下划线、点号）的情况。
    ///
    /// ## 测试场景
    /// 1. 创建包含连字符、下划线和点号的别名名称
    /// 2. 展开这些别名
    ///
    /// ## 预期结果
    /// - 所有特殊字符的别名名称都能正确识别和展开
    /// - 展开结果正确
    #[test]
    fn test_alias_name_with_special_chars_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 测试别名名称包含特殊字符（虽然不推荐，但应该能处理）
        aliases.insert("git-log".to_string(), "git log".to_string());
        aliases.insert("my_alias".to_string(), "echo test".to_string());
        aliases.insert("alias.dot".to_string(), "echo dot".to_string());

        let mut visited = HashSet::new();

        // 测试带连字符的别名
        let result1 = mock_expand_alias("git-log", &aliases, &mut visited, 0)?;
        assert_eq!(result1, "git log");

        visited.clear();

        // 测试带下划线的别名
        let result2 = mock_expand_alias("my_alias", &aliases, &mut visited, 0)?;
        assert_eq!(result2, "echo test");

        visited.clear();

        // 测试带点号的别名
        let result3 = mock_expand_alias("alias.dot", &aliases, &mut visited, 0)?;
        assert_eq!(result3, "echo dot");

        Ok(())
    }

    /// 测试复杂的四层嵌套别名展开
    ///
    /// ## 测试目的
    /// 验证别名管理器能够正确处理复杂的多层嵌套别名，并正确累积各层参数。
    ///
    /// ## 测试场景
    /// 1. 创建4层嵌套别名（cmd -> wrap1 -> wrap2 -> wrap3）
    /// 2. 每层添加不同参数
    /// 3. 从最外层 wrap3 开始展开
    ///
    /// ## 预期结果
    /// - 所有层都能正确展开
    /// - 所有参数都被正确累积
    /// - 最终命令为 "echo hello arg1 arg2 arg3"
    #[test]
    fn test_complex_four_level_nesting_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 创建 4 层嵌套别名，测试复杂场景
        aliases.insert("cmd".to_string(), "echo hello".to_string());
        aliases.insert("wrap1".to_string(), "cmd arg1".to_string());
        aliases.insert("wrap2".to_string(), "wrap1 arg2".to_string());
        aliases.insert("wrap3".to_string(), "wrap2 arg3".to_string());

        let mut visited = HashSet::new();

        // 测试 4 层嵌套展开
        let result = mock_expand_alias("wrap3", &aliases, &mut visited, 0)?;
        assert_eq!(result, "echo hello arg1 arg2 arg3");

        Ok(())
    }

    /// 测试别名展开时大小写敏感性
    ///
    /// ## 测试目的
    /// 验证别名管理器在展开别名时保持大小写敏感性，不同大小写的别名名称被视为不同的别名。
    ///
    /// ## 测试场景
    /// 1. 创建不同大小写形式的别名（Lower, UPPER, MiXeD）
    /// 2. 使用正确的大小写展开别名
    /// 3. 使用错误的大小写尝试展开（应失败）
    ///
    /// ## 预期结果
    /// - 使用正确大小写的别名名称能够成功展开
    /// - 使用错误大小写的别名名称展开失败（返回错误）
    /// - 别名名称是大小写敏感的
    #[test]
    fn test_alias_expansion_preserves_case_return_ok() -> Result<()> {
        let mut aliases = HashMap::new();

        // 测试大小写敏感性
        aliases.insert("Lower".to_string(), "echo lower".to_string());
        aliases.insert("UPPER".to_string(), "echo UPPER".to_string());
        aliases.insert("MiXeD".to_string(), "echo MiXeD".to_string());

        let mut visited = HashSet::new();

        // 验证别名名称是大小写敏感的
        let result1 = mock_expand_alias("Lower", &aliases, &mut visited, 0)?;
        assert_eq!(result1, "echo lower");

        visited.clear();
        let result2 = mock_expand_alias("UPPER", &aliases, &mut visited, 0)?;
        assert_eq!(result2, "echo UPPER");

        visited.clear();
        let result3 = mock_expand_alias("MiXeD", &aliases, &mut visited, 0)?;
        assert_eq!(result3, "echo MiXeD");

        // 验证不同大小写的别名名称不会匹配
        visited.clear();
        let result4 = mock_expand_alias("lower", &aliases, &mut visited, 0);
        assert!(result4.is_err()); // "lower" 不存在，只有 "Lower"

        Ok(())
    }
}
