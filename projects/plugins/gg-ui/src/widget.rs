use crate::node::{UiNodeId, UiTree};

/// 控件 trait
///
/// 所有 UI 控件必须实现此 trait，定义控件的构建、更新和查询接口。
pub trait Widget {
    /// 构建控件节点树，返回根节点 ID
    fn build(&self, tree: &mut UiTree) -> UiNodeId;

    /// 更新控件状态
    fn update(&self, tree: &mut UiTree);

    /// 获取控件根节点 ID
    fn node_id(&self) -> Option<UiNodeId>;
}
