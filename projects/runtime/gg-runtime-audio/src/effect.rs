/// 效果器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    /// 混响效果器
    Reverb,
    /// 低通滤波效果器
    LowPass,
    /// 音高变化效果器
    PitchShift,
}

/// 音频效果器参数
#[derive(Debug, Clone)]
pub struct AudioEffectParam {
    /// 参数名称
    pub name: String,
    /// 参数值
    pub value: f32,
}

/// 音频效果器种类
///
/// 定义内置的音频效果器类型及其参数。
#[derive(Debug, Clone)]
pub enum AudioEffectKind {
    /// 混响效果器
    ///
    /// 模拟声音在空间中的反射和衰减。
    Reverb {
        /// 衰减系数 [0.0, 1.0]，值越大混响越长
        decay: f32,
        /// 干湿混合比 [0.0, 1.0]，0.0 为全干声，1.0 为全湿声
        mix: f32,
    },
    /// 低通滤波效果器
    ///
    /// 衰减高于截止频率的信号成分。
    LowPass {
        /// 截止频率（Hz）
        cutoff_frequency: f32,
    },
    /// 音高变化效果器
    ///
    /// 改变音频的播放音高。
    PitchShift {
        /// 音高倍率，1.0 为原始音高，2.0 为升高一个八度
        pitch: f32,
    },
}

impl AudioEffectKind {
    /// 获取效果器类型
    pub fn effect_type(&self) -> EffectType {
        match self {
            AudioEffectKind::Reverb { .. } => EffectType::Reverb,
            AudioEffectKind::LowPass { .. } => EffectType::LowPass,
            AudioEffectKind::PitchShift { .. } => EffectType::PitchShift,
        }
    }

    /// 设置效果器参数
    ///
    /// 根据参数名修改效果器的对应参数值。
    /// 支持的参数名因效果器类型而异：
    /// - Reverb: "decay", "mix"
    /// - LowPass: "cutoff_frequency"
    /// - PitchShift: "pitch"
    pub fn set_param(&mut self, param: &str, value: f32) {
        match self {
            AudioEffectKind::Reverb { decay, mix } => match param {
                "decay" => *decay = value.clamp(0.0, 1.0),
                "mix" => *mix = value.clamp(0.0, 1.0),
                _ => {}
            },
            AudioEffectKind::LowPass { cutoff_frequency } => match param {
                "cutoff_frequency" => *cutoff_frequency = value.max(0.0),
                _ => {}
            },
            AudioEffectKind::PitchShift { pitch } => match param {
                "pitch" => *pitch = value.max(0.0),
                _ => {}
            },
        }
    }
}

/// 音频效果器链
///
/// 管理按顺序应用的效果器列表，音频信号依次经过链中的每个效果器处理。
#[derive(Debug, Clone, Default)]
pub struct AudioEffectChain {
    /// 效果器列表
    effects: Vec<AudioEffectKind>,
}

impl AudioEffectChain {
    /// 创建空的效果器链
    pub fn new() -> Self {
        Self::default()
    }

    /// 附加效果器到链末尾
    pub fn attach(&mut self, effect: AudioEffectKind) {
        self.effects.push(effect);
    }

    /// 移除指定类型的效果器
    ///
    /// 移除链中第一个匹配指定类型的效果器。
    /// 返回 true 表示成功移除，false 表示未找到。
    pub fn detach(&mut self, effect_type: EffectType) -> bool {
        let pos = self.effects.iter().position(|e| e.effect_type() == effect_type);
        match pos {
            Some(idx) => {
                self.effects.remove(idx);
                true
            }
            None => false,
        }
    }

    /// 设置效果器参数
    ///
    /// 查找链中第一个匹配指定类型的效果器，并设置其参数。
    /// 返回 true 表示成功设置，false 表示未找到。
    pub fn set_param(&mut self, effect_type: EffectType, param: &str, value: f32) -> bool {
        let effect = self.effects.iter_mut().find(|e| e.effect_type() == effect_type);
        match effect {
            Some(e) => {
                e.set_param(param, value);
                true
            }
            None => false,
        }
    }

    /// 获取效果器列表的引用
    pub fn effects(&self) -> &[AudioEffectKind] {
        &self.effects
    }

    /// 获取效果器列表的可变引用
    pub fn effects_mut(&mut self) -> &mut Vec<AudioEffectKind> {
        &mut self.effects
    }

    /// 检查是否包含指定类型的效果器
    pub fn has_effect(&self, effect_type: EffectType) -> bool {
        self.effects.iter().any(|e| e.effect_type() == effect_type)
    }

    /// 获取效果器数量
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// 检查效果器链是否为空
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}
