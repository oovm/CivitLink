use std::path::Path;

use gg_core::{GError, GErrorKind, GResult};

/// 字体管理器
///
/// 管理字体数据的加载和字体族名称的存储。
/// 字体数据以原始字节形式保存，供渲染时通过 piet 的 RenderContext 创建文本布局使用。
pub struct FontManager {
    /// 当前字体族名称
    font_family: Option<String>,
    /// 字体原始字节数据
    font_data: Option<Vec<u8>>,
}

impl FontManager {
    /// 创建新的字体管理器
    ///
    /// 返回一个未加载任何字体的空字体管理器实例。
    pub fn new() -> Self {
        Self {
            font_family: None,
            font_data: None,
        }
    }

    /// 从字节数据加载字体
    ///
    /// 将字体原始字节数据保存到管理器中，并尝试解析字体族名称。
    /// 使用 piet_common 的 FontFamily 来指定字体族。
    ///
    /// # 参数
    ///
    /// - `data` - 字体文件的原始字节数据（如 TTF、OTF 格式）
    pub fn load_font(&mut self, data: Vec<u8>) -> GResult<()> {
        let font = ab_glyph::FontArc::try_from_vec(data.clone()).map_err(|e| GError {
            kind: GErrorKind::Asset,
            message: format!("无法解析字体数据: {}", e),
        })?;

        let family_name = font.family_name();
        self.font_family = Some(family_name);
        self.font_data = Some(data);

        Ok(())
    }

    /// 从文件路径加载字体
    ///
    /// 读取指定路径的字体文件，并将字节数据加载到管理器中。
    ///
    /// # 参数
    ///
    /// - `path` - 字体文件路径
    pub fn load_font_from_path(&mut self, path: &Path) -> GResult<()> {
        let data = std::fs::read(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("无法读取字体文件 '{}': {}", path.display(), e),
        })?;

        self.load_font(data)
    }

    /// 获取当前字体族名称
    ///
    /// 返回当前加载字体的族名称引用。
    /// 若未加载任何字体，返回 `None`。
    pub fn font_family(&self) -> Option<&str> {
        self.font_family.as_deref()
    }

    /// 检查是否已加载字体
    ///
    /// 当管理器中存在字体数据时返回 `true`，否则返回 `false`。
    pub fn has_font(&self) -> bool {
        self.font_data.is_some()
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}
