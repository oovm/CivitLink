//! Web Audio API 后端实现
//!
//! 通过 `wasm-bindgen` 绑定 Web Audio API，
//! 为 WebAssembly 环境提供音频播放能力。

use std::{collections::HashMap, path::Path};

use gg_core::{GError, GErrorKind, GResult};

use crate::{AudioCommand, AudioContext, AudioEngine, SoundDescriptor, SoundFormat, SoundId};

/// 活跃的音频播放
///
/// 追踪正在播放的音频节点，用于停止、音量调节等控制操作。
#[cfg(target_arch = "wasm32")]
struct ActivePlayback {
    /// 音频缓冲源节点
    source: web_sys::AudioBufferSourceNode,
    /// 增益节点
    gain: web_sys::GainNode,
}

/// 活跃的音频播放（非 wasm32 占位）
#[cfg(not(target_arch = "wasm32"))]
struct ActivePlayback;

/// AudioContext 包装器
///
/// 在 wasm32 目标下持有真实的 `web_sys::AudioContext`，
/// 在非 wasm32 目标下为空结构体。
#[cfg(target_arch = "wasm32")]
struct AudioCtxWrapper {
    /// 内部 AudioContext
    ctx: web_sys::AudioContext,
}

/// AudioContext 包装器（非 wasm32 占位）
#[cfg(not(target_arch = "wasm32"))]
struct AudioCtxWrapper;

/// 基于 Web Audio API 的音频引擎实现
///
/// 通过浏览器的 Web Audio API 提供音频播放能力，
/// 仅在 `wasm32` 目标下可用。
pub struct WebAudioEngine {
    /// 已加载的声音数据缓存（路径 -> 原始数据）
    sounds: HashMap<SoundId, Vec<u8>>,
    /// 声音描述符缓存
    descriptors: HashMap<SoundId, SoundDescriptor>,
    /// 路径到声音 ID 的映射
    path_to_id: HashMap<String, SoundId>,
    /// 下一个声音 ID
    next_sound_id: u64,
    /// Web Audio 上下文（单例）
    audio_ctx: Option<AudioCtxWrapper>,
    /// 活跃的音频播放
    active_playbacks: HashMap<SoundId, ActivePlayback>,
}

impl WebAudioEngine {
    /// 创建新的 Web Audio 引擎
    pub fn new() -> GResult<Self> {
        Ok(Self {
            sounds: HashMap::new(),
            descriptors: HashMap::new(),
            path_to_id: HashMap::new(),
            next_sound_id: 1,
            audio_ctx: None,
            active_playbacks: HashMap::new(),
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

    /// 获取或创建 AudioContext 单例
    #[cfg(target_arch = "wasm32")]
    fn get_or_create_audio_ctx(&mut self) -> GResult<&web_sys::AudioContext> {
        if self.audio_ctx.is_none() {
            let ctx = web_sys::AudioContext::new().map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: "Failed to create AudioContext".to_string(),
            })?;
            self.audio_ctx = Some(AudioCtxWrapper { ctx });
        }
        Ok(&self.audio_ctx.as_ref().unwrap().ctx)
    }

    /// 获取或创建 AudioContext 单例（非 wasm32 占位）
    #[cfg(not(target_arch = "wasm32"))]
    fn get_or_create_audio_ctx(&mut self) -> GResult<&AudioCtxWrapper> {
        Err(GError {
            kind: GErrorKind::Platform,
            message: "WebAudioEngine is only available on wasm32 target".to_string(),
        })
    }

    /// 通过 JavaScript 解码音频并获取时长
    ///
    /// 使用 Web Audio API 的 `decodeAudioData` 解码音频数据，
    /// 返回 (sample_rate, channels, duration)。
    #[cfg(target_arch = "wasm32")]
    fn decode_audio_info_with_ctx(
        audio_ctx: &web_sys::AudioContext,
        data: &[u8],
    ) -> GResult<(u32, u16, f64)> {
        let js_array = js_sys::Uint8Array::new_with_length(data.len() as u32);
        js_array.copy_from(data);

        let promise = audio_ctx
            .decode_audio_data_with_array_buffer(&js_array.buffer())
            .map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: "Failed to decode audio data".to_string(),
            })?;

