# wae-ai

AI 服务抽象层，提供统一的 AI 能力抽象，支持多种 AI 服务商。

## 概述

wae-ai 模块为 AI 服务提供统一的抽象接口，支持对话、文生图、视频生成等能力。深度融合 tokio 运行时，所有 API 都是异步优先设计。目前支持腾讯混元和火山引擎两大服务商。

核心设计理念：
- **能力抽象**：通过 trait 定义统一的能力接口，服务商实现具体逻辑
- **配置分离**：认证配置与业务逻辑分离，便于环境管理
- **多模态支持**：图像生成支持文本、图片等多种输入模态

## 快速开始

添加依赖：

```toml
[dependencies]
wae-ai = { path = "backends/wae-ai" }
tokio = { version = "1", features = ["full"] }
```

最小可运行示例：

```rust
use wae_ai::{HunyuanProvider, ChatCapability, ChatParams, ChatMessage, AiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = HunyuanProvider::new();
    let config = AiConfig {
        secret_id: std::env::var("TENCENT_SECRET_ID")?,
        secret_key: std::env::var("TENCENT_SECRET_KEY")?,
        endpoint: None,
        region: None,
    };

    let response = provider.chat(&ChatParams {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "你好，请介绍一下自己".to_string(),
        }],
        temperature: Some(0.7),
        max_tokens: None,
    }, &config).await?;

    println!("{}", response);
    Ok(())
}
```

## 核心用法

### 对话能力

使用 `ChatCapability` trait 进行对话：

```rust
use wae_ai::{HunyuanProvider, ChatCapability, ChatParams, ChatMessage, AiConfig};

let provider = HunyuanProvider::new();
let config = AiConfig {
    secret_id: "your-secret-id".to_string(),
    secret_key: "your-secret-key".to_string(),
    endpoint: None,
    region: None,
};

let response = provider.chat(&ChatParams {
    messages: vec![
        ChatMessage {
            role: "system".to_string(),
            content: "你是一个有帮助的助手".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Rust 的所有权机制是什么？".to_string(),
        },
    ],
    temperature: Some(0.7),
    max_tokens: Some(1000),
}, &config).await?;
```

使用火山引擎豆包：

```rust
use wae_ai::{VolcengineProvider, ChatCapability, ChatParams, ChatMessage, AiConfig};

let provider = VolcengineProvider::new();

let config = AiConfig {
    secret_id: "your-api-key".to_string(),
    secret_key: String::new(),
    endpoint: Some("doubao-pro-4k".to_string()),
    region: None,
};

let response = provider.chat(&ChatParams {
    messages: vec![ChatMessage {
        role: "user".to_string(),
        content: "Hello".to_string(),
    }],
    temperature: None,
    max_tokens: None,
}, &config).await?;
```

### 文生图能力

简单文生图使用 `TextToImageCapability`：

```rust
use wae_ai::{VolcengineProvider, TextToImageCapability, TextToImageParams, AiConfig};

let provider = VolcengineProvider::new();
let config = AiConfig {
    secret_id: "your-access-key-id".to_string(),
    secret_key: "your-secret-access-key".to_string(),
    endpoint: None,
    region: Some("cn-beijing".to_string()),
};

let urls = provider.text_to_image(&TextToImageParams {
    prompt: "一只可爱的橘猫在阳光下打盹".to_string(),
    negative_prompt: Some("模糊, 低质量".to_string()),
    width: Some(1024),
    height: Some(1024),
    num_images: Some(1),
    seed: None,
}, &config).await?;

for url in &urls {
    println!("Generated image: {}", url);
}
```

高级图像生成使用 `AiImageCapability`，支持多模态输入：

