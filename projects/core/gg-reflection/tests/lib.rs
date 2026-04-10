use gg_reflection::prelude::*;
use gg_reflection::PartialReflect;

#[derive(Reflect, Clone)]
struct TestStruct {
    name: String,
    value: i32,
}

#[derive(Reflect, Clone)]
struct TupleStruct(f32, f32);

#[derive(Reflect, Clone)]
struct UnitStruct;

#[test]
fn test_reflection_registry_register_and_is_registered() {
    let mut registry = ReflectionRegistry::new();
    assert!(!registry.is_registered(std::any::TypeId::of::<TestStruct>()));
    registry.register::<TestStruct>();
    assert!(registry.is_registered(std::any::TypeId::of::<TestStruct>()));
}

#[test]
fn test_type_info_creation() {
    let info = TypeInfo::new::<TestStruct>();
    assert_eq!(info.type_id, std::any::TypeId::of::<TestStruct>());
    assert!(info.type_name.contains("TestStruct"));
    assert_eq!(info.short_name, "TestStruct");
}

#[test]
fn test_property_info_creation_with_description() {
    let prop = PropertyInfo::new("health", "i32", true).with_description("Player health points");
    assert_eq!(prop.name, "health");
    assert_eq!(prop.type_name, "i32");
    assert!(prop.writable);
    assert_eq!(prop.description, Some("Player health points".to_string()));
}

#[test]
fn test_field_access_on_derived_struct() {
    let s = TestStruct { name: "hello".to_string(), value: 42 };
    assert_eq!(s.field_names(), &["name", "value"]);

    let name_field = s.field("name").unwrap();
    let name_any = name_field.as_any();
    let name_val = name_any.downcast_ref::<String>().unwrap();
    assert_eq!(name_val, "hello");

    let value_field = s.field("value").unwrap();
    let value_any = value_field.as_any();
    let value_val = value_any.downcast_ref::<i32>().unwrap();
    assert_eq!(*value_val, 42);

    assert!(s.field("nonexistent").is_none());
}

#[test]
fn test_field_mut_access() {
    let mut s = TestStruct { name: "hello".to_string(), value: 42 };
    {
        let name_field = s.field_mut("name").unwrap();
        let name_any = name_field.as_any_mut();
        let name_val = name_any.downcast_mut::<String>().unwrap();
        name_val.push_str(" world");
    }
    assert_eq!(s.name, "hello world");

    {
        let value_field = s.field_mut("value").unwrap();
        let value_any = value_field.as_any_mut();
        let value_val = value_any.downcast_mut::<i32>().unwrap();
        *value_val = 100;
    }
    assert_eq!(s.value, 100);
}

#[test]
fn test_struct_property_editor_editable_properties() {
    let s = TestStruct { name: "test".to_string(), value: 10 };
    let editor = StructPropertyEditor::new(s);
    let props = editor.editable_properties();
    assert_eq!(props.len(), 2);
    assert_eq!(props[0].name, "name");
    assert_eq!(props[1].name, "value");
    assert!(props[0].writable);
    assert!(props[1].writable);
}

#[test]
fn test_struct_property_editor_get_property() {
    let s = TestStruct { name: "test".to_string(), value: 10 };
    let editor = StructPropertyEditor::new(s);
    let prop = editor.get_property("name").unwrap();
    let name_val = prop.as_any().downcast_ref::<String>().unwrap();
    assert_eq!(name_val, "test");
}

#[test]
fn test_struct_property_editor_set_property_success() {
    let s = TestStruct { name: "test".to_string(), value: 10 };
    let mut editor = StructPropertyEditor::new(s);
    let result = editor.set_property("value", Box::new(99_i32));
    assert!(result.is_ok());
    assert_eq!(editor.value.value, 99);
}

#[test]
fn test_struct_property_editor_set_property_type_mismatch() {
    let s = TestStruct { name: "test".to_string(), value: 10 };
    let mut editor = StructPropertyEditor::new(s);
    let result = editor.set_property("value", Box::new("wrong".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_struct_property_editor_set_property_missing_field() {
    let s = TestStruct { name: "test".to_string(), value: 10 };
    let mut editor = StructPropertyEditor::new(s);
    let result = editor.set_property("nonexistent", Box::new(99_i32));
    assert!(result.is_err());
}

#[test]
fn test_named_struct_reflect() {
    let s = TestStruct { name: "hello".to_string(), value: 42 };
    assert_eq!(s.field_names(), &["name", "value"]);
    assert!(s.field("name").is_some());
    assert!(s.field("value").is_some());
    assert!(s.field("nonexistent").is_none());
}

#[test]
fn test_tuple_struct_reflect() {
    let s = TupleStruct(1.0, 2.0);
    assert_eq!(s.field_names(), &["0", "1"]);
    assert!(s.field("0").is_some());
    assert!(s.field("1").is_some());
    assert!(s.field("2").is_none());
}

#[test]
fn test_unit_struct_reflect() {
    let s = UnitStruct;
    assert_eq!(s.field_names(), &[] as &[&str]);
    assert!(s.field("x").is_none());
}
