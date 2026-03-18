# config-basic

配置管理基础示例 - 演示 WAE Config 模块的基本用法。

## 示例说明

本示例展示如何使用 `wae-config` 模块加载和管理配置，包括：
- 默认配置
- TOML/YAML 配置文件
- 环境变量覆盖
- 多层级配置合并

## 主要功能演示

- ConfigLoader 的基本使用
- 默认配置设置
- 环境变量加载
- 配置类型转换

## 运行示例

```bash
cd examples/config-basic
cargo run
```

## 代码示例

```rust
use serde::{Deserialize, Serialize};
use wae_config::ConfigLoader;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    app_name: String,
    version: String,
    database: DatabaseConfig,
    server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "WAE App".to_string(),
            version: "0.1.0".to_string(),
            database: DatabaseConfig {
                url: "sqlite://:memory:".to_string(),
                max_connections: 10,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: AppConfig = ConfigLoader::new()
        .with_defaults(&AppConfig::default())
        .with_toml("config.toml")
        .with_env("APP_")
        .extract()?;
    
    println!("应用名称: {}", config.app_name);
    println!("版本: {}", config.version);
    
    Ok(())
}
```

## 输出示例

```
=== WAE Config 基础示例 ===

1. 使用默认配置:
   应用名称: WAE App
   版本: 0.1.0
   数据库 URL: sqlite://:memory:
   数据库最大连接数: 10
   服务器地址: 127.0.0.1:3000

2. 使用 ConfigLoader 加载配置（仅默认值）:
   应用名称: WAE App
   版本: 0.1.0

3. 尝试从环境变量加载（设置 APP_NAME）:
   应用名称: My Awesome App
   服务器端口: 8080

=== 示例完成 ===
```

## 依赖

- `wae-config` - 配置管理核心模块
- `tokio` - 异步运行时
- `serde` - 序列化/反序列化