```rust
use wae_ai::{
    VolcengineProvider, AiImageCapability, AiImageTask, AiImageInput, AiConfig
};

let provider = VolcengineProvider::new();
let config = AiConfig {
    secret_id: "your-access-key-id".to_string(),
    secret_key: "your-secret-access-key".to_string(),
    endpoint: None,
    region: Some("cn-beijing".to_string()),
};

let output = provider.generate_image(&AiImageTask {
    inputs: vec![
        AiImageInput::Text("将这张图片转换为水彩画风格".to_string()),
        AiImageInput::Image("https://example.com/reference.jpg".to_string()),
    ],
    width: Some(1024),
    height: Some(768),
    num_images: Some(2),
    negative_prompt: Some("模糊, 变形".to_string()),
    seed: Some(42),
    loras: vec![],
    extra: None,
}, &config).await?;

println!("Generated {} images", output.images.len());
println!("Usage: {:?}", output.usage);
```

预估图像生成费用：

```rust
let task = AiImageTask {
    inputs: vec![AiImageInput::Text("test prompt".to_string())],
    width: Some(1024),
    height: Some(1024),
    num_images: Some(4),
    ..Default::default()
};

let usage = provider.estimate_usage(&task)?;
println!("Estimated output pixels: {}", usage.output_pixels);
```

### 视频生成能力

```rust
use wae_ai::{
    HunyuanProvider, VideoGenerationCapability, AiVideoTask, AiVideoInput, AiConfig
};

let provider = HunyuanProvider::new();
let config = AiConfig {
    secret_id: "your-secret-id".to_string(),
    secret_key: "your-secret-key".to_string(),
    endpoint: None,
    region: None,
};

let task = AiVideoTask {
    inputs: vec![
        AiVideoInput::Text("一只蝴蝶在花丛中飞舞".to_string()),
    ],
    duration: Some(5),
    width: Some(1280),
    height: Some(720),
    fps: Some(24),
    extra: None,
};

let output = provider.generate_video(&task, &config).await?;
println!("Video URL: {}", output.video_url);

if let Some(cover) = output.cover_url {
    println!("Cover URL: {}", cover);
}
```

查询异步视频任务状态：

```rust
let status = provider.get_video_task_status("task-id", &config).await?;
println!("Video ready: {}", status.video_url);
```

## 服务商选择

### 腾讯混元 (HunyuanProvider)

**适用场景：**
- 国内业务，需要低延迟
- 与腾讯云生态深度集成
- 需要混元大模型特有的中文优化

**认证方式：**
- 使用 Secret ID 和 Secret Key
- 自动处理腾讯云签名算法

```rust
let config = AiConfig {
    secret_id: env::var("TENCENT_SECRET_ID").unwrap(),
    secret_key: env::var("TENCENT_SECRET_KEY").unwrap(),
    endpoint: None,
    region: Some("ap-guangzhou".to_string()),
};
```

### 火山引擎 (VolcengineProvider)

**适用场景：**
- 使用豆包大模型
- 需要火山引擎视觉 AI 能力
- 与字节跳动生态集成

**认证方式：**
- 豆包对话：仅需 API Key（设置到 `secret_id`）
- 视觉服务：需要 Access Key ID 和 Secret Access Key

```rust
// 豆包对话配置
let chat_config = AiConfig {
    secret_id: env::var("DOUBAO_API_KEY").unwrap(),
    secret_key: String::new(),
    endpoint: Some("doubao-pro-4k".to_string()),
    region: None,
};

// 视觉服务配置
let vision_config = AiConfig {
    secret_id: env::var("VOLC_ACCESS_KEY").unwrap(),
    secret_key: env::var("VOLC_SECRET_KEY").unwrap(),
    endpoint: None,
    region: Some("cn-beijing".to_string()),
};
```

**使用自定义 HTTP 客户端：**

```rust
use wae_request::HttpClient;

let http_client = HttpClient::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build();

let provider = VolcengineProvider::with_client(http_client);
```

## 错误处理

所有 AI 操作返回 `AiResult<T>`，即 `CloudResult<T>` 的类型别名：

```rust
use wae_ai::{AiError, ChatCapability};

match provider.chat(&params, &config).await {
    Ok(response) => println!("Response: {}", response),
    Err(AiError::Network(msg)) => eprintln!("网络错误: {}", msg),
    Err(AiError::InvalidParams(msg)) => eprintln!("参数错误: {}", msg),
    Err(AiError::Internal(msg)) => eprintln!("服务内部错误: {}", msg),
    Err(e) => eprintln!("其他错误: {}", e),
}
```

结合 wae-resilience 实现重试：

```rust
use wae_resilience::Retry;

let retry = Retry::new()
    .max_attempts(3)
    .initial_delay(std::time::Duration::from_millis(100));

let response = retry.execute(|| {
    provider.chat(&params, &config)
}).await?;
```

## 最佳实践

### 配置管理

从环境变量加载配置：

```rust
fn load_ai_config() -> AiConfig {
    AiConfig {
        secret_id: std::env::var("AI_SECRET_ID")
            .expect("AI_SECRET_ID must be set"),
        secret_key: std::env::var("AI_SECRET_KEY")
            .expect("AI_SECRET_KEY must be set"),
        endpoint: std::env::var("AI_ENDPOINT").ok(),
        region: std::env::var("AI_REGION").ok(),
    }
}
```

### Provider 选择策略

根据业务需求动态选择服务商：

```rust
enum AiProviderKind {
    Hunyuan,
    Volcengine,
}

fn create_provider(kind: AiProviderKind) -> Box<dyn ChatCapability> {
    match kind {
        AiProviderKind::Hunyuan => Box::new(HunyuanProvider::new()),
        AiProviderKind::Volcengine => Box::new(VolcengineProvider::new()),
    }
}
```

### 超时控制

为长时间运行的任务设置超时：

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(60),
    provider.generate_image(&task, &config)
).await??;
```

### 批量生成

控制并发数量避免触发限流：

```rust
use futures::stream::{self, StreamExt};

let prompts = vec!["prompt1", "prompt2", "prompt3"];

let results: Vec<_> = stream::iter(prompts)
    .map(|prompt| async {
        provider.text_to_image(&TextToImageParams {
            prompt: prompt.to_string(),
            ..Default::default()
        }, &config).await
    })
    .buffer_unordered(2)  // 最多 2 个并发
    .collect()
    .await;
```

## 常见问题

### Q: 如何选择 HunyuanProvider 和 VolcengineProvider？

A: 根据业务场景选择：
- 中文对话优化优先 → 腾讯混元
- 需要豆包模型 → 火山引擎
- 图像生成质量要求高 → 建议实际测试对比
- 已有云厂商绑定 → 选择对应服务商

### Q: 图像生成返回的是 URL 还是 Base64？

A: 取决于服务商返回格式。火山引擎视觉服务可能返回 Base64 数据（带 `data:image/png;base64,` 前缀）或 URL。建议统一处理后使用：

```rust
for image in &output.images {
    if image.starts_with("data:") {
        // Base64 数据，需要解码保存
    } else {
        // URL，可以直接下载或展示
    }
}
```

### Q: 视频生成为什么需要查询状态？

A: 视频生成通常是异步任务，生成时间较长（数秒到数分钟）。调用 `generate_video` 后可能返回任务 ID，需要通过 `get_video_task_status` 轮询获取结果。

### Q: 如何处理敏感信息？

A: 切勿将 Secret ID/Key 硬编码在代码中。推荐做法：
1. 使用环境变量
2. 使用配置文件（加入 .gitignore）
3. 使用密钥管理服务（如腾讯云 KMS）

### Q: 支持流式响应吗？

A: 当前版本的 `ChatCapability` 返回完整响应。流式支持计划在后续版本中添加。

### Q: 如何调试 API 调用？

A: 启用日志追踪：

```rust
tracing_subscriber::fmt::init();
```

或使用自定义 HTTP 客户端添加日志中间件。
