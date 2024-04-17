//! 立绘 Z 轴排序模块
//! 提供立绘的 Z 轴排序计算和按位置排序功能

use gg_ecs::Entity;
use gg_galgame_schema::components::{PortraitPosition, PortraitState};

/// 计算 Z 轴排序值
///
/// 根据立绘在场景中的索引和总数计算 Z 轴排序值。
/// 索引越大，Z 值越大，立绘越靠前显示。
///
/// # 参数
///
/// - `index` - 立绘在场景中的索引
/// - `total` - 场景中立绘总数
///
/// # 返回
///
/// Z 轴排序值，值越大越靠前
pub fn calculate_z_order(index: usize, total: usize) -> i32 {
    if total <= 1 {
        return 0;
    }
    index as i32
}

/// 获取位置枚举的排序权重
///
/// 位置排序优先级：`Left` < `Center` < `Right` < `Custom`。
fn position_weight(position: &PortraitPosition) -> u8 {
    match position {
        PortraitPosition::Left => 0,
        PortraitPosition::Center => 1,
        PortraitPosition::Right => 2,
        PortraitPosition::Custom { .. } => 3,
    }
}

/// 按位置对立绘列表排序
///
/// 将立绘按照位置从左到右排序（`Left` < `Center` < `Right` < `Custom`）。
/// 排序结果直接影响立绘的渲染顺序。
///
/// # 参数
///
/// - `portraits` - 待排序的立绘列表，包含实体 ID 和立绘状态
pub fn sort_portraits_by_position(portraits: &mut [(Entity, PortraitState)]) {
    portraits.sort_by_key(|(_, state)| position_weight(&state.position));
}
