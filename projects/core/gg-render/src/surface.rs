/// 渲染表面信息
///
/// 描述渲染目标窗口的属性。
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceInfo {
    /// 表面宽度（像素）
    pub width: u32,
    /// 表面高度（像素）
    pub height: u32,
    /// 窗口标题
    pub title: String,
    /// 是否全屏
    pub fullscreen: bool,
}

impl SurfaceInfo {
    /// 创建新的渲染表面信息
    ///
    /// # 参数
    ///
    /// - `width` - 表面宽度（像素）
    /// - `height` - 表面高度（像素）
    /// - `title` - 窗口标题
    pub fn new(width: u32, height: u32, title: impl Into<String>) -> Self {
        Self {
            width,
            height,
            title: title.into(),
            fullscreen: false,
        }
    }

    /// 设置是否全屏
    ///
    /// # 参数
    ///
    /// - `fullscreen` - 是否全屏
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }
}

/// 窗口事件
///
/// 描述从窗口系统接收到的事件。
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// 窗口大小改变
    Resized {
        /// 新的宽度（像素）
        width: u32,
        /// 新的高度（像素）
        height: u32,
    },
    /// 窗口关闭请求
    CloseRequested,
    /// 窗口焦点变化
    Focused(bool),
    /// 缩放因子变化
    ScaleFactorChanged(f64),
}
