use gg_reflection::{PartialReflect, ReflectionRegistry};
use gg_world::{ComponentData, EntityData, GameWorld, SceneData, SceneDeserializer, SceneSerializer, WorldData};
use std::any::Any;

mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestPosition {
        x: f32,
        y: f32,
    }

    impl PartialReflect for TestPosition {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn type_name(&self) -> &'static str {
            std::any::type_name::<Self>()
        }

        fn clone_reflect(&self) -> Box<dyn PartialReflect> {
            Box::new(self.clone())
        }

        fn field_names(&self) -> &[&str] {
            &["x", "y"]
        }

        fn field(&self, name: &str) -> Option<&dyn PartialReflect> {
            match name {
                "x" => Some(&self.x),
                "y" => Some(&self.y),
                _ => None,
            }
        }

        fn field_mut(&mut self, name: &str) -> Option<&mut dyn PartialReflect> {
            match name {
                "x" => Some(&mut self.x),
                "y" => Some(&mut self.y),
                _ => None,
            }
        }

        fn try_assign(&mut self, source: &dyn PartialReflect) -> Result<(), String> {
            if let Some(val) = source.as_any().downcast_ref::<Self>() {
                *self = val.clone();
                Ok(())
            }
            else {
                Err(format!("type mismatch: expected {}, got {}", self.type_name(), source.type_name()))
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct TestHealth(f32);

    impl PartialReflect for TestHealth {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn type_name(&self) -> &'static str {
            std::any::type_name::<Self>()
        }

        fn clone_reflect(&self) -> Box<dyn PartialReflect> {
            Box::new(self.clone())
        }

        fn try_assign(&mut self, source: &dyn PartialReflect) -> Result<(), String> {
            if let Some(val) = source.as_any().downcast_ref::<Self>() {
                *self = val.clone();
                Ok(())
            }
            else {
                Err(format!("type mismatch: expected {}, got {}", self.type_name(), source.type_name()))
            }
        }
    }

    #[test]
    fn test_scene_data_ron_roundtrip() {
        let scene = SceneData {
            version: "1.0".to_string(),
            name: "test_scene".to_string(),
            entities: vec![EntityData {
                id: 0,
                name: "Entity_0".to_string(),
                parent_id: None,
                components: vec![ComponentData {
                    type_name: "TestPosition".to_string(),
                    properties: serde_json::json!({"x": 1.0, "y": 2.0}),
                }],
            }],
        };

        let ron_str = scene.to_ron().expect("RON serialization failed");
        let deserialized = SceneData::from_ron(&ron_str).expect("RON deserialization failed");

        assert_eq!(deserialized.version, "1.0");
        assert_eq!(deserialized.name, "test_scene");
        assert_eq!(deserialized.entities.len(), 1);
        assert_eq!(deserialized.entities[0].name, "Entity_0");
        assert_eq!(deserialized.entities[0].components.len(), 1);
        assert_eq!(deserialized.entities[0].components[0].type_name, "TestPosition");
    }

    #[test]
    fn test_scene_serializer() {
        let mut world = GameWorld::new("test_world".to_string());
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<TestPosition>();
        registry.register_component::<TestHealth>();

        let entity = world.spawn().id();
        let _ = world.add_component(entity, TestPosition { x: 10.0, y: 20.0 });
        let _ = world.add_component(entity, TestHealth(100.0));

        let scene_data = SceneSerializer::serialize(&world, &registry);

        assert_eq!(scene_data.version, "1.0");
        assert_eq!(scene_data.name, "test_world");
        assert_eq!(scene_data.entities.len(), 1);

        let entity_data = &scene_data.entities[0];
        assert_eq!(entity_data.components.len(), 2);

        let type_names: Vec<&str> = entity_data
            .components
            .iter()
            .map(|c| {
                let short = c.type_name.rsplit("::").next().unwrap_or(&c.type_name);
                short
            })
            .collect();
        assert!(type_names.contains(&"TestPosition"));
        assert!(type_names.contains(&"TestHealth"));

        let pos_comp = entity_data.components.iter().find(|c| c.type_name.contains("TestPosition")).unwrap();
        assert!(pos_comp.properties.is_object());
        let props = pos_comp.properties.as_object().unwrap();
        assert!(props.contains_key("x"));
        assert!(props.contains_key("y"));
    }

    #[test]
    fn test_scene_deserializer_unknown_components_skipped() {
        let scene_data = SceneData {
            version: "1.0".to_string(),
            name: "deser_test".to_string(),
            entities: vec![EntityData {
                id: 0,
                name: "Entity_0".to_string(),
                parent_id: None,
                components: vec![
                    ComponentData {
                        type_name: "UnknownComponent".to_string(),
                        properties: serde_json::Value::Object(serde_json::Map::new()),
                    },
                    ComponentData { type_name: "AnotherUnknown".to_string(), properties: serde_json::Value::Null },
                ],
            }],
        };

        let registry = ReflectionRegistry::new();
        let world = SceneDeserializer::deserialize(&scene_data, &registry);

        assert_eq!(world.name(), "deser_test");
        assert_eq!(world.ecs_world.entities().len(), 1);
    }

    #[test]
    fn test_world_data_from_world() {
        let mut world = GameWorld::new("test_world".to_string());
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<TestPosition>();
        registry.register_component::<TestHealth>();

        let entity = world.spawn().id();
        let _ = world.add_component(entity, TestPosition { x: 10.0, y: 20.0 });
        let _ = world.add_component(entity, TestHealth(100.0));

        let world_data = WorldData::from_world(&world, &registry);

        assert_eq!(world_data.entities.len(), 1);

        let entity_data = &world_data.entities[0];
        assert_eq!(entity_data.components.len(), 2);

        let type_names: Vec<&str> =
            entity_data.components.iter().map(|c| c.type_name.rsplit("::").next().unwrap_or(&c.type_name)).collect();
        assert!(type_names.contains(&"TestPosition"));
        assert!(type_names.contains(&"TestHealth"));
    }

    #[test]
    fn test_world_data_apply_to_world() {
        let world_data = WorldData {
            entities: vec![EntityData {
                id: 0,
                name: "Entity_0".to_string(),
                parent_id: None,
                components: vec![ComponentData {
                    type_name: "UnknownComponent".to_string(),
                    properties: serde_json::Value::Object(serde_json::Map::new()),
                }],
            }],
        };

        let mut world = GameWorld::new("target_world".to_string());
        let registry = ReflectionRegistry::new();

        world_data.apply_to_world(&mut world, &registry);

        assert_eq!(world.ecs_world.entities().len(), 1);
    }

    #[test]
    fn test_gameworld_serialize_deserialize() {
        let mut world = GameWorld::new("serialize_test".to_string());
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<TestPosition>();

        let entity = world.spawn().id();
        let _ = world.add_component(entity, TestPosition { x: 5.0, y: 15.0 });

        let world_data = world.serialize(&registry);
        assert_eq!(world_data.entities.len(), 1);

        let mut target_world = GameWorld::new("target".to_string());
        target_world.deserialize(&world_data, &registry);
        assert_eq!(target_world.ecs_world.entities().len(), 1);
    }
}
