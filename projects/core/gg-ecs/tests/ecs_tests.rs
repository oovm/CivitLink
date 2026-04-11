use gg_ecs::{Component, Resource, TypeId, World};

mod tests_component_derive {
    use super::*;

    #[derive(Component, Debug, PartialEq)]
    struct Health(f32);

    #[derive(Component, Debug, PartialEq)]
    struct Name(String);

    #[test]
    fn test_component_derive() {
        let mut world = World::new();
        let entity = world.spawn().id();
        world.add_component(entity, Health(100.0)).unwrap();
        world.add_component(entity, Name("Hero".to_string())).unwrap();

        let health = world.get_component::<Health>(entity).unwrap();
        assert_eq!(health.0, 100.0);

        let name = world.get_component::<Name>(entity).unwrap();
        assert_eq!(name.0, "Hero");
    }
}

mod tests_add_component_raw {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    #[test]
    fn test_add_component_raw_basic() {
        let mut world = World::new();
        let entity = world.spawn().id();

        let pos = Position { x: 1.0, y: 2.0 };
        let type_id = TypeId::of::<Position>();
        world.add_component_raw(entity, Box::new(pos), type_id).unwrap();

        let result = world.get_component::<Position>(entity);
        assert!(result.is_some());
        assert_eq!(result.unwrap().x, 1.0);
        assert_eq!(result.unwrap().y, 2.0);
    }

    #[test]
    fn test_add_component_raw_overwrite() {
        let mut world = World::new();
        let entity = world.spawn().id();

        world.add_component(entity, Position { x: 1.0, y: 2.0 }).unwrap();

        let new_pos = Position { x: 3.0, y: 4.0 };
        world.add_component_raw(entity, Box::new(new_pos), TypeId::of::<Position>()).unwrap();

        let result = world.get_component::<Position>(entity).unwrap();
        assert_eq!(result.x, 3.0);
        assert_eq!(result.y, 4.0);
    }

    #[test]
    fn test_add_component_raw_with_migration() {
        let mut world = World::new();
        let entity = world.spawn().id();

        world.add_component(entity, Position { x: 1.0, y: 2.0 }).unwrap();

        let vel = Velocity { dx: 0.5, dy: 0.3 };
        world.add_component_raw(entity, Box::new(vel), TypeId::of::<Velocity>()).unwrap();

        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);

        let vel = world.get_component::<Velocity>(entity).unwrap();
        assert_eq!(vel.dx, 0.5);
        assert_eq!(vel.dy, 0.3);
    }
}

mod tests_resource_derive {
    use super::*;

    #[derive(Resource, Debug, PartialEq)]
    struct GameTime {
        delta: f32,
        elapsed: f32,
    }

    #[test]
    fn test_resource_derive() {
        let mut world = World::new();
        world.insert_resource(GameTime { delta: 0.016, elapsed: 0.0 });

        let time = world.get_resource::<GameTime>().unwrap();
        assert_eq!(time.delta, 0.016);
    }
}
