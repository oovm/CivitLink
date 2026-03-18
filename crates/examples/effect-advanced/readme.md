# effect-advanced

Effect 依赖注入进阶示例 - 演示 WAE Effect 模块的高级用法。

## 示例说明

本示例展示 `wae-effect` 模块的进阶用法，包括：
- 多个服务的依赖注入
- 类型安全的依赖获取
- 服务之间的协作
- 配置与服务分离

## 主要功能演示

- AlgebraicEffect 构建器的使用
- 字符串键依赖注册/获取
- 类型安全的依赖注册/获取
- 多个服务协作模式

## 运行示例

```bash
cd examples/effect-advanced
cargo run
```

## 代码示例

```rust
use serde::{Deserialize, Serialize};
use wae_effect::{AlgebraicEffect, Dependencies};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    app_name: String,
    version: String,
}

#[derive(Debug, Clone)]
struct UserRepository {
    users: Vec<User>,
}

#[derive(Debug, Clone)]
struct LoggingService {
    service_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig {
        app_name: "WAE Effect Demo".to_string(),
        version: "1.0.0".to_string(),
    };
    
    let user_repo = UserRepository::new();
    let logging = LoggingService::new("effect-demo".to_string());

    let deps = AlgebraicEffect::new()
        .with("config", config)
        .with("user_repo", user_repo)
        .with("logging", logging)
        .build();

    let config: AppConfig = deps.get("config")?;
    let repo: UserRepository = deps.get("user_repo")?;
    let logger: LoggingService = deps.get("logging")?;

    logger.info(&format!("应用: {}", config.app_name));
    
    Ok(())
}
```

## 输出示例

```
=== WAE Effect 进阶示例 ===

1. 创建配置对象:
   配置创建完成

2. 创建服务实例:
   服务实例创建完成

3. 使用 AlgebraicEffect 构建依赖容器:
   依赖容器构建完成

4. 使用字符串键获取依赖:
   应用名称: WAE Effect Demo
   版本: 1.0.0
   数据库 URL: postgres://localhost:5432/wae

5. 使用用户仓储服务:
   用户数量: 2
   - 张三 (zhangsan@example.com)
   - 李四 (lisi@example.com)

6. 按类型注册和获取依赖:
   通过类型获取配置: WAE Effect Demo

7. 使用日志服务:
[INFO] [effect-demo] 这是一条信息日志
[ERROR] [effect-demo] 这是一条错误日志

8. 演示服务协作:
[INFO] [effect-demo] 找到用户: 张三 (zhangsan@example.com)

=== 示例总结 ===
本示例展示了:
  - 使用字符串键注册和获取依赖
  - 使用类型安全的方式注册和获取依赖
  - 多个服务之间的协作
  - 配置管理与服务分离
```

## 依赖

- `wae-effect` - 代数效应/依赖注入核心模块
- `tokio` - 异步运行时
- `serde` - 序列化/反序列化
