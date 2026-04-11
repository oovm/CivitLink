use gg_render::{Color, DrawCommand, Rect, RenderContext, Transform};

use crate::node::{UiNodeData, UiNodeId, UiTree};
use crate::style::Overflow;

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

        match &node.data {
            UiNodeData::Text { content } => {
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
            UiNodeData::Image { texture_id, size } => {
                if let Some(tid) = texture_id {
                    let (img_w, img_h) = size.unwrap_or((layout.width, layout.height));
                    context.draw(DrawCommand::Sprite {
                        texture_id: *tid,
                        transform: Transform::with_position([abs_x, abs_y]),
                        size: [img_w, img_h],
                        tint: Color::WHITE,
                        clip_rect: context.clip_rect().copied(),
                    });
                }
            }
            UiNodeData::Custom { .. } | UiNodeData::Container => {
                // 对于 Custom 和 Container 类型的节点，只渲染背景和边框，然后继续渲染子节点
            }
        }

        let saved_clip = if node.style.overflow == Overflow::Clip {
            let node_rect = Rect::new(abs_x, abs_y, layout.width, layout.height);
            let new_clip = match context.clip_rect() {
                Some(existing) => Self::intersect_rects(existing, &node_rect),
                None => node_rect,
            };
            let saved = context.clip_rect().copied();
            context.set_clip_rect(new_clip);
            saved
        } else {
            None
        };

        let children = node.children.clone();
        for &child_id in &children {
            Self::render_node(tree, child_id, context, abs_x, abs_y);
        }

        if node.style.overflow == Overflow::Clip {
            match saved_clip {
                Some(rect) => context.set_clip_rect(rect),
                None => context.clear_clip_rect(),
            }
        }
    }

    /// 计算两个矩形的交集
    ///
    /// 返回两个矩形重叠的区域。如果无交集，返回零尺寸矩形。
    fn intersect_rects(a: &Rect, b: &Rect) -> Rect {
        let x = a.x.max(b.x);
        let y = a.y.max(b.y);
        let right = (a.x + a.width).min(b.x + b.width);
        let bottom = (a.y + a.height).min(b.y + b.height);
        Rect::new(x, y, (right - x).max(0.0), (bottom - y).max(0.0))
    }
}
