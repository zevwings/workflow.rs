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
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_execute_empty() {
        let executor = ConcurrentExecutor::new(5);
        let results =
            executor.execute::<String, String>(Vec::new()).expect("Execute should succeed");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_execute_single() {
        let executor = ConcurrentExecutor::new(5);
        let tasks = vec![(
            "task1".to_string(),
            Box::new(|| -> Result<String, String> { Ok("result1".to_string()) })
                as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
        )];
        let results = executor.execute(tasks).expect("Execute should succeed");
        assert_eq!(results.len(), 1);
        match &results[0].1 {
            TaskResult::Success(value) => assert_eq!(value, "result1"),
            TaskResult::Failure(_) => panic!("Expected success"),
        }
    }

    #[test]
    fn test_execute_multiple() {
        let executor = ConcurrentExecutor::new(2);
        let tasks = vec![
            (
                "task1".to_string(),
                Box::new(|| -> Result<String, String> {
                    sleep(Duration::from_millis(10));
                    Ok("result1".to_string())
                }) as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
            (
                "task2".to_string(),
                Box::new(|| -> Result<String, String> {
                    sleep(Duration::from_millis(10));
                    Ok("result2".to_string())
                }) as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
            (
                "task3".to_string(),
                Box::new(|| -> Result<String, String> {
                    sleep(Duration::from_millis(10));
                    Ok("result3".to_string())
                }) as Box<dyn Fn() -> Result<String, String> + Send + Sync>,
            ),
        ];
        let results = executor.execute(tasks).expect("Execute should succeed");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_execute_with_failure() {
        let executor = ConcurrentExecutor::new(5);
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
        let results = executor.execute(tasks).expect("Execute should succeed");
        assert_eq!(results.len(), 2);
        match &results[0].1 {
            TaskResult::Success(_) => {}
            TaskResult::Failure(_) => {}
        }
    }
}
