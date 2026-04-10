use gg_ecs::{Component, Entity, Resource, World};

#[test]
fn test_spawn_and_despawn() {
    let mut world = World::new();
    let entity = world.spawn().id();
    assert!(world.contains_entity(entity));

    world.despawn(entity).unwrap();
    assert!(!world.contains_entity(entity));
}

#[test]
fn test_entity_generation() {
    let mut world = World::new();
    let entity1 = world.spawn().id();
    let idx = entity1.index();

    world.despawn(entity1).unwrap();
    let entity2 = world.spawn().id();

    assert_eq!(entity2.index(), idx);
    assert!(entity2.generation() > entity1.generation());
    assert!(!world.contains_entity(entity1));
    assert!(world.contains_entity(entity2));
}

#[test]
fn test_add_and_get_component() {
    let mut world = World::new();
    let entity = world.spawn().id();

    world.add_component(entity, 42i32).unwrap();
    world.add_component(entity, "hello".to_string()).unwrap();

    assert_eq!(*world.get_component::<i32>(entity).unwrap(), 42);
    assert_eq!(world.get_component::<String>(entity).unwrap(), "hello");
}

#[test]
fn test_component_mutation() {
    let mut world = World::new();
    let entity = world.spawn().id();

    world.add_component(entity, 10i32).unwrap();
    *world.get_component_mut::<i32>(entity).unwrap() = 20;
    assert_eq!(*world.get_component::<i32>(entity).unwrap(), 20);
}

#[test]
fn test_remove_component() {
    let mut world = World::new();
    let entity = world.spawn().id();

    world.add_component(entity, 42i32).unwrap();
    world.add_component(entity, "hello".to_string()).unwrap();

    let removed = world.remove_component::<i32>(entity).unwrap();
    assert_eq!(*removed, 42);
    assert!(world.get_component::<i32>(entity).is_none());
    assert!(world.get_component::<String>(entity).is_some());
}

#[test]
fn test_resources() {
    let mut world = World::new();

    world.insert_resource(42i32);
    assert_eq!(*world.get_resource::<i32>().unwrap(), 42);

    *world.get_resource_mut::<i32>().unwrap() = 100;
    assert_eq!(*world.get_resource::<i32>().unwrap(), 100);

    let removed = world.remove_resource::<i32>().unwrap();
    assert_eq!(*removed.downcast::<i32>().unwrap(), 100);
    assert!(world.get_resource::<i32>().is_none());
}

#[test]
fn test_query() {
    let mut world = World::new();

    let e1 = world.spawn().id();
    world.add_component(e1, 1i32).unwrap();

    let e2 = world.spawn().id();
    world.add_component(e2, 2i32).unwrap();

    let _e3 = world.spawn().id();

    let results: Vec<(Entity, &i32)> = world.query::<i32>().collect();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_multiple_entities_despawn() {
    let mut world = World::new();

    let e1 = world.spawn().id();
    world.add_component(e1, 1i32).unwrap();

    let e2 = world.spawn().id();
    world.add_component(e2, 2i32).unwrap();

    let e3 = world.spawn().id();
    world.add_component(e3, 3i32).unwrap();

    world.despawn(e2).unwrap();

    assert!(world.contains_entity(e1));
    assert!(!world.contains_entity(e2));
    assert!(world.contains_entity(e3));

    assert_eq!(*world.get_component::<i32>(e1).unwrap(), 1);
    assert_eq!(*world.get_component::<i32>(e3).unwrap(), 3);
}

#[test]
fn test_entity_builder() {
    let mut world = World::new();
    let entity = world.spawn().insert(42i32).insert("hello".to_string()).id();

    assert_eq!(*world.get_component::<i32>(entity).unwrap(), 42);
    assert_eq!(world.get_component::<String>(entity).unwrap(), "hello");
}

#[test]
fn test_multi_query_two_components() {
    let mut world = World::new();

    let _e1 = world.spawn().insert(1i32).insert(10.0f64).id();
    let _e2 = world.spawn().insert(2i32).id();
    let _e3 = world.spawn().insert(3i32).insert(30.0f64).id();

    let mut results: Vec<(Entity, i32, f64)> = Vec::new();
    world.query_multi::<(&i32, &f64)>().for_each(|entity, (a, b): (&i32, &f64)| {
        results.push((entity, *a, *b));
    });

    assert_eq!(results.len(), 2);
    let values: Vec<i32> = results.iter().map(|r| r.1).collect();
    assert!(values.contains(&1));
    assert!(values.contains(&3));
}

#[test]
fn test_changed_filter() {
    let mut world = World::new();

    let e1 = world.spawn().id();
    world.add_component(e1, 10i32).unwrap();

    let e2 = world.spawn().id();
    world.add_component(e2, 20i32).unwrap();

    // Initial state - no changes
    let initial: Vec<(Entity, &i32)> = world.query_changed::<i32>().collect();
    assert_eq!(initial.len(), 0);

    // Modify component
    *world.get_component_mut::<i32>(e1).unwrap() = 15;

    // Now e1 should be in changed query
    let changed: Vec<(Entity, &i32)> = world.query_changed::<i32>().collect();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].0, e1);
    assert_eq!(*changed[0].1, 15);

    // Second query should not include it unless changed again
    let second: Vec<(Entity, &i32)> = world.query_changed::<i32>().collect();
    assert_eq!(second.len(), 0);
}

