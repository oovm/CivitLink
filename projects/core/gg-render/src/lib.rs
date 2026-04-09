#![warn(missing_docs)]

//! GG 引擎渲染硬件抽象层
//!
//! 提供与具体图形 API 无关的渲染抽象接口，
//! 支持精灵绘制、文本渲染、矩形绘制和场景过渡动画。
//!
//! # 模块结构
//!
//! - [`color`] - 颜色类型
//! - [`command`] - 绘制命令
//! - [`surface`] - 渲染表面与窗口事件
//! - [`texture`] - 纹理标识与描述
//! - [`renderer`] - 渲染器 trait 与渲染上下文
//! - [`transform`] - 二维变换
//! - [`rect`] - 矩形区域

/// 颜色类型
pub mod color;
/// 绘制命令
pub mod command;
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
pub use rect::Rect;
pub use renderer::{RenderContext, Renderer};
pub use surface::{SurfaceInfo, WindowEvent};
pub use texture::{PixelFormat, TextureDescriptor, TextureId};
pub use transform::Transform;

/// 预导入模块
///
/// 包含 gg-render 中最常用的类型，方便一次性导入。
pub mod prelude {
    pub use crate::color::Color;
    pub use crate::command::{DrawCommand, TransitionKind};
    pub use crate::rect::Rect;
    pub use crate::renderer::{RenderContext, Renderer};
    pub use crate::surface::{SurfaceInfo, WindowEvent};
    pub use crate::texture::{PixelFormat, TextureDescriptor, TextureId};
    pub use crate::transform::Transform;
}
