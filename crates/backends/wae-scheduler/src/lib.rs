#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

mod cron;
mod delayed;
mod interval;
mod task;

pub use cron::{CronExpression, CronField, CronScheduler, CronSchedulerConfig, CronTask, cron_scheduler, interval_scheduler};
pub use delayed::{DelayedQueue, DelayedQueueConfig, DelayedTask, DelayedTaskExecutor};
pub use interval::{IntervalScheduler, IntervalSchedulerConfig};
pub use task::{ScheduledTask, TaskHandle, TaskId, TaskState};
pub use wae_types::{WaeError, WaeErrorKind, WaeResult};
