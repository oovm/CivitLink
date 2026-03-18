# wae-config

配置模块 - 提供灵活的配置管理功能。

## 主要功能

- **多格式支持**: 支持 TOML、YAML、JSON 配置文件
- **环境变量**: 支持环境变量覆盖
- **热重载**: 配置文件变更自动重载
- **类型安全**: 强类型配置解析

## 技术栈

- **配置解析**: config-rs
- **序列化**: serde
- **异步运行时**: Tokio

## 使用示例

```rust
use wae_config::{ConfigLoader, ConfigSource};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
}

#[tokio::main]
async fn main() {
    let config: AppConfig = ConfigLoader::new()
        .file("config.toml")
        .env_prefix("APP")
        .load()
        .await?;
    
    println!("Server: {}:{}", config.server.host, config.server.port);
}
```

## 配置优先级

1. 环境变量 (最高优先级)
2. 命令行参数
3. 配置文件
4. 默认值 (最低优先级)
