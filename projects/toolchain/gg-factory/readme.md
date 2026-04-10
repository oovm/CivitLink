# gg-factory

**GG Game Engine 的项目生成器，负责生成和管理项目模板。**

## 📋 模块简介

gg-factory 是 GG Game Engine 的项目生成器，负责生成和管理项目模板，为开发者提供快速创建项目的能力。

## ✨ 核心功能

- **项目生成**：根据模板生成新项目
- **模板管理**：管理和更新项目模板
- **配置生成**：生成项目配置文件
- **依赖管理**：管理项目依赖
- **代码生成**：生成基础代码结构
- **自定义模板**：支持自定义项目模板

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-factory = { path = "projects/toolchain/gg-factory" }
```

### 基础示例

```rust
use gg_factory::prelude::*;

fn main() {
    // 创建项目生成器
    let mut factory = ProjectFactory::new();
    
    // 生成新项目
    factory.generate_project(
        "my-game",
        ProjectType::Galgame,
        Template::default()
    ).unwrap();
    
    println!("Project generated successfully!");
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [工具链设计](../../../design/architecture/overview.md) - 了解工具链系统的设计理念