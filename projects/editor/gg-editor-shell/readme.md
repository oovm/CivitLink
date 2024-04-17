# gg-editor-shell

**GG Game Engine 的编辑器外壳和基础框架，提供编辑器的核心功能和插件系统。**

## 📋 模块简介

gg-editor-shell 是 GG Game Engine 的编辑器外壳和基础框架，提供编辑器的核心功能和插件系统，是整个编辑器的基础。

## ✨ 核心功能

- **编辑器外壳**：提供编辑器的主界面和窗口管理
- **插件系统**：支持通过插件扩展编辑器功能
- **命令系统**：提供命令注册和执行机制
- **事件系统**：支持编辑器内部事件的发送和接收
- **服务管理**：管理编辑器的各种服务
- **面板管理**：管理编辑器的各种面板和视图

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-shell = { path = "projects/editor/gg-editor-shell" }
```

### 基础示例

```rust
use gg_editor_shell::prelude::*;

fn main() {
    // 创建编辑器外壳
    let mut shell = EditorShell::new();
    
    // 注册命令
    shell.register_command("hello", |ctx| {
        println!("Hello, GG Editor!");
        Ok(())
    });
    
    // 添加插件
    shell.add_plugin(Box::new(MyPlugin));
    
    // 运行编辑器
    shell.run();
}

// 自定义插件
struct MyPlugin;
impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    
    fn initialize(&mut self, context: &mut EditorContext) -> Result<()> {
        println!("My plugin initialized!");
        Ok(())
    }
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统
- **gg-reflection**：反射系统

## 📖 相关文档

- [编辑器设计](../../../design/architecture/overview.md) - 了解编辑器系统的设计理念