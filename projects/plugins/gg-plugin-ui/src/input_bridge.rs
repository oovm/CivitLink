//! UI 输入桥接模块
//! 将平台输入事件转换为 UI 事件并分发到 EventSystem

use gg_core::GResult;
use gg_ecs::System;
use gg_ui::UiEvent;

use crate::{EventSystemResource, FocusManager, UiTreeResource};

/// 触摸阶段
#[derive(Debug, Clone, PartialEq)]
pub enum TouchPhase {
    /// 触摸开始
    Began,
    /// 触摸移动
    Moved,
    /// 触摸结束
    Ended,
}

/// 触摸事件
#[derive(Debug, Clone, PartialEq)]
pub struct TouchEvent {
    /// 触摸点标识符
    pub touch_id: u64,
    /// 触摸 X 坐标
    pub x: f32,
    /// 触摸 Y 坐标
    pub y: f32,
    /// 触摸阶段
    pub phase: TouchPhase,
}

/// 输入状态资源
///
/// 存储当前帧的平台输入状态，由外部运行时在每帧开始时更新。
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// 鼠标 X 坐标
    pub mouse_x: f32,
    /// 鼠标 Y 坐标
    pub mouse_y: f32,
    /// 鼠标左键是否按下
    pub mouse_pressed: bool,
    /// 本帧鼠标是否刚按下（从非按下变为按下）
    pub mouse_just_pressed: bool,
    /// 本帧鼠标是否刚释放（从按下变为非按下）
    pub mouse_just_released: bool,
    /// 本帧鼠标是否移动
    pub mouse_moved: bool,
    /// 本帧键盘事件列表
    pub key_events: Vec<String>,
    /// 鼠标滚动增量
    pub scroll_delta: f32,
    /// 本帧触摸事件列表
    pub touch_events: Vec<TouchEvent>,
}

impl InputState {
    /// 创建新的输入状态
    pub fn new() -> Self {
        Self::default()
    }
}

/// 输入桥接系统
///
/// 每帧从 InputState 资源读取平台输入事件，
/// 转换为 UiEvent 并分发到 EventSystem。
pub struct InputBridgeSystem;

impl System for InputBridgeSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_input_bridge"
    }

    /// 执行输入桥接系统逻辑
    ///
    /// 从 World 获取 InputState，将鼠标、键盘、滚动和触摸事件转换为 UiEvent，
    /// 通过 EventSystem::dispatch 分发到 UI 节点。
    fn execute(&mut self, world: &mut gg_ecs::World) -> GResult<()> {
        let input = world.get_resource::<InputState>().cloned();
        let input = match input {
            Some(i) => i,
            None => return Ok(()),
        };

        let has_mouse_events = input.mouse_moved || input.mouse_just_pressed || input.mouse_just_released;
        let has_key_events = !input.key_events.is_empty();
        let has_scroll = input.scroll_delta != 0.0;
        let has_touch = !input.touch_events.is_empty();

        if !has_mouse_events && !has_key_events && !has_scroll && !has_touch {
            return Ok(());
        }

        let tree = world.get_resource::<UiTreeResource>().map(|r| r.0.clone());

        if has_mouse_events {
            if let Some(event_system) = world.get_resource_mut::<EventSystemResource>() {
                if let Some(ref tree) = tree {
                    if input.mouse_just_pressed {
                        event_system.0.dispatch(&UiEvent::MouseDown { x: input.mouse_x, y: input.mouse_y }, tree);
                    }
                    if input.mouse_just_released {
                        event_system.0.dispatch(&UiEvent::MouseUp { x: input.mouse_x, y: input.mouse_y }, tree);
                        event_system.0.dispatch(&UiEvent::Click { x: input.mouse_x, y: input.mouse_y }, tree);
                    }
                    if input.mouse_moved {
                        event_system.0.dispatch(&UiEvent::MouseMove { x: input.mouse_x, y: input.mouse_y }, tree);
                    }
                }
            }
        }

        if has_scroll {
            if let Some(event_system) = world.get_resource_mut::<EventSystemResource>() {
                if let Some(ref tree) = tree {
                    let event = UiEvent::Scroll {
                        delta_x: 0.0,
                        delta_y: input.scroll_delta,
                        x: input.mouse_x,
                        y: input.mouse_y,
                    };
                    event_system.0.dispatch(&event, tree);
                }
            }
        }

        if has_touch {
            if let Some(event_system) = world.get_resource_mut::<EventSystemResource>() {
                if let Some(ref tree) = tree {
                    for touch in &input.touch_events {
                        let event = match touch.phase {
                            TouchPhase::Began => UiEvent::MouseDown { x: touch.x, y: touch.y },
                            TouchPhase::Moved => UiEvent::MouseMove { x: touch.x, y: touch.y },
                            TouchPhase::Ended => UiEvent::MouseUp { x: touch.x, y: touch.y },
                        };
                        event_system.0.dispatch(&event, tree);
                    }
                }
            }
        }

        if has_key_events {
            let focused = world.get_resource::<FocusManager>().and_then(|fm| fm.focused());
            if let Some(focused_id) = focused {
                if let Some(event_system) = world.get_resource_mut::<EventSystemResource>() {
                    for key in &input.key_events {
                        event_system.0.dispatch_to_node(focused_id, &UiEvent::KeyInput { key: key.clone() });
                    }
                }
            }
        }

        Ok(())
    }
}
