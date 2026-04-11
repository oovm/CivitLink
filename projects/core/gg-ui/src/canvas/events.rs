//! 游戏UI事件系统模块
//!
//! 处理游戏UI的输入事件，如鼠标点击、键盘输入等

use gg_ecs::Entity;
use std::any::Any;

/// 事件系统资源
#[derive(Debug, Clone)]
pub struct EventSystemResource {
    /// 事件队列
    pub events: Vec<UiEvent>,
    /// 选中的对象
    pub currently_selected: Option<Entity>,
    /// 首次选中的对象
    pub first_selected: Option<Entity>,
    /// 是否发送导航事件
    pub send_navigation_events: bool,
    /// 是否像素完美
    pub pixel_perfect: bool,
}

impl Default for EventSystemResource {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            currently_selected: None,
            first_selected: None,
            send_navigation_events: true,
            pixel_perfect: false,
        }
    }
}

/// UI事件
#[derive(Debug)]
pub enum UiEvent {
    /// 点击事件
    Click {
        /// X坐标
        x: f32,
        /// Y坐标
        y: f32,
        /// 按钮
        button: MouseButton,
    },
    /// 鼠标按下事件
    MouseDown {
        /// X坐标
        x: f32,
        /// Y坐标
        y: f32,
        /// 按钮
        button: MouseButton,
    },
    /// 鼠标释放事件
    MouseUp {
        /// X坐标
        x: f32,
        /// Y坐标
        y: f32,
        /// 按钮
        button: MouseButton,
    },
    /// 鼠标移动事件
    MouseMove {
        /// X坐标
        x: f32,
        /// Y坐标
        y: f32,
    },
    /// 鼠标进入事件
    MouseEnter {
        /// 实体
        entity: Entity,
    },
    /// 鼠标离开事件
    MouseExit {
        /// 实体
        entity: Entity,
    },
    /// 键盘按下事件
    KeyDown {
        /// 键
        key: Key,
        /// 修饰键
        modifiers: KeyModifiers,
    },
    /// 键盘释放事件
    KeyUp {
        /// 键
        key: Key,
        /// 修饰键
        modifiers: KeyModifiers,
    },
    /// 文本输入事件
    TextInput {
        /// 文本
        text: String,
    },
    /// 选择事件
    Select {
        /// 实体
        entity: Entity,
    },
    /// 取消选择事件
    Deselect {
        /// 实体
        entity: Entity,
    },
    /// 提交事件
    Submit {
        /// 实体
        entity: Entity,
    },
    /// 更新选择事件
    UpdateSelected {
        /// 实体
        entity: Entity,
    },
    /// 自定义事件
    Custom {
        /// 事件名称
        name: String,
        /// 事件数据
        data: Box<dyn Any + Send + Sync>,
    },
}

impl Clone for UiEvent {
    fn clone(&self) -> Self {
        match self {
            UiEvent::Click { x, y, button } => UiEvent::Click { x: *x, y: *y, button: *button },
            UiEvent::MouseDown { x, y, button } => UiEvent::MouseDown { x: *x, y: *y, button: *button },
            UiEvent::MouseUp { x, y, button } => UiEvent::MouseUp { x: *x, y: *y, button: *button },
            UiEvent::MouseMove { x, y } => UiEvent::MouseMove { x: *x, y: *y },
            UiEvent::MouseEnter { entity } => UiEvent::MouseEnter { entity: *entity },
            UiEvent::MouseExit { entity } => UiEvent::MouseExit { entity: *entity },
            UiEvent::KeyDown { key, modifiers } => UiEvent::KeyDown { key: key.clone(), modifiers: modifiers.clone() },
            UiEvent::KeyUp { key, modifiers } => UiEvent::KeyUp { key: key.clone(), modifiers: modifiers.clone() },
            UiEvent::TextInput { text } => UiEvent::TextInput { text: text.clone() },
            UiEvent::Select { entity } => UiEvent::Select { entity: *entity },
            UiEvent::Deselect { entity } => UiEvent::Deselect { entity: *entity },
            UiEvent::Submit { entity } => UiEvent::Submit { entity: *entity },
            UiEvent::UpdateSelected { entity } => UiEvent::UpdateSelected { entity: *entity },
            UiEvent::Custom { name, .. } => UiEvent::Custom { name: name.clone(), data: Box::new(()) },
        }
    }
}

/// 鼠标按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// 左键
    Left,
    /// 右键
    Right,
    /// 中键
    Middle,
    /// 其他按钮
    Other(u32),
}

