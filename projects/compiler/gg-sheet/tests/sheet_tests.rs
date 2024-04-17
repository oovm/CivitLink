use gg_sheet::{
    codegen::{self, CodegenConfig},
    compiler::SheetCompiler,
    reader::FileFormat,
    schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
    types::{DecimalKind, IntegerKind, SheetType, SheetValue, VectorKind},
};

use std::path::Path;

#[test]
fn test_parse_boolean_type() {
    assert_eq!(SheetType::parse("bool").unwrap(), SheetType::Boolean);
    assert_eq!(SheetType::parse("boolean").unwrap(), SheetType::Boolean);
}

#[test]
fn test_parse_integer_types() {
    assert_eq!(SheetType::parse("i8").unwrap(), SheetType::Integer(IntegerKind::Integer8));
    assert_eq!(SheetType::parse("i16").unwrap(), SheetType::Integer(IntegerKind::Integer16));
    assert_eq!(SheetType::parse("i32").unwrap(), SheetType::Integer(IntegerKind::Integer32));
    assert_eq!(SheetType::parse("i64").unwrap(), SheetType::Integer(IntegerKind::Integer64));
    assert_eq!(SheetType::parse("u8").unwrap(), SheetType::Integer(IntegerKind::Unsigned8));
    assert_eq!(SheetType::parse("u16").unwrap(), SheetType::Integer(IntegerKind::Unsigned16));
    assert_eq!(SheetType::parse("u32").unwrap(), SheetType::Integer(IntegerKind::Unsigned32));
    assert_eq!(SheetType::parse("u64").unwrap(), SheetType::Integer(IntegerKind::Unsigned64));
}

#[test]
fn test_parse_integer_aliases() {
    assert_eq!(SheetType::parse("int").unwrap(), SheetType::Integer(IntegerKind::Integer32));
    assert_eq!(SheetType::parse("long").unwrap(), SheetType::Integer(IntegerKind::Integer64));
    assert_eq!(SheetType::parse("byte").unwrap(), SheetType::Integer(IntegerKind::Unsigned8));
    assert_eq!(SheetType::parse("uint").unwrap(), SheetType::Integer(IntegerKind::Unsigned32));
}

#[test]
fn test_parse_decimal_types() {
    assert_eq!(SheetType::parse("f32").unwrap(), SheetType::Decimal(DecimalKind::Float32));
    assert_eq!(SheetType::parse("f64").unwrap(), SheetType::Decimal(DecimalKind::Float64));
    assert_eq!(SheetType::parse("float").unwrap(), SheetType::Decimal(DecimalKind::Float32));
    assert_eq!(SheetType::parse("double").unwrap(), SheetType::Decimal(DecimalKind::Float64));
}

#[test]
fn test_parse_string_type() {
    assert_eq!(SheetType::parse("string").unwrap(), SheetType::String);
    assert_eq!(SheetType::parse("str").unwrap(), SheetType::String);
    assert_eq!(SheetType::parse("text").unwrap(), SheetType::String);
}

#[test]
fn test_parse_color_type() {
    assert_eq!(SheetType::parse("color").unwrap(), SheetType::Color);
    assert_eq!(SheetType::parse("colour").unwrap(), SheetType::Color);
}

#[test]
fn test_parse_vector_types() {
    assert_eq!(SheetType::parse("vec2").unwrap(), SheetType::Vector(VectorKind::Vec2));
    assert_eq!(SheetType::parse("vec3").unwrap(), SheetType::Vector(VectorKind::Vec3));
    assert_eq!(SheetType::parse("vec4").unwrap(), SheetType::Vector(VectorKind::Vec4));
    assert_eq!(SheetType::parse("v2").unwrap(), SheetType::Vector(VectorKind::Vec2));
    assert_eq!(SheetType::parse("v3").unwrap(), SheetType::Vector(VectorKind::Vec3));
}

#[test]
fn test_parse_list_type() {
    let result = SheetType::parse("[string]").unwrap();
    assert_eq!(result, SheetType::List(Box::new(SheetType::String)));
}

