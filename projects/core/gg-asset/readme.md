# GG Asset

资源管理模块，提供资源加载和管理功能。

## 功能

- 资源加载和管理
- 支持自定义资源加载器
- 资源类型安全

## 使用示例

```rust
use gg_asset::{AssetManager, TextLoader, TextAsset};

let mut asset_manager = AssetManager::new();
asset_manager.register_loader("txt", TextLoader);

let text_asset = asset_manager.load::<TextAsset>(Path::new("assets/text.txt")).unwrap();
println!("Text content: {}", text_asset.content());
```
