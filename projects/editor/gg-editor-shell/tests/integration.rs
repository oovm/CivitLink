//! 编辑器壳程序集成测试
//!
//! 验证 WinitWindowService、ShortcutRegistry、EventBus 和 CommandManager 的核心功能。

use std::cell::RefCell;
use std::rc::Rc;

use gg_editor_shell::{
    Command, CommandManager, EditorContext, EditorEvent, EventBus, Key, ModifierState,
    PendingWindowCreate, ServiceRegistry, ShortcutKey, ShortcutRegistry, WindowId, WindowService,
    WinitWindowService,
};
use gg_world::GameWorld;

/// 测试计数器命令
///
/// 用于验证命令管理器的执行、撤销和重做逻辑。
struct CounterCommand {
    /// 计数器值
    value: i32,
}

impl Command for CounterCommand {
    fn execute(&mut self, _context: &mut EditorContext) -> gg_core::GResult<()> {
        Ok(())
    }

    fn undo(&mut self, _context: &mut EditorContext) -> gg_core::GResult<()> {
        Ok(())
    }

    fn description(&self) -> &str {
        "counter"
    }
}

/// 测试 WinitWindowService 的窗口创建和销毁流程
///
/// 验证创建浮动窗口后 pending_creates 包含对应请求，
/// 销毁窗口后 pending_destroys 包含对应 ID。
#[test]
fn test_winit_window_service_create_destroy() {
    let mut service = WinitWindowService::new();

    let window_id = service.create_floating_window("test_window", (800.0, 600.0));

    let creates: Vec<PendingWindowCreate> = service.drain_pending_creates();
    assert_eq!(creates.len(), 1);
    assert_eq!(creates[0].window_id, window_id);
    assert_eq!(creates[0].title, "test_window");
    assert_eq!(creates[0].size, (800.0, 600.0));

    let creates_again: Vec<PendingWindowCreate> = service.drain_pending_creates();
    assert!(creates_again.is_empty());

    service.destroy_window(window_id);

    let destroys: Vec<WindowId> = service.drain_pending_destroys();
    assert_eq!(destroys.len(), 1);
    assert_eq!(destroys[0], window_id);

    let destroys_again: Vec<WindowId> = service.drain_pending_destroys();
    assert!(destroys_again.is_empty());
}

/// 测试 ShortcutRegistry 的注册和查找功能
///
/// 验证注册 Ctrl+Z 为 "undo" 后，匹配查询返回 "undo"，
/// 不匹配的查询返回 None。
#[test]
fn test_shortcut_registry_register_find() {
    let mut registry = ShortcutRegistry::new();

    registry.register(ShortcutKey::new(Key::Z).with_ctrl(), "undo".to_string());

    let ctrl_modifier = ModifierState {
        ctrl: true,
        shift: false,
        alt: false,
    };
    let result = registry.find_command(&Key::Z, &ctrl_modifier);
    assert_eq!(result, Some("undo"));

    let no_modifier = ModifierState::default();
    let result = registry.find_command(&Key::Z, &no_modifier);
    assert!(result.is_none());

    let result = registry.find_command(&Key::A, &ctrl_modifier);
    assert!(result.is_none());
}

/// 测试 EventBus 的事件订阅和处理功能
///
/// 验证订阅 WindowCreated 事件后，发布并处理该事件时处理器被调用。
#[test]
fn test_event_bus_window_events() {
    let mut bus = EventBus::new();
    let called = Rc::new(RefCell::new(false));
    let received_id = Rc::new(RefCell::new(0u64));

    let called_clone = Rc::clone(&called);
    let received_id_clone = Rc::clone(&received_id);
    bus.subscribe(Box::new(move |event| {
        if let EditorEvent::WindowCreated { window_id } = event {
            *called_clone.borrow_mut() = true;
            *received_id_clone.borrow_mut() = *window_id;
        }
    }));

    bus.publish(EditorEvent::WindowCreated { window_id: 42 });

    assert!(!*called.borrow());
    assert_eq!(*received_id.borrow(), 0);

    bus.process_pending();

    assert!(*called.borrow());
    assert_eq!(*received_id.borrow(), 42);
}

/// 测试 CommandManager 的撤销和重做功能
///
/// 验证执行命令后 can_undo 为 true，撤销后 can_redo 为 true，
/// 重做后 can_undo 再次为 true。
#[test]
fn test_command_manager_undo_redo() {
    let mut services = ServiceRegistry::new();
    let mut commands = CommandManager::new();
    let mut events = EventBus::new();
    let mut world = GameWorld::new("test_world".to_string());

    let cmd = Box::new(CounterCommand { value: 42 });
    {
        let mut context =
            EditorContext::new(&mut services, &mut commands, &mut events, &mut world);
        context.execute_command(cmd);
    }

    assert!(commands.can_undo());
    assert!(!commands.can_redo());

    {
        let mut context =
            EditorContext::new(&mut services, &mut commands, &mut events, &mut world);
        context.undo_command().unwrap();
    }

    assert!(!commands.can_undo());
    assert!(commands.can_redo());

    {
        let mut context =
            EditorContext::new(&mut services, &mut commands, &mut events, &mut world);
        context.redo_command().unwrap();
    }

    assert!(commands.can_undo());
    assert!(!commands.can_redo());
}
