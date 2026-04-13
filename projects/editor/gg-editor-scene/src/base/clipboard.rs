//! 剪贴板实体数据

use crate::components::{RectRenderer, SpriteRenderer, Transform2D};
use super::types::SceneEntityKind;

/// 实体快照
///
/// 存储实体的组件数据，用于删除实体后恢复。
/// 包含 `Transform2D` 和可选的渲染器组件数据。
#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    /// 变换组件数据
    pub transform: Transform2D,
    /// 矩形渲染器组件数据（如果存在）
    pub rect_renderer: Option<RectRenderer>,
    /// 精灵渲染器组件数据（如果存在）
    pub sprite_renderer: Option<SpriteRenderer>,
}

/// 剪贴板实体数据
///
/// 存储复制到剪贴板的实体组件信息，用于粘贴时创建新实体。
#[derive(Debug, Clone)]
pub struct ClipboardEntity {
    /// 实体的变换组件数据
    pub transform: Transform2D,
    /// 矩形渲染器组件数据（如果存在）
    pub rect_renderer: Option<RectRenderer>,
    /// 精灵渲染器组件数据（如果存在）
    pub sprite_renderer: Option<SpriteRenderer>,
    /// 实体类型
    pub kind: SceneEntityKind,
}