#[test]
fn test_parse_map_type() {
    let result = SheetType::parse("Map<string, i32>").unwrap();
    match result {
        SheetType::Map { key, value } => {
            assert_eq!(*key, SheetType::String);
            assert_eq!(*value, SheetType::Integer(IntegerKind::Integer32));
        }
        _ => panic!("Expected Map type"),
    }
}

#[test]
fn test_parse_optional_type() {
    let result = SheetType::parse("i32?").unwrap();
    assert_eq!(result, SheetType::Optional(Box::new(SheetType::Integer(IntegerKind::Integer32))));
}

#[test]
fn test_parse_reference_type() {
    let result = SheetType::parse("&Item").unwrap();
    assert_eq!(result, SheetType::Reference("Item".to_string()));
}

#[test]
fn test_parse_enumerate_type() {
    let result = SheetType::parse("Quality").unwrap();
    assert_eq!(result, SheetType::Enumerate("Quality".to_string()));
}

#[test]
fn test_to_valkyrie_type() {
    assert_eq!(SheetType::Boolean.to_valkyrie_type(), "bool");
    assert_eq!(SheetType::Integer(IntegerKind::Integer32).to_valkyrie_type(), "i32");
    assert_eq!(SheetType::Decimal(DecimalKind::Float32).to_valkyrie_type(), "f32");
    assert_eq!(SheetType::String.to_valkyrie_type(), "string");
    assert_eq!(SheetType::Color.to_valkyrie_type(), "Color");
    assert_eq!(SheetType::Vector(VectorKind::Vec3).to_valkyrie_type(), "Vec3");
    assert_eq!(SheetType::List(Box::new(SheetType::String)).to_valkyrie_type(), "List<string>");
    assert_eq!(SheetType::Reference("Item".to_string()).to_valkyrie_type(), "i32");
    assert_eq!(SheetType::Enumerate("Quality".to_string()).to_valkyrie_type(), "Quality");
}

#[test]
fn test_parse_value_boolean() {
    assert_eq!(SheetValue::parse_from_str("true", &SheetType::Boolean).unwrap(), SheetValue::Boolean(true));
    assert_eq!(SheetValue::parse_from_str("false", &SheetType::Boolean).unwrap(), SheetValue::Boolean(false));
    assert_eq!(SheetValue::parse_from_str("1", &SheetType::Boolean).unwrap(), SheetValue::Boolean(true));
    assert_eq!(SheetValue::parse_from_str("0", &SheetType::Boolean).unwrap(), SheetValue::Boolean(false));
}

#[test]
fn test_parse_value_integer() {
    assert_eq!(
        SheetValue::parse_from_str("42", &SheetType::Integer(IntegerKind::Integer32)).unwrap(),
        SheetValue::Integer32(42)
    );
    assert_eq!(SheetValue::parse_from_str("0", &SheetType::Integer(IntegerKind::Integer32)).unwrap(), SheetValue::Integer32(0));
}

#[test]
fn test_parse_value_string() {
    assert_eq!(SheetValue::parse_from_str("hello", &SheetType::String).unwrap(), SheetValue::String("hello".to_string()));
}

#[test]
fn test_parse_value_optional() {
    let result =
        SheetValue::parse_from_str("", &SheetType::Optional(Box::new(SheetType::Integer(IntegerKind::Integer32)))).unwrap();
    assert_eq!(result, SheetValue::Optional(None));

    let result =
        SheetValue::parse_from_str("42", &SheetType::Optional(Box::new(SheetType::Integer(IntegerKind::Integer32)))).unwrap();
    assert_eq!(result, SheetValue::Optional(Some(Box::new(SheetValue::Integer32(42)))));
}

#[test]
fn test_table_kind_detect() {
    assert_eq!(TableKind::detect("id"), TableKind::List);
    assert_eq!(TableKind::detect("key"), TableKind::Dict);
    assert_eq!(TableKind::detect("enum"), TableKind::Enumerate);
    assert_eq!(TableKind::detect("name"), TableKind::List);
}

#[test]
fn test_file_format_detection() {
    assert_eq!(FileFormat::from_extension(Path::new("test.xlsx")).unwrap(), FileFormat::Xlsx);
    assert_eq!(FileFormat::from_extension(Path::new("test.csv")).unwrap(), FileFormat::Csv);
    assert_eq!(FileFormat::from_extension(Path::new("test.tsv")).unwrap(), FileFormat::Tsv);
    assert!(FileFormat::from_extension(Path::new("test.txt")).is_err());
}

