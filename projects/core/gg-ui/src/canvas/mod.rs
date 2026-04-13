//! GG Game UI 系统
//!
//! 基于 GameObject 和 Component 的游戏UI系统，参考 Unity uGUI 设计理念
//! 为游戏提供灵活、支持复杂动画和3D效果的UI解决方案

#![warn(missing_docs)]

/// 动画系统模块
pub mod animation;
/// 核心组件模块
pub mod components;
/// 事件系统模块
pub mod events;
/// 布局系统模块
pub mod layout;
/// 渲染系统模块
pub mod renderer;

use gg_ecs::{Component, Entity, World};
use gg_render::{Color, DrawCommand, RenderContext};
use std::sync::Arc;

/// Canvas 渲染模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasRenderMode {
    /// 屏幕空间覆盖
    ScreenSpaceOverlay,
    /// 屏幕空间相机
    ScreenSpaceCamera,
    /// 世界空间
    WorldSpace,
}

/// Canvas 组件，管理UI元素的渲染
#[derive(Debug, Clone, Component)]
pub struct Canvas {
    /// 渲染模式
    pub render_mode: CanvasRenderMode,
    /// 是否像素完美
    pub pixel_perfect: bool,
    /// 排序顺序
    pub sorting_order: i32,
    /// 目标显示
    pub target_display: i32,
    /// 世界相机（仅在 ScreenSpaceCamera 和 WorldSpace 模式下使用）
    pub world_camera: Option<Entity>,
    /// 平面距离（仅在 ScreenSpaceCamera 模式下使用）
    pub plane_distance: f32,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            render_mode: CanvasRenderMode::ScreenSpaceOverlay,
            pixel_perfect: false,
            sorting_order: 0,
            target_display: 0,
            world_camera: None,
            plane_distance: 1.0,
        }
    }
}

/// RectTransform 组件，管理UI元素的位置、旋转、缩放
#[derive(Debug, Clone, Component)]
pub struct RectTransform {
    /// 锚点最小值
    pub anchor_min: [f32; 2],
    /// 锚点最大值
    pub anchor_max: [f32; 2],
    /// 锚点位置
    pub anchored_position: [f32; 2],
    /// 尺寸增量
    pub size_delta: [f32; 2],
    /// 轴心点
    pub pivot: [f32; 2],
    /// 旋转
    pub rotation: [f32; 4],
    /// 本地缩放
    pub local_scale: [f32; 3],
}

impl Default for RectTransform {
    fn default() -> Self {
        Self {
            anchor_min: [0.0, 0.0],
            anchor_max: [1.0, 1.0],
            anchored_position: [0.0, 0.0],
            size_delta: [0.0, 0.0],
            pivot: [0.5, 0.5],
            rotation: [0.0, 0.0, 0.0, 1.0],
            local_scale: [1.0, 1.0, 1.0],
        }
    }
}

/// CanvasRenderer 组件，负责UI元素的渲染
#[derive(Debug, Clone, Component)]
pub struct CanvasRenderer {
    /// 是否裁剪透明网格
    pub cull_transparent_mesh: bool,
    /// 是否有网格
    pub has_mesh: bool,
}

impl Default for CanvasRenderer {
    fn default() -> Self {
        Self { cull_transparent_mesh: true, has_mesh: false }
    }
}

/// Graphic 组件基类，所有可渲染的UI元素都继承自此类
#[derive(Debug, Clone, Component)]
pub struct Graphic {
    /// 颜色
    pub color: Color,
    /// 是否接收射线
    pub raycast_target: bool,
    /// 材质
    pub material: Option<Entity>,
    /// 纹理
    pub texture: Option<Entity>,
}

impl Default for Graphic {
    fn default() -> Self {
        Self { color: Color::WHITE, raycast_target: true, material: None, texture: None }
    }
}

/// Image 组件，用于显示图片
#[derive(Debug, Clone, Component)]
pub struct Image {
    /// 图片源
    pub sprite: Option<Entity>,
    /// 填充方式
    pub fill_method: ImageFillMethod,
    /// 填充原点
    pub fill_origin: u32,
    /// 填充量
    pub fill_amount: f32,
    /// 是否保持纵横比
    pub preserve_aspect: bool,
}

/// 图片填充方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFillMethod {
    /// 不填充
    None,
    /// 水平填充
    Horizontal,
    /// 垂直填充
    Vertical,
    /// 径向填充
    Radial360,
    /// 径向填充（180度）
    Radial180,
    /// 径向填充（90度）
    Radial90,
    /// 径向填充（30度）
    Radial30,
}

impl Default for Image {
    fn default() -> Self {
        Self { sprite: None, fill_method: ImageFillMethod::None, fill_origin: 0, fill_amount: 1.0, preserve_aspect: false }
    }
}

