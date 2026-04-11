use crate::{Color, Rect, TextureId, Transform};

/// 过渡动画类型
///
/// 定义场景切换时的过渡动画效果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionKind {
    /// 淡入淡出
    Fade,
    /// 交叉溶解
    CrossDissolve,
    /// 从右向左滑入
    SlideLeft,
    /// 从左向右滑入
    SlideRight,
    /// 从下向上滑入
    SlideUp,
    /// 从上向下滑入
    SlideDown,
}

/// 绘制命令
///
/// 描述一次绘制操作的类型和参数。
/// 渲染器会按照命令的顺序依次执行绘制。
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    /// 精灵绘制
    Sprite {
        /// 纹理标识符
        texture_id: TextureId,
        /// 变换信息
        transform: Transform,
        /// 精灵尺寸 `[width, height]`
        size: [f32; 2],
        /// 着色颜色
        tint: Color,
        /// 裁剪矩形
        clip_rect: Option<Rect>,
    },
    /// 文本绘制
    Text {
        /// 文本内容
        text: String,
        /// 文本位置 `[x, y]`
        position: [f32; 2],
        /// 字体大小
        font_size: f32,
        /// 文本颜色
        color: Color,
        /// 最大行宽限制
        max_width: Option<f32>,
    },
    /// 矩形绘制
    Rect {
        /// 矩形区域
        rect: Rect,
        /// 填充颜色
        color: Color,
        /// 圆角半径
        corner_radius: f32,
    },
    /// 线段绘制
    Line {
        /// 起点 `[x, y]`
        start: [f32; 2],
        /// 终点 `[x, y]`
        end: [f32; 2],
        /// 线段颜色
        color: Color,
        /// 线段宽度（像素）
        width: f32,
    },
    /// 圆形绘制
    Circle {
        /// 圆心 `[x, y]`
        center: [f32; 2],
        /// 半径（像素）
        radius: f32,
        /// 填充颜色
        color: Color,
        /// 是否填充；`true` 为填充，`false` 为描边
        filled: bool,
        /// 描边宽度（像素）；`0.0` 表示不描边，`>0` 表示描边宽度
        border_width: f32,
        /// 描边颜色 RGBA `[r, g, b, a]`
        border_color: [f32; 4],
    },
    /// 椭圆绘制
    Ellipse {
        /// 中心 `[x, y]`
        center: [f32; 2],
        /// 半轴 `[rx, ry]`
        radii: [f32; 2],
        /// 填充颜色
        color: Color,
        /// 是否填充；`true` 为填充，`false` 为描边
        filled: bool,
        /// 描边宽度（像素）；`0.0` 表示不描边，`>0` 表示描边宽度
        border_width: f32,
        /// 描边颜色 RGBA `[r, g, b, a]`
        border_color: [f32; 4],
    },
    /// 过渡动画
    Transition {
        /// 旧纹理
        old_texture: Option<TextureId>,
        /// 新纹理
        new_texture: Option<TextureId>,
        /// 过渡进度 `[0.0, 1.0]`
        progress: f32,
        /// 过渡类型
        kind: TransitionKind,
    },
}
