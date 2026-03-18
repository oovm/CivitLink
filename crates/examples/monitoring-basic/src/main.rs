//! 监控系统示例
//!
//! 演示如何使用 wae-monitoring 模块进行系统资源监控和告警。

use std::sync::Arc;
use tokio::time;
use wae_monitoring::{MonitorConfig, MonitorService};

#[tokio::main]
async fn main() {
    println!("启动监控系统示例...");

    // 创建监控配置
    let config = MonitorConfig::default();

    // 创建监控服务（不使用告警服务）
    let monitor_service = Arc::new(MonitorService::new(config, None));

    // 启动监控服务
    let monitor_service_clone = monitor_service.clone();
    tokio::spawn(async move {
        monitor_service_clone.start().await;
    });

    // 等待一段时间，让监控服务采集一些数据
    time::sleep(time::Duration::from_secs(10)).await;

    // 获取最新的监控数据
    if let Some(resource) = monitor_service.get_latest_resource().await {
        println!("\n最新监控数据:");
        println!("时间戳: {}", resource.timestamp);
        println!("CPU 使用率: {:.2}%", resource.cpu_usage);
        println!(
            "内存使用: {:.2}% ({} MB / {} MB)",
            resource.memory.usage_percent,
            resource.memory.used / 1024 / 1024,
            resource.memory.total / 1024 / 1024
        );
        println!(
            "网络流量: 接收 {} MB, 发送 {} MB",
            resource.network.bytes_received / 1024 / 1024,
            resource.network.bytes_sent / 1024 / 1024
        );
        println!(
            "磁盘使用: {:.2}% ({} GB / {} GB)",
            resource.disk.usage_percent,
            resource.disk.used / 1024 / 1024 / 1024,
            resource.disk.total / 1024 / 1024 / 1024
        );
    }

    // 获取历史监控数据
    let start_time = Some(chrono::Utc::now().timestamp() - 60);
    let end_time = Some(chrono::Utc::now().timestamp());

    let resources = monitor_service.get_resources(start_time, end_time).await;
    println!("\n过去 60 秒的监控数据点数量: {}", resources.len());

    println!("\n监控系统示例运行中... 按 Ctrl+C 退出");

    // 保持程序运行
    tokio::signal::ctrl_c().await.unwrap();
    println!("\n监控系统示例已退出");
}
