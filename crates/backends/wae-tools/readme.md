# wae-tools

工具模块 - 提供通用工具函数和扩展。

## 主要功能

- **字符串处理**: 字符串工具函数
- **时间处理**: 时间格式化和解析
- **加密工具**: 哈希和加密函数
- **ID 生成**: 唯一标识符生成

## 技术栈

- **ID 生成**: uuid, snowflake
- **加密**: sha2, md5
- **异步运行时**: Tokio

## 使用示例

```rust
use wae_tools::{
    id::generate_uuid,
    id::generate_snowflake_id,
    crypto::sha256,
    crypto::md5_hash,
    string::random_string,
    time::format_timestamp,
};

fn main() {
    let uuid = generate_uuid();
    let snowflake_id = generate_snowflake_id();
    
    let hash = sha256("hello world");
    let md5 = md5_hash("hello world");
    
    let random = random_string(16);
    let formatted = format_timestamp(chrono::Utc::now(), "%Y-%m-%d %H:%M:%S");
}
```

## ID 生成器

```rust
use wae_tools::id::SnowflakeIdGenerator;

let generator = SnowflakeIdGenerator::new(1);
let id = generator.next_id();
```

## 字符串工具

```rust
use wae_tools::string::{truncate, camel_to_snake, snake_to_camel};

let truncated = truncate("很长的字符串...", 10);
let snake = camel_to_snake("userName");
let camel = snake_to_camel("user_name");
```