#[test]
fn test_generate_list_table() {
    let table = SheetTable {
        name: "Item".to_string(),
        kind: TableKind::List,
        headers: vec![
            SheetHeader {
                column: 0,
                field_name: "id".to_string(),
                typing: SheetType::Integer(IntegerKind::Integer32),
                comment: "物品ID".to_string(),
                constraint: FieldConstraint::Primary,
            },
            SheetHeader {
                column: 1,
                field_name: "name".to_string(),
                typing: SheetType::String,
                comment: "物品名称".to_string(),
                constraint: FieldConstraint::None,
            },
        ],
        rows: vec![vec!["1".to_string(), "铁剑".to_string()], vec!["2".to_string(), "木盾".to_string()]],
    };

    let config = CodegenConfig::new(std::path::PathBuf::from("."));
    let result = codegen::generate_list_table(&table, &config).unwrap();

    assert!(result.contains("struct ItemRow"));
    assert!(result.contains("class ItemTable"));
    assert!(result.contains("id: i32"));
    assert!(result.contains("name: string"));
    assert!(result.contains("rows: Map<i32, ItemRow>"));
    assert!(result.contains("micro get(id: i32)"));
}

#[test]
fn test_generate_enum_table() {
    let table = SheetTable {
        name: "Quality".to_string(),
        kind: TableKind::Enumerate,
        headers: vec![
            SheetHeader {
                column: 0,
                field_name: "enum".to_string(),
                typing: SheetType::Enumerate("Quality".to_string()),
                comment: String::new(),
                constraint: FieldConstraint::None,
            },
            SheetHeader {
                column: 1,
                field_name: "id".to_string(),
                typing: SheetType::Integer(IntegerKind::Integer32),
                comment: String::new(),
                constraint: FieldConstraint::None,
            },
            SheetHeader {
                column: 2,
                field_name: "name".to_string(),
                typing: SheetType::String,
                comment: String::new(),
                constraint: FieldConstraint::None,
            },
        ],
        rows: vec![
            vec!["Common".to_string(), "1".to_string(), "普通".to_string()],
            vec!["Rare".to_string(), "3".to_string(), "稀有".to_string()],
        ],
    };

    let config = CodegenConfig::new(std::path::PathBuf::from("."));
    let result = codegen::generate_enum_table(&table, &config).unwrap();

    assert!(result.contains("enum Quality"));
    assert!(result.contains("Common = 1"));
    assert!(result.contains("Rare = 3"));
    assert!(result.contains("class QualityTable"));
    assert!(result.contains("name: Map<Quality, string>"));
}

#[test]
fn test_compiler_csv_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sheet_dir = temp_dir.path().join("sheet");
    let output_dir = temp_dir.path().join("output");
    std::fs::create_dir_all(&sheet_dir).unwrap();

    let csv_content = "id,name,level\ni32,string,i32\n物品ID,名称,等级\n1,铁剑,1\n2,木盾,2\n";
    std::fs::write(sheet_dir.join("Item.csv"), csv_content).unwrap();

    let mut compiler = SheetCompiler::new(&sheet_dir, &output_dir);
    compiler.compile().unwrap();

    let output_path = output_dir.join("ItemTable.v");
    assert!(output_path.exists());

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("struct ItemRow"));
    assert!(content.contains("class ItemTable"));
}

#[test]
fn test_compiler_incremental() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sheet_dir = temp_dir.path().join("sheet");
    let output_dir = temp_dir.path().join("output");
    std::fs::create_dir_all(&sheet_dir).unwrap();

    let csv_content = "id,name\ni32,string\n物品ID,名称\n1,铁剑\n";
    std::fs::write(sheet_dir.join("Item.csv"), csv_content).unwrap();

    let mut compiler = SheetCompiler::new(&sheet_dir, &output_dir);
    compiler.compile().unwrap();

    let first_content = std::fs::read_to_string(output_dir.join("ItemTable.v")).unwrap();

    compiler.compile().unwrap();

    let second_content = std::fs::read_to_string(output_dir.join("ItemTable.v")).unwrap();
    assert_eq!(first_content, second_content);
}
