use std::collections::HashMap;

use gg_core::GResult;
use gg_ecs::System;
use gg_render::RenderContext;
use gg_ui::{LayoutCache, LayoutEngine, UiNodeId, UiTree};

use super::UiTreeResource;

/// 脏标记资源
///
/// 跟踪哪些 UI 节点需要重新计算布局。
/// 当节点或其祖先被标记为脏时，该节点的子树将在下一帧重新布局。
#[derive(Debug, Clone, Default)]
pub struct DirtyFlags {
    /// 节点到脏标记的映射，true 表示需要重新布局
    pub flags: HashMap<u64, bool>,
}

impl DirtyFlags {
    /// 创建新的脏标记资源
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查指定节点是否被标记为脏
    pub fn is_dirty(&self, node_id: UiNodeId) -> bool {
        self.flags.get(&node_id).copied().unwrap_or(false)
    }

    /// 标记指定节点为脏
    pub fn mark(&mut self, node_id: UiNodeId) {
        self.flags.insert(node_id, true);
    }

    /// 清除指定节点的脏标记
    pub fn clear(&mut self, node_id: UiNodeId) {
        self.flags.insert(node_id, false);
    }

    /// 清除所有脏标记
    pub fn clear_all(&mut self) {
        for flag in self.flags.values_mut() {
            *flag = false;
        }
    }

    /// 检查是否存在任何脏节点
    pub fn has_dirty(&self) -> bool {
        self.flags.values().any(|&d| d)
    }

    /// 获取所有脏节点 ID 列表
    pub fn dirty_nodes(&self) -> Vec<UiNodeId> {
        self.flags.iter().filter_map(|(&id, &dirty)| if dirty { Some(id) } else { None }).collect()
    }
}

/// 标记节点及其所有祖先为脏
///
/// 当节点样式或尺寸发生变化时，需要从该节点向上到根节点
/// 全部标记为脏，以确保布局正确传播。
pub fn mark_dirty(tree: &UiTree, node_id: UiNodeId, flags: &mut DirtyFlags) {
    flags.mark(node_id);

    let mut current = node_id;
    while let Some(node) = tree.get(current) {
        if let Some(parent_id) = node.parent {
            flags.mark(parent_id);
            current = parent_id;
        }
        else {
            break;
        }
    }
}

/// 布局缓存资源
///
/// 将 gg-ui 的 LayoutCache 包装为 ECS 全局资源，
/// 以便通过 World 的资源系统进行存取。
pub struct LayoutCacheResource(pub LayoutCache);

impl LayoutCacheResource {
    /// 创建新的布局缓存资源
    pub fn new() -> Self {
        Self(LayoutCache::new())
    }
}

impl Default for LayoutCacheResource {
    fn default() -> Self {
        Self::new()
    }
}

/// UI 布局系统
///
/// 基于脏标记驱动的增量布局更新系统，并集成布局缓存。
/// 仅对被标记为脏的子树执行布局计算，利用缓存跳过未变化的节点。
pub struct UiLayoutSystem;

impl System for UiLayoutSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_layout"
    }

    /// 执行增量布局系统逻辑
    ///
    /// 检查脏标记资源，对脏节点执行增量布局：
    /// - 若根节点为脏，执行全树布局计算（带缓存）
    /// - 否则，对每个脏子树执行局部布局计算（带缓存）
    /// - 计算完成后清除所有脏标记
    fn execute(&mut self, world: &mut gg_ecs::World) -> GResult<()> {
        let dirty_flags = world.get_resource::<DirtyFlags>().cloned();
        let dirty_flags = match dirty_flags {
            Some(f) => f,
            None => return Ok(()),
        };

        if !dirty_flags.has_dirty() {
            return Ok(());
        }

        let (width, height) = world
            .get_resource::<RenderContext>()
            .map(|ctx| (ctx.surface_width() as f32, ctx.surface_height() as f32))
            .unwrap_or((800.0, 600.0));

        let root_id = world.get_resource::<UiTreeResource>().and_then(|r| r.0.root());

        let root_is_dirty = root_id.map_or(false, |id| dirty_flags.is_dirty(id));

        if root_is_dirty {
            let tree_opt = world.get_resource::<UiTreeResource>().map(|r| r.0.clone());
            if let Some(mut tree) = tree_opt {
                if let Some(mut cache_res) = world.get_resource_mut::<LayoutCacheResource>() {
                    LayoutEngine::compute_cached(&mut tree, width, height, &mut cache_res.0);
                }
                else {
                    LayoutEngine::compute(&mut tree, width, height);
                }
                if let Some(world_tree) = world.get_resource_mut::<UiTreeResource>() {
                    world_tree.0 = tree;
                }
            }
        }
        else {
            let dirty_nodes = dirty_flags.dirty_nodes();
            for node_id in dirty_nodes {
                let tree_opt = world.get_resource::<UiTreeResource>().map(|r| r.0.clone());
                if let Some(mut tree) = tree_opt {
                    if let Some(mut cache_res) = world.get_resource_mut::<LayoutCacheResource>() {
                        LayoutEngine::compute_subtree_cached(&mut tree, node_id, width, height, &mut cache_res.0);
                    }
                    else {
                        LayoutEngine::compute_subtree(&mut tree, node_id, width, height);
                    }
                    if let Some(world_tree) = world.get_resource_mut::<UiTreeResource>() {
                        world_tree.0 = tree;
                    }
                }
            }
        }

        if let Some(mut flags) = world.get_resource_mut::<DirtyFlags>() {
            flags.clear_all();
        }

        Ok(())
    }
}
