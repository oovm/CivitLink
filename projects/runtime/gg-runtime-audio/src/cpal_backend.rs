use std::{collections::HashMap, io::Cursor, path::Path, time::Duration};

use gg_core::{GError, GErrorKind, GResult};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::{
    AudioCommand, AudioContext, AudioEngine, SoundDescriptor, SoundFormat, SoundId,
    effect::{AudioEffectChain, AudioEffectKind, EffectType},
    group::AudioGroupManager,
    playback::{PlaybackId, PlaybackState},
};

/// cpal 播放实例
///
/// 追踪一个活跃的音频播放实例，包含播放控制和效果器信息。
struct CpalPlayback {
    /// rodio Sink
    sink: Sink,
    /// 声音资源标识
    sound_id: SoundId,
    /// 播放音量
    volume: f32,
    /// 是否循环播放
    looped: bool,
    /// 所属音频分组
    group: Option<String>,
    /// 效果器链
    effect_chain: AudioEffectChain,
    /// 播放状态
    state: PlaybackState,
}

/// 基于 cpal + rodio 的音频引擎实现
pub struct CpalAudioEngine {
    /// rodio 输出流
    _stream: OutputStream,
    /// rodio 输出流句柄
    stream_handle: OutputStreamHandle,
    /// 已加载的声音数据缓存
    sounds: HashMap<SoundId, Vec<u8>>,
    /// 声音描述符缓存
    descriptors: HashMap<SoundId, SoundDescriptor>,
    /// 活跃的播放实例
    active_playbacks: HashMap<PlaybackId, CpalPlayback>,
    /// 音频分组管理器
    group_manager: AudioGroupManager,
    /// 下一个声音 ID
    next_sound_id: u64,
    /// 下一个播放实例 ID
    next_playback_id: u64,
}

impl CpalAudioEngine {
    /// 创建新的 cpal 音频引擎
    pub fn new() -> GResult<Self> {
        let (stream, stream_handle) = OutputStream::try_default().map_err(|e| GError {
            kind: GErrorKind::Platform,
            message: format!("Failed to get default audio output stream: {}", e),
        })?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            sounds: HashMap::new(),
            descriptors: HashMap::new(),
            active_playbacks: HashMap::new(),
            group_manager: AudioGroupManager::new(),
            next_sound_id: 1,
            next_playback_id: 1,
        })
    }

    /// 分配下一个声音 ID
    fn allocate_sound_id(&mut self) -> SoundId {
        let id = self.next_sound_id;
        self.next_sound_id += 1;
        SoundId(id)
    }

    /// 分配下一个播放实例 ID
    fn allocate_playback_id(&mut self) -> PlaybackId {
        let id = self.next_playback_id;
        self.next_playback_id += 1;
        PlaybackId(id)
    }

    /// 根据文件扩展名推断声音格式
    fn guess_format(path: &Path) -> Option<SoundFormat> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("wav") => Some(SoundFormat::Wav),
            Some("ogg") => Some(SoundFormat::Ogg),
            Some("mp3") => Some(SoundFormat::Mp3),
            Some("flac") => Some(SoundFormat::Flac),
            _ => None,
        }
    }

    /// 从缓存的音频数据中估算时长
    fn estimate_duration(data: &[u8], format: SoundFormat) -> f64 {
        match format {
            SoundFormat::Wav => {
                if data.len() > 28 {
                    let sample_rate = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
                    let byte_rate = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
                    if byte_rate > 0 {
                        return (data.len() as f64 - 44.0) / byte_rate as f64;
                    }
                    if sample_rate > 0 {
                        return (data.len() as f64 - 44.0) / (sample_rate as f64 * 2.0 * 2.0);
                    }
                }
                0.0
            }
            _ => 0.0,
        }
    }

    /// 计算播放实例的有效音量
    ///
    /// 有效音量 = 声音自身音量 × 分组有效音量
    fn effective_volume(&self, volume: f32, group: Option<&str>) -> f32 {
        match group {
            Some(g) => {
                let group_vol = self.group_manager.effective_volume(g).unwrap_or(1.0);
                volume * group_vol
            }
            None => volume,
        }
    }

    /// 更新所有播放实例的分组音量
    fn update_group_volumes(&mut self) {
        let playback_ids: Vec<PlaybackId> = self.active_playbacks.keys().copied().collect();
        for pid in playback_ids {
            if let Some(playback) = self.active_playbacks.get(&pid) {
                let vol = playback.volume;
                let group = playback.group.clone();
                let effective = self.effective_volume(vol, group.as_deref());
                if let Some(playback) = self.active_playbacks.get(&pid) {
                    playback.sink.set_volume(effective);
                }
            }
        }
    }
}

