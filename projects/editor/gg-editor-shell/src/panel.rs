//! 编辑器面板 trait 和注册机制

use gg_core::GResult;
use gg_ecs::{Entity, World};

/// 面板共享数据
pub struct PanelData {
    /// World 的原始指针
    world_ptr: *mut World,
    /// 当前选中的实体
    selected_entity: Option<Entity>,
    /// 项目路径
    project_path: Option<String>,
}

impl PanelData {
    /// 创建新的面板数据
    pub fn new() -> Self {
        Self {
            world_ptr: std::ptr::null_mut(),
            selected_entity: None,
            project_path: None,
        }
    }

    /// 设置 World 引用
    pub fn set_world(&mut self, world: &mut World) {
        self.world_ptr = world as *mut World;
    }

    /// 获取 World 的可变引用
    ///
    /// # Safety
    ///
    /// 调用者必须确保在获取引用期间没有其他可变引用指向同一 World。
    pub fn world(&mut self) -> Option<&mut World> {
        if self.world_ptr.is_null() {
            None
        } else {
            unsafe { Some(&mut *self.world_ptr) }
        }
    }

    /// 获取选中实体
    pub fn selected_entity(&self) -> Option<Entity> {
        self.selected_entity
    }

    /// 设置选中实体
    pub fn set_selected_entity(&mut self, entity: Option<Entity>) {
        self.selected_entity = entity;
    }

    /// 获取项目路径
    pub fn project_path(&self) -> Option<&str> {
        self.project_path.as_deref()
    }

    /// 设置项目路径
    pub fn set_project_path(&mut self, path: Option<String>) {
        self.project_path = path;
    }
}

impl Default for PanelData {
    fn default() -> Self {
        Self::new()
    }
}

/// 面板渲染上下文
pub struct PanelContext<'a> {
    /// 面板共享数据
    pub panel_data: &'a mut PanelData,
}

/// 编辑器面板 trait
pub trait EditorPanel {
    /// 面板名称
    fn name(&self) -> &str;

    /// 是否可见
    fn is_visible(&self) -> bool;

    /// 设置可见性
    fn set_visible(&mut self, visible: bool);

    /// 渲染面板
    fn render(&mut self, context: &mut PanelContext) -> GResult<()>;
}
