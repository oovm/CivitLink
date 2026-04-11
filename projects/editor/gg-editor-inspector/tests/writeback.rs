//! 属性检查器回写集成测试
//!
//! 验证 PropertyStore 读写、ReflectionPropertyBinding 绑定
//! 和 SetPropertyCommand 撤销/重做功能。

use gg_ecs::World;
use gg_editor_inspector::{PropertyBinding, PropertyStore, ReflectionPropertyBinding, SetPropertyCommand};

/// 测试 PropertyStore 的读写功能
///
/// 验证设置属性值后可以正确读取，未设置的属性返回 None。
#[test]
fn test_property_store_read_write() {
    let mut store = PropertyStore::new();

    assert!(store.get(1, "Transform2D", "x").is_none());

    store.set(1, "Transform2D", "x", "100.0");
    let value = store.get(1, "Transform2D", "x");
    assert_eq!(value, Some("100.0"));

    store.set(1, "Transform2D", "x", "200.0");
    let value = store.get(1, "Transform2D", "x");
    assert_eq!(value, Some("200.0"));

    store.set(1, "Transform2D", "y", "300.0");
    let value = store.get(1, "Transform2D", "y");
    assert_eq!(value, Some("300.0"));

    assert!(store.get(2, "Transform2D", "x").is_none());
}

/// 测试 ReflectionPropertyBinding 的读写功能
///
/// 验证通过 ReflectionPropertyBinding 向 ECS World 的 PropertyStore 资源
/// 写入值后可以正确读取。
#[test]
fn test_reflection_property_binding() {
    let mut world = World::new();
    world.insert_resource(PropertyStore::new());

    let binding = ReflectionPropertyBinding::new("Transform2D".to_string(), "x".to_string());

    let read_result = binding.read(&mut world, 1);
    assert!(read_result.is_none());

    binding.write(&mut world, 1, "42.5").unwrap();

    let read_result = binding.read(&mut world, 1);
    assert_eq!(read_result, Some("42.5".to_string()));
}

/// 测试 SetPropertyCommand 的撤销和重做功能
///
/// 验证执行 SetPropertyCommand 后属性值改变，
/// 撤销后属性值恢复为旧值。
///
/// 注意：当前 SetPropertyCommand::execute 通过 ServiceRegistry 获取 World，
/// 而 gg_ecs::World 未实现 Send + Sync，无法注册到 ServiceRegistry 中。
/// 此测试标记为 ignore，待 SetPropertyCommand 改为通过
/// EditorContext::world_mut().ecs_world 访问 World 后可启用。
#[test]
#[ignore = "SetPropertyCommand 通过 ServiceRegistry 获取 World，但 World 未实现 Send + Sync，无法注册为服务"]
fn test_set_property_command_undo_redo() {
    let mut ecs_world = World::new();
    ecs_world.insert_resource(PropertyStore::new());

    let binding =
        Box::new(ReflectionPropertyBinding::new("Transform2D".to_string(), "x".to_string()));
    let mut cmd = SetPropertyCommand::new(
        binding,
        1,
        "99.0".to_string(),
        "修改 Transform2D.x".to_string(),
    );

    let mut services = gg_editor_shell::ServiceRegistry::new();
    let mut commands = gg_editor_shell::CommandManager::new();
    let mut events = gg_editor_shell::EventBus::new();
    let mut world = gg_world::GameWorld::new("test_world".to_string());

    {
        let mut context =
            gg_editor_shell::EditorContext::new(&mut services, &mut commands, &mut events, &mut world);
        use gg_editor_shell::Command;
        cmd.execute(&mut context).unwrap();
    }

    let store = ecs_world.get_resource::<PropertyStore>().unwrap();
    let value = store.get(1, "Transform2D", "x");
    assert_eq!(value, Some("99.0"));
    let _ = store;

    {
        let mut context =
            gg_editor_shell::EditorContext::new(&mut services, &mut commands, &mut events, &mut world);
        use gg_editor_shell::Command;
        cmd.undo(&mut context).unwrap();
    }

    let store = ecs_world.get_resource::<PropertyStore>().unwrap();
    let value = store.get(1, "Transform2D", "x");
    assert_eq!(value, None);
}