impl AudioEngine for CpalAudioEngine {
    fn load_sound(&mut self, path: &Path) -> GResult<SoundId> {
        let data = std::fs::read(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read audio file {:?}: {}", path, e) })?;

        let format = Self::guess_format(path).ok_or_else(|| GError {
            kind: GErrorKind::Asset,
            message: format!("Unsupported audio format for file: {:?}", path),
        })?;

        let cursor = Cursor::new(data.clone());
        let source = Decoder::new(cursor).map_err(|e| GError {
            kind: GErrorKind::Asset,
            message: format!("Failed to decode audio file {:?}: {}", path, e),
        })?;

        let sample_rate = source.sample_rate();
        let channels = source.channels();
        let duration_secs =
            source.duration().unwrap_or_else(|| Duration::from_secs_f64(Self::estimate_duration(&data, format))).as_secs_f64();

        let sound_id = self.allocate_sound_id();

        let descriptor = SoundDescriptor { format, duration_secs, channels, sample_rate };

        self.sounds.insert(sound_id, data);
        self.descriptors.insert(sound_id, descriptor);

        Ok(sound_id)
    }

    fn play(&mut self, sound_id: SoundId, volume: f32, looped: bool, group: Option<&str>) -> GResult<PlaybackId> {
        let data = self
            .sounds
            .get(&sound_id)
            .ok_or_else(|| GError { kind: GErrorKind::Asset, message: format!("Sound not found: {:?}", sound_id) })?;

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to create audio sink: {}", e) })?;

        let effective_vol = self.effective_volume(volume, group);
        sink.set_volume(effective_vol);

        let cursor = Cursor::new(data.clone());
        let source = Decoder::new(cursor)
            .map_err(|e| GError { kind: GErrorKind::Asset, message: format!("Failed to decode audio for playback: {}", e) })?;

        if looped {
            sink.append(source.repeat_infinite());
        }
        else {
            sink.append(source);
        }

        let playback_id = self.allocate_playback_id();

        let playback = CpalPlayback {
            sink,
            sound_id,
            volume,
            looped,
            group: group.map(|g| g.to_string()),
            effect_chain: AudioEffectChain::new(),
            state: PlaybackState::Playing,
        };

        self.active_playbacks.insert(playback_id, playback);

        Ok(playback_id)
    }

    fn stop(&mut self, sound_id: SoundId) -> GResult<()> {
        let ids: Vec<PlaybackId> =
            self.active_playbacks.iter().filter(|(_, p)| p.sound_id == sound_id).map(|(id, _)| *id).collect();
        for id in ids {
            if let Some(playback) = self.active_playbacks.remove(&id) {
                playback.sink.stop();
            }
        }
        Ok(())
    }

    fn set_volume(&mut self, sound_id: SoundId, volume: f32) -> GResult<()> {
        for (_, playback) in self.active_playbacks.iter_mut().filter(|(_, p)| p.sound_id == sound_id) {
            playback.volume = volume;
            let effective = self.effective_volume(volume, playback.group.as_deref());
            playback.sink.set_volume(effective);
        }
        Ok(())
    }

    fn pause(&mut self, sound_id: SoundId) -> GResult<()> {
        for (_, playback) in self.active_playbacks.iter_mut().filter(|(_, p)| p.sound_id == sound_id) {
            playback.sink.pause();
            playback.state = PlaybackState::Paused;
        }
        Ok(())
    }

    fn resume(&mut self, sound_id: SoundId) -> GResult<()> {
        for (_, playback) in self.active_playbacks.iter_mut().filter(|(_, p)| p.sound_id == sound_id) {
            playback.sink.play();
            playback.state = PlaybackState::Playing;
        }
        Ok(())
    }

    fn update(&mut self, context: &AudioContext) -> GResult<()> {
        for command in context.commands() {
            match command.clone() {
                AudioCommand::Play { sound_id, volume, looped, group } => {
                    self.play(sound_id, volume, looped, group.as_deref())?;
                }
                AudioCommand::Stop { sound_id } => {
                    self.stop(sound_id)?;
                }
                AudioCommand::SetVolume { sound_id, volume } => {
                    self.set_volume(sound_id, volume)?;
                }
                AudioCommand::Pause { sound_id } => {
                    self.pause(sound_id)?;
                }
                AudioCommand::Resume { sound_id } => {
                    self.resume(sound_id)?;
                }
                AudioCommand::SetGroupVolume { group, volume } => {
                    self.group_manager.set_group_volume(&group, volume);
                    self.update_group_volumes();
                }
                AudioCommand::SetGroupMute { group, mute } => {
                    self.group_manager.set_group_mute(&group, mute);
                    self.update_group_volumes();
                }
                AudioCommand::AttachEffect { playback_id, effect } => {
                    if let Some(playback) = self.active_playbacks.get_mut(&playback_id) {
                        playback.effect_chain.attach(effect);
                    }
                }
                AudioCommand::DetachEffect { playback_id, effect_type } => {
                    if let Some(playback) = self.active_playbacks.get_mut(&playback_id) {
                        playback.effect_chain.detach(effect_type);
                    }
                }
                AudioCommand::SetEffectParam { playback_id, effect_type, param, value } => {
                    if let Some(playback) = self.active_playbacks.get_mut(&playback_id) {
                        playback.effect_chain.set_param(effect_type, &param, value);
                    }
                }
                AudioCommand::StopPlayback { playback_id } => {
                    self.stop_playback(playback_id)?;
                }
                AudioCommand::SetPlaybackVolume { playback_id, volume } => {
                    self.set_playback_volume(playback_id, volume)?;
                }
                AudioCommand::PausePlayback { playback_id } => {
                    self.pause_playback(playback_id)?;
                }
                AudioCommand::ResumePlayback { playback_id } => {
                    self.resume_playback(playback_id)?;
                }
            }
        }

        self.active_playbacks.retain(|_, playback| !playback.sink.empty() || playback.state == PlaybackState::Paused);

        Ok(())
    }

    fn sound_descriptor(&self, sound_id: SoundId) -> Option<&SoundDescriptor> {
        self.descriptors.get(&sound_id)
    }

    fn reload_sound(&mut self, path: &str) -> GResult<()> {
        let path_ref = std::path::Path::new(path);
        self.load_sound(path_ref)?;
        Ok(())
    }

    fn stop_playback(&mut self, playback_id: PlaybackId) -> GResult<()> {
        if let Some(playback) = self.active_playbacks.remove(&playback_id) {
            playback.sink.stop();
        }
        Ok(())
    }

    fn set_playback_volume(&mut self, playback_id: PlaybackId, volume: f32) -> GResult<()> {
        if let Some(playback) = self.active_playbacks.get_mut(&playback_id) {
            playback.volume = volume;
            let effective = self.effective_volume(volume, playback.group.as_deref());
            playback.sink.set_volume(effective);
            Ok(())
        }
        else {
            Err(GError { kind: GErrorKind::Asset, message: format!("Playback not found: {:?}", playback_id) })
        }
    }

    fn pause_playback(&mut self, playback_id: PlaybackId) -> GResult<()> {
        if let Some(playback) = self.active_playbacks.get_mut(&playback_id) {
            playback.sink.pause();
            playback.state = PlaybackState::Paused;
            Ok(())
        }
        else {
            Err(GError { kind: GErrorKind::Asset, message: format!("Playback not found: {:?}", playback_id) })
        }
    }

    fn resume_playback(&mut self, playback_id: PlaybackId) -> GResult<()> {
        if let Some(playback) = self.active_playbacks.get_mut(&playback_id) {
            playback.sink.play();
            playback.state = PlaybackState::Playing;
            Ok(())
        }
        else {
            Err(GError { kind: GErrorKind::Asset, message: format!("Playback not found: {:?}", playback_id) })
        }
    }

    fn create_group(&mut self, name: &str) -> GResult<()> {
        self.group_manager.create_group(name)
    }

    fn set_group_volume(&mut self, name: &str, volume: f32) -> GResult<()> {
        self.group_manager.set_group_volume(name, volume);
        self.update_group_volumes();
        Ok(())
    }

    fn set_group_mute(&mut self, name: &str, mute: bool) -> GResult<()> {
        self.group_manager.set_group_mute(name, mute);
        self.update_group_volumes();
        Ok(())
    }

    fn group_volume(&self, name: &str) -> Option<f32> {
        self.group_manager.group_volume(name)
    }

    fn is_group_muted(&self, name: &str) -> Option<bool> {
        self.group_manager.is_muted(name)
    }

    fn active_playback_count(&self) -> usize {
        self.active_playbacks.len()
    }

    fn is_playing(&self, playback_id: PlaybackId) -> bool {
        self.active_playbacks.contains_key(&playback_id)
    }
}
