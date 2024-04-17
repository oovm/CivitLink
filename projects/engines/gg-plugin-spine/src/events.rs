//! Spine 动画事件回调模块
//! 提供动画时间线事件触发功能

use serde::{Deserialize, Serialize};

/// Spine 动画事件
///
/// 定义在动画播放到特定时间点时触发的事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineEvent {
    /// 事件名称
    pub name: String,
    /// 触发时间点（秒）
    pub time: f32,
    /// 事件附加数据
    pub data: Option<String>,
}

/// Spine 事件回调 trait
///
/// 实现此 trait 以接收 Spine 动画事件回调。
pub trait SpineEventCallback: Send + Sync {
    /// 处理 Spine 动画事件
    fn on_event(&self, event: &SpineEvent);
}

/// Spine 事件管理器
///
/// 管理动画事件和已触发事件的记录。
#[derive(Debug, Clone, Default)]
pub struct SpineEventManager {
    /// 已注册的事件列表
    pub events: Vec<SpineEvent>,
    /// 已触发的事件时间点集合（防止重复触发）
    pub fired_events: Vec<String>,
}

impl SpineEventManager {
    /// 创建新的事件管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册动画事件
    pub fn register_event(&mut self, event: SpineEvent) {
        self.events.push(event);
    }

    /// 检查并触发在指定时间范围内的事件
    ///
    /// 当动画时间从 `prev_time` 推进到 `current_time` 时，
    /// 检查是否有事件的时间点落在这个范围内。
    pub fn check_events(&mut self, prev_time: f32, current_time: f32) -> Vec<SpineEvent> {
        let mut triggered = Vec::new();

        for event in &self.events {
            if prev_time < event.time && current_time >= event.time {
                if !self.fired_events.contains(&event.name) {
                    triggered.push(event.clone());
                    self.fired_events.push(event.name.clone());
                }
            }
        }

        triggered
    }

    /// 清除已触发事件记录
    ///
    /// 在动画循环时调用，允许事件在新循环中重新触发。
    pub fn clear_fired(&mut self) {
        self.fired_events.clear();
    }
}
