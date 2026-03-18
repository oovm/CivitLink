# wae-scheduler

任务调度模块，提供多种任务调度策略，支持间隔调度、延迟队列和 Cron 表达式调度。

## 概述

wae-scheduler 深度融合 tokio 运行时，所有 API 都是异步优先设计。支持任务状态追踪、优先级排序、并发控制等特性。三种调度器可独立使用，也可组合使用：

- **IntervalScheduler** - 固定间隔周期执行
- **DelayedQueue** - 延迟执行，支持优先级
- **CronScheduler** - 基于 Cron 表达式的灵活调度

## 快速开始

实现 `ScheduledTask` trait 并注册到调度器：

```rust
use wae::scheduler::{IntervalScheduler, ScheduledTask, SchedulerResult};
use std::sync::Arc;
use std::time::Duration;

struct HelloTask;

impl ScheduledTask for HelloTask {
    async fn execute(&self) -> SchedulerResult<()> {
        println!("Hello from scheduler!");
        Ok(())
    }

    fn name(&self) -> &str {
        "hello-task"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = IntervalScheduler::default();
    let handle = scheduler.schedule_interval(Arc::new(HelloTask), Duration::from_secs(10)).await?;
    
    tokio::time::sleep(Duration::from_secs(35)).await;
    scheduler.shutdown().await;
    Ok(())
}
```

## 间隔调度

IntervalScheduler 适合周期性任务，如心跳检测、数据同步、缓存刷新等。

### 基本使用

```rust
use wae::scheduler::{IntervalScheduler, IntervalSchedulerConfig, ScheduledTask, SchedulerResult};
use std::sync::Arc;
use std::time::Duration;

struct CacheRefreshTask;

impl ScheduledTask for CacheRefreshTask {
    async fn execute(&self) -> SchedulerResult<()> {
        refresh_cache().await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "cache-refresh"
    }
}

let scheduler = IntervalScheduler::new(IntervalSchedulerConfig {
    max_concurrent_tasks: 100,
    task_timeout: Duration::from_secs(300),
    retry_count: 3,
    retry_interval: Duration::from_secs(1),
});

let handle = scheduler.schedule_interval(Arc::new(CacheRefreshTask), Duration::from_secs(60)).await?;
```

### 使用默认配置

```rust
use wae::scheduler::interval_scheduler;

let scheduler = interval_scheduler();
let handle = scheduler.schedule_interval(Arc::new(MyTask), Duration::from_secs(30)).await?;
```

### 带状态的任务

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct CounterTask {
    count: Arc<AtomicU64>,
}

impl ScheduledTask for CounterTask {
    async fn execute(&self) -> SchedulerResult<()> {
        let n = self.count.fetch_add(1, Ordering::SeqCst);
        println!("Execution #{}", n + 1);
        Ok(())
    }

    fn name(&self) -> &str {
        "counter"
    }
}

let counter = Arc::new(AtomicU64::new(0));
let task = CounterTask { count: counter.clone() };
let handle = scheduler.schedule_interval(Arc::new(task), Duration::from_secs(1)).await?;
```

## 延迟队列

DelayedQueue 适合一次性延迟任务，如订单超时取消、定时推送通知、延迟重试等。支持优先级排序。

### 基本使用

```rust
use wae::scheduler::{DelayedQueue, DelayedQueueConfig, DelayedTask, DelayedTaskExecutor, SchedulerResult};
use std::sync::Arc;
use chrono::{Utc, Duration as ChronoDuration};

#[derive(Clone)]
struct OrderTimeout {
    order_id: String,
}

struct OrderExecutor;

impl DelayedTaskExecutor<OrderTimeout> for OrderExecutor {
    async fn execute(&self, task: DelayedTask<OrderTimeout>) -> SchedulerResult<()> {
        cancel_order(&task.data.order_id).await?;
        println!("Order {} cancelled due to timeout", task.data.order_id);
        Ok(())
    }
}

let queue = DelayedQueue::new(DelayedQueueConfig::default(), Arc::new(OrderExecutor));

let task = DelayedTask::new(
    "order-123-timeout".to_string(),
    Utc::now() + ChronoDuration::minutes(30),
    OrderTimeout { order_id: "123".to_string() },
);

