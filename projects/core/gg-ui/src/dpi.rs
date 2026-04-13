/// 封装逻辑像素到物理像素的缩放因子
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DpiScale {
    /// 缩放因子
    scale_factor: f32,
}

impl DpiScale {
    /// 创建 DPI 缩放实例，scale_factor 必须 > 0.0，否则使用 1.0
    pub fn new(scale_factor: f32) -> Self {
        Self { scale_factor: if scale_factor > 0.0 { scale_factor } else { 1.0 } }
    }

    /// 返回 1.0 缩放（无 DPI 缩放）
    pub fn identity() -> Self {
        Self { scale_factor: 1.0 }
    }

    /// 逻辑像素转物理像素
    pub fn logical_to_physical(&self, logical: f32) -> f32 {
        logical * self.scale_factor
    }

    /// 物理像素转逻辑像素
    pub fn physical_to_logical(&self, physical: f32) -> f32 {
        physical / self.scale_factor
    }

    /// 获取缩放因子
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// 缩放尺寸
    pub fn scale_size(&self, width: f32, height: f32) -> (f32, f32) {
        (width * self.scale_factor, height * self.scale_factor)
    }
}

impl Default for DpiScale {
    fn default() -> Self {
        Self::identity()
    }
}

/// DPI 感知接口
pub trait DpiAware {
    /// 设置 DPI 缩放因子
    fn set_dpi_scale(&mut self, scale: DpiScale);

    /// 获取当前 DPI 缩放因子
    fn dpi_scale(&self) -> &DpiScale;
}
