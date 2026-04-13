use std::collections::{HashMap, HashSet};

use super::types::ComponentRegistry;

impl ComponentRegistry {
    /// 创建空的组件注册表
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            builtins: HashSet::new(),
        }
    }

    /// 创建包含所有内置组件的注册表
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register_builtins();
        registry
    }

    /// 注册一个组件
    pub fn register(&mut self, schema: super::types::ComponentSchema) {
        self.components.insert(schema.type_name.clone(), schema);
    }

    /// 查询指定名称的组件
    pub fn lookup(&self, type_name: &str) -> Option<&super::types::ComponentSchema> {
        self.components.get(type_name)
    }

    /// 注册所有内置组件
    pub fn register_builtins(&mut self) {
        self.register_layout();
        self.register_stack();
        self.register_button();
        self.register_text();
        self.register_input();
        self.register_image();
        self.register_panel();
        self.register_scroll_view();
        self.register_editor_components();
        self.register_widget_components();
    }

    /// 注册编辑器专用组件
    pub(super) fn register_editor_components(&mut self) {
        self.register_inspector_panel();
        self.register_hierarchy_view();
        self.register_asset_browser();
        self.register_scene_view();
        self.register_toolbar();
        self.register_menu_bar();
        self.register_tab_container();
        self.register_split_view();
        self.register_tree_view();
        self.register_property_field();
    }

    /// 注册控件组件
    pub(super) fn register_widget_components(&mut self) {
        self.register_checkbox();
        self.register_combo_box();
        self.register_context_menu();
        self.register_dialog();
        self.register_dropdown();
        self.register_progress_bar();
        self.register_slider();
        self.register_spinner();
        self.register_tab_bar();
        self.register_text_box();
        self.register_tooltip();
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
