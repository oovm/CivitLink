//! 打字机效果模块
//! 提供逐字显示文本的打字机效果，支持富文本标签和延迟

/// 打字机状态
///
/// 管理文本逐字显示的状态，支持按速度逐字显示和跳过动画。
/// 自动实现 `Component` 和 `Resource` trait（通过 blanket impl）。
pub struct TypewriterState {
    /// 完整文本
    pub full_text: String,
    /// 当前显示字符位置（字符索引，非字节索引）
    pub current_position: usize,
    /// 显示速度（字符/秒）
    pub speed: f32,
    /// 已过时间
    pub elapsed: f32,
    /// 是否完成
    pub is_complete: bool,
    /// 延迟暂停剩余时间（秒）
    pub pending_delay: f32,
    /// 逐字显示回调
    pub on_char_display: Option<Box<dyn FnMut(char) + Send + Sync>>,
}

impl TypewriterState {
    /// 创建新的打字机状态
    ///
    /// 使用指定的完整文本和显示速度初始化打字机。
    pub fn new(text: String, speed: f32) -> Self {
        Self {
            full_text: text,
            current_position: 0,
            speed,
            elapsed: 0.0,
            is_complete: false,
            pending_delay: 0.0,
            on_char_display: None,
        }
    }

    /// 更新打字机状态
    ///
    /// 根据经过的时间增量推进当前显示位置。
    /// 当显示位置到达文本末尾时，标记为完成。
    /// 如果存在延迟暂停，先消耗延迟时间再推进显示。
    pub fn update(&mut self, delta_secs: f32) {
        if self.is_complete {
            return;
        }

        if self.pending_delay > 0.0 {
            self.pending_delay -= delta_secs;
            return;
        }

        let prev_position = self.current_position;
        self.elapsed += delta_secs;
        let total_chars = self.full_text.chars().count();
        let chars_to_show = (self.elapsed * self.speed) as usize;
        self.current_position = chars_to_show.min(total_chars);

        if self.current_position >= total_chars {
            self.current_position = total_chars;
            self.is_complete = true;
        }

        if let Some(ref mut callback) = self.on_char_display {
            for (i, ch) in self.full_text.chars().enumerate() {
                if i >= prev_position && i < self.current_position {
                    callback(ch);
                }
            }
        }
    }

    /// 获取当前显示的文本
    ///
    /// 返回从文本开头到当前显示位置的子字符串。
    /// 使用字符边界进行安全切片，不会因多字节 UTF-8 字符导致 panic。
    pub fn current_text(&self) -> &str {
        let byte_pos = self.full_text.char_indices().nth(self.current_position).map(|(i, _)| i).unwrap_or(self.full_text.len());
        &self.full_text[..byte_pos]
    }

    /// 跳过动画，显示全部文本
    ///
    /// 立即将显示位置推进到文本末尾，标记为完成。
    pub fn skip(&mut self) {
        self.current_position = self.full_text.chars().count();
        self.pending_delay = 0.0;
        self.is_complete = true;
    }

    /// 是否完成
    ///
    /// 返回打字机是否已经完成全部文本的显示。
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    /// 设置延迟暂停
    ///
    /// 打字机将在指定秒数内暂停显示。
    pub fn set_delay(&mut self, seconds: f32) {
        self.pending_delay = seconds;
    }
}

/// 富文本片段
#[derive(Debug, Clone)]
pub enum RichTextSegment {
    /// 普通文本
    Text(String),
    /// 带颜色的文本
    Colored {
        /// 文本内容
        text: String,
        /// RGBA 颜色值
        color: [f32; 4],
    },
    /// 带大小的文本
    Sized {
        /// 文本内容
        text: String,
        /// 字体大小
        size: f32,
    },
    /// 延迟标记
    Delay {
        /// 延迟秒数
        seconds: f32,
    },
}

/// 解析富文本标签
///
/// 支持以下标签格式：
/// - `{color=#rrggbb}文本{/color}` - 颜色标签
/// - `{size=N}文本{/size}` - 大小标签
/// - `{w=N}` - 延迟标签
pub fn parse_rich_text(text: &str) -> Vec<RichTextSegment> {
    let mut segments = Vec::new();
    let mut remaining = text;
    let mut plain = String::new();

    while !remaining.is_empty() {
        if let Some(pos) = remaining.find('{') {
            if pos > 0 {
                plain.push_str(&remaining[..pos]);
            }
            remaining = &remaining[pos..];

            if remaining.starts_with("{color=") {
                if !plain.is_empty() {
                    segments.push(RichTextSegment::Text(plain.clone()));
                    plain.clear();
                }
                let end_tag = remaining.find('}').unwrap_or(remaining.len());
                let color_str = &remaining[7..end_tag];
                let color = parse_color_hex(color_str);
                remaining = &remaining[end_tag + 1..];

                let close_pos = remaining.find("{/color}").unwrap_or(remaining.len());
                let colored_text = &remaining[..close_pos];
                segments.push(RichTextSegment::Colored { text: colored_text.to_string(), color });
                remaining = if close_pos < remaining.len() { &remaining[close_pos + 8..] } else { "" };
            }
            else if remaining.starts_with("{size=") {
                if !plain.is_empty() {
                    segments.push(RichTextSegment::Text(plain.clone()));
                    plain.clear();
                }
                let end_tag = remaining.find('}').unwrap_or(remaining.len());
                let size_str = &remaining[6..end_tag];
                let size: f32 = size_str.parse().unwrap_or(18.0);
                remaining = &remaining[end_tag + 1..];

                let close_pos = remaining.find("{/size}").unwrap_or(remaining.len());
                let sized_text = &remaining[..close_pos];
                segments.push(RichTextSegment::Sized { text: sized_text.to_string(), size });
                remaining = if close_pos < remaining.len() { &remaining[close_pos + 7..] } else { "" };
            }
            else if remaining.starts_with("{w=") {
                if !plain.is_empty() {
                    segments.push(RichTextSegment::Text(plain.clone()));
                    plain.clear();
                }
                let end_tag = remaining.find('}').unwrap_or(remaining.len());
                let delay_str = &remaining[3..end_tag];
                let seconds: f32 = delay_str.parse().unwrap_or(0.0);
                segments.push(RichTextSegment::Delay { seconds });
                remaining = &remaining[end_tag + 1..];
            }
            else {
                plain.push('{');
                remaining = &remaining[1..];
            }
        }
        else {
            plain.push_str(remaining);
            break;
        }
    }

    if !plain.is_empty() {
        segments.push(RichTextSegment::Text(plain));
    }

    segments
}

/// 解析十六进制颜色字符串为 RGBA
fn parse_color_hex(hex: &str) -> [f32; 4] {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
        let a = if hex.len() >= 8 { u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0 } else { 1.0 };
        [r, g, b, a]
    }
    else {
        [1.0, 1.0, 1.0, 1.0]
    }
}