/// Text 组件，用于显示文本
#[derive(Debug, Clone, Component)]
pub struct Text {
    /// 文本内容
    pub text: String,
    /// 字体
    pub font: Option<Entity>,
    /// 字体大小
    pub font_size: f32,
    /// 字体样式
    pub font_style: FontStyle,
    /// 对齐方式
    pub alignment: TextAlignment,
    /// 水平溢出
    pub horizontal_overflow: TextOverflow,
    /// 垂直溢出
    pub vertical_overflow: TextOverflow,
    /// 行间距
    pub line_spacing: f32,
    /// 是否支持富文本
    pub rich_text: bool,
}

/// 字体样式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    /// 正常
    Normal,
    /// 粗体
    Bold,
    /// 斜体
    Italic,
    /// 粗斜体
    BoldAndItalic,
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    /// 左上角
    UpperLeft,
    /// 中上
    UpperCenter,
    /// 右上角
    UpperRight,
    /// 左中
    MiddleLeft,
    /// 中中
    MiddleCenter,
    /// 右中
    MiddleRight,
    /// 左下角
    LowerLeft,
    /// 中下
    LowerCenter,
    /// 右下角
    LowerRight,
}

/// 文本溢出方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOverflow {
    /// 截断
    Truncate,
    /// 溢出
    Overflow,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            font: None,
            font_size: 14.0,
            font_style: FontStyle::Normal,
            alignment: TextAlignment::UpperLeft,
            horizontal_overflow: TextOverflow::Truncate,
            vertical_overflow: TextOverflow::Truncate,
            line_spacing: 1.0,
            rich_text: true,
        }
    }
}

/// Button 组件，用于交互
#[derive(Debug, Clone, Component)]
pub struct Button {
    /// 过渡方式
    pub transition: ButtonTransition,
    /// 目标图形
    pub target_graphic: Option<Entity>,
    /// 高亮颜色
    pub highlight_color: Color,
    /// 按下颜色
    pub pressed_color: Color,
    /// 禁用颜色
    pub disabled_color: Color,
    /// 颜色倍增器
    pub color_multiplier: f32,
    /// 动画持续时间
    pub fade_duration: f32,
    /// 是否可交互
    pub interactable: bool,
}

/// 按钮过渡方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTransition {
    /// 颜色过渡
    ColorTint,
    /// 精灵过渡
    SpriteSwap,
    /// 动画过渡
    Animation,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            transition: ButtonTransition::ColorTint,
            target_graphic: None,
            highlight_color: Color::new(0.9f32, 0.9f32, 0.9f32, 1.0f32),
            pressed_color: Color::new(0.7f32, 0.7f32, 0.7f32, 1.0f32),
            disabled_color: Color::new(0.5f32, 0.5f32, 0.5f32, 0.5f32),
            color_multiplier: 1.0,
            fade_duration: 0.1,
            interactable: true,
        }
    }
}

/// Canvas 系统，管理Canvas的渲染
pub struct CanvasSystem {
    /// 渲染器
    renderer: Arc<dyn renderer::CanvasRenderer>,
}

impl CanvasSystem {
    /// 创建新的Canvas系统
    pub fn new(renderer: Arc<dyn renderer::CanvasRenderer>) -> Self {
        Self { renderer }
    }
}

impl gg_ecs::System for CanvasSystem {
    fn name(&self) -> &str {
        "CanvasSystem"
    }

    fn execute(&mut self, world: &mut World) -> gg_error::GResult<()> {
        self.renderer.render(world)?;
        Ok(())
    }
}

/// 创建UI元素的辅助函数
pub mod utils {
    use super::*;
    use gg_ecs::EntityBuilder;

    /// 创建Canvas
    pub fn create_canvas(world: &mut World, render_mode: CanvasRenderMode) -> Entity {
        world.spawn().insert(Canvas { render_mode, ..Default::default() }).id()
    }

    /// 创建Image
    pub fn create_image(world: &mut World, _parent: Entity, sprite: Option<Entity>) -> Entity {
        world
            .spawn()
            .insert(RectTransform::default())
            .insert(CanvasRenderer::default())
            .insert(Graphic::default())
            .insert(Image { sprite, ..Default::default() })
            .id()
    }

    /// 创建Text
    pub fn create_text(world: &mut World, _parent: Entity, text: &str) -> Entity {
        world
            .spawn()
            .insert(RectTransform::default())
            .insert(CanvasRenderer::default())
            .insert(Graphic::default())
            .insert(Text { text: text.to_string(), ..Default::default() })
            .id()
    }

    /// 创建Button
    pub fn create_button(world: &mut World, _parent: Entity, text: &str) -> Entity {
        let button_entity = world
            .spawn()
            .insert(RectTransform::default())
            .insert(CanvasRenderer::default())
            .insert(Graphic::default())
            .insert(Image::default())
            .insert(Button::default())
            .id();

        let _text_entity = create_text(world, button_entity, text);

        button_entity
    }
}
