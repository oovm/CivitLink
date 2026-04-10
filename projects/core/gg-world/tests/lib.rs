use gg_world::{GameWorld, WorldManager};

#[test]
fn test_spawn_and_add_component() {
    let mut world = GameWorld::new("test".to_string());
    let entity = world.spawn().id();

    world.add_component(entity, 42i32).unwrap();
    world.add_component(entity, "hello".to_string()).unwrap();

    assert_eq!(*world.get_component::<i32>(entity).unwrap(), 42);
    assert_eq!(world.get_component::<String>(entity).unwrap(), "hello");
}

#[test]
fn test_get_component_and_mut() {
    let mut world = GameWorld::new("test".to_string());
    let entity = world.spawn().id();

    world.add_component(entity, 10i32).unwrap();

    assert_eq!(*world.get_component::<i32>(entity).unwrap(), 10);

    *world.get_component_mut::<i32>(entity).unwrap() = 20;
    assert_eq!(*world.get_component::<i32>(entity).unwrap(), 20);
}

#[test]
fn test_remove_component() {
    let mut world = GameWorld::new("test".to_string());
    let entity = world.spawn().id();

    world.add_component(entity, 42i32).unwrap();
    world.add_component(entity, "hello".to_string()).unwrap();

    let removed = world.remove_component::<i32>(entity).unwrap();
    assert_eq!(*removed, 42);
    assert!(world.get_component::<i32>(entity).is_none());
    assert!(world.get_component::<String>(entity).is_some());
}

#[test]
fn test_world_manager_create_and_get() {
    let mut manager = WorldManager::new();

    let id1 = manager.create_world("world1".to_string());
    let id2 = manager.create_world("world2".to_string());

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);

    assert_eq!(manager.get_world(id1).unwrap().name(), "world1");
    assert_eq!(manager.get_world(id2).unwrap().name(), "world2");
    assert!(manager.get_world(999).is_none());
}

#[test]
fn test_world_manager_active_switching() {
    let mut manager = WorldManager::new();

    let id1 = manager.create_world("world1".to_string());
    let id2 = manager.create_world("world2".to_string());

    assert!(manager.active_world().is_none());

    assert!(manager.set_active_world(id1));
    assert_eq!(manager.active_world().unwrap().name(), "world1");

    assert!(manager.set_active_world(id2));
    assert_eq!(manager.active_world().unwrap().name(), "world2");

    assert!(!manager.set_active_world(999));
    assert_eq!(manager.active_world().unwrap().name(), "world2");
}