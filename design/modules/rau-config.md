# wae-config

配置管理模块，提供统一的配置加载和管理能力，基于 figment 库实现。

## 概述

支持多层级配置合并：环境变量 > 配置文件 > 默认值。支持 TOML、YAML 配置文件和环境变量。

## 快速开始

```rust
use wae_config::ConfigLoader;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    name: String,
    port: u16,
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: AppConfig = ConfigLoader::new()
        .with_toml("config.toml")
        .with_env("APP_")
        .extract()?;
    
    println!("App: {} on port {}", config.name, config.port);
    Ok(())
}
```

## 核心用法

### 从不同来源加载

```rust
use wae_config::ConfigLoader;

let config = ConfigLoader::new()
    .with_toml("config.toml")
    .with_yaml("config.yaml")
    .with_env("APP_")
    .with_env_separator("APP_", "__")
    .extract::<AppConfig>()?;
```

### 多层级配置合并

配置按添加顺序合并，后添加的覆盖先添加的：

```rust
let config: AppConfig = ConfigLoader::new()
    .with_defaults(&AppConfig {
        name: "default".to_string(),
        port: 8080,
        debug: false,
    })
    .with_toml("config.default.toml")
    .with_toml("config.toml")
    .with_env("APP_")
    .extract()?;
```

优先级：环境变量 > config.toml > config.default.toml > 默认值

### 嵌套配置与环境变量

```rust
#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    name: String,
    database: DatabaseConfig,
}

let config: AppConfig = ConfigLoader::new()
    .with_env_separator("APP_", "__")
    .extract()?;
```

环境变量映射：
- `APP_NAME` -> `name`
- `APP__DATABASE__HOST` -> `database.host`
- `APP__DATABASE__PORT` -> `database.port`

### 带错误上下文

```rust
let config: AppConfig = ConfigLoader::new()
    .with_toml("config.toml")
    .extract_with_context("Failed to load application config")?;
```

### 便捷函数

```rust
use wae_config::{load_config, from_env};

let config: AppConfig = load_config("config.toml", "APP_")?;

let config: AppConfig = from_env("APP_")?;
```

## 配置文件示例

### TOML

```toml
name = "my-app"
port = 3000
debug = true

[database]
host = "localhost"
port = 5432
```

### YAML

```yaml
name: my-app
port: 3000
debug: true

database:
  host: localhost
  port: 5432
```

## 最佳实践

### 生产环境配置结构

```
config/
├── default.toml      # 默认配置
├── development.toml  # 开发环境
├── production.toml   # 生产环境
└── local.toml        # 本地覆盖（不提交）
```

```rust
let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());

let config: AppConfig = ConfigLoader::new()
    .with_toml("config/default.toml")
    .with_toml(&format!("config/{}.toml", env))
    .with_toml("config/local.toml")
    .with_env("APP_")
    .extract()?;
```

### 配置验证

```rust
impl AppConfig {
    fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("port cannot be 0".to_string());
        }
        if self.name.is_empty() {
            return Err("name cannot be empty".to_string());
        }
        Ok(())
    }
}

let config: AppConfig = ConfigLoader::new()
    .with_toml("config.toml")
    .with_env("APP_")
    .extract()?;

config.validate()?;
```

### 敏感配置处理

敏感配置（密码、密钥）应通过环境变量传递，不要写入配置文件：

```rust
#[derive(Debug, Deserialize)]
struct AppConfig {
    database_url: String,
    api_key: String,
}

let config: AppConfig = ConfigLoader::new()
    .with_toml("config.toml")
    .with_env("APP_")
    .extract()?;

let config: AppConfig = ConfigLoader::new()
    .with_toml("config.toml")
    .with_env("APP_")
    .extract()?;
```

## 常见问题

### 如何处理可选配置项？

使用 `Option<T>`：

```rust
#[derive(Debug, Deserialize)]
struct AppConfig {
    name: String,
    #[serde(default)]
    redis_url: Option<String>,
}
```

### 如何设置配置项默认值？

使用 `#[serde(default)]` 配合 `Default` trait：

```rust
#[derive(Debug, Deserialize)]
struct AppConfig {
    #[serde(default = "default_port")]
    port: u16,
}

fn default_port() -> u16 { 8080 }
```

### 如何调试配置加载问题？

使用 `extract_with_context` 获取更详细的错误信息：

```rust
let config: AppConfig = ConfigLoader::new()
    .with_toml("config.toml")
    .extract_with_context("加载应用配置失败")?;
```

### 如何在测试中使用不同配置？

```rust
#[cfg(test)]
fn test_config() -> AppConfig {
    ConfigLoader::new()
        .with_defaults(&AppConfig {
            name: "test".to_string(),
            port: 0,
            debug: true,
        })
        .extract()
        .unwrap()
}
```

### 如何处理配置热更新？

当前版本不支持热更新，需要重启服务。如需动态配置，考虑使用 `wae-cache` 配合外部配置中心。
