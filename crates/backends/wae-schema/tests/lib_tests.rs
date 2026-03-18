use serde_json::json;
use wae_schema::*;

#[test]
fn test_schema_type_default() {
    let schema_type = SchemaType::default();
    assert_eq!(schema_type, SchemaType::Object);
}

#[test]
fn test_string_schema() {
    let schema = Schema::string()
        .title("用户名")
        .description("用户的用户名")
        .min_length(1)
        .max_length(100)
        .format("email")
        .pattern(r"^[a-zA-Z0-9]+$")
        .example(json!("test@example.com"))
        .nullable(true);

    assert_eq!(schema.schema_type, SchemaType::String);
    assert_eq!(schema.title, Some("用户名".to_string()));
    assert_eq!(schema.description, Some("用户的用户名".to_string()));
    assert_eq!(schema.min_length, Some(1));
    assert_eq!(schema.max_length, Some(100));
    assert_eq!(schema.format, Some("email".to_string()));
    assert_eq!(schema.pattern, Some(r"^[a-zA-Z0-9]+$".to_string()));
    assert_eq!(schema.example, Some(json!("test@example.com")));
    assert_eq!(schema.nullable, true);
}

#[test]
fn test_integer_schema() {
    let schema = Schema::integer().minimum(0.0).maximum(100.0).with_default(json!(0)).read_only(true).write_only(false);

    assert_eq!(schema.schema_type, SchemaType::Integer);
    assert_eq!(schema.minimum, Some(0.0));
    assert_eq!(schema.maximum, Some(100.0));
    assert_eq!(schema.default, Some(json!(0)));
    assert_eq!(schema.read_only, true);
    assert_eq!(schema.write_only, false);
}

#[test]
fn test_number_schema() {
    let schema = Schema::number().minimum(0.5).maximum(99.9);

    assert_eq!(schema.schema_type, SchemaType::Number);
    assert_eq!(schema.minimum, Some(0.5));
    assert_eq!(schema.maximum, Some(99.9));
}

#[test]
fn test_boolean_schema() {
    let schema = Schema::boolean().with_default(json!(true));

    assert_eq!(schema.schema_type, SchemaType::Boolean);
    assert_eq!(schema.default, Some(json!(true)));
}

#[test]
fn test_array_schema() {
    let items = Schema::string();
    let schema = Schema::array(items);

    assert_eq!(schema.schema_type, SchemaType::Array);
    assert!(schema.items.is_some());
}

#[test]
fn test_object_schema() {
    let schema = Schema::object()
        .title("User")
        .property("id", Schema::integer())
        .property("name", Schema::string())
        .property("email", Schema::string().format("email"))
        .required(vec!["id", "name"]);

    assert_eq!(schema.schema_type, SchemaType::Object);
    assert_eq!(schema.title, Some("User".to_string()));
    assert!(schema.properties.is_some());

    let props = schema.properties.as_ref().unwrap();
    assert!(props.contains_key("id"));
    assert!(props.contains_key("name"));
    assert!(props.contains_key("email"));

    assert_eq!(schema.required, Some(vec!["id".to_string(), "name".to_string()]));
}

#[test]
fn test_enum_schema() {
    let enum_values = vec![json!("option1"), json!("option2"), json!("option3")];
    let schema = Schema::string().enum_values(enum_values.clone());

    assert_eq!(schema.enum_values, Some(enum_values));
}

#[test]
fn test_one_of_schema() {
    let schemas = vec![Schema::string(), Schema::integer()];
    let schema = Schema::default().one_of(schemas.clone());

    assert_eq!(schema.one_of, Some(schemas));
}

#[test]
fn test_any_of_schema() {
    let schemas = vec![Schema::string(), Schema::integer()];
    let schema = Schema::default().any_of(schemas.clone());

    assert_eq!(schema.any_of, Some(schemas));
}

#[test]
fn test_all_of_schema() {
    let schemas = vec![Schema::object().property("id", Schema::integer()), Schema::object().property("name", Schema::string())];
    let schema = Schema::default().all_of(schemas.clone());

    assert_eq!(schema.all_of, Some(schemas));
}

#[test]
fn test_schema_to_json() {
    let schema = Schema::string().title("Test");
    let json = schema.to_json_schema();
    assert!(json.is_object());
}

#[test]
fn test_schema_default() {
    let schema = Schema::default();
    assert_eq!(schema.schema_type, SchemaType::Object);
}

#[test]
fn test_to_schema_for_string() {
    let schema = String::schema();
    assert_eq!(schema.schema_type, SchemaType::String);
}

#[test]
fn test_to_schema_for_i64() {
    let schema = i64::schema();
    assert_eq!(schema.schema_type, SchemaType::Integer);
}

#[test]
fn test_to_schema_for_i32() {
    let schema = i32::schema();
    assert_eq!(schema.schema_type, SchemaType::Integer);
}

#[test]
fn test_to_schema_for_i16() {
    let schema = i16::schema();
    assert_eq!(schema.schema_type, SchemaType::Integer);
}

#[test]
fn test_to_schema_for_i8() {
    let schema = i8::schema();
    assert_eq!(schema.schema_type, SchemaType::Integer);
}

#[test]
fn test_to_schema_for_u64() {
    let schema = u64::schema();
    assert_eq!(schema.schema_type, SchemaType::Integer);
}

#[test]
fn test_to_schema_for_u32() {
    let schema = u32::schema();
    assert_eq!(schema.schema_type, SchemaType::Integer);
}

#[test]
fn test_to_schema_for_u16() {
    let schema = u16::schema();
    assert_eq!(schema.schema_type, SchemaType::Integer);
}

#[test]
fn test_to_schema_for_u8() {
    let schema = u8::schema();
    assert_eq!(schema.schema_type, SchemaType::Integer);
}

#[test]
fn test_to_schema_for_f64() {
    let schema = f64::schema();
    assert_eq!(schema.schema_type, SchemaType::Number);
}

#[test]
fn test_to_schema_for_f32() {
    let schema = f32::schema();
    assert_eq!(schema.schema_type, SchemaType::Number);
}

#[test]
fn test_to_schema_for_bool() {
    let schema = bool::schema();
    assert_eq!(schema.schema_type, SchemaType::Boolean);
}

#[test]
fn test_to_schema_for_vec() {
    let schema = Vec::<String>::schema();
    assert_eq!(schema.schema_type, SchemaType::Array);
}

#[test]
fn test_to_schema_for_option() {
    let schema = Option::<String>::schema();
    assert_eq!(schema.schema_type, SchemaType::String);
    assert_eq!(schema.nullable, true);
}

#[test]
fn test_to_schema_for_json_value() {
    let schema = serde_json::Value::schema();
    assert_eq!(schema.schema_type, SchemaType::Object);
}
