use std::collections::HashMap;
use wae_types::*;

#[test]
fn test_value_constructors() {
    assert_eq!(Value::null(), Value::Null);
    assert_eq!(Value::bool(true), Value::Bool(true));
    assert_eq!(Value::integer(42), Value::Integer(42));
    assert_eq!(Value::float(3.14), Value::Float(3.14));
    assert_eq!(Value::string("hello"), Value::String("hello".to_string()));
    assert_eq!(Value::bytes(vec![1, 2, 3]), Value::Bytes(vec![1, 2, 3]));
    assert_eq!(Value::array(vec![]), Value::Array(vec![]));
    assert_eq!(Value::object(HashMap::new()), Value::Object(HashMap::new()));
}

#[test]
fn test_value_type_checks() {
    assert!(Value::Null.is_null());
    assert!(!Value::Null.is_bool());
    assert!(!Value::Null.is_integer());
    assert!(!Value::Null.is_float());
    assert!(!Value::Null.is_string());
    assert!(!Value::Null.is_array());
    assert!(!Value::Null.is_object());

    assert!(!Value::Bool(true).is_null());
    assert!(Value::Bool(true).is_bool());
    assert!(!Value::Bool(true).is_number());

    assert!(!Value::Integer(42).is_null());
    assert!(Value::Integer(42).is_integer());
    assert!(Value::Integer(42).is_number());

    assert!(!Value::Float(3.14).is_null());
    assert!(Value::Float(3.14).is_float());
    assert!(Value::Float(3.14).is_number());

    assert!(!Value::String("hello".to_string()).is_null());
    assert!(Value::String("hello".to_string()).is_string());

    assert!(!Value::Array(vec![]).is_null());
    assert!(Value::Array(vec![]).is_array());

    assert!(!Value::Object(HashMap::new()).is_null());
    assert!(Value::Object(HashMap::new()).is_object());
}

#[test]
fn test_value_as_conversions() {
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert_eq!(Value::Bool(false).as_bool(), Some(false));
    assert_eq!(Value::Null.as_bool(), None);

    assert_eq!(Value::Integer(42).as_integer(), Some(42));
    assert_eq!(Value::Float(3.14).as_integer(), None);

    assert_eq!(Value::Float(3.14).as_float(), Some(3.14));
    assert_eq!(Value::Integer(42).as_float(), Some(42.0));

    assert_eq!(Value::String("hello".to_string()).as_str(), Some("hello"));
    assert_eq!(Value::Null.as_str(), None);

    let arr = vec![Value::Integer(1), Value::Integer(2)];
    assert_eq!(Value::Array(arr.clone()).as_array(), Some(&arr));

    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Integer(42));
    assert_eq!(Value::Object(map.clone()).as_object(), Some(&map));
}

#[test]
fn test_value_mut_conversions() {
    let mut arr = Value::Array(vec![Value::Integer(1)]);
    if let Some(a) = arr.as_array_mut() {
        a.push(Value::Integer(2));
    }
    assert_eq!(arr.as_array().unwrap().len(), 2);

    let mut obj = Value::Object(HashMap::new());
    if let Some(o) = obj.as_object_mut() {
        o.insert("key".to_string(), Value::Integer(42));
    }
    assert_eq!(obj.as_object().unwrap().len(), 1);
}

#[test]
fn test_value_get_and_get_index() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Integer(42));
    let obj = Value::Object(map);
    assert_eq!(obj.get("key"), Some(&Value::Integer(42)));
    assert_eq!(obj.get("nonexistent"), None);

    let arr = Value::Array(vec![Value::Integer(1), Value::Integer(2)]);
    assert_eq!(arr.get_index(0), Some(&Value::Integer(1)));
    assert_eq!(arr.get_index(1), Some(&Value::Integer(2)));
    assert_eq!(arr.get_index(2), None);
}

