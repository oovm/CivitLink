//! 音频系统
//! 读取 AudioBus 中的音频事件，映射为音频命令，委托给运行时音频后端执行

use crate::components::*;
use crate::config::AudioConfig;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};
use std::collections::HashMap;

/// 音频系统
/// 读取 AudioBus 中的音频事件，映射为音频命令，委托给运行时音频后端执行
pub struct AudioSystem {
    /// 音频事件到片段路径的映射
    event_clip_map: HashMap<AudioEvent, String>,
    /// 主音量
    master_volume: f32,
    /// 音效音量
    sfx_volume: f32,
}

impl AudioSystem {
    /// 创建新的音频系统
    pub fn new(config: &AudioConfig) -> Self {
        Self {
            event_clip_map: config.event_clip_map.clone(),
            master_volume: config.master_volume,
            sfx_volume: config.sfx_volume,
        }
    }
}

impl System for AudioSystem {
    fn name(&self) -> &str {
        "audio_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let events: Vec<AudioEvent> = world.get_resource::<AudioBus>().map(|bus| bus.events.clone()).unwrap_or_default();

        for event in &events {
            if let Some(clip_path) = self.event_clip_map.get(event) {
                let final_volume = self.master_volume * self.sfx_volume;
                if let Some(bus) = world.get_resource_mut::<AudioBus>() {
                    bus.send(AudioCommand::Play(clip_path.clone()));
                    bus.send(AudioCommand::SetVolume(final_volume));
                }
                log::info!("[AudioSystem] 事件 {:?} -> 播放 {:?}", event, clip_path);
            }
            else {
                log::debug!("[AudioSystem] 事件 {:?} 无对应片段映射，已跳过", event);
            }
        }

        let entities: Vec<Entity> = world.query::<AudioSource>().map(|(e, _)| e).collect();
        for entity in entities {
            let play_on_start = world.get_component::<AudioSource>(entity).map(|s| s.play_on_start).unwrap_or(false);
            if !play_on_start {
                continue;
            }

            let (clip_path, volume) =
                world.get_component::<AudioSource>(entity).map(|s| (s.clip_path.clone(), s.volume)).unwrap_or_default();

            if clip_path.is_empty() {
                continue;
            }

            let final_volume = self.master_volume * self.sfx_volume * volume;
            if let Some(bus) = world.get_resource_mut::<AudioBus>() {
                bus.send(AudioCommand::Play(clip_path.clone()));
                bus.send(AudioCommand::SetVolume(final_volume));
            }
            log::info!("[AudioSystem] AudioSource 自动播放: {:?}", clip_path);

            if let Some(source) = world.get_component_mut::<AudioSource>(entity) {
                source.play_on_start = false;
            }
        }

        if let Some(bus) = world.get_resource_mut::<AudioBus>() {
            bus.events.clear();
        }

        Ok(())
    }
}
