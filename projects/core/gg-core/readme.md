# GG Core

GG 引擎核心模块，提供基础类型、错误处理和平台抽象接口。

## 功能

- 错误处理系统
- 平台抽象层
- 插件系统

## 使用示例

```rust
use gg_core::{GResult, GError, GErrorKind};

fn example() -> GResult<()> {
    // 成功返回
    Ok(())
}

fn example_with_error() -> GResult<()> {
    // 返回错误
    Err(GError {
        kind: GErrorKind::Other,
        message: "An error occurred".to_string(),
    })
}
```
