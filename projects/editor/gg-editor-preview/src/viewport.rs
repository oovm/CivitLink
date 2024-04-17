//! 视口渲染器模块
//! 提供将游戏引擎渲染输出到纹理的功能，以及视口输入事件转发

use gg_core::GResult;
use gg_galgame::engine::GalgameEngine;
use gg_render::{TextureId, WgpuRenderer};

/// 视口输入事件
///
/// 描述从预览面板视口转发到游戏引擎的输入事件类型。
#[derive(Debug, Clone)]
pub enum ViewportInputEvent {
    /// 鼠标移动事件
    MouseMove {
        /// 屏幕 X 坐标
        x: f32,
        /// 屏幕 Y 坐标
        y: f32,
    },
    /// 鼠标按钮事件
    MouseButton {
        /// 按钮索引（0=左键，1=中键，2=右键）
        button: u32,
        /// 是否按下
        pressed: bool,
        /// 屏幕 X 坐标
        x: f32,
        /// 屏幕 Y 坐标
        y: f32,
    },
    /// 键盘事件
    Key {
        /// 按键代码
        key_code: u32,
        /// 是否按下
        pressed: bool,
    },
    /// 滚轮事件
    Scroll {
        /// 水平滚动量
        delta_x: f32,
        /// 垂直滚动量
        delta_y: f32,
    },
}

/// 视口渲染器
///
/// 负责将游戏引擎的渲染输出捕获到纹理，
/// 并在预览面板的视口区域中显示。
/// 同时管理视口大小调整和输入事件转发。
pub struct ViewportRenderer {
    /// 渲染纹理标识
    render_texture_id: Option<TextureId>,
    /// 视口宽度（像素）
    width: u32,
    /// 视口高度（像素）
    height: u32,
    /// 是否需要重新创建纹理
    needs_resize: bool,
}

impl ViewportRenderer {
    /// 创建新的视口渲染器
    ///
    /// 使用指定的初始宽高创建渲染器。
    /// 初始状态下没有渲染纹理，需要在首次渲染时创建。
    pub fn new(width: u32, height: u32) -> Self {
        Self { render_texture_id: None, width, height, needs_resize: false }
    }

    /// 获取当前渲染纹理标识
    ///
    /// 如果尚未创建纹理或纹理需要重建，返回 None。
    pub fn render_texture_id(&self) -> Option<TextureId> {
        self.render_texture_id
    }

    /// 调整视口大小
    ///
    /// 标记需要重新创建渲染纹理，实际创建在下次 ensure_texture 时执行。
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.needs_resize = true;
        }
    }

    /// 确保渲染纹理已创建
    ///
    /// 如果纹理不存在或需要重建，则创建新的渲染纹理。
    /// 返回创建或已有的纹理标识。
    pub fn ensure_texture(&mut self, renderer: &mut WgpuRenderer) -> GResult<TextureId> {
        if self.render_texture_id.is_none() || self.needs_resize {
            let texture_id = renderer.create_render_texture(self.width, self.height)?;
            self.render_texture_id = Some(texture_id);
            self.needs_resize = false;
        }
        Ok(self.render_texture_id.unwrap())
    }

    /// 将游戏引擎渲染输出到纹理
    ///
    /// 调用游戏引擎的渲染方法，将输出捕获到渲染纹理中。
    /// 如果纹理不存在或需要重建，先创建纹理。
    /// 返回渲染后的纹理标识。
    pub fn render_engine_to_texture(&mut self, engine: &mut GalgameEngine, renderer: &mut WgpuRenderer) -> GResult<TextureId> {
        let texture_id = self.ensure_texture(renderer)?;
        engine.render_to_texture(texture_id)?;
        Ok(texture_id)
    }

    /// 将输入事件转发给游戏引擎
    ///
    /// 将视口中的用户输入事件传递给游戏引擎实例，
    /// 使游戏能够响应预览面板中的交互操作。
    pub fn forward_input_event(&self, event: &ViewportInputEvent, engine: &mut GalgameEngine) -> GResult<()> {
        match event {
            ViewportInputEvent::MouseMove { x, y } => {
                engine.input_mouse_move(*x, *y)?;
            }
            ViewportInputEvent::MouseButton { button, pressed, x, y } => {
                engine.input_mouse_button(*button, *pressed, *x, *y)?;
            }
            ViewportInputEvent::Key { key_code, pressed } => {
                engine.input_key(*key_code, *pressed)?;
            }
            ViewportInputEvent::Scroll { delta_x, delta_y } => {
                engine.input_scroll(*delta_x, *delta_y)?;
            }
        }
        Ok(())
    }

    /// 获取视口宽度
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取视口高度
    pub fn height(&self) -> u32 {
        self.height
    }
}