#[test]
fn test_add_multiple_components() {
    let mut world = World::new();
    let entity = world.spawn().id();

    world.add_component(entity, 42i32).unwrap();
    world.add_component(entity, "hello".to_string()).unwrap();
    world.add_component(entity, 3.14f64).unwrap();

    assert_eq!(*world.get_component::<i32>(entity).unwrap(), 42);
    assert_eq!(world.get_component::<String>(entity).unwrap(), "hello");
    assert_eq!(*world.get_component::<f64>(entity).unwrap(), 3.14);
}

#[test]
fn test_query_with_missing_components() {
    let mut world = World::new();

    let e1 = world.spawn().id();
    world.add_component(e1, 1i32).unwrap();

    let e2 = world.spawn().id();
    world.add_component(e2, "test".to_string()).unwrap();

    let e3 = world.spawn().id();

    let int_results: Vec<(Entity, &i32)> = world.query::<i32>().collect();
    assert_eq!(int_results.len(), 1);
    assert_eq!(int_results[0].0, e1);

    let string_results: Vec<(Entity, &String)> = world.query::<String>().collect();
    assert_eq!(string_results.len(), 1);
    assert_eq!(string_results[0].0, e2);

    let no_results: Vec<(Entity, &f64)> = world.query::<f64>().collect();
    assert_eq!(no_results.len(), 0);
}

#[test]
fn test_component_removal_clears_changed_flag() {
    let mut world = World::new();
    let entity = world.spawn().id();

    world.add_component(entity, 10i32).unwrap();
    *world.get_component_mut::<i32>(entity).unwrap() = 20;

    // Should be in changed query
    let changed: Vec<(Entity, &i32)> = world.query_changed::<i32>().collect();
    assert_eq!(changed.len(), 1);

    // Remove and re-add component
    world.remove_component::<i32>(entity).unwrap();
    world.add_component(entity, 30i32).unwrap();

    // Should be in changed query again
    let re_added: Vec<(Entity, &i32)> = world.query_changed::<i32>().collect();
    assert_eq!(re_added.len(), 1);
    assert_eq!(*re_added[0].1, 30);
}

#[test]
fn test_entity_spawn_after_despawn() {
    let mut world = World::new();

    let e1 = world.spawn().id();
    let e2 = world.spawn().id();
    let e3 = world.spawn().id();

    assert!(world.contains_entity(e1));
    assert!(world.contains_entity(e2));
    assert!(world.contains_entity(e3));

    world.despawn(e2).unwrap();
    assert!(!world.contains_entity(e2));

    let e4 = world.spawn().id();
    assert!(world.contains_entity(e4));
    assert_ne!(e4, e2); // Should have different ID
}

#[test]
fn test_resource_insertion_and_retrieval() {
    let mut world = World::new();

    // Insert resources
    world.insert_resource(42i32);
    world.insert_resource("hello".to_string());
    world.insert_resource(3.14f64);

    // Retrieve resources
    assert_eq!(*world.get_resource::<i32>().unwrap(), 42);
    assert_eq!(world.get_resource::<String>().unwrap(), "hello");
    assert_eq!(*world.get_resource::<f64>().unwrap(), 3.14);

    // Overwrite resource
    world.insert_resource(100i32);
    assert_eq!(*world.get_resource::<i32>().unwrap(), 100);

    // Remove resource
    world.remove_resource::<String>().unwrap();
    assert!(world.get_resource::<String>().is_none());
}

#[test]
fn test_component_type_checking() {
    let mut world = World::new();
    let entity = world.spawn().id();

    world.add_component(entity, 42i32).unwrap();

    // Should be able to get i32 component
    assert!(world.get_component::<i32>(entity).is_some());

    // Should not be able to get String component
    assert!(world.get_component::<String>(entity).is_none());

    // Should be able to add String component
    world.add_component(entity, "hello".to_string()).unwrap();
    assert!(world.get_component::<String>(entity).is_some());
}

#[test]
fn test_entity_operations_on_non_existent() {
    let mut world = World::new();
    let non_existent = Entity::new(999, 0);

    assert!(!world.contains_entity(non_existent));
    assert!(world.despawn(non_existent).is_err());
    assert!(world.get_component::<i32>(non_existent).is_none());
    assert!(world.get_component_mut::<i32>(non_existent).is_none());
    assert!(world.remove_component::<i32>(non_existent).is_err());
}
