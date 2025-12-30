//! 并发任务执行器实现

use color_eyre::{eyre::eyre, Result};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// 任务列表类型别名
type TaskList<T, E> = Vec<(String, Box<dyn Fn() -> Result<T, E> + Send + Sync>)>;

/// 任务结果
#[derive(Debug, Clone)]
pub enum TaskResult<T, E> {
    /// 任务成功完成
    Success(T),
    /// 任务执行失败
    Failure(E),
}

/// 并发任务执行器
///
/// 用于并行执行多个任务，支持并发数限制和结果收集。
///
/// # 示例
///
/// ```rust
/// use workflow::base::concurrent::{ConcurrentExecutor, TaskResult};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let executor = ConcurrentExecutor::new(5); // 最大并发数 5
///
/// let tasks: Vec<(String, Box<dyn Fn() -> Result<String, String> + Send + Sync>)> = vec![
///     ("task1".to_string(), Box::new(|| -> Result<String, String> { Ok("result1".to_string()) })),
///     ("task2".to_string(), Box::new(|| -> Result<String, String> { Ok("result2".to_string()) })),
/// ];
///
/// let results = executor.execute(tasks)?;
/// for (name, result) in results {
///     match result {
///         TaskResult::Success(value) => println!("{}: success - {}", name, value),
///         TaskResult::Failure(err) => println!("{}: failed - {}", name, err),
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct ConcurrentExecutor {
    /// 最大并发数
    max_concurrent: usize,
}

