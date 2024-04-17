# gg-editor-lsp

**GG Game Engine 的语言服务器协议支持，为脚本编辑器提供高级代码分析功能。**

## 📋 模块简介

gg-editor-lsp 是 GG Game Engine 的语言服务器协议（LSP）支持模块，为脚本编辑器提供高级代码分析功能，如代码补全、定义跳转、引用查找等。

## ✨ 核心功能

- **LSP 客户端**：实现语言服务器协议客户端
- **代码补全**：提供智能代码补全
- **定义跳转**：支持跳转到符号定义
- **引用查找**：查找符号的所有引用
- **诊断信息**：提供代码错误和警告
- **代码格式化**：支持代码格式化
- **文档提示**：提供符号的文档和提示

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-lsp = { path = "projects/editor/gg-editor-lsp" }
```

### 基础示例

```rust
use gg_editor_lsp::prelude::*;

fn main() {
    // 创建 LSP 客户端
    let mut client = LspClient::new();
    
    // 连接到语言服务器
    client.connect("tcp://localhost:8080");
    
    // 发送初始化请求
    client.initialize(InitializeParams {
        process_id: Some(1234),
        root_uri: Some(Url::parse("file:///path/to/project").unwrap()),
        capabilities: ClientCapabilities::default(),
        ..Default::default()
    });
    
    // 发送文本变更通知
    client.did_change(DidChangeTextDocumentParams {
        text_document: TextDocumentIdentifier {
            uri: Url::parse("file:///path/to/script.valkyrie").unwrap(),
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "function main() { print('Hello'); }".to_string(),
        }],
    });
}
```

## 📦 依赖关系

- **gg-editor-shell**：编辑器外壳和基础框架
- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [编辑器设计](../../../design/architecture/overview.md) - 了解编辑器系统的设计理念