        use wasm_bindgen_futures::JsFuture;
        let result = wasm_bindgen_futures::futures::block_on(JsFuture::from(promise))
            .map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: "Audio decode failed".to_string(),
            })?;

        let audio_buffer = web_sys::AudioBuffer::from(result);
        let sample_rate = audio_buffer.sample_rate() as u32;
        let channels = audio_buffer.number_of_channels() as u16;
        let duration = audio_buffer.duration();

        Ok((sample_rate, channels, duration))
    }

    /// 通过 JavaScript 解码音频并获取时长（非 wasm32 占位）
    #[cfg(not(target_arch = "wasm32"))]
    fn decode_audio_info_with_ctx(
        _ctx: &AudioCtxWrapper,
        _data: &[u8],
    ) -> GResult<(u32, u16, f64)> {
        Err(GError {
            kind: GErrorKind::Platform,
            message: "WebAudioEngine is only available on wasm32 target".to_string(),
        })
    }

    /// 清理已结束的活跃播放
    fn cleanup_finished_playbacks(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            let finished: Vec<SoundId> = self
                .active_playbacks
                .iter()
                .filter(|(_, playback)| playback.source.playback_state() == 3)
                .map(|(id, _)| *id)
                .collect();
            for id in finished {
                self.active_playbacks.remove(&id);
            }
        }
    }
}

impl AudioEngine for WebAudioEngine {
    fn load_sound(&mut self, path: &Path) -> GResult<SoundId> {
        let path_str = path.to_string_lossy().to_string();

        if let Some(existing_id) = self.path_to_id.get(&path_str) {
            return Ok(*existing_id);
        }

        let data = self.fetch_file_data(path)?;

        let format = Self::guess_format(path).ok_or_else(|| GError {
            kind: GErrorKind::Asset,
            message: format!("Unsupported audio format for file: {:?}", path),
        })?;

        let audio_ctx = self.get_or_create_audio_ctx()?;
        let (sample_rate, channels, duration_secs) =
            Self::decode_audio_info_with_ctx(audio_ctx, &data)?;

        let sound_id = self.allocate_sound_id();
        let descriptor = SoundDescriptor {
            format,
            duration_secs,
            channels,
            sample_rate,
        };

        self.sounds.insert(sound_id, data);
        self.descriptors.insert(sound_id, descriptor);
        self.path_to_id.insert(path_str, sound_id);

        Ok(sound_id)
    }

    fn play(&mut self, sound_id: SoundId, volume: f32, looped: bool) -> GResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let data = self.sounds.get(&sound_id).ok_or_else(|| GError {
                kind: GErrorKind::Asset,
                message: format!("Sound not found: {:?}", sound_id),
            })?;

            let audio_ctx = self.get_or_create_audio_ctx()?;

            let js_array = js_sys::Uint8Array::new_with_length(data.len() as u32);
            js_array.copy_from(data);

            let promise = audio_ctx
                .decode_audio_data_with_array_buffer(&js_array.buffer())
                .map_err(|_| GError {
                    kind: GErrorKind::Platform,
                    message: "Failed to decode audio data".to_string(),
                })?;

            let result =
                wasm_bindgen_futures::futures::block_on(wasm_bindgen_futures::JsFuture::from(
                    promise,
                ))
                .map_err(|_| GError {
                    kind: GErrorKind::Platform,
                    message: "Audio decode failed".to_string(),
                })?;

            let audio_buffer = web_sys::AudioBuffer::from(result);