let handle = queue.schedule_delayed(task).await?;
```

### 使用优先级

优先级数值越小越先执行：

```rust
let high_priority = DelayedTask::new(
    "urgent-notification".to_string(),
    Utc::now() + ChronoDuration::minutes(5),
    Notification { user_id: "vip-001".to_string() },
).with_priority(1);

let normal_priority = DelayedTask::new(
    "normal-notification".to_string(),
    Utc::now() + ChronoDuration::minutes(5),
    Notification { user_id: "user-002".to_string() },
).with_priority(10);
```

### 动态添加任务

```rust
async fn create_order(order_id: String) -> Result<(), SchedulerError> {
    save_order(&order_id).await?;
    
    let timeout_task = DelayedTask::new(
        format!("timeout-{}", order_id),
        Utc::now() + ChronoDuration::minutes(30),
        OrderTimeout { order_id },
    );
    
    DELAYED_QUEUE.schedule_delayed(timeout_task).await?;
    Ok(())
}
```

## Cron 调度

CronScheduler 适合复杂的时间调度需求，如每天凌晨报表生成、每周清理任务等。

### Cron 表达式格式

格式: `秒 分 时 日 月 周`

| 表达式 | 说明 |
|--------|------|
| `0 * * * * *` | 每分钟执行 |
| `0 0 * * * *` | 每小时执行 |
| `0 0 0 * * *` | 每天零点执行 |
| `0 30 14 * * *` | 每天 14:30 执行 |
| `0 0 9-17 * * 1-5` | 周一到周五 9-17 点每小时执行 |
| `0 0 0 1 * *` | 每月 1 号零点执行 |
| `0 0 0 * * 0` | 每周日零点执行 |
| `0 0 */2 * * *` | 每 2 小时执行 |
| `0 30 8,12,18 * * *` | 每天 8:30、12:30、18:30 执行 |

### 基本使用

```rust
use wae::scheduler::{CronScheduler, ScheduledTask, SchedulerResult};
use std::sync::Arc;

struct DailyReport;

impl ScheduledTask for DailyReport {
    async fn execute(&self) -> SchedulerResult<()> {
        generate_daily_report().await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "daily-report"
    }
}

let scheduler = CronScheduler::default_config();
let handle = scheduler.schedule_cron(Arc::new(DailyReport), "0 0 9 * * *").await?;

let next_exec = scheduler.get_next_execution(&handle.id).await;
println!("Next execution: {:?}", next_exec);
```

### 使用便捷函数

```rust
use wae::scheduler::cron_scheduler;

let scheduler = cron_scheduler();

let handle = scheduler.schedule_cron(Arc::new(WeeklyCleanup), "0 0 3 * * 0").await?;
```

### 查询下次执行时间

```rust
use wae::scheduler::CronExpression;
use chrono::Utc;

let cron = CronExpression::parse("0 30 14 * * *")?;
let next = cron.next_execution(Utc::now());
println!("Next execution at: {:?}", next);
```

### 多任务调度

```rust
let scheduler = CronScheduler::default_config();

scheduler.schedule_cron(Arc::new(MorningTask), "0 0 8 * * *").await?;
scheduler.schedule_cron(Arc::new(NoonTask), "0 0 12 * * *").await?;
scheduler.schedule_cron(Arc::new(EveningTask), "0 0 18 * * *").await?;
```

## 任务管理

### 查询任务状态

```rust
let handle = scheduler.schedule_interval(Arc::new(MyTask), Duration::from_secs(60)).await?;

let state = handle.state().await;
println!("Task state: {:?}", state);

let count = handle.execution_count();
println!("Executed {} times", count);

if let Some(last) = handle.last_execution().await {
    println!("Last execution: {:?}", last);
}

if let Some(error) = handle.last_error().await {
    println!("Last error: {}", error);
}
```

### 取消任务

```rust
let handle = scheduler.schedule_interval(Arc::new(MyTask), Duration::from_secs(60)).await?;

if some_condition {
    handle.cancel();
}

if handle.is_cancelled() {
    println!("Task was cancelled");
}
```

### 错误处理

```rust
use wae::scheduler::SchedulerError;

