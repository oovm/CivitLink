//! 打字机效果模块
//! 提供逐字显示文本的打字机效果

use gg_ecs::{Component, Resource};

/// 打字机状态
///
/// 管理文本逐字显示的状态，支持按速度逐字显示和跳过动画。
pub struct TypewriterState {
    /// 完整文本
    pub full_text: String,
    /// 当前显示位置
    pub current_position: usize,
    /// 显示速度（字符/秒）
    pub speed: f32,
    /// 已过时间
    pub elapsed: f32,
    /// 是否完成
    pub is_complete: bool,
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
        }
    }

    /// 更新打字机状态
    ///
    /// 根据经过的时间增量推进当前显示位置。
    /// 当显示位置到达文本末尾时，标记为完成。
    pub fn update(&mut self, delta_secs: f32) {
        if self.is_complete {
            return;
        }
        self.elapsed += delta_secs;
        let chars_to_show = (self.elapsed * self.speed) as usize;
        self.current_position = chars_to_show.min(self.full_text.len());
        if self.current_position >= self.full_text.len() {
            self.current_position = self.full_text.len();
            self.is_complete = true;
        }
    }

    /// 获取当前显示的文本
    ///
    /// 返回从文本开头到当前显示位置的子字符串。
    pub fn current_text(&self) -> &str {
        &self.full_text[..self.current_position]
    }

    /// 跳过动画，显示全部文本
    ///
    /// 立即将显示位置推进到文本末尾，标记为完成。
    pub fn skip(&mut self) {
        self.current_position = self.full_text.len();
        self.is_complete = true;
    }

    /// 是否完成
    ///
    /// 返回打字机是否已经完成全部文本的显示。
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }
}

impl Component for TypewriterState {}

impl Resource for TypewriterState {}
