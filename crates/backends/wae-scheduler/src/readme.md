# WAE Scheduler - 任务调度抽象层

提供统一的任务调度能力抽象，支持多种调度模式。

深度融合 tokio 运行时，所有 API 都是异步优先设计。
微服务架构友好，支持定时任务、延迟任务、Cron 表达式等特性。

## 模块结构

- [`IntervalScheduler`] - 固定间隔任务调度器
- [`DelayedQueue`] - 延迟任务队列
- [`CronScheduler`] - Cron 表达式任务调度器

---

调度模块 - 提供任务调度功能。

## 主要功能

- **定时任务**: Cron 表达式定时执行
- **延迟任务**: 延迟执行任务
- **周期任务**: 固定间隔执行
- **任务管理**: 启动、停止、暂停任务

## 技术栈

- **调度器**: tokio-cron-scheduler
- **异步运行时**: Tokio

## 使用示例

```rust,no_run
use wae_scheduler::{IntervalScheduler, ScheduledTask, SchedulerResult};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;


struct MyTask;


#[async_trait]
impl ScheduledTask for MyTask {
    async fn execute(&self) -> SchedulerResult<()> {
        println!("Task executed!");
        Ok(())
    }

    fn name(&self) -> &str {
        "my_task"
    }
}

#[tokio::main]
async fn main() {
    let scheduler = IntervalScheduler::default_config();
    let task = Arc::new(MyTask);

    let handle = scheduler
        .schedule_interval(task, Duration::from_secs(60))
        .await
        .unwrap();

    println!("Task scheduled: {}", handle.name);

    scheduler.shutdown().await;
}
```

## Cron 表达式

```text
秒 分 时 日 月 周
*  *  *  *  *  *
|  |  |  |  |  |
|  |  |  |  |  +-- 周几 (0-6, 0=周日)
|  |  |  |  +----- 月份 (1-12)
|  |  |  +-------- 日期 (1-31)
|  |  +----------- 小时 (0-23)
|  +-------------- 分钟 (0-59)
+----------------- 秒 (0-59)
```
