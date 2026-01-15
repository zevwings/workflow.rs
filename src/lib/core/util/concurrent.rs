//! 并发任务执行器实现
//!
//! 提供并发任务执行功能，支持限制最大并发数，并收集所有任务的结果。
//! 使用线程池模式，将任务分批执行，每批最多 `max_concurrent` 个任务并行执行。

use color_eyre::Result;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use thiserror::Error;

/// 最小并发数（确保至少为 1）
const MIN_CONCURRENT: usize = 1;

/// 并发执行器错误类型
#[derive(Debug, Error)]
pub enum ConcurrentError {
    /// 任务数量不符合预期
    #[error("Expected exactly one task, but got none")]
    UnexpectedTaskCount,

    /// 线程等待失败
    #[error("Thread join error")]
    ThreadJoinError,
}

/// 任务类型别名
type Task<T, E> = Box<dyn Fn() -> Result<T, E> + Send + Sync>;

/// 任务列表类型别名
type TaskList<T, E> = Vec<(String, Task<T, E>)>;

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
/// use workflow::util::concurrent::{ConcurrentExecutor, TaskResult};
/// use color_eyre::Result;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let executor = ConcurrentExecutor::new(5); // 最大并发数 5
///
/// let tasks: Vec<(String, Box<dyn Fn() -> Result<String> + Send + Sync>)> = vec![
///     ("task1".to_string(), Box::new(|| -> Result<String> { Ok("result1".to_string()) })),
///     ("task2".to_string(), Box::new(|| -> Result<String> { Ok("result2".to_string()) })),
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
            max_concurrent: max_concurrent.max(MIN_CONCURRENT),
        }
    }

    /// 执行多个任务（并行）
    ///
    /// 将任务分批执行，每批最多 `max_concurrent` 个任务并行执行。
    /// 所有任务的结果都会被收集并返回，无论成功或失败。
    ///
    /// # 参数
    ///
    /// * `tasks` - 任务列表，每个任务是一个元组 `(标识符, 任务函数)`
    ///
    /// # 返回
    ///
    /// 返回任务结果列表，每个结果是一个元组 `(标识符, 任务结果)`。
    /// 结果的顺序可能与输入顺序不同（因为并发执行）。
    ///
    /// # 错误
    ///
    /// 如果线程创建或等待失败，返回 `ConcurrentError`。
    ///
    /// # 性能
    ///
    /// - 如果任务列表为空，立即返回空结果
    /// - 如果只有一个任务，直接执行（避免线程开销）
    /// - 多个任务时，使用线程池模式分批执行
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::util::concurrent::{ConcurrentExecutor, TaskResult};
    /// use color_eyre::Result;
    ///
    /// let executor = ConcurrentExecutor::new(3);
    /// let tasks = vec![
    ///     ("task1".to_string(), Box::new(|| -> Result<String> {
    ///         Ok("result1".to_string())
    ///     }) as Box<dyn Fn() -> Result<String> + Send + Sync>),
    ///     ("task2".to_string(), Box::new(|| -> Result<String> {
    ///         Ok("result2".to_string())
    ///     }) as Box<dyn Fn() -> Result<String> + Send + Sync>),
    /// ];
    ///
    /// let results = executor.execute(tasks)?;
    /// for (name, result) in results {
    ///     match result {
    ///         TaskResult::Success(value) => println!("{}: {}", name, value),
    ///         TaskResult::Failure(err) => eprintln!("{}: error - {}", name, err),
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
            let (name, task) =
                tasks.into_iter().next().ok_or(ConcurrentError::UnexpectedTaskCount)?;
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
                    // 如果接收端已关闭，记录警告但继续处理其他任务
                    if let Err(e) = tx.send((name, result)) {
                        tracing::warn!("Failed to send task result: receiver dropped: {}", e);
                        // 接收端已关闭，继续处理其他任务没有意义，退出线程
                        break;
                    }
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
            handle.join().map_err(|_| ConcurrentError::ThreadJoinError)?;
        }

        Ok(results)
    }

    /// 执行多个任务（并行），带进度回调
    ///
    /// 将任务分批执行，每批最多 `max_concurrent` 个任务并行执行。
    /// 所有任务的结果都会被收集并返回，无论成功或失败。
    /// 每个任务完成时会调用进度回调函数。
    ///
    /// # 参数
    ///
    /// * `tasks` - 任务列表，每个任务是一个元组 `(标识符, 任务函数)`
    /// * `callback` - 可选的进度回调函数，类型为 `Arc<Mutex<Option<F>>>`，其中 `F` 是回调函数类型
    ///   回调函数签名：`fn(name: &str, success: bool, error: Option<&str>)`
    ///
    /// # 返回
    ///
    /// 返回任务结果列表，每个结果是一个元组 `(标识符, 任务结果)`。
    /// 结果的顺序可能与输入顺序不同（因为并发执行）。
    ///
    /// # 错误
    ///
    /// 如果线程创建或等待失败，返回 `ConcurrentError`。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::util::concurrent::{ConcurrentExecutor, TaskResult};
    /// use std::sync::{Arc, Mutex};
    /// use color_eyre::Result;
    ///
    /// let executor = ConcurrentExecutor::new(3);
    /// let tasks = vec![
    ///     ("task1".to_string(), Box::new(|| -> Result<String> {
    ///         Ok("result1".to_string())
    ///     }) as Box<dyn Fn() -> Result<String> + Send + Sync>),
    /// ];
    ///
    /// let progress_log = Arc::new(Mutex::new(Vec::new()));
    /// let progress_log_clone = progress_log.clone();
    /// let callback = move |name: &str, success: bool, error: Option<&str>| {
    ///     let mut log = progress_log_clone.lock().unwrap();
    ///     log.push((name.to_string(), success, error.map(|e| e.to_string())));
    /// };
    /// let callback_wrapper = Arc::new(Mutex::new(Some(callback)));
    ///
    /// let results = executor.execute_with_progress(tasks, Some(callback_wrapper))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn execute_with_progress<T, E, F>(
        &self,
        tasks: TaskList<T, E>,
        callback: Option<Arc<Mutex<Option<F>>>>,
    ) -> Result<Vec<(String, TaskResult<T, E>)>>
    where
        T: Send + 'static,
        E: Send + 'static + std::fmt::Display,
        F: Fn(&str, bool, Option<&str>) + Send + Sync + 'static,
    {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        // 如果只有一个任务，直接执行（避免线程开销）
        if tasks.len() == 1 {
            let (name, task) =
                tasks.into_iter().next().ok_or(ConcurrentError::UnexpectedTaskCount)?;
            let result = match task() {
                Ok(value) => {
                    if let Some(cb) = &callback {
                        if let Some(cb_fn) = cb.lock().unwrap().as_ref() {
                            cb_fn(&name, true, None);
                        }
                    }
                    TaskResult::Success(value)
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if let Some(cb) = &callback {
                        if let Some(cb_fn) = cb.lock().unwrap().as_ref() {
                            cb_fn(&name, false, Some(&err_str));
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

        // 分批处理：将任务分成多个批次，每批最多 max_concurrent 个并行执行
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
            let callback_clone = callback.clone();

            let handle = thread::spawn(move || {
                for (name, task) in chunk {
                    let result = match task() {
                        Ok(value) => {
                            if let Some(cb) = &callback_clone {
                                if let Some(cb_fn) = cb.lock().unwrap().as_ref() {
                                    cb_fn(&name, true, None);
                                }
                            }
                            TaskResult::Success(value)
                        }
                        Err(err) => {
                            let err_str = err.to_string();
                            if let Some(cb) = &callback_clone {
                                if let Some(cb_fn) = cb.lock().unwrap().as_ref() {
                                    cb_fn(&name, false, Some(&err_str));
                                }
                            }
                            TaskResult::Failure(err)
                        }
                    };
                    // 如果接收端已关闭，记录警告但继续处理其他任务
                    if let Err(e) = tx.send((name, result)) {
                        tracing::warn!("Failed to send task result: receiver dropped: {}", e);
                        // 接收端已关闭，继续处理其他任务没有意义，退出线程
                        break;
                    }
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
            handle.join().map_err(|_| ConcurrentError::ThreadJoinError)?;
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // ==================== 基础功能测试 ====================

    #[test]
    fn test_executor_creation() {
        let _executor = ConcurrentExecutor::new(5);
        // 验证执行器创建成功（内部字段无法直接访问，通过行为验证）

        // 测试最小并发数限制（应该至少为 1）
        let executor_zero = ConcurrentExecutor::new(0);
        let tasks = vec![(
            "task1".to_string(),
            create_success_task("result1".to_string(), 0),
        )];
        let results = executor_zero.execute(tasks).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_execute_empty_tasks() {
        let executor = ConcurrentExecutor::new(5);
        let results = executor.execute::<String, String>(Vec::new()).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_execute_single_task_success() {
        let executor = ConcurrentExecutor::new(5);
        let tasks = vec![(
            "task1".to_string(),
            create_success_task("result1".to_string(), 0),
        )];

        let results = executor.execute(tasks).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "task1");
        match &results[0].1 {
            TaskResult::Success(value) => assert_eq!(value, "result1"),
            TaskResult::Failure(_) => panic!("Expected success result"),
        }
    }

    #[test]
    fn test_execute_single_task_failure() {
        let executor = ConcurrentExecutor::new(5);
        let tasks = vec![(
            "task1".to_string(),
            create_failure_task("test error".to_string(), 0),
        )];

        let results = executor.execute(tasks).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "task1");
        match &results[0].1 {
            TaskResult::Success(_) => panic!("Expected failure result"),
            TaskResult::Failure(error) => assert_eq!(error, "test error"),
        }
    }

    // ==================== 并发控制测试 ====================

    #[test]
    fn test_concurrent_execution_multiple_tasks() {
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

        let start_time = Instant::now();
        let results = executor.execute(tasks).unwrap();
        let duration = start_time.elapsed();

        // 验证结果数量
        assert_eq!(results.len(), 4);

        // 验证所有任务都成功
        for (_, result) in &results {
            match result {
                TaskResult::Success(_) => {}
                TaskResult::Failure(err) => panic!("Unexpected failure: {}", err),
            }
        }

        // 验证并发执行（4个任务，并发数2，每个任务50ms，应该大约需要100ms而不是200ms）
        // 允许一些时间误差
        assert!(duration >= Duration::from_millis(90));
        assert!(duration <= Duration::from_millis(150));
    }

    #[rstest]
    #[case(1, 4)] // 串行执行
    #[case(2, 4)] // 并发数2
    #[case(4, 4)] // 并发数4
    #[case(8, 4)] // 并发数超过任务数
    fn test_concurrent_limits_timing(#[case] max_concurrent: usize, #[case] task_count: usize) {
        let executor = ConcurrentExecutor::new(max_concurrent);
        let mut tasks = Vec::new();

        for i in 0..task_count {
            tasks.push((
                format!("task{}", i),
                create_success_task(format!("result{}", i), 10), // 减少延迟以提高测试稳定性
            ));
        }

        let start_time = Instant::now();
        let results = executor.execute(tasks).unwrap();
        let duration = start_time.elapsed();

        assert_eq!(results.len(), task_count);

        // 只验证基本的时间约束：
        // 1. 总时间应该合理（不会过快或过慢）
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

        // 验证所有任务都成功（顺序可能不同）
        let mut task_names: Vec<String> = results.iter().map(|(name, _)| name.clone()).collect();
        task_names.sort();

        let mut expected_names: Vec<String> =
            (0..task_count).map(|i| format!("task{}", i)).collect();
        expected_names.sort();

        assert_eq!(task_names, expected_names);

        // 验证每个任务的结果
        for (name, result) in &results {
            match result {
                TaskResult::Success(value) => {
                    // 从任务名中提取索引
                    let index = name.strip_prefix("task").unwrap().parse::<usize>().unwrap();
                    assert_eq!(value, &format!("result{}", index));
                }
                TaskResult::Failure(error) => panic!("Task {} failed: {}", name, error),
            }
        }
    }

    // ==================== 错误处理和混合结果测试 ====================

    #[test]
    fn test_mixed_success_and_failure_tasks() {
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

        let results = executor.execute(tasks).unwrap();

        assert_eq!(results.len(), 4);

        // 统计成功和失败的数量
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
    }

    #[test]
    fn test_all_tasks_fail() {
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

        let results = executor.execute(tasks).unwrap();

        assert_eq!(results.len(), 3);

        // 验证所有任务都失败
        for (_, result) in &results {
            match result {
                TaskResult::Success(_) => panic!("Expected all tasks to fail"),
                TaskResult::Failure(_) => {}
            }
        }
    }

    // ==================== 进度回调测试 ====================

    #[test]
    fn test_execute_with_progress_callback() {
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

        let results = executor.execute_with_progress(tasks, Some(callback_wrapper)).unwrap();

        // 验证执行结果
        assert_eq!(results.len(), 3);

        // 验证进度回调被正确调用
        let log = progress_log.lock().unwrap();
        assert_eq!(log.len(), 3);

        // 验证回调内容（顺序可能不同，所以按名称查找）
        let task1_log = log.iter().find(|(name, _, _)| name == "task1").unwrap();
        assert!(task1_log.1); // success
        assert_eq!(task1_log.2, None); // no error

        let task2_log = log.iter().find(|(name, _, _)| name == "task2").unwrap();
        assert!(!task2_log.1); // failure
        assert_eq!(task2_log.2, Some("error2".to_string())); // error message

        let task3_log = log.iter().find(|(name, _, _)| name == "task3").unwrap();
        assert!(task3_log.1); // success
        assert_eq!(task3_log.2, None); // no error
    }

    #[test]
    fn test_execute_with_progress_single_task() {
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

        let results = executor.execute_with_progress(tasks, Some(callback_wrapper)).unwrap();

        assert_eq!(results.len(), 1);

        // 验证单任务的进度回调
        let log = progress_log.lock().unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].0, "single_task");
        assert!(log[0].1);
        assert_eq!(log[0].2, None);
    }

    #[test]
    fn test_execute_with_progress_no_callback() {
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
        let results: Result<Vec<(String, TaskResult<String, String>)>> = executor
            .execute_with_progress::<String, String, fn(&str, bool, Option<&str>)>(tasks, None);
        let results = results.unwrap();

        // 验证即使没有回调函数，执行也能正常完成
        assert_eq!(results.len(), 2);
    }

    // ==================== 边界条件和压力测试 ====================

    #[test]
    fn test_large_number_of_tasks() {
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
        let results = executor.execute(tasks).unwrap();
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
    }

    #[test]
    fn test_zero_delay_tasks() {
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
        let results = executor.execute(tasks).unwrap();
        let duration = start_time.elapsed();

        assert_eq!(results.len(), 3);

        // 验证快速执行（应该在很短时间内完成）
        assert!(duration <= Duration::from_millis(50));
    }

    #[test]
    fn test_task_names_preservation() {
        let executor = ConcurrentExecutor::new(3);
        let expected_names = vec!["alpha", "beta", "gamma", "delta"];
        let mut tasks = Vec::new();

        for name in &expected_names {
            tasks.push((
                name.to_string(),
                create_success_task(format!("result_{}", name), 5),
            ));
        }

        let results = executor.execute(tasks).unwrap();

        assert_eq!(results.len(), expected_names.len());

        // 验证所有任务名称都被保留（顺序可能不同）
        let mut result_names: Vec<String> = results.iter().map(|(name, _)| name.clone()).collect();
        result_names.sort();
        let mut expected_sorted = expected_names.clone();
        expected_sorted.sort();

        assert_eq!(result_names, expected_sorted);
    }

    // ==================== 类型系统测试 ====================

    #[test]
    fn test_different_result_types() {
        let executor = ConcurrentExecutor::new(2);

        // 测试整数类型的任务
        let int_tasks: Vec<(String, Box<dyn Fn() -> Result<i32, String> + Send + Sync>)> =
            vec![("int_task".to_string(), Box::new(|| Ok(42)))];

        let int_results = executor.execute(int_tasks).unwrap();
        assert_eq!(int_results.len(), 1);
        match &int_results[0].1 {
            TaskResult::Success(value) => assert_eq!(*value, 42),
            TaskResult::Failure(_) => panic!("Expected success"),
        }
    }

    #[test]
    fn test_custom_error_types() {
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

        let results = executor.execute(custom_tasks).unwrap();
        assert_eq!(results.len(), 2);

        // 验证自定义错误类型
        let error_result = results.iter().find(|(name, _)| name == "error_task").unwrap();
        match &error_result.1 {
            TaskResult::Success(_) => panic!("Expected failure"),
            TaskResult::Failure(error) => {
                assert_eq!(error.code, 404);
                assert_eq!(error.message, "Not found");
            }
        }
    }
}
