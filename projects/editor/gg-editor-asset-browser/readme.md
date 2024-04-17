# gg-editor-asset-browser

**GG Game Engine 的资源浏览器，负责管理和浏览游戏资源。**

## 📋 模块简介

gg-editor-asset-browser 是 GG Game Engine 的资源浏览器，负责管理和浏览游戏资源，提供资源的预览、导入和管理功能。

## ✨ 核心功能

- **资源浏览**：浏览项目中的资源文件和文件夹
- **资源预览**：预览各种类型的资源（纹理、模型、音频等）
- **资源导入**：导入外部资源到项目中
- **资源管理**：创建、删除、重命名资源和文件夹
- **资源搜索**：搜索项目中的资源
- **拖放支持**：支持通过拖放操作管理资源

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-asset-browser = { path = "projects/editor/gg-editor-asset-browser" }
```

### 基础示例

```rust
use gg_editor_asset_browser::prelude::*;

fn main() {
    // 创建资源浏览器面板
    let mut asset_browser = AssetBrowserPanel::new();
    
    // 设置资源根目录
    asset_browser.set_root_path("assets/");
    
    // 注册资源处理器
    asset_browser.register_handler(Box::new(TextureHandler));
    
    // 显示面板
    asset_browser.show();
}

// 自定义资源处理器
struct TextureHandler;
impl AssetHandler for TextureHandler {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "png" || ext == "jpg")
    }
    
    fn preview(&self, path: &Path) -> Result<Preview> {
        // 生成预览
        Ok(Preview::Texture(/* 纹理数据 */))
    }
}
```

## 📦 依赖关系

- **gg-editor-shell**：编辑器外壳和基础框架
- **gg-core**：核心功能和平台抽象
- **gg-asset**：资源管理系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [编辑器设计](../../../design/architecture/overview.md) - 了解编辑器系统的设计理念