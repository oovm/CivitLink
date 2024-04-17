//! 动画预览模块
//! 提供动画播放状态管理、时间轴控制和关键帧显示功能

use gg_core::GResult;

/// 动画播放状态
///
/// 描述动画的当前播放状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationPlaybackState {
    /// 已停止
    Stopped,
    /// 播放中
    Playing,
    /// 已暂停
    Paused,
}

/// 缓动函数类型
///
/// 定义关键帧之间的插值方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingType {
    /// 线性插值
    Linear,
    /// 缓入（慢启动）
    EaseIn,
    /// 缓出（慢结束）
    EaseOut,
    /// 缓入缓出（慢启动和结束）
    EaseInOut,
    /// 阶跃（无插值，直接跳变）
    Step,
}

impl Default for EasingType {
    fn default() -> Self {
        EasingType::Linear
    }
}

/// 关键帧数据
///
/// 描述动画轨道中一个关键帧的时间和缓动信息。
#[derive(Debug, Clone)]
pub struct KeyframeData {
    /// 关键帧时间（秒）
    pub time: f32,
    /// 缓动函数类型
    pub easing: EasingType,
}

/// 动画轨道数据
///
/// 描述动画中的一个轨道，包含轨道名称、目标组件和关键帧列表。
#[derive(Debug, Clone)]
pub struct AnimationTrackData {
    /// 轨道名称
    pub name: String,
    /// 轨道目标组件路径
    pub target: String,
    /// 关键帧列表（按时间排序）
    pub keyframes: Vec<KeyframeData>,
}

/// 动画事件数据
///
/// 描述动画在特定时间点触发的事件。
#[derive(Debug, Clone)]
pub struct AnimationEventData {
    /// 事件触发时间（秒）
    pub time: f32,
    /// 事件名称
    pub name: String,
}

/// 动画时间轴状态
///
/// 管理动画播放的时间位置、时长、速度和循环设置。
pub struct AnimationTimelineState {
    /// 当前播放位置（秒）
    pub position: f32,
    /// 动画总时长（秒）
    pub duration: f32,
    /// 播放速度倍率
    pub speed: f32,
    /// 是否循环播放
    pub is_looping: bool,
}

impl AnimationTimelineState {
    /// 创建默认的时间轴状态
    ///
    /// 初始位置为 0，时长 1 秒，速度 1x，不循环。
    pub fn new() -> Self {
        Self { position: 0.0, duration: 1.0, speed: 1.0, is_looping: false }
    }

    /// 创建指定时长的时间轴状态
    pub fn with_duration(duration: f32) -> Self {
        Self { position: 0.0, duration: duration.max(0.001), speed: 1.0, is_looping: false }
    }

    /// 推进时间轴
    ///
    /// 根据经过的时间和播放速度推进时间轴位置。
    /// 如果启用循环，到达末尾时从头开始；否则停在末尾。
    pub fn advance(&mut self, delta_secs: f32) {
        self.position += delta_secs * self.speed;
        if self.position >= self.duration {
            if self.is_looping {
                self.position %= self.duration;
            }
            else {
                self.position = self.duration;
            }
        }
        if self.position < 0.0 {
            self.position = 0.0;
        }
    }

    /// 获取归一化的播放位置（0.0 ~ 1.0）
    pub fn normalized_position(&self) -> f32 {
        if self.duration > 0.0 { (self.position / self.duration).clamp(0.0, 1.0) } else { 0.0 }
    }
}

/// 动画工具栏按钮类型
///
/// 定义动画预览工具栏中可用的操作按钮。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationToolbarKind {
    /// 播放按钮
    Play,
    /// 暂停按钮
    Pause,
    /// 停止按钮
    Stop,
    /// 循环切换按钮
    LoopToggle,
}

/// 动画工具栏项
///
/// 描述工具栏中的一个按钮及其激活状态。
pub struct AnimationToolbarItem {
    /// 按钮类型
    pub kind: AnimationToolbarKind,
    /// 是否激活（例如循环按钮在启用循环时为激活状态）
    pub active: bool,
}

/// 时间轴控制状态
///
/// 管理时间轴的拖拽交互和坐标转换。
pub struct TimelineControl {
    /// 是否正在拖拽时间轴指示器
    pub is_dragging: bool,
    /// 拖拽起始位置（像素 X 坐标）
    pub drag_start_position: f32,
    /// 时间轴显示宽度（像素）
    pub display_width: f32,
}

impl TimelineControl {
    /// 创建新的时间轴控制
    ///
    /// 使用指定的显示宽度初始化。
    pub fn new(display_width: f32) -> Self {
        Self { is_dragging: false, drag_start_position: 0.0, display_width: display_width.max(1.0) }
    }

    /// 开始拖拽时间轴指示器
    pub fn start_drag(&mut self, position: f32) {
        self.is_dragging = true;
        self.drag_start_position = position;
    }