match scheduler.schedule_cron(Arc::new(MyTask), "invalid").await {
    Ok(handle) => println!("Scheduled: {}", handle.id),
    Err(SchedulerError::InvalidCronExpression(msg)) => {
        eprintln!("Invalid cron: {}", msg);
    }
    Err(SchedulerError::TaskAlreadyExists(id)) => {
        eprintln!("Task already exists: {}", id);
    }
    Err(e) => eprintln!("Error: {:?}", e),
}
```

## 最佳实践

### 任务执行时间控制

任务执行时间不应超过调度间隔，否则会导致任务堆积：

```rust
impl ScheduledTask for FastTask {
    async fn execute(&self) -> SchedulerResult<()> {
        tokio::time::timeout(
            Duration::from_secs(30),
            do_work()
        ).await.map_err(|_| SchedulerError::ExecutionFailed("timeout".into()))?;
        Ok(())
    }
    fn name(&self) -> &str { "fast-task" }
}
```

### 错误重试策略

在任务内部实现重试逻辑：

```rust
impl ScheduledTask for RetryTask {
    async fn execute(&self) -> SchedulerResult<()> {
        let mut retries = 0;
        loop {
            match do_external_call().await {
                Ok(result) => return Ok(()),
                Err(e) if retries < 3 => {
                    retries += 1;
                    tokio::time::sleep(Duration::from_secs(retries)).await;
                }
                Err(e) => return Err(SchedulerError::ExecutionFailed(e.to_string())),
            }
        }
    }
    fn name(&self) -> &str { "retry-task" }
}
```

### 全局调度器管理

使用 `once_cell` 或 `lazy_static` 管理全局调度器实例：

```rust
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static SCHEDULER: Lazy<Arc<IntervalScheduler>> = Lazy::new(|| {
    Arc::new(IntervalScheduler::default())
});

pub async fn schedule_periodic_task() -> Result<TaskHandle, SchedulerError> {
    SCHEDULER.schedule_interval(Arc::new(MyTask), Duration::from_secs(60)).await
}
```

### 优雅关闭

```rust
async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
    println!("Shutting down scheduler...");
}

#[tokio::main]
async fn main() {
    let scheduler = IntervalScheduler::default();
    
    tokio::spawn(async move {
        shutdown_signal().await;
        scheduler.shutdown().await;
    });
    
    scheduler.run().await;
}
```

### 任务隔离

不同类型的任务使用独立的调度器实例，避免相互影响：

```rust
struct Schedulers {
    heartbeat: IntervalScheduler,
    cleanup: CronScheduler,
    notifications: DelayedQueue<Notification>,
}
```

## 常见问题

### Q: 任务执行失败后如何处理？

A: 任务返回 `Err` 时，调度器会记录错误。可以在 `execute` 方法内实现重试逻辑，或使用 `IntervalSchedulerConfig.retry_count` 配置自动重试次数。

### Q: 如何实现任务只执行一次？

A: 使用 `DelayedQueue` 并在任务执行后不再重新调度，或在 `IntervalScheduler` 任务中手动取消：

```rust
impl ScheduledTask for OneTimeTask {
    async fn execute(&self) -> SchedulerResult<()> {
        do_once().await?;
        self_handle.cancel();
        Ok(())
    }
    fn name(&self) -> &str { "one-time" }
}
```

### Q: Cron 表达式支持哪些特殊字符？

A: 支持 `*`（任意值）、`-`（范围）、`/`（步进）、`,`（列表）。例如 `*/5` 表示每 5 个单位，`1,3,5` 表示第 1、3、5 个单位。

### Q: 如何处理时区问题？

A: 调度器内部使用 UTC 时间。如果需要本地时区，在计算 `next_execution` 时进行转换：

```rust
use chrono::Local;

let utc_next = cron.next_execution(Utc::now());
let local_next = utc_next.map(|t| t.with_timezone(&Local));
```

### Q: 任务可以动态修改执行间隔吗？

A: 需要取消当前任务并重新调度：

```rust
async fn reschedule(handle: &TaskHandle, new_interval: Duration) -> Result<TaskHandle, SchedulerError> {
    handle.cancel();
    SCHEDULER.schedule_interval(Arc::new(MyTask), new_interval).await
}
```

### Q: 多个调度器可以共享同一个 tokio runtime 吗？

A: 可以。所有调度器都是异步的，会在当前 tokio runtime 中运行。建议在 main 函数中创建 runtime，然后启动各个调度器。
