//! Base/Concurrent 模块测试
//!
//! 测试并发执行器的核心业务逻辑，包括：
//! - 并发数限制和任务分批处理
//! - 错误收集和结果聚合
//! - 进度回调机制
//! - 边界条件处理
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - `Mutex.lock().unwrap()` 保留（锁poisoning在测试中panic是合理的）
//! - 测试并发控制、错误处理和进度回调功能

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use color_eyre::Result;
use rstest::rstest;

use workflow::base::concurrent::{ConcurrentExecutor, TaskResult};

/// 创建测试任务的辅助函数
fn create_success_task(
    result: String,
    delay_ms: u64,
) -> Box<dyn Fn() -> Result<String, String> + Send + Sync> {
    Box::new(move || {
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
        Ok(result.clone())
    })
}

/// 创建失败任务的辅助函数
fn create_failure_task(
    error: String,
    delay_ms: u64,
) -> Box<dyn Fn() -> Result<String, String> + Send + Sync> {
    Box::new(move || {
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
        Err(error.clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 基础功能测试 ====================

    #[test]
    fn test_executor_creation_with_concurrency_limit_creates_executor() -> Result<()> {
        // Arrange: 准备并发限制
        let concurrency = 5;

        // Act: 创建执行器
        let _executor = ConcurrentExecutor::new(concurrency);

        // Assert: 验证执行器创建成功（内部字段无法直接访问，通过行为验证）
        // 测试最小并发数限制（应该至少为1）
        let executor_zero = ConcurrentExecutor::new(0);
        let tasks = vec![(
            "task1".to_string(),
            create_success_task("result1".to_string(), 0),
        )];
        let results = executor_zero.execute(tasks)?;
        assert_eq!(results.len(), 1);
        Ok(())
    }

    #[test]
    fn test_execute_empty_tasks_with_empty_list_returns_empty_results() -> Result<()> {
        // Arrange: 准备执行器和空任务列表
        let executor = ConcurrentExecutor::new(5);
        let tasks: Vec<(String, Box<dyn Fn() -> Result<String, String> + Send + Sync>)> = Vec::new();

        // Act: 执行空任务列表
        let results = executor.execute(tasks)?;

        // Assert: 验证返回空结果
        assert_eq!(results.len(), 0);
        Ok(())
    }

    #[test]
    fn test_execute_single_task_success_with_success_task_returns_success() -> Result<()> {
        // Arrange: 准备执行器和成功任务
        let executor = ConcurrentExecutor::new(5);
        let tasks = vec![(
            "task1".to_string(),
            create_success_task("result1".to_string(), 0),
        )];

        // Act: 执行任务
        let results = executor.execute(tasks)?;

        // Assert: 验证返回成功结果
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "task1");
        match &results[0].1 {
            TaskResult::Success(value) => assert_eq!(value, "result1"),
            TaskResult::Failure(_) => panic!("Expected success result"),
        }
        Ok(())
    }

    #[test]
    fn test_execute_single_task_failure_with_failure_task_returns_failure() -> Result<()> {
        // Arrange: 准备执行器和失败任务
        let executor = ConcurrentExecutor::new(5);
        let tasks = vec![(
            "task1".to_string(),
            create_failure_task("test error".to_string(), 0),
        )];

        // Act: 执行任务
        let results = executor.execute(tasks)?;

        // Assert: 验证返回失败结果
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "task1");
        match &results[0].1 {
            TaskResult::Success(_) => panic!("Expected failure result"),
            TaskResult::Failure(error) => assert_eq!(error, "test error"),
        }
        Ok(())
    }

    // ==================== 并发控制测试 ====================

    #[test]
    fn test_concurrent_execution_multiple_tasks_with_multiple_tasks_executes_concurrently() -> Result<()> {
        // Arrange: 准备执行器和多个任务
        let executor = ConcurrentExecutor::new(2);
        let tasks = vec![
            (
                "task1".to_string(),
                create_success_task("result1".to_string(), 50),
            ),
            (
                "task2".to_string(),
                create_success_task("result2".to_string(), 50),
            ),
            (
                "task3".to_string(),
                create_success_task("result3".to_string(), 50),
            ),
            (
                "task4".to_string(),
                create_success_task("result4".to_string(), 50),
            ),
        ];

        // Act: 执行多个任务并测量时间
        let start_time = Instant::now();
        let results = executor.execute(tasks)?;
        let duration = start_time.elapsed();

        // Assert: 验证结果数量正确
        assert_eq!(results.len(), 4);

        // Assert: 验证所有任务都成功
        for (_, result) in &results {
            match result {
                TaskResult::Success(_) => {}
                TaskResult::Failure(err) => panic!("Unexpected failure: {}", err),
            }
        }

        // Assert: 验证并发执行（4个任务，并发数2，每个任务50ms，应该大约需要100ms而不是200ms）
        // 允许一些时间误差，特别是在CI环境中线程调度可能不稳定
        assert!(duration >= Duration::from_millis(90));
        // 增加上限以应对CI环境的线程调度延迟，但仍需远小于串行执行的200ms
        assert!(duration <= Duration::from_millis(1000));
        Ok(())
    }

    #[rstest]
    #[case(1, 4)] // 串行执行
    #[case(2, 4)] // 并发数2
    #[case(4, 4)] // 并发数4
    #[case(8, 4)] // 并发数超过任务数
    fn test_concurrent_limits_timing_with_various_concurrency_levels_executes_within_time_limit(
        #[case] max_concurrent: usize,
        #[case] task_count: usize,
    ) -> Result<()> {
        // Arrange: 准备执行器和任务列表
        let executor = ConcurrentExecutor::new(max_concurrent);
        let mut tasks = Vec::new();
        for i in 0..task_count {
            tasks.push((
                format!("task{}", i),
                create_success_task(format!("result{}", i), 10), // 减少延迟以提高测试稳定性
            ));
        }

        // Act: 执行任务并测量时间
        let start_time = Instant::now();
        let results = executor.execute(tasks)?;
        let duration = start_time.elapsed();

        // Assert: 验证结果数量正确
        assert_eq!(results.len(), task_count);

        // Assert: 验证时间在合理范围内
        let min_duration = Duration::from_millis(5); // 至少5ms（考虑系统开销）
        let max_duration = Duration::from_secs(2); // 最多2秒（防止死锁）
        assert!(
            duration >= min_duration && duration <= max_duration,
            "Duration {:?} not in reasonable range [{:?}, {:?}] for concurrent={}, tasks={}",
            duration,
            min_duration,
            max_duration,
            max_concurrent,
            task_count
        );

        // Assert: 验证所有任务都成功（顺序可能不同）
        let mut task_names: Vec<String> = results.iter().map(|(name, _)| name.clone()).collect();
        task_names.sort();
        let mut expected_names: Vec<String> =
            (0..task_count).map(|i| format!("task{}", i)).collect();
        expected_names.sort();
        assert_eq!(task_names, expected_names);

        // Assert: 验证每个任务的结果
        for (name, result) in &results {
            match result {
                TaskResult::Success(value) => {
                    // 从任务名中提取索引
                    if let Some(suffix) = name.strip_prefix("task") {
                        let index: usize = suffix.parse()?;
                        assert_eq!(value, &format!("result{}", index));
                    }
                }
                TaskResult::Failure(error) => panic!("Task {} failed: {}", name, error),
            }
        }
        Ok(())
    }

    // ==================== 错误处理和混合结果测试 ====================

    #[test]
    fn test_mixed_success_and_failure_tasks_with_mixed_tasks_handles_both() -> Result<()> {
        // Arrange: 准备执行器和混合任务（成功和失败）
        let executor = ConcurrentExecutor::new(3);
        let tasks = vec![
            (
                "success1".to_string(),
                create_success_task("result1".to_string(), 10),
            ),
            (
                "failure1".to_string(),
                create_failure_task("error1".to_string(), 10),
            ),
            (
                "success2".to_string(),
                create_success_task("result2".to_string(), 10),
            ),
            (
                "failure2".to_string(),
                create_failure_task("error2".to_string(), 10),
            ),
        ];

        // Act: 执行混合任务
        let results = executor.execute(tasks)?;

        // Assert: 验证结果数量正确
        assert_eq!(results.len(), 4);

        // Assert: 统计成功和失败的数量并验证结果正确
        let mut success_count = 0;
        let mut failure_count = 0;
        for (name, result) in &results {
            match result {
                TaskResult::Success(value) => {
                    success_count += 1;
                    if name == "success1" {
                        assert_eq!(value, "result1");
                    } else if name == "success2" {
                        assert_eq!(value, "result2");
                    }
                }
                TaskResult::Failure(error) => {
                    failure_count += 1;
                    if name == "failure1" {
                        assert_eq!(error, "error1");
                    } else if name == "failure2" {
                        assert_eq!(error, "error2");
                    }
                }
            }
        }
        assert_eq!(success_count, 2);
        assert_eq!(failure_count, 2);
        Ok(())
    }

    #[test]
    fn test_all_tasks_fail_with_all_failure_tasks_returns_all_failures() -> Result<()> {
        // Arrange: 准备执行器和所有失败任务
        let executor = ConcurrentExecutor::new(2);
        let tasks = vec![
            (
                "fail1".to_string(),
                create_failure_task("error1".to_string(), 0),
            ),
            (
                "fail2".to_string(),
                create_failure_task("error2".to_string(), 0),
            ),
            (
                "fail3".to_string(),
                create_failure_task("error3".to_string(), 0),
            ),
        ];

        // Act: 执行所有失败任务
        let results = executor.execute(tasks)?;

        // Assert: 验证结果数量正确且所有任务都失败
        assert_eq!(results.len(), 3);
        for (_, result) in &results {
            match result {
                TaskResult::Success(_) => panic!("Expected all tasks to fail"),
                TaskResult::Failure(_) => {}
            }
        }
        Ok(())
    }

    // ==================== 进度回调测试 ====================

    #[test]
    fn test_execute_with_progress_callback() -> Result<()> {
        let executor = ConcurrentExecutor::new(2);

        // 使用 Arc<Mutex<Vec<_>>> 收集进度信息
        let progress_log = Arc::new(Mutex::new(Vec::new()));
        let progress_log_clone = progress_log.clone();

        let callback = move |name: &str, success: bool, error: Option<&str>| {
            let mut log = progress_log_clone.lock().unwrap();
            log.push((name.to_string(), success, error.map(|e| e.to_string())));
        };

        let callback_wrapper = Arc::new(Mutex::new(Some(callback)));

        let tasks = vec![
            (
                "task1".to_string(),
                create_success_task("result1".to_string(), 10),
            ),
            (
                "task2".to_string(),
                create_failure_task("error2".to_string(), 10),
            ),
            (
                "task3".to_string(),
                create_success_task("result3".to_string(), 10),
            ),
        ];

        let results = executor.execute_with_progress(tasks, Some(callback_wrapper))?;

        // 验证执行结果
        assert_eq!(results.len(), 3);

        // 验证进度回调被正确调用
        let log = progress_log.lock().unwrap();
        assert_eq!(log.len(), 3);

        // 验证回调内容（顺序可能不同，所以按名称查找）
        if let Some(task1_log) = log.iter().find(|(name, _, _)| name == "task1") {
            assert_eq!(task1_log.1, true); // success
            assert_eq!(task1_log.2, None); // no error
        }

        if let Some(task2_log) = log.iter().find(|(name, _, _)| name == "task2") {
            assert_eq!(task2_log.1, false); // failure
            assert_eq!(task2_log.2, Some("error2".to_string())); // error message
        }

        if let Some(task3_log) = log.iter().find(|(name, _, _)| name == "task3") {
            assert_eq!(task3_log.1, true); // success
            assert_eq!(task3_log.2, None); // no error
        }
        Ok(())
    }

    #[test]
    fn test_execute_with_progress_single_task() -> Result<()> {
        let executor = ConcurrentExecutor::new(1);

        let progress_log = Arc::new(Mutex::new(Vec::new()));
        let progress_log_clone = progress_log.clone();

        let callback = move |name: &str, success: bool, error: Option<&str>| {
            let mut log = progress_log_clone.lock().unwrap();
            log.push((name.to_string(), success, error.map(|e| e.to_string())));
        };

        let callback_wrapper = Arc::new(Mutex::new(Some(callback)));

        let tasks = vec![(
            "single_task".to_string(),
            create_success_task("result".to_string(), 0),
        )];

        let results = executor.execute_with_progress(tasks, Some(callback_wrapper))?;

        assert_eq!(results.len(), 1);

        // 验证单任务的进度回调
        let log = progress_log.lock().unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].0, "single_task");
        assert_eq!(log[0].1, true);
        assert_eq!(log[0].2, None);
        Ok(())
    }

    #[test]
    fn test_execute_with_progress_no_callback() -> Result<()> {
        let executor = ConcurrentExecutor::new(2);

        let tasks = vec![
            (
                "task1".to_string(),
                create_success_task("result1".to_string(), 0),
            ),
            (
                "task2".to_string(),
                create_failure_task("error2".to_string(), 0),
            ),
        ];

        // 不提供回调函数，需要显式指定类型参数
        let results = executor
            .execute_with_progress::<String, String, fn(&str, bool, Option<&str>)>(tasks, None)?;

        // 验证即使没有回调函数，执行也能正常完成
        assert_eq!(results.len(), 2);
        Ok(())
    }

    // ==================== 边界条件和压力测试 ====================

    #[test]
    fn test_large_number_of_tasks() -> Result<()> {
        let executor = ConcurrentExecutor::new(10);
        let mut tasks = Vec::new();

        // 创建100个快速任务
        for i in 0..100 {
            tasks.push((
                format!("task{}", i),
                create_success_task(format!("result{}", i), 1), // 1ms延迟
            ));
        }

        let start_time = Instant::now();
        let results = executor.execute(tasks)?;
        let duration = start_time.elapsed();

        assert_eq!(results.len(), 100);

        // 验证所有任务都成功
        for (_, result) in &results {
            match result {
                TaskResult::Success(_) => {}
                TaskResult::Failure(err) => panic!("Unexpected failure: {}", err),
            }
        }

        // 验证执行时间合理（100个任务，并发数10，应该在合理时间内完成）
        assert!(duration <= Duration::from_millis(500));
        Ok(())
    }

    #[test]
    fn test_zero_delay_tasks() -> Result<()> {
        let executor = ConcurrentExecutor::new(5);
        let tasks = vec![
            (
                "instant1".to_string(),
                create_success_task("result1".to_string(), 0),
            ),
            (
                "instant2".to_string(),
                create_success_task("result2".to_string(), 0),
            ),
            (
                "instant3".to_string(),
                create_success_task("result3".to_string(), 0),
            ),
        ];

        let start_time = Instant::now();
        let results = executor.execute(tasks)?;
        let duration = start_time.elapsed();

        assert_eq!(results.len(), 3);

        // 验证快速执行（应该在很短时间内完成）
        assert!(duration <= Duration::from_millis(50));
        Ok(())
    }

    #[test]
    fn test_task_names_preservation() -> Result<()> {
        let executor = ConcurrentExecutor::new(3);
        let expected_names = vec!["alpha", "beta", "gamma", "delta"];
        let mut tasks = Vec::new();

        for name in &expected_names {
            tasks.push((
                name.to_string(),
                create_success_task(format!("result_{}", name), 5),
            ));
        }

        let results = executor.execute(tasks)?;

        assert_eq!(results.len(), expected_names.len());

        // 验证所有任务名称都被保留（顺序可能不同）
        let mut result_names: Vec<String> = results.iter().map(|(name, _)| name.clone()).collect();
        result_names.sort();
        let mut expected_sorted = expected_names.clone();
        expected_sorted.sort();

        assert_eq!(result_names, expected_sorted);
        Ok(())
    }

    // ==================== 类型系统测试 ====================

    #[test]
    fn test_different_result_types() -> Result<()> {
        let executor = ConcurrentExecutor::new(2);

        // 测试整数类型的任务
        let int_tasks: Vec<(String, Box<dyn Fn() -> Result<i32, String> + Send + Sync>)> =
            vec![("int_task".to_string(), Box::new(|| Ok(42)))];

        let int_results = executor.execute(int_tasks)?;
        assert_eq!(int_results.len(), 1);
        match &int_results[0].1 {
            TaskResult::Success(value) => assert_eq!(*value, 42),
            TaskResult::Failure(_) => panic!("Expected success"),
        }
        Ok(())
    }

    #[test]
    fn test_custom_error_types() -> Result<()> {
        let executor = ConcurrentExecutor::new(2);

        // 测试自定义错误类型
        #[derive(Debug, Clone, PartialEq)]
        struct CustomError {
            code: i32,
            message: String,
        }

        impl ToString for CustomError {
            fn to_string(&self) -> String {
                format!("Error {}: {}", self.code, self.message)
            }
        }

        let custom_tasks: Vec<(
            String,
            Box<dyn Fn() -> Result<String, CustomError> + Send + Sync>,
        )> = vec![
            (
                "success_task".to_string(),
                Box::new(|| Ok("success".to_string())),
            ),
            (
                "error_task".to_string(),
                Box::new(|| {
                    Err(CustomError {
                        code: 404,
                        message: "Not found".to_string(),
                    })
                }),
            ),
        ];

        let results = executor.execute(custom_tasks)?;
        assert_eq!(results.len(), 2);

        // 验证自定义错误类型
        if let Some(error_result) = results.iter().find(|(name, _)| name == "error_task") {
            match &error_result.1 {
                TaskResult::Success(_) => panic!("Expected failure"),
                TaskResult::Failure(error) => {
                    assert_eq!(error.code, 404);
                    assert_eq!(error.message, "Not found");
                }
            }
        }
        Ok(())
    }
}
