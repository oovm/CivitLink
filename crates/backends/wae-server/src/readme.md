# WAE - Rust Async Utilities

WAE 是一个微服务优先的 Rust 异步框架，完全替代 axum，深度融合 tokio 运行时。

提供从 HTTP 服务到云服务的一站式解决方案：
- AI 服务抽象 (wae-ai)
- 存储服务抽象 (wae-storage)
- 邮件服务抽象 (wae-email)
- 数据库抽象 (wae-database)
- 配置管理 (wae-config)
- HTTPS 服务核心 (wae-https)
- 服务发现与注册 (wae-service)
- 代数效应依赖注入 (wae-effect)
- 弹性容错 (wae-resilience)
- 任务调度 (wae-scheduler)
- WebSocket 实时通信 (wae-websocket)
- 事件驱动 (wae-event)
- 分布式能力 (wae-distributed)
- 测试支持 (wae-testing)
- 开发工具 (wae-tools)
- 基础类型定义 (wae-types)

---

WAE 框架入口包 - 提供统一的框架功能导出和便捷的 re-export。

## 主要功能

- **统一入口**: 重新导出所有 WAE 子模块的核心功能
- **简化依赖**: 只需引入一个包即可使用全部框架功能
- **版本管理**: 统一管理各子模块版本

## 包含模块

| 模块 | 说明 |
|------|------|
| `wae-effect` | 代数效应系统 |
| `wae-database` | 数据库抽象层 |
| `wae-config` | 配置管理 |
| `wae-authentication` | 认证模块 |
| `wae-cache` | 缓存模块 |
| `wae-request` | HTTP 客户端 |
| `wae-websocket` | WebSocket 通信 |
| `wae-queue` | 消息队列 |
| `wae-storage` | 对象存储 |
| `wae-email` | 邮件服务 |
| `wae-event` | 事件系统 |
| `wae-scheduler` | 任务调度 |
| `wae-resilience` | 弹性模式 |
| `wae-service` | 服务发现 |
| `wae-schema` | Schema 定义 |
| `wae-ai` | AI 功能 |
| `wae-https` | HTTPS 客户端 |
| `wae-distributed` | 分布式支持 |
| `wae-testing` | 测试工具 |
| `wae-tools` | 工具函数 |
| `wae-types` | 类型定义 |
| `wae-macros` | 过程宏 |

## 使用示例

```rust,no_run
use wae::effect::AlgebraicEffect;

fn main() {
    let _deps = AlgebraicEffect::new();
}
```
