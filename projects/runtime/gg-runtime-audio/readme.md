# gg-runtime-audio

**GG Game Engine 的音频系统，负责游戏的音频播放和管理。**

## 📋 模块简介

gg-runtime-audio 是 GG Game Engine 的音频系统，负责游戏的音频播放和管理，提供音效和音乐的播放功能。

## ✨ 核心功能

- **音频播放**：播放音效和音乐
- **音频管理**：管理音频的加载和释放
- **音量控制**：控制音频的音量和混音
- **多后端支持**：支持不同的音频后端（如 CPAL）
- **空间音频**：支持空间音频效果
- **音频命令**：支持音频的暂停、 resume 和停止

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-runtime-audio = { path = "projects/runtime/gg-runtime-audio" }
```

### 基础示例

```rust
use gg_runtime_audio::prelude::*;

fn main() {
    // 创建音频引擎
    let mut audio_engine = AudioEngine::new();
    
    // 初始化音频引擎
    audio_engine.initialize().unwrap();
    
    // 加载音效
    let sound = audio_engine.load_sound("assets/sounds/explosion.wav").unwrap();
    
    // 播放音效
    audio_engine.play_sound(sound);
    
    // 加载音乐
    let music = audio_engine.load_music("assets/music/theme.mp3").unwrap();
    
    // 播放音乐（循环）
    audio_engine.play_music(music, true);
    
    // 设置音量
    audio_engine.set_volume(0.8);
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统
- **cpal**：跨平台音频库

## 📖 相关文档

- [音频系统设计](../../../design/architecture/overview.md) - 了解音频系统的设计理念