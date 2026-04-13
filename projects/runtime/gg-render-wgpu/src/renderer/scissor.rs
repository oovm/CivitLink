/// 裁剪矩形
///
/// 用于渲染通道中的像素裁剪区域。
pub(crate) struct ScissorRect {
    /// x 坐标
    pub x: u32,
    /// y 坐标
    pub y: u32,
    /// 宽度
    pub w: u32,
    /// 高度
    pub h: u32,
}

/// 带索引的绘制命令，用于 Z 排序时保持原始顺序
pub(crate) struct IndexedCommand {
    /// 原始命令索引
    pub index: usize,
    /// Z 层级值
    pub z_index: f32,
    /// 是否为过渡命令（过渡命令始终最后渲染）
    pub is_transition: bool,
}

/// 圆角矩形渲染项
///
/// 包含圆角矩形绘制所需的 uniform 绑定组和缓冲区。
pub(crate) struct RoundedRectRenderItem {
    /// uniform 绑定组
    pub uniform_bind_group: wgpu::BindGroup,
    /// uniform 缓冲区（用于帧间复用）
    pub uniform_buffer: wgpu::Buffer,
}

/// 椭圆渲染项
///
/// 包含椭圆绘制所需的 uniform 绑定组和缓冲区。
pub(crate) struct EllipseRenderItem {
    /// uniform 绑定组
    pub uniform_bind_group: wgpu::BindGroup,
    /// uniform 缓冲区（用于帧间复用）
    pub uniform_buffer: wgpu::Buffer,
}