/// 键盘按键
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// A 键
    A,
    /// B 键
    B,
    /// C 键
    C,
    /// D 键
    D,
    /// E 键
    E,
    /// F 功能键
    FKey(u32),
    /// G 键
    G,
    /// H 键
    H,
    /// I 键
    I,
    /// J 键
    J,
    /// K 键
    K,
    /// L 键
    L,
    /// M 键
    M,
    /// N 键
    N,
    /// O 键
    O,
    /// P 键
    P,
    /// Q 键
    Q,
    /// R 键
    R,
    /// S 键
    S,
    /// T 键
    T,
    /// U 键
    U,
    /// V 键
    V,
    /// W 键
    W,
    /// X 键
    X,
    /// Y 键
    Y,
    /// Z 键
    Z,
    /// 数字键
    Number(u32),
    /// 空格键
    Space,
    /// 回车键
    Enter,
    /// ESC 键
    Escape,
    /// 退格键
    Backspace,
    /// Tab 键
    Tab,
    /// Shift 键
    Shift,
    /// Control 键
    Control,
    /// Alt 键
    Alt,
    /// 方向键上
    UpArrow,
    /// 方向键下
    DownArrow,
    /// 方向键左
    LeftArrow,
    /// 方向键右
    RightArrow,
    /// 其他键
    Other(String),
}

/// 键盘修饰键
#[derive(Debug, Clone, Default)]
pub struct KeyModifiers {
    /// Shift 是否按下
    pub shift: bool,
    /// Control 是否按下
    pub control: bool,
    /// Alt 是否按下
    pub alt: bool,
    /// Meta 是否按下
    pub meta: bool,
}

/// 输入模块基类
pub trait BaseInputModule {
    /// 处理输入
    fn process(&mut self, event_system: &mut EventSystemResource, world: &mut gg_ecs::World);
    /// 停用模块
    fn deactivate_module(&mut self);
    /// 激活模块
    fn activate_module(&mut self);
}

/// 独立输入模块
pub struct StandaloneInputModule {
    /// 水平轴
    pub horizontal_axis: String,
    /// 垂直轴
    pub vertical_axis: String,
    /// 提交按钮
    pub submit_button: String,
    /// 取消按钮
    pub cancel_button: String,
}

impl Default for StandaloneInputModule {
    fn default() -> Self {
        Self {
            horizontal_axis: "Horizontal".to_string(),
            vertical_axis: "Vertical".to_string(),
            submit_button: "Submit".to_string(),
            cancel_button: "Cancel".to_string(),
        }
    }
}

impl BaseInputModule for StandaloneInputModule {
    fn process(&mut self, event_system: &mut EventSystemResource, world: &mut gg_ecs::World) {
        let click_events: Vec<(f32, f32, MouseButton)> = event_system
            .events
            .iter()
            .filter_map(|event| match event {
                UiEvent::Click { x, y, button } => Some((*x, *y, *button)),
                _ => None,
            })
            .collect();

        for (x, y, button) in click_events {
            self.handle_click(x, y, button, event_system, world);
        }

        event_system.events.clear();
    }

    fn deactivate_module(&mut self) {}

    fn activate_module(&mut self) {}
}

impl StandaloneInputModule {
    /// 处理点击事件
    fn handle_click(
        &self,
        x: f32,
        y: f32,
        _button: MouseButton,
        event_system: &mut EventSystemResource,
        world: &mut gg_ecs::World,
    ) {
        if let Some(entity) = self.raycast(x, y, world) {
            self.process_click(entity, event_system, world);
        }
    }

    /// 射线检测
    fn raycast(&self, _x: f32, _y: f32, _world: &mut gg_ecs::World) -> Option<Entity> {
        None
    }

    /// 处理点击
    fn process_click(&self, entity: Entity, event_system: &mut EventSystemResource, world: &mut gg_ecs::World) {
        if let Some(button) = world.get_component::<super::components::Button>(entity) {
            if button.interactable {
                if let Some(onclick) = &button.onclick {
                    onclick();
                }
            }
        }

        event_system.currently_selected = Some(entity);
    }
}

/// 事件系统
pub struct EventSystem {
    /// 输入模块
    input_module: Box<dyn BaseInputModule>,
}

impl EventSystem {
    /// 创建新的事件系统
    pub fn new(input_module: Box<dyn BaseInputModule>) -> Self {
        Self { input_module }
    }

    /// 处理输入
    pub fn process(&mut self, world: &mut gg_ecs::World) {
        let mut event_system = match world.remove_resource::<EventSystemResource>() {
            Some(boxed) => match boxed.downcast::<EventSystemResource>() {
                Ok(typed) => *typed,
                Err(_) => return,
            },
            None => return,
        };

        self.input_module.process(&mut event_system, world);

        world.insert_resource(event_system);
    }
}

impl gg_ecs::System for EventSystem {
    fn name(&self) -> &str {
        "EventSystem"
    }

    fn execute(&mut self, world: &mut gg_ecs::World) -> gg_error::GResult<()> {
        self.process(world);
        Ok(())
    }
}
