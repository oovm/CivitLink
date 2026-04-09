#![warn(missing_docs)]

//! GG 引擎音频运行时模块
//! 提供音频硬件抽象层（HAL）定义，包含声音资源描述、音频命令和音频引擎 trait

/// 音频命令定义
pub mod command;

/// 音频引擎 trait 和音频上下文
pub mod engine;

/// 声音资源标识与描述
pub mod sound;

#[cfg(feature = "cpal-backend")]
/// 基于 cpal + rodio 的音频后端实现
pub mod cpal_backend;

pub use command::AudioCommand;
pub use engine::{AudioContext, AudioEngine};
pub use sound::{SoundDescriptor, SoundFormat, SoundId};

#[cfg(feature = "cpal-backend")]
pub use cpal_backend::CpalAudioEngine;
