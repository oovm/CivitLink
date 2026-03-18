# wae-ai

AI 功能模块 - 提供人工智能服务的统一抽象和集成。

## 主要功能

- **LLM 集成**: 支持多种大语言模型后端
- **Embedding 服务**: 文本向量化支持
- **对话管理**: 多轮对话上下文管理
- **流式响应**: 支持 SSE 流式输出

## 技术栈

- **HTTP 客户端**: hyper, hyper-tls
- **异步运行时**: Tokio
- **序列化**: serde

## 使用示例

```rust
use wae_ai::{AiClient, ChatRequest, ChatMessage};

#[tokio::main]
async fn main() {
    let client = AiClient::new("your-api-key");
    
    let request = ChatRequest {
        messages: vec![
            ChatMessage::user("你好，请介绍一下自己"),
        ],
        model: "gpt-4",
    };
    
    let response = client.chat(request).await?;
    println!("{}", response.content);
}
```

## 支持的后端

- OpenAI API
- Azure OpenAI
- 本地模型服务
