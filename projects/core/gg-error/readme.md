# gg-error

**GG Game Engine 的错误处理系统，提供统一的错误类型和处理机制。**

## 📋 模块简介

gg-error 是 GG Game Engine 的错误处理系统，提供统一的错误类型和处理机制，使错误处理更加一致和简洁。

## ✨ 核心功能

- **统一错误类型**：定义统一的错误类型和错误码
- **错误链**：支持错误链，保留原始错误信息
- **错误转换**：提供从其他错误类型到统一错误类型的转换
- **错误格式化**：提供友好的错误格式化和展示
- **错误追踪**：支持错误发生的位置和上下文信息

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-error = { path = "projects/core/gg-error" }
```

### 基础示例

```rust
use gg_error::prelude::*;

fn read_config() -> Result<String> {
    let config_path = "assets/config.toml";
    
    // 读取文件，自动转换错误类型
    std::fs::read_to_string(config_path)
        .map_err(|e| Error::from_io(e, config_path))
}

fn main() {
    match read_config() {
        Ok(config) => println!("Config loaded successfully: {}", config),
        Err(e) => println!("Error loading config: {}", e),
    }
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象

## 📖 相关文档

- [错误处理设计](../../../design/architecture/overview.md) - 了解错误处理系统的设计理念