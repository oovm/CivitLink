use crate::{
    node::{UiNodeData, UiNodeId, UiTree},
    style::Style,
    widget::Widget,
};
use gg_core::GResult;
use gg_render::TextureId;

/// 图片控件
///
/// 显示纹理图片的 UI 元素，支持设置纹理和尺寸。
pub struct Image {
    /// 纹理标识符
    texture_id: TextureId,
    /// 图片尺寸 `[width, height]`
    size: [f32; 2],
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl Image {
    /// 创建图片控件
    ///
    /// # 参数
    ///
    /// - `texture_id` - 纹理标识符
    /// - `size` - 图片尺寸 `[width, height]`
    pub fn new(texture_id: TextureId, size: [f32; 2]) -> Self {
        Self { texture_id, size, node_id: None }
    }

    /// 设置纹理标识符
    pub fn set_texture_id(&mut self, texture_id: TextureId) {
        self.texture_id = texture_id;
    }

    /// 设置图片尺寸
    pub fn set_size(&mut self, size: [f32; 2]) {
        self.size = size;
    }
}

impl Widget for Image {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let id = tree.create_node(
            "Image",
            Style::new(),
            UiNodeData::Image {
                texture_id: Some(self.texture_id),
                size: Some((self.size[0], self.size[1])),
            },
        );

        self.node_id = Some(id);
        Ok(id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(id) = self.node_id {
            if let Some(node) = tree.get_mut(id) {
                node.data = UiNodeData::Image {
                    texture_id: Some(self.texture_id),
                    size: Some((self.size[0], self.size[1])),
                };
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
