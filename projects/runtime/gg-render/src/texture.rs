/// 像素格式
///
/// 定义纹理数据的像素排列方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGBA 8位无符号归一化 sRGB 格式
    Rgba8UnormSrgb,
    /// BGRA 8位无符号归一化 sRGB 格式
    Bgra8UnormSrgb,
    /// R 8位无符号归一化格式
    R8Unorm,
}

/// 纹理标识符
///
/// 用于在渲染器中唯一标识一个已加载的纹理资源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u64);

impl Default for TextureId {
    /// 默认值为 `TextureId::INVALID`
    fn default() -> Self {
        Self::INVALID
    }
}

impl TextureId {
    /// 创建一个新的纹理标识符
    ///
    /// # 参数
    ///
    /// - `id` - 纹理的唯一标识值
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取纹理标识符的原始值
    pub fn id(&self) -> u64 {
        self.0
    }

    /// 无效纹理标识符
    ///
    /// 值为 `0`，表示未加载或无效的纹理。
    pub const INVALID: Self = Self(0);
}

/// 纹理描述符
///
/// 描述纹理的尺寸和像素格式等属性。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureDescriptor {
    /// 纹理宽度（像素）
    pub width: u32,
    /// 纹理高度（像素）
    pub height: u32,
    /// 像素格式
    pub format: PixelFormat,
}

impl TextureDescriptor {
    /// 创建一个新的纹理描述符
    ///
    /// # 参数
    ///
    /// - `width` - 纹理宽度（像素）
    /// - `height` - 纹理高度（像素）
    /// - `format` - 像素格式
    pub fn new(width: u32, height: u32, format: PixelFormat) -> Self {
        Self { width, height, format }
    }
}