#[test]
fn test_value_from_traits() {
    assert_eq!(Value::from(true), Value::Bool(true));
    assert_eq!(Value::from(42i32), Value::Integer(42));
    assert_eq!(Value::from(42i64), Value::Integer(42));
    assert_eq!(Value::from(42u64), Value::Integer(42));
    assert_eq!(Value::from(3.14), Value::Float(3.14));
    assert_eq!(Value::from("hello"), Value::String("hello".to_string()));
    assert_eq!(Value::from("hello".to_string()), Value::String("hello".to_string()));
    assert_eq!(Value::from(vec![Value::Integer(1)]), Value::Array(vec![Value::Integer(1)]));
}

#[test]
fn test_object_and_array_macros() {
    let obj = object! {
        "name" => "test",
        "age" => 42,
    };
    assert!(obj.is_object());
    assert_eq!(obj.get("name"), Some(&Value::String("test".to_string())));
    assert_eq!(obj.get("age"), Some(&Value::Integer(42)));

    let empty_obj = object! {};
    assert!(empty_obj.is_object());
    assert_eq!(empty_obj.as_object().unwrap().len(), 0);

    let arr = array![1, 2, "three"];
    assert!(arr.is_array());
    assert_eq!(arr.get_index(0), Some(&Value::Integer(1)));
    assert_eq!(arr.get_index(1), Some(&Value::Integer(2)));
    assert_eq!(arr.get_index(2), Some(&Value::String("three".to_string())));

    let empty_arr = array![];
    assert!(empty_arr.is_array());
    assert_eq!(empty_arr.as_array().unwrap().len(), 0);
}

#[test]
fn test_value_json_serialization() {
    assert_eq!(Value::Null.to_json_string(), "null");
    assert_eq!(Value::Bool(true).to_json_string(), "true");
    assert_eq!(Value::Bool(false).to_json_string(), "false");
    assert_eq!(Value::Integer(42).to_json_string(), "42");
    assert_eq!(Value::Float(3.14).to_json_string(), "3.14");
    assert_eq!(Value::String("hello".to_string()).to_json_string(), "\"hello\"");
}

#[test]
fn test_value_json_parsing() {
    assert_eq!(Value::from_json_str("null").unwrap(), Value::Null);
    assert_eq!(Value::from_json_str("true").unwrap(), Value::Bool(true));
    assert_eq!(Value::from_json_str("false").unwrap(), Value::Bool(false));
    assert_eq!(Value::from_json_str("42").unwrap(), Value::Integer(42));
    assert_eq!(Value::from_json_str("3.14").unwrap(), Value::Float(3.14));
    assert_eq!(Value::from_json_str("\"hello\"").unwrap(), Value::String("hello".to_string()));

    let parsed_array = Value::from_json_str("[1, 2, 3]").unwrap();
    assert!(parsed_array.is_array());

    let parsed_object = Value::from_json_str("{\"key\": 42}").unwrap();
    assert!(parsed_object.is_object());
}

#[test]
fn test_value_deep_clone() {
    let original = object! {
        "nested" => object! {
            "array" => array! [1, 2, 3],
        },
    };
    let cloned = original.deep_clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_value_merge() {
    let mut a = object! {
        "common" => "old",
        "only_a" => 42,
    };
    let b = object! {
        "common" => "new",
        "only_b" => "hello",
    };
    a.merge(b);

    assert_eq!(a.get("common"), Some(&Value::String("new".to_string())));
    assert_eq!(a.get("only_a"), Some(&Value::Integer(42)));
    assert_eq!(a.get("only_b"), Some(&Value::String("hello".to_string())));

    let mut arr1 = array![1, 2];
    let arr2 = array![3, 4];
    arr1.merge(arr2);
    assert_eq!(arr1.as_array().unwrap().len(), 4);
}

#[test]
fn test_billing_dimensions_default() {
    let dims = BillingDimensions::default();
    assert_eq!(dims.input_text, 0);
    assert_eq!(dims.output_text, 0);
    assert_eq!(dims.input_pixels, 0);
    assert_eq!(dims.output_pixels, 0);
}