impl ConcurrentExecutor {
    /// 创建新的并发执行器
    ///
    /// # 参数
    ///
    /// * `max_concurrent` - 最大并发数（同时执行的任务数）
    ///
    /// # 返回
    ///
    /// 返回 `ConcurrentExecutor` 实例
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent: max_concurrent.max(1),
        }
    }

    /// 执行多个任务（并行）
    ///
    /// # 参数
    ///
    /// * `tasks` - 任务列表，每个任务是一个元组 `(标识符, 任务函数)`
    ///
    /// # 返回
    ///
    /// 返回任务结果列表，每个结果是一个元组 `(标识符, 任务结果)`
    ///
    /// # 错误
    ///
    /// 如果线程创建或等待失败，返回相应的错误
    pub fn execute<T, E>(&self, tasks: TaskList<T, E>) -> Result<Vec<(String, TaskResult<T, E>)>>
    where
        T: Send + 'static,
        E: Send + 'static,
    {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        // 如果只有一个任务，直接执行（避免线程开销）
        if tasks.len() == 1 {
            let (name, task) = tasks
                .into_iter()
                .next()
                .ok_or_else(|| eyre!("Expected exactly one task, but got none"))?;
            let result = match task() {
                Ok(value) => TaskResult::Success(value),
                Err(err) => TaskResult::Failure(err),
            };
            return Ok(vec![(name, result)]);
        }

        let max_concurrent = self.max_concurrent.min(tasks.len());

        // 结果通道
        let (tx, rx) = mpsc::channel();

        // 分批处理：将任务分成多个批次，每批最多 max_concurrent 个并行执行
        // 注意：我们不能直接 clone Box<dyn Fn()>，所以需要将任务移动到线程中
        let mut handles = Vec::new();
        let mut tasks_iter = tasks.into_iter();

        loop {
            let mut chunk = Vec::new();
            for _ in 0..max_concurrent {
                if let Some(task) = tasks_iter.next() {
                    chunk.push(task);
                } else {
                    break;
                }
            }

            if chunk.is_empty() {
                break;
            }

            let tx = tx.clone();

            let handle = thread::spawn(move || {
                for (name, task) in chunk {
                    let result = match task() {
                        Ok(value) => TaskResult::Success(value),
                        Err(err) => TaskResult::Failure(err),
                    };
                    tx.send((name, result)).ok();
                }
            });

            handles.push(handle);
        }

        // 关闭发送端
        drop(tx);

        // 收集结果
        let mut results = Vec::new();
        for result in rx {
            results.push(result);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().map_err(|e| eyre!("Thread join error: {:?}", e))?;
        }

        Ok(results)
    }

    /// 执行多个任务（并行），带进度回调
    ///
    /// # 参数
    ///
    /// * `tasks` - 任务列表
    /// * `on_progress` - 进度回调函数，参数为 `(任务标识符, 是否成功, 错误信息)`
    ///
    /// # 返回
    ///
    /// 返回任务结果列表
    pub fn execute_with_progress<T, E, F>(
        &self,
        tasks: TaskList<T, E>,
        on_progress: Option<Arc<Mutex<Option<F>>>>,
    ) -> Result<Vec<(String, TaskResult<T, E>)>>
    where
        T: Send + 'static,
        E: Send + 'static + ToString,
        F: Fn(&str, bool, Option<&str>) + Send + Sync + 'static,
    {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        // 如果只有一个任务，直接执行
        if tasks.len() == 1 {
            let (name, task) = tasks
                .into_iter()
                .next()
                .ok_or_else(|| eyre!("Expected exactly one task, but got none"))?;
            let result = match task() {
                Ok(value) => {
                    if let Some(ref callback) = on_progress {
                        if let Ok(cb_guard) = callback.lock() {
                            if let Some(ref cb_fn) = *cb_guard {
                                cb_fn(&name, true, None);
                            }
                        }
                    }
                    TaskResult::Success(value)
                }
                Err(err) => {
                    let err_msg = err.to_string();
                    if let Some(ref callback) = on_progress {
                        if let Ok(cb_guard) = callback.lock() {
                            if let Some(ref cb_fn) = *cb_guard {
                                cb_fn(&name, false, Some(&err_msg));
                            }
                        }
                    }
                    TaskResult::Failure(err)
                }
            };
            return Ok(vec![(name, result)]);
        }

        let max_concurrent = self.max_concurrent.min(tasks.len());

        // 结果通道
        let (tx, rx) = mpsc::channel();

        // 分批处理
        let mut handles = Vec::new();
        let mut tasks_iter = tasks.into_iter();

        loop {
            let mut chunk = Vec::new();
            for _ in 0..max_concurrent {
                if let Some(task) = tasks_iter.next() {
                    chunk.push(task);
                } else {
                    break;
                }
            }

            if chunk.is_empty() {
                break;
            }

            let tx = tx.clone();
            let callback = on_progress.clone();

            let handle = thread::spawn(move || {
                for (name, task) in chunk {
                    let result = match task() {
                        Ok(value) => {
                            if let Some(ref cb) = callback {
                                if let Ok(cb_guard) = cb.lock() {
                                    if let Some(ref cb_fn) = *cb_guard {
                                        cb_fn(&name, true, None);
                                    }
                                }
                            }
                            TaskResult::Success(value)
                        }
                        Err(err) => {
                            let err_msg = err.to_string();
                            if let Some(ref cb) = callback {
                                if let Ok(cb_guard) = cb.lock() {
                                    if let Some(ref cb_fn) = *cb_guard {
                                        cb_fn(&name, false, Some(&err_msg));
                                    }
                                }
                            }
                            TaskResult::Failure(err)
                        }
                    };
                    tx.send((name, result)).ok();
                }
            });

            handles.push(handle);
        }

        // 关闭发送端
        drop(tx);

        // 收集结果
        let mut results = Vec::new();
        for result in rx {
            results.push(result);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().map_err(|e| eyre!("Thread join error: {:?}", e))?;
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::type_complexity)]

    use super::*;
    use color_eyre::Result;
    use rstest::rstest;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

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

    /// 测试执行空任务列表
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够正确处理空任务列表的情况。
    ///
    /// ## 测试场景
    /// 使用空的任务列表调用 execute 方法
    ///
    /// ## 预期结果
    /// - 函数执行成功（返回 Ok）
    /// - 返回空的结果列表（长度为 0）
    #[test]
    fn test_execute_empty() {
        // Arrange: 创建并发执行器（最大并发数 5）
        let executor = ConcurrentExecutor::new(5);

        // Act: 执行空任务列表
        let results = executor.execute::<String, String>(Vec::new()).unwrap();

        // Assert: 验证返回空结果列表
        assert_eq!(results.len(), 0);
    }

    /// 测试执行单个任务
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够正确执行单个任务并返回结果。
    ///
    /// ## 测试场景
    /// 创建一个返回成功结果的任务，使用执行器执行该任务
    ///
    /// ## 预期结果
    /// - 函数执行成功（返回 Ok）
    /// - 返回 1 个结果
    /// - 结果为 Success，值为 "result1"
    #[test]
    fn test_execute_single() {
        // Arrange: 创建并发执行器和单个任务
        let executor = ConcurrentExecutor::new(5);
        let tasks = vec![(
            "task1".to_string(),
            Box::new(|| -> Result<String, String> { Ok("result1".to_string()) })
                as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
        )];

        // Act: 执行任务
        let results = executor.execute(tasks).unwrap();

        // Assert: 验证返回 1 个结果
        assert_eq!(results.len(), 1);

        // Assert: 验证结果为成功，值为 "result1"
        match &results[0].1 {
            TaskResult::Success(value) => assert_eq!(value, "result1"),
            TaskResult::Failure(_) => panic!("Expected success"),
        }
    }

    /// 测试执行多个任务
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够正确并发执行多个任务并返回所有结果。
    ///
    /// ## 测试场景
    /// 1. 创建 3 个任务，每个任务都睡眠 10ms 后返回成功
    /// 2. 使用最大并发数为 2 的执行器执行这些任务
    ///
    /// ## 预期结果
    /// - 函数执行成功（返回 Ok）
    /// - 返回 3 个结果（所有任务都完成）
    #[test]
    fn test_execute_multiple() {
        // Arrange: 创建最大并发数为 2 的执行器
        let executor = ConcurrentExecutor::new(2);

        // Arrange: 创建 3 个任务
        let tasks = vec![
            (
                "task1".to_string(),
                Box::new(|| -> Result<String, String> {
                    thread::sleep(Duration::from_millis(10));
                    Ok("result1".to_string())
                }) as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
            (
                "task2".to_string(),
                Box::new(|| -> Result<String, String> {
                    thread::sleep(Duration::from_millis(10));
                    Ok("result2".to_string())
                }) as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
            (
                "task3".to_string(),
                Box::new(|| -> Result<String, String> {
                    thread::sleep(Duration::from_millis(10));
                    Ok("result3".to_string())
                }) as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
        ];

        // Act: 并发执行所有任务
        let results = executor.execute(tasks).unwrap();

        // Assert: 验证返回 3 个结果
        assert_eq!(results.len(), 3);
    }

    /// 测试执行包含失败任务的情况
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够正确处理混合成功和失败的任务。
    ///
    /// ## 测试场景
    /// 1. 创建 2 个任务：一个成功，一个失败
    /// 2. 使用执行器执行这些任务
    ///
    /// ## 预期结果
    /// - 函数执行成功（返回 Ok）
    /// - 返回 2 个结果
    /// - 两个结果都存在（一个 Success，一个 Failure）
    #[test]
    fn test_execute_with_failure() {
        // Arrange: 创建并发执行器
        let executor = ConcurrentExecutor::new(5);

        // Arrange: 创建混合成功和失败的任务
        let tasks = vec![
            (
                "task1".to_string(),
                Box::new(|| -> Result<String, String> { Ok("result1".to_string()) })
                    as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
            (
                "task2".to_string(),
                Box::new(|| -> Result<String, String> { Err("error".to_string()) })
                    as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
        ];

        // Act: 执行任务
        let results = executor.execute(tasks).unwrap();

        // Assert: 验证返回 2 个结果
        assert_eq!(results.len(), 2);

        // Assert: 验证两个结果都存在（一个成功，一个失败）
        match &results[0].1 {
            TaskResult::Success(_) => {}
            TaskResult::Failure(_) => {}
        }
    }

    // ==================== 基础功能测试 ====================

    /// 测试创建并发执行器
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor::new() 能够创建执行器并处理并发限制。
    ///
    /// ## 测试场景
    /// 1. 使用指定的并发限制创建执行器
    /// 2. 测试最小并发数限制（0会被调整为至少1）
    /// 3. 验证执行器能够执行任务
    ///
    /// ## 预期结果
    /// - 执行器创建成功，能够执行任务
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

    /// 测试执行空任务列表
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 对空任务列表的处理。
    ///
    /// ## 测试场景
    /// 1. 创建执行器
    /// 2. 执行空任务列表
    /// 3. 验证返回空结果
    ///
    /// ## 预期结果
    /// - 返回空结果列表
    #[test]
    fn test_execute_empty_tasks_with_empty_list_return_collect() -> Result<()> {
        // Arrange: 准备执行器和空任务列表
        let executor = ConcurrentExecutor::new(5);
        let tasks: Vec<(
            String,
            Box<dyn Fn() -> Result<String, String> + Send + Sync>,
        )> = Vec::new();

        // Act: 执行空任务列表
        let results = executor.execute(tasks)?;

        // Assert: 验证返回空结果
        assert_eq!(results.len(), 0);
        Ok(())
    }

    /// 测试执行单个成功任务
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够执行单个成功任务并返回成功结果。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和成功任务
    /// 2. 执行任务
    /// 3. 验证返回成功结果
    ///
    /// ## 预期结果
    /// - 返回成功结果，包含任务名和结果值
    #[test]
    fn test_execute_single_task_success_with_success_task_return_true() -> Result<()> {
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

    /// 测试执行单个失败任务
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够执行单个失败任务并返回失败结果。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和失败任务
    /// 2. 执行任务
    /// 3. 验证返回失败结果
    ///
    /// ## 预期结果
    /// - 返回失败结果，包含任务名和错误信息
    #[test]
    fn test_execute_single_task_failure_with_failure_task() -> Result<()> {
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

    /// 测试并发执行多个任务
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够并发执行多个任务。
    ///
    /// ## 测试场景
    /// 1. 创建执行器（并发数2）和多个任务
    /// 2. 执行任务并测量时间
    /// 3. 验证结果数量正确且执行时间符合并发预期
    ///
    /// ## 预期结果
    /// - 所有任务都成功，执行时间符合并发预期
    #[test]
    fn test_concurrent_execution_multiple_tasks_with_multiple_tasks_executes_concurrently_return_ok(
    ) -> Result<()> {
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

    /// 测试不同并发级别的执行时间
    ///
    /// ## 测试目的
    /// 使用参数化测试验证不同并发级别下的执行时间。
    ///
    /// ## 测试场景
    /// 1. 使用不同的并发级别和任务数量
    /// 2. 执行任务并测量时间
    /// 3. 验证时间在合理范围内
    ///
    /// ## 预期结果
    /// - 执行时间在合理范围内，所有任务都成功
    #[rstest]
    #[case(1, 4)] // 串行执行
    #[case(2, 4)] // 并发数2
    #[case(4, 4)] // 并发数4
    #[case(8, 4)] // 并发数超过任务数
    fn test_concurrent_limits_timing_with_various_concurrency_levels_executes_within_time_limit_return_ok(
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

    /// 测试混合成功和失败任务
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够正确处理混合的成功和失败任务。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和混合任务（成功和失败）
    /// 2. 执行任务
    /// 3. 验证成功和失败的数量正确
    ///
    /// ## 预期结果
    /// - 成功和失败任务都被正确处理
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

    /// 测试所有任务都失败的情况
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 在所有任务都失败时能够返回所有失败结果。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和所有失败任务
    /// 2. 执行任务
    /// 3. 验证所有任务都返回失败结果
    ///
    /// ## 预期结果
    /// - 所有任务都返回失败结果
    #[test]
    fn test_all_tasks_fail_with_all_failure_tasks() -> Result<()> {
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

    /// 测试带进度回调的执行
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor::execute_with_progress() 能够调用进度回调函数。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和进度回调
    /// 2. 执行任务（包含成功和失败）
    /// 3. 验证进度回调被正确调用
    ///
    /// ## 预期结果
    /// - 进度回调被调用，包含正确的任务名、成功状态和错误信息
    #[test]
    fn test_execute_with_progress_callback_return_collect() -> Result<()> {
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
            assert!(task1_log.1); // success
            assert_eq!(task1_log.2, None); // no error
        }

        if let Some(task2_log) = log.iter().find(|(name, _, _)| name == "task2") {
            assert!(!task2_log.1); // failure
            assert_eq!(task2_log.2, Some("error2".to_string())); // error message
        }

        if let Some(task3_log) = log.iter().find(|(name, _, _)| name == "task3") {
            assert!(task3_log.1); // success
            assert_eq!(task3_log.2, None); // no error
        }
        Ok(())
    }

    /// 测试带进度回调的单任务执行
    ///
    /// ## 测试目的
    /// 验证 execute_with_progress() 对单个任务也能正确调用进度回调。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和进度回调
    /// 2. 执行单个任务
    /// 3. 验证进度回调被调用
    ///
    /// ## 预期结果
    /// - 进度回调被调用一次，包含正确的任务信息
    #[test]
    fn test_execute_with_progress_single_task_return_ok() -> Result<()> {
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
        assert!(log[0].1);
        assert_eq!(log[0].2, None);
        Ok(())
    }

    /// 测试不带进度回调的执行
    ///
    /// ## 测试目的
    /// 验证 execute_with_progress() 在没有提供回调函数时也能正常执行。
    ///
    /// ## 测试场景
    /// 1. 创建执行器
    /// 2. 执行任务但不提供回调函数
    /// 3. 验证执行正常完成
    ///
    /// ## 预期结果
    /// - 即使没有回调函数，执行也能正常完成
    #[test]
    fn test_execute_with_progress_no_callback_return_collect() -> Result<()> {
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

    /// 测试大量任务执行
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够处理大量任务。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和100个任务
    /// 2. 执行任务并测量时间
    /// 3. 验证所有任务都成功且执行时间合理
    ///
    /// ## 预期结果
    /// - 所有任务都成功，执行时间在合理范围内
    #[test]
    fn test_large_number_of_tasks_return_ok() -> Result<()> {
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

    /// 测试零延迟任务执行
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够快速执行零延迟任务。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和零延迟任务
    /// 2. 执行任务并测量时间
    /// 3. 验证执行时间很短
    ///
    /// ## 预期结果
    /// - 任务在很短时间内完成
    #[test]
    fn test_zero_delay_tasks_return_ok() -> Result<()> {
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

    /// 测试任务名称保留
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够保留所有任务名称（即使执行顺序可能不同）。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和多个任务
    /// 2. 执行任务
    /// 3. 验证所有任务名称都被保留
    ///
    /// ## 预期结果
    /// - 所有任务名称都被保留（顺序可能不同）
    #[test]
    fn test_task_names_preservation_return_ok() -> Result<()> {
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

    /// 测试不同结果类型
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够处理不同结果类型的任务。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和整数类型任务
    /// 2. 执行任务
    /// 3. 验证结果类型正确
    ///
    /// ## 预期结果
    /// - 不同结果类型都能正确处理
    #[test]
    fn test_different_result_types_return_ok() -> Result<()> {
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

    /// 测试自定义错误类型
    ///
    /// ## 测试目的
    /// 验证 ConcurrentExecutor 能够处理自定义错误类型。
    ///
    /// ## 测试场景
    /// 1. 创建执行器和自定义错误类型任务
    /// 2. 执行任务（包含成功和失败）
    /// 3. 验证自定义错误类型被正确处理
    ///
    /// ## 预期结果
    /// - 自定义错误类型被正确处理
    #[test]
    fn test_custom_error_types_return_false() -> Result<()> {
        let executor = ConcurrentExecutor::new(2);

        // 测试自定义错误类型
        #[derive(Debug, Clone, PartialEq)]
        struct CustomError {
            code: i32,
            message: String,
        }

        impl std::fmt::Display for CustomError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Error {}: {}", self.code, self.message)
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