            let source = audio_ctx.create_buffer_source().map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: "Failed to create BufferSourceNode".to_string(),
            })?;

            source.set_buffer(Some(&audio_buffer));
            source.set_loop(looped);

            let gain_node = audio_ctx.create_gain().map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: "Failed to create GainNode".to_string(),
            })?;

            gain_node.gain().set_value(volume);

            source
                .connect_with_audio_node(&gain_node)
                .map_err(|_| GError {
                    kind: GErrorKind::Platform,
                    message: "Failed to connect source to gain node".to_string(),
                })?;

            gain_node
                .connect_with_audio_node(&audio_ctx.destination())
                .map_err(|_| GError {
                    kind: GErrorKind::Platform,
                    message: "Failed to connect gain node to destination".to_string(),
                })?;

            source.start().map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: "Failed to start audio playback".to_string(),
            })?;

            self.active_playbacks.remove(&sound_id);

            self.active_playbacks
                .insert(sound_id, ActivePlayback { source, gain: gain_node });

            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (sound_id, volume, looped);
            Err(GError {
                kind: GErrorKind::Platform,
                message: "WebAudioEngine is only available on wasm32 target".to_string(),
            })
        }
    }

    fn stop(&mut self, sound_id: SoundId) -> GResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(playback) = self.active_playbacks.remove(&sound_id) {
                let _ = playback.source.stop();
                let _ = playback.source.disconnect();
                let _ = playback.gain.disconnect();
            }
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = sound_id;
            Err(GError {
                kind: GErrorKind::Platform,
                message: "WebAudioEngine is only available on wasm32 target".to_string(),
            })
        }
    }

    fn set_volume(&mut self, sound_id: SoundId, volume: f32) -> GResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(playback) = self.active_playbacks.get(&sound_id) {
                playback.gain.gain().set_value(volume);
            }
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (sound_id, volume);
            Err(GError {
                kind: GErrorKind::Platform,
                message: "WebAudioEngine is only available on wasm32 target".to_string(),
            })
        }
    }

    fn pause(&mut self, _sound_id: SoundId) -> GResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(ref wrapper) = self.audio_ctx {
                let _ = wrapper.ctx.suspend();
            }
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = _sound_id;
            Err(GError {
                kind: GErrorKind::Platform,
                message: "WebAudioEngine is only available on wasm32 target".to_string(),
            })
        }
    }

    fn resume(&mut self, _sound_id: SoundId) -> GResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(ref wrapper) = self.audio_ctx {
                let _ = wrapper.ctx.resume();
            }
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = _sound_id;
            Err(GError {
                kind: GErrorKind::Platform,
                message: "WebAudioEngine is only available on wasm32 target".to_string(),
            })
        }
    }

    fn update(&mut self, context: &AudioContext) -> GResult<()> {
        self.cleanup_finished_playbacks();
        for command in context.commands() {
            match command.clone() {
                AudioCommand::Play {
                    sound_id,
                    volume,
                    looped,
                } => {
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

    fn reload_sound(&mut self, path: &str) -> GResult<()> {
        let path_ref = std::path::Path::new(path);
        self.load_sound(path_ref)?;
        Ok(())
    }
}

impl WebAudioEngine {
    /// 通过 fetch API 获取文件数据
    #[cfg(target_arch = "wasm32")]
    fn fetch_file_data(&self, path: &Path) -> GResult<Vec<u8>> {
        let url = path.to_string_lossy().to_string();

        let window = web_sys::window().ok_or_else(|| GError {
            kind: GErrorKind::Platform,
            message: "No window object available".to_string(),
        })?;

        let promise = window.fetch_with_str(&url);

        let resp =
            wasm_bindgen_futures::futures::block_on(wasm_bindgen_futures::JsFuture::from(promise))
                .map_err(|_| GError {
                    kind: GErrorKind::Io,
                    message: format!("Failed to fetch '{}'", url),
                })?;

        let response = web_sys::Response::from(resp);

        if !response.ok() {
            return Err(GError {
                kind: GErrorKind::Io,
                message: format!("HTTP error fetching '{}': {}", url, response.status()),
            });
        }

        let array_buffer_promise = response.array_buffer().map_err(|_| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to get array buffer for '{}'", url),
        })?;

        let array_buffer =
            wasm_bindgen_futures::futures::block_on(wasm_bindgen_futures::JsFuture::from(
                array_buffer_promise,
            ))
            .map_err(|_| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to read array buffer for '{}'", url),
            })?;

        let js_array = js_sys::Uint8Array::new(&array_buffer);
        let mut data = vec![0u8; js_array.length() as usize];
        js_array.copy_to(&mut data);

        Ok(data)
    }

    /// 通过 fetch API 获取文件数据（非 wasm32 占位）
    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_file_data(&self, path: &Path) -> GResult<Vec<u8>> {
        Err(GError {
            kind: GErrorKind::Platform,
            message: format!(
                "WebAudioEngine is only available on wasm32 target, cannot fetch '{}'",
                path.display()
            ),
        })
    }
}
