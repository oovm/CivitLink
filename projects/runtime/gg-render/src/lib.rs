#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

/// 颜色类型
pub mod color;
/// 绘制命令
pub mod command;
/// 渲染管线与渲染通道
pub mod pipeline;
/// 矩形区域
pub mod rect;
/// 渲染器 trait 与渲染上下文
pub mod renderer;
/// 渲染表面与窗口事件
pub mod surface;
/// 纹理标识与描述
pub mod texture;
/// 二维变换
pub mod transform;

pub use color::Color;
pub use command::{DrawCommand, TransitionKind};
pub use pipeline::{DefaultRenderPipeline, DrawFilter, RenderPass, RenderPipeline, RenderTarget};
pub use rect::Rect;
pub use renderer::{Camera, Camera2D, RenderContext, Renderer, Viewport};
pub use surface::{SurfaceInfo, WindowEvent};
pub use texture::{PixelFormat, TextureDescriptor, TextureId};
pub use transform::Transform;

/// 预导入模块
///
/// 包含 gg-render 中最常用的类型，方便一次性导入。
pub mod prelude {
    pub use crate::{
        color::Color,
        command::{DrawCommand, TransitionKind},
        pipeline::{DefaultRenderPipeline, DrawFilter, RenderPass, RenderPipeline, RenderTarget},
        rect::Rect,
        renderer::{Camera, Camera2D, RenderContext, Renderer, Viewport},
        surface::{SurfaceInfo, WindowEvent},
        texture::{PixelFormat, TextureDescriptor, TextureId},
        transform::Transform,
    };
}
