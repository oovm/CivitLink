use std::{collections::HashMap, io::Cursor, path::Path};

use gg_core::{GError, GErrorKind, GResult};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::{AudioCommand, AudioContext, AudioEngine, SoundDescriptor, SoundFormat, SoundId};

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
    /// 活跃的播放 Sink
    active_sinks: HashMap<SoundId, Sink>,
    /// 下一个声音 ID
    next_sound_id: u64,
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
            active_sinks: HashMap::new(),
            next_sound_id: 1,
        })
    }

    /// 分配下一个声音 ID
    fn allocate_sound_id(&mut self) -> SoundId {
        let id = self.next_sound_id;
        self.next_sound_id += 1;
        SoundId(id)
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
        let duration_secs = Self::estimate_duration(&data, format);

        let sound_id = self.allocate_sound_id();

        let descriptor = SoundDescriptor { format, duration_secs, channels, sample_rate };

        self.sounds.insert(sound_id, data);
        self.descriptors.insert(sound_id, descriptor);

        Ok(sound_id)
    }

    fn play(&mut self, sound_id: SoundId, volume: f32, looped: bool) -> GResult<()> {
        let data = self
            .sounds
            .get(&sound_id)
            .ok_or_else(|| GError { kind: GErrorKind::Asset, message: format!("Sound not found: {:?}", sound_id) })?;

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to create audio sink: {}", e) })?;

        sink.set_volume(volume);

        let cursor = Cursor::new(data.clone());
        let source = Decoder::new(cursor)
            .map_err(|e| GError { kind: GErrorKind::Asset, message: format!("Failed to decode audio for playback: {}", e) })?;

        if looped {
            sink.append(source.repeat_infinite());
        }
        else {
            sink.append(source);
        }

        self.active_sinks.insert(sound_id, sink);

        Ok(())
    }

    fn stop(&mut self, sound_id: SoundId) -> GResult<()> {
        if let Some(sink) = self.active_sinks.remove(&sound_id) {
            sink.stop();
        }
        Ok(())
    }

    fn set_volume(&mut self, sound_id: SoundId, volume: f32) -> GResult<()> {
        if let Some(sink) = self.active_sinks.get(&sound_id) {
            sink.set_volume(volume);
            Ok(())
        }
        else {
            Err(GError { kind: GErrorKind::Asset, message: format!("Active sound not found: {:?}", sound_id) })
        }
    }

    fn pause(&mut self, sound_id: SoundId) -> GResult<()> {
        if let Some(sink) = self.active_sinks.get(&sound_id) {
            sink.pause();
            Ok(())
        }
        else {
            Err(GError { kind: GErrorKind::Asset, message: format!("Active sound not found: {:?}", sound_id) })
        }
    }

    fn resume(&mut self, sound_id: SoundId) -> GResult<()> {
        if let Some(sink) = self.active_sinks.get(&sound_id) {
            sink.play();
            Ok(())
        }
        else {
            Err(GError { kind: GErrorKind::Asset, message: format!("Active sound not found: {:?}", sound_id) })
        }
    }

    fn update(&mut self, context: &AudioContext) -> GResult<()> {
        for command in context.commands() {
            match command.clone() {
                AudioCommand::Play { sound_id, volume, looped } => {
                    self.play(sound_id, volume, looped)?;
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
            }
        }
        Ok(())
    }

    fn sound_descriptor(&self, sound_id: SoundId) -> Option<&SoundDescriptor> {
        self.descriptors.get(&sound_id)
    }

    /// 重新加载音频（用于 HMR 热更新）
    fn reload_sound(&mut self, path: &str) -> GResult<()> {
        let path_ref = std::path::Path::new(path);
        self.load_sound(path_ref)?;
        Ok(())
    }
}
