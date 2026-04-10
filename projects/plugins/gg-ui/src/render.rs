use gg_render::{DrawCommand, Rect, RenderContext};

use crate::node::{UiNodeData, UiNodeId, UiTree};

/// UI 渲染器
///
/// 将 UI 节点树转换为绘制命令序列，供渲染器执行。
pub struct UiRenderer;

impl UiRenderer {
    /// 将 UI 树转换为 DrawCommand 序列
    ///
    /// 遍历 UI 树中所有可见节点，根据其样式和数据生成对应的绘制命令。
    pub fn render(tree: &UiTree, context: &mut RenderContext) {
        if let Some(root_id) = tree.root() {
            Self::render_node(tree, root_id, context, 0.0, 0.0);
        }
    }

    /// 渲染单个节点（递归）
    fn render_node(tree: &UiTree, node_id: UiNodeId, context: &mut RenderContext, offset_x: f32, offset_y: f32) {
        let node = tree.get(node_id);
        let node = match node {
            Some(n) => n,
            None => return,
        };

        if !node.visible {
            return;
        }

        let layout = match node.layout_result {
            Some(l) => l,
            None => return,
        };

        let abs_x = layout.x + offset_x;
        let abs_y = layout.y + offset_y;

        if let Some(bg_color) = node.style.background_color {
            context.draw(DrawCommand::Rect {
                rect: Rect::new(abs_x, abs_y, layout.width, layout.height),
                color: bg_color,
                corner_radius: node.style.corner_radius,
            });
        }

        if let Some(ref border_color) = node.style.border_color {
            if node.style.border_width > 0.0 {
                context.draw(DrawCommand::Rect {
                    rect: Rect::new(abs_x, abs_y, layout.width, layout.height),
                    color: *border_color,
                    corner_radius: node.style.corner_radius,
                });
            }
        }

        if let UiNodeData::Text { ref content } = node.data {
            if let Some(ref font) = node.style.font {
                context.draw(DrawCommand::Text {
                    text: content.clone(),
                    position: [abs_x + node.style.layout.padding, abs_y + node.style.layout.padding],
                    font_size: font.size,
                    color: font.color,
                    max_width: if layout.width > 0.0 { Some(layout.width - 2.0 * node.style.layout.padding) } else { None },
                });
            }
        }

        let children = node.children.clone();
        for &child_id in &children {
            Self::render_node(tree, child_id, context, offset_x, offset_y);
        }
    }
}
