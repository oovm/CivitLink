use gg_reflection::{EnumReflect, ListReflect, MapReflect, PartialReflect, ReflectionRegistry}; 
use gg_ecs::World;
use std::any::TypeId;
use std::collections::HashMap;

mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Health(f32);

    impl PartialReflect for Health {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
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
            } else {
                Err(format!("type mismatch: expected {}, got {}", self.type_name(), source.type_name()))
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Name(String);

    impl PartialReflect for Name {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
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
            } else {
                Err(format!("type mismatch: expected {}, got {}", self.type_name(), source.type_name()))
            }
        }
    }

    #[test]
    fn test_reflect_component_insert_get_remove() {
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<Health>();

        let mut world = World::new();
        let entity = world.spawn().id();

        let rc = registry.get_component_reflect(TypeId::of::<Health>()).unwrap();
        rc.insert(&mut world, entity, Box::new(Health(100.0)));

        let value = rc.get(&world, entity).unwrap();
        let health = value.as_any().downcast_ref::<Health>().unwrap();
        assert_eq!(health.0, 100.0);

        rc.remove(&mut world, entity);
        assert!(rc.get(&world, entity).is_none());
    }

    #[test]
    fn test_reflect_component_get_mut() {
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<Health>();

        let mut world = World::new();
        let entity = world.spawn().id();

        let rc = registry.get_component_reflect(TypeId::of::<Health>()).unwrap();
        rc.insert(&mut world, entity, Box::new(Health(50.0)));

        {
            let value_mut = rc.get_mut(&mut world, entity).unwrap();
            let health = value_mut.as_any_mut().downcast_mut::<Health>().unwrap();
            health.0 = 75.0;
        }

        let value = rc.get(&world, entity).unwrap();
        let health = value.as_any().downcast_ref::<Health>().unwrap();
        assert_eq!(health.0, 75.0);
    }

    #[test]
    fn test_reflect_component_clone_value() {
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<Health>();

        let mut world = World::new();
        let entity = world.spawn().id();

        let rc = registry.get_component_reflect(TypeId::of::<Health>()).unwrap();
        rc.insert(&mut world, entity, Box::new(Health(42.0)));

        let cloned = rc.clone_value(&world, entity).unwrap();
        let health = cloned.as_any().downcast_ref::<Health>().unwrap();
        assert_eq!(health.0, 42.0);
    }

    #[test]
    fn test_registry_get_by_name() {
        let mut registry = ReflectionRegistry::new();
        registry.register::<Health>();

        let type_name = std::any::type_name::<Health>();
        let found = registry.get_by_name(type_name).unwrap();
        assert_eq!(found.type_info().type_name, type_name);

        assert!(registry.get_by_name("nonexistent::Type").is_none());
    }

    #[test]
    fn test_registry_get_by_short_name() {
        let mut registry = ReflectionRegistry::new();
        registry.register::<Health>();

        let found = registry.get_by_short_name("Health").unwrap();
        assert_eq!(found.type_info().short_name, "Health");

        assert!(registry.get_by_short_name("NonExistent").is_none());
    }

    #[test]
    fn test_registry_iter() {
        let mut registry = ReflectionRegistry::new();
        registry.register::<Health>();
        registry.register::<Name>();

        let names: Vec<&str> = registry.iter().map(|r| r.type_info().short_name.as_str()).collect();
        assert!(names.contains(&"Health"));
        assert!(names.contains(&"Name"));
    }

    #[test]
    fn test_registry_iter_component_types() {
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<Health>();
        registry.register_component::<Name>();

        let type_ids: Vec<TypeId> = registry.iter_component_types().map(|(id, _)| id).collect();
        assert!(type_ids.contains(&TypeId::of::<Health>()));
        assert!(type_ids.contains(&TypeId::of::<Name>()));
    }

    #[test]
    fn test_list_reflect_vec() {
        let mut list: Vec<i32> = vec![1, 2, 3];

        assert_eq!(ListReflect::len(&list), 3);
        assert!(!ListReflect::is_empty(&list));

        assert_eq!(ListReflect::get(&list, 0).unwrap().as_any().downcast_ref::<i32>().unwrap(), &1);
        assert_eq!(ListReflect::get(&list, 2).unwrap().as_any().downcast_ref::<i32>().unwrap(), &3);
        assert!(ListReflect::get(&list, 3).is_none());

        ListReflect::push(&mut list, Box::new(4i32));
        assert_eq!(ListReflect::len(&list), 4);

        let removed = ListReflect::remove(&mut list, 1);
        assert_eq!(removed.unwrap().as_any().downcast_ref::<i32>().unwrap(), &2);
        assert_eq!(ListReflect::len(&list), 3);
    }

    #[test]
    fn test_map_reflect_hashmap() {
        let mut map: HashMap<String, i32> = HashMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);

        assert_eq!(MapReflect::len(&map), 2);
        assert!(!MapReflect::is_empty(&map));

        assert_eq!(MapReflect::get(&map, "a").unwrap().as_any().downcast_ref::<i32>().unwrap(), &1);
        assert!(MapReflect::get(&map, "c").is_none());

        MapReflect::insert(&mut map, "c".to_string(), Box::new(3i32));
        assert_eq!(MapReflect::len(&map), 3);

        let keys = MapReflect::keys(&map);
        assert_eq!(keys.len(), 3);

        let removed = MapReflect::remove(&mut map, "b");
        assert_eq!(removed.unwrap().as_any().downcast_ref::<i32>().unwrap(), &2);
        assert_eq!(MapReflect::len(&map), 2);
    }

    #[test]
    fn test_enum_reflect() {
        #[derive(Clone, Debug, PartialEq)]
        enum Color {
            Red,
            Green,
            Blue,
            Rgb(f32, f32, f32),
        }

        static VARIANTS: &[&str] = &["Red", "Green", "Blue", "Rgb"];

        impl PartialReflect for Color {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
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
                } else {
                    Err(format!("type mismatch: expected {}, got {}", self.type_name(), source.type_name()))
                }
            }
        }

        impl EnumReflect for Color {
            fn variants(&self) -> &[&str] {
                VARIANTS
            }

            fn variant_name(&self) -> &str {
                match self {
                    Color::Red => "Red",
                    Color::Green => "Green",
                    Color::Blue => "Blue",
                    Color::Rgb(_, _, _) => "Rgb",
                }
            }

            fn field_at(&self, index: usize) -> Option<&dyn PartialReflect> {
                match self {
                    Color::Rgb(r, g, b) => match index {
                        0 => Some(r as &dyn PartialReflect),
                        1 => Some(g as &dyn PartialReflect),
                        2 => Some(b as &dyn PartialReflect),
                        _ => None,
                    },
                    _ => None,
                }
            }

            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
                match self {
                    Color::Rgb(r, g, b) => match index {
                        0 => Some(r as &mut dyn PartialReflect),
                        1 => Some(g as &mut dyn PartialReflect),
                        2 => Some(b as &mut dyn PartialReflect),
                        _ => None,
                    },
                    _ => None,
                }
            }

            fn field_count(&self) -> usize {
                match self {
                    Color::Rgb(_, _, _) => 3,
                    _ => 0,
                }
            }

            fn set_variant(&mut self, name: &str) -> Result<(), String> {
                match name {
                    "Red" => {
                        *self = Color::Red;
                        Ok(())
                    }
                    "Green" => {
                        *self = Color::Green;
                        Ok(())
                    }
                    "Blue" => {
                        *self = Color::Blue;
                        Ok(())
                    }
                    "Rgb" => {
                        *self = Color::Rgb(0.0, 0.0, 0.0);
                        Ok(())
                    }
                    _ => Err(format!("unknown variant: {}", name)),
                }
            }
        }

        let mut color = Color::Rgb(1.0, 0.5, 0.0);

        assert_eq!(color.variants(), &["Red", "Green", "Blue", "Rgb"] as &[&str]);
        assert_eq!(color.variant_name(), "Rgb");
        assert_eq!(color.field_count(), 3);

        let r = color.field_at(0).unwrap().as_any().downcast_ref::<f32>().unwrap();
        assert_eq!(*r, 1.0);

        color.set_variant("Red").unwrap();
        assert_eq!(color.variant_name(), "Red");
        assert_eq!(color.field_count(), 0);

        assert!(color.set_variant("Unknown").is_err());
    }
}
