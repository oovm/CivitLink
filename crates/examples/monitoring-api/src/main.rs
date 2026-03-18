//! 监控系统 API 示例
//!
//! 演示如何使用 wae-monitoring 模块和 wae-https 框架创建监控 API。

use http::{Method, Response, StatusCode, header};
use http_body_util::Full;
use hyper::body::Bytes;
use std::sync::Arc;
use tokio;
use url;
use wae_https::{HttpsServerBuilder, Router};
use wae_monitoring::{MonitorConfig, MonitorService};

fn full_body(data: impl Into<Bytes>) -> Full<Bytes> {
    Full::new(data.into())
}

#[tokio::main]
async fn main() {
    println!("启动监控系统 API 示例...");

    // 创建监控配置
    let config = MonitorConfig::default();

    // 创建监控服务（不使用告警服务）
    let monitor_service = Arc::new(MonitorService::new(config, None));

    // 启动监控服务
    let monitor_service_clone = monitor_service.clone();
    tokio::spawn(async move {
        monitor_service_clone.start().await;
    });

    // 创建路由
    let mut router = Router::new();

    // 添加获取最新监控数据的路由
    let monitor_service_clone = monitor_service.clone();
    router.add_route(Method::GET, "/api/monitoring/latest", move |_parts: wae_https::extract::RequestParts| {
        let service = monitor_service_clone.clone();
        let response = tokio::runtime::Handle::current().block_on(async move {
            match service.get_latest_resource().await {
                Some(resource) => {
                    let json = serde_json::to_string(&resource).unwrap();
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(full_body(json))
                        .unwrap()
                }
                None => Response::builder().status(StatusCode::NO_CONTENT).body(full_body(Bytes::new())).unwrap(),
            }
        });
        response
    });

    // 添加获取监控数据的路由
    let monitor_service_clone = monitor_service.clone();
    router.add_route(Method::GET, "/api/monitoring/resources", move |parts: wae_https::extract::RequestParts| {
        let service = monitor_service_clone.clone();
        let response = tokio::runtime::Handle::current().block_on(async move {
            let query = parts.uri.query().unwrap_or("");
            let params: std::collections::HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes()).into_owned().collect();

            let start_time = params.get("start_time").and_then(|s| s.parse::<i64>().ok());
            let end_time = params.get("end_time").and_then(|s| s.parse::<i64>().ok());

            let resources = service.get_resources(start_time, end_time).await;
            let json = serde_json::to_string(&resources).unwrap();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(full_body(json))
                .unwrap()
        });
        response
    });

    // 启动 HTTP 服务器
    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("监控 API 服务器运行在 http://{}", addr);

    let server = HttpsServerBuilder::new().addr(addr).router(router).build();

    server.serve().await.unwrap();
}