    /// 更新拖拽位置
    ///
    /// 将鼠标位置转换为时间并更新时间轴位置。
    pub fn update_drag(&mut self, position: f32, timeline: &mut AnimationTimelineState, duration: f32) {
        let time = self.x_to_time(position, duration);
        timeline.position = time.clamp(0.0, duration);
    }

    /// 结束拖拽
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
    }

    /// 将时间转换为 X 坐标
    pub fn time_to_x(&self, time: f32, duration: f32) -> f32 {
        if duration > 0.0 { (time / duration) * self.display_width } else { 0.0 }
    }

    /// 将 X 坐标转换为时间
    pub fn x_to_time(&self, x: f32, duration: f32) -> f32 {
        if self.display_width > 0.0 { (x / self.display_width) * duration } else { 0.0 }
    }

    /// 设置时间轴显示宽度
    pub fn set_display_width(&mut self, width: f32) {
        self.display_width = width.max(1.0);
    }
}

/// 动画预览状态
///
/// 组合动画播放状态、时间轴、轨道数据和事件数据，
/// 提供完整的动画预览控制能力。
pub struct AnimationPreviewState {
    /// 播放状态
    pub playback_state: AnimationPlaybackState,
    /// 时间轴状态
    pub timeline: AnimationTimelineState,
    /// 动画名称
    pub animation_name: Option<String>,
    /// 动画轨道列表
    pub tracks: Vec<AnimationTrackData>,
    /// 动画事件列表
    pub events: Vec<AnimationEventData>,
    /// 动画总时长（秒）
    pub duration: f32,
    /// 是否循环播放
    pub is_looping: bool,
    /// 播放速度倍率（0.1 ~ 3.0）
    pub speed: f32,
}

impl AnimationPreviewState {
    /// 创建新的动画预览状态
    ///
    /// 初始状态为停止，无轨道和事件数据。
    pub fn new() -> Self {
        Self {
            playback_state: AnimationPlaybackState::Stopped,
            timeline: AnimationTimelineState::new(),
            animation_name: None,
            tracks: Vec::new(),
            events: Vec::new(),
            duration: 0.0,
            is_looping: false,
            speed: 1.0,
        }
    }

    /// 创建指定名称的动画预览状态
    pub fn with_name(name: String) -> Self {
        Self { animation_name: Some(name), ..Self::new() }
    }

    /// 从动画数据加载
    ///
    /// 设置轨道、事件、时长和循环属性，并重置播放位置。
    pub fn load_from_data(
        &mut self,
        tracks: Vec<AnimationTrackData>,
        events: Vec<AnimationEventData>,
        duration: f32,
        is_looping: bool,
    ) {
        self.tracks = tracks;
        self.events = events;
        self.duration = duration;
        self.is_looping = is_looping;
        self.timeline = AnimationTimelineState::with_duration(duration);
        self.timeline.is_looping = is_looping;
        self.playback_state = AnimationPlaybackState::Stopped;
    }

    /// 开始播放动画
    pub fn play(&mut self) {
        if self.playback_state == AnimationPlaybackState::Stopped {
            self.timeline.position = 0.0;
        }
        self.playback_state = AnimationPlaybackState::Playing;
    }

    /// 暂停动画播放
    pub fn pause(&mut self) {
        if self.playback_state == AnimationPlaybackState::Playing {
            self.playback_state = AnimationPlaybackState::Paused;
        }
    }

    /// 停止动画播放
    ///
    /// 重置播放位置到起始点。
    pub fn stop(&mut self) {
        self.playback_state = AnimationPlaybackState::Stopped;
        self.timeline.position = 0.0;
    }

    /// 跳转到指定时间位置
    pub fn seek(&mut self, position_secs: f32) {
        self.timeline.position = position_secs.clamp(0.0, self.duration);
    }

