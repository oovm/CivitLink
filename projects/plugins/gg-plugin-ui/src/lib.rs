//! GG Editor UI 插件

use gg_ecs::{Resource, World};
use gg_runtime_gui::GuiRuntime;
use std::sync::Arc;

/// UI 插件
pub struct UiPlugin {
    gui_runtime: Option<Arc<GuiRuntime>>,
}

impl Resource for UiPlugin {}

impl UiPlugin {
    /// 创建新的 UI 插件
    pub fn new() -> Self {
        Self {
            gui_runtime: None,
        }
    }
    
    /// 设置 GUI 运行时
    pub fn set_gui_runtime(&mut self, gui_runtime: Arc<GuiRuntime>) {
        self.gui_runtime = Some(gui_runtime);
    }
    
    /// 获取 GUI 运行时
    pub fn get_gui_runtime(&self) -> Option<Arc<GuiRuntime>> {
        self.gui_runtime.clone()
    }
    
    /// 更新 UI
    pub fn update(&mut self) {
        if let Some(runtime) = &mut self.gui_runtime {
            // 这里需要可变引用，实际实现可能需要更复杂的处理
        }
    }
}

/// ECS 系统：UI 更新系统
pub fn ui_update_system(world: &mut World) {
    // 获取 UI 插件
    let mut ui_plugin = world.get_resource_mut::<UiPlugin>().unwrap();
    
    // 更新 UI
    ui_plugin.update();
}

/// ECS 系统：UI 渲染系统
pub fn ui_render_system(world: &mut World) {
    // 获取 UI 插件
    let ui_plugin = world.get_resource::<UiPlugin>().unwrap();
    
    // 渲染 UI
    if let Some(runtime) = ui_plugin.get_gui_runtime() {
        // 这里需要调用渲染方法
    }
}

/// 扩展 World 以支持 UI
pub trait WorldUiExt {
    /// 初始化 UI 系统
    fn init_ui<T: gg_runtime_gui::GuiRenderer + 'static>(&mut self, renderer: T);
    
    /// 获取 UI 插件
    fn get_ui_plugin(&self) -> Option<&UiPlugin>;
    
    /// 获取可变 UI 插件
    fn get_ui_plugin_mut(&mut self) -> Option<&mut UiPlugin>;
}

impl WorldUiExt for World {
    fn init_ui<T: gg_runtime_gui::GuiRenderer + 'static>(&mut self, renderer: T) {
        let gui_runtime = GuiRuntime::new(renderer);
        let mut ui_plugin = UiPlugin::new();
        ui_plugin.set_gui_runtime(Arc::new(gui_runtime));
        self.insert_resource(ui_plugin);
    }
    
    fn get_ui_plugin(&self) -> Option<&UiPlugin> {
        self.get_resource::<UiPlugin>()
    }
    
    fn get_ui_plugin_mut(&mut self) -> Option<&mut UiPlugin> {
        self.get_resource_mut::<UiPlugin>()
    }
}

/// UI 事件
pub enum UiEvent {
    /// 按钮点击事件
    ButtonClick {
        button_id: String,
    },
    /// 文本输入事件
    TextInput {
        input_id: String,
        text: String,
    },
    /// 其他 UI 事件
    Other {
        event_type: String,
        data: Box<dyn std::any::Any + Send + Sync>,
    },
}

/// UI 组件
#[derive(Default)]
pub struct UiComponent {
    pub ui_events: Vec<UiEvent>,
}

impl Resource for UiComponent {}

// impl Default for UiComponent {
//     fn default() -> Self {
//         Self {
//             ui_events: Vec::new(),
//         }
//     }
// }

/// 发送 UI 事件到 ECS
pub fn send_ui_event(world: &mut World, event: UiEvent) {
    // 检查 UI 组件是否存在
    if world.get_resource::<UiComponent>().is_none() {
        // 创建并插入 UI 组件
        let component = UiComponent::default();
        world.insert_resource(component);
    }
    
    // 获取 UI 组件并添加事件
    if let Some(mut ui_component) = world.get_resource_mut::<UiComponent>() {
        ui_component.ui_events.push(event);
    }
}

/// 处理 UI 事件的系统
pub fn handle_ui_events_system(world: &mut World) {
    // 获取 UI 组件
    if let Some(ui_component) = world.get_resource_mut::<UiComponent>() {
        // 处理事件
        for event in &ui_component.ui_events {
            match event {
                UiEvent::ButtonClick { button_id } => {
                    println!("Button clicked: {}", button_id);
                    // 这里可以添加游戏逻辑处理
                }
                UiEvent::TextInput { input_id, text } => {
                    println!("Text input: {} = {}", input_id, text);
                    // 这里可以添加游戏逻辑处理
                }
                UiEvent::Other { event_type, data } => {
                    println!("Other UI event: {}", event_type);
                    // 这里可以添加游戏逻辑处理
                }
            }
        }
        
        // 清空事件
        ui_component.ui_events.clear();
    }
}