    /// 设置播放速度
    ///
    /// 速度范围限制在 0.1x ~ 3.0x 之间。
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.1, 3.0);
        self.timeline.speed = self.speed;
    }

    /// 切换循环播放
    pub fn toggle_loop(&mut self) {
        self.is_looping = !self.is_looping;
        self.timeline.is_looping = self.is_looping;
    }

    /// 推进动画一帧
    ///
    /// 如果正在播放，根据 delta 时间推进时间轴。
    /// 检查是否到达动画末尾，非循环时自动停止。
    pub fn tick(&mut self, delta_secs: f32) {
        if self.playback_state == AnimationPlaybackState::Playing {
            self.timeline.advance(delta_secs);
            if !self.is_looping && self.timeline.position >= self.duration {
                self.playback_state = AnimationPlaybackState::Stopped;
            }
        }
    }

    /// 是否正在播放
    pub fn is_playing(&self) -> bool {
        self.playback_state == AnimationPlaybackState::Playing
    }

    /// 是否已暂停
    pub fn is_paused(&self) -> bool {
        self.playback_state == AnimationPlaybackState::Paused
    }

    /// 是否已停止
    pub fn is_stopped(&self) -> bool {
        self.playback_state == AnimationPlaybackState::Stopped
    }

    /// 获取指定轨道的当前关键帧索引
    ///
    /// 返回时间轴当前位置之前最近的关键帧索引。
    /// 如果轨道不存在或没有关键帧，返回 None。
    pub fn current_keyframe_index(&self, track_index: usize) -> Option<usize> {
        self.tracks.get(track_index).and_then(|track| {
            let current_time = self.timeline.position;
            track.keyframes.iter().rposition(|kf| kf.time <= current_time)
        })
    }

    /// 获取所有关键帧的唯一时间点
    ///
    /// 收集所有轨道中的关键帧时间，去重并按时间排序。
    pub fn all_keyframe_times(&self) -> Vec<f32> {
        let mut times: Vec<f32> = self.tracks.iter().flat_map(|track| track.keyframes.iter().map(|kf| kf.time)).collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        times.dedup_by(|a, b| (a - b).abs() < f32::EPSILON);
        times
    }

    /// 构建动画预览工具栏 UI 项列表
    ///
    /// 根据当前播放状态返回应显示的工具栏按钮。
    pub fn build_toolbar_ui(&self) -> Vec<AnimationToolbarItem> {
        let mut items = Vec::new();
        match self.playback_state {
            AnimationPlaybackState::Stopped => {
                items.push(AnimationToolbarItem { kind: AnimationToolbarKind::Play, active: false });
            }
            AnimationPlaybackState::Playing => {
                items.push(AnimationToolbarItem { kind: AnimationToolbarKind::Pause, active: false });
                items.push(AnimationToolbarItem { kind: AnimationToolbarKind::Stop, active: false });
            }
            AnimationPlaybackState::Paused => {
                items.push(AnimationToolbarItem { kind: AnimationToolbarKind::Play, active: false });
                items.push(AnimationToolbarItem { kind: AnimationToolbarKind::Stop, active: false });
            }
        }
        items.push(AnimationToolbarItem { kind: AnimationToolbarKind::LoopToggle, active: self.is_looping });
        items
    }
}

/// 动画文件加载器
///
/// 提供从文件或 VON 格式文本加载动画数据的功能。
pub struct AnimationLoader;

impl AnimationLoader {
    /// 从文件路径加载动画数据
    ///
    /// 读取指定路径的 .animation 文件并解析为 AnimationPreviewState。
    pub fn load_from_file(path: &str) -> GResult<AnimationPreviewState> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_from_von(&content)
    }

    /// 从 VON 格式文本解析动画数据
    ///
    /// 解析 VON 格式的动画定义，提取轨道、关键帧和事件信息。
    pub fn parse_from_von(content: &str) -> GResult<AnimationPreviewState> {
        let mut state = AnimationPreviewState::new();
        let mut tracks: Vec<AnimationTrackData> = Vec::new();
        let mut events: Vec<AnimationEventData> = Vec::new();
        let mut duration: f32 = 1.0;
        let mut is_looping: bool = false;
        let mut name: Option<String> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("name:") {
                name = Some(rest.trim().trim_matches('"').to_string());
            }
            else if let Some(rest) = trimmed.strip_prefix("duration:") {
                duration = rest.trim().parse::<f32>().unwrap_or(1.0);
            }
            else if let Some(rest) = trimmed.strip_prefix("loop:") {
                is_looping = rest.trim() == "true";
            }
            else if let Some(rest) = trimmed.strip_prefix("track:") {
                let parts: Vec<&str> = rest.trim().split(',').collect();
                if parts.len() >= 2 {
                    tracks.push(AnimationTrackData {
                        name: parts[0].trim().to_string(),
                        target: parts[1].trim().to_string(),
                        keyframes: Vec::new(),
                    });
                }
            }
            else if let Some(rest) = trimmed.strip_prefix("keyframe:") {
                let parts: Vec<&str> = rest.trim().split(',').collect();
                if parts.len() >= 2 {
                    if let Ok(time) = parts[0].trim().parse::<f32>() {
                        let easing = match parts.get(1).map(|s| s.trim()) {
                            Some("ease_in") => EasingType::EaseIn,
                            Some("ease_out") => EasingType::EaseOut,
                            Some("ease_in_out") => EasingType::EaseInOut,
                            Some("step") => EasingType::Step,
                            _ => EasingType::Linear,
                        };
                        if let Some(last_track) = tracks.last_mut() {
                            last_track.keyframes.push(KeyframeData { time, easing });
                        }
                    }
                }
            }
            else if let Some(rest) = trimmed.strip_prefix("event:") {
                let parts: Vec<&str> = rest.trim().split(',').collect();
                if parts.len() >= 2 {
                    if let Ok(time) = parts[0].trim().parse::<f32>() {
                        events.push(AnimationEventData { time, name: parts[1].trim().to_string() });
                    }
                }
            }
        }

        state.animation_name = name;
        state.load_from_data(tracks, events, duration, is_looping);
        Ok(state)
    }
}
