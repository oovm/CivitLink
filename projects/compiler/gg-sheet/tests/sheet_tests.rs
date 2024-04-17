use gg_sheet::{
    codegen::{self, CodegenConfig},
    compiler::SheetCompiler,
    reader::FileFormat,
    schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
    types::{DecimalKind, IntegerKind, SheetType, SheetValue, VectorKind, parse_color_str, parse_map_str, parse_vector_str},
    von_codegen,
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
fn test_parse_utf8_type() {
    assert_eq!(SheetType::parse("utf8").unwrap(), SheetType::String);
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
fn test_parse_list_generic_type() {
    let result = SheetType::parse("list<string>").unwrap();
    assert_eq!(result, SheetType::List(Box::new(SheetType::String)));
}

#[test]
fn test_parse_vec_type() {
    let result = SheetType::parse("Vec<string>").unwrap();
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
fn test_parse_hashmap_type() {
    let result = SheetType::parse("HashMap<string, i32>").unwrap();
    match result {
        SheetType::Map { key, value } => {
            assert_eq!(*key, SheetType::String);
            assert_eq!(*value, SheetType::Integer(IntegerKind::Integer32));
        }
        _ => panic!("Expected Map type"),
    }
}

#[test]
fn test_parse_dict_type() {
    let result = SheetType::parse("dict<string, i32>").unwrap();
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
    assert_eq!(SheetType::String.to_valkyrie_type(), "UTF8Text");
    assert_eq!(SheetType::Color.to_valkyrie_type(), "Color");
    assert_eq!(SheetType::Vector(VectorKind::Vec3).to_valkyrie_type(), "Vec3");
    assert_eq!(SheetType::List(Box::new(SheetType::String)).to_valkyrie_type(), "List<UTF8Text>");
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
                validation_rules: vec![],
                default_value: None,
            },
            SheetHeader {
                column: 1,
                field_name: "name".to_string(),
                typing: SheetType::String,
                comment: "物品名称".to_string(),
                constraint: FieldConstraint::None,
                validation_rules: vec![],
                default_value: None,
            },
        ],
        rows: vec![vec!["1".to_string(), "铁剑".to_string()], vec!["2".to_string(), "木盾".to_string()]],
    };

    let config = CodegenConfig::new(std::path::PathBuf::from("."));
    let result = codegen::generate_table(&table, &config).unwrap();

    assert!(result.contains("class Item"));
    assert!(result.contains("class ItemTable"));
    assert!(result.contains("id: i32"));
    assert!(result.contains("name: UTF8Text"));
    assert!(result.contains("private _data: HashMap<i32, Item>"));
    assert!(result.contains("micro find"));
    assert!(result.contains("micro all"));
    assert!(result.contains("namespace package::table"));
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
                validation_rules: vec![],
                default_value: None,
            },
            SheetHeader {
                column: 1,
                field_name: "id".to_string(),
                typing: SheetType::Integer(IntegerKind::Integer32),
                comment: String::new(),
                constraint: FieldConstraint::None,
                validation_rules: vec![],
                default_value: None,
            },
            SheetHeader {
                column: 2,
                field_name: "name".to_string(),
                typing: SheetType::String,
                comment: String::new(),
                constraint: FieldConstraint::None,
                validation_rules: vec![],
                default_value: None,
            },
        ],
        rows: vec![
            vec!["Common".to_string(), "1".to_string(), "普通".to_string()],
            vec!["Rare".to_string(), "3".to_string(), "稀有".to_string()],
        ],
    };

    let config = CodegenConfig::new(std::path::PathBuf::from("."));
    let result = codegen::generate_table(&table, &config).unwrap();

    assert!(result.contains("class Quality"));
    assert!(result.contains("class QualityTable"));
    assert!(result.contains("const QualityEnum"));
    assert!(result.contains("private _data: HashMap<UTF8Text, Quality>"));
    assert!(result.contains("private _values: List<Quality>"));
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

    let output_path = output_dir.join("ItemTable.script");
    assert!(output_path.exists());

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("class Item"));
    assert!(content.contains("class ItemTable"));

    let von_path = output_dir.join("ItemTable.von");
    assert!(von_path.exists());
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

    let first_content = std::fs::read_to_string(output_dir.join("ItemTable.script")).unwrap();

    compiler.compile().unwrap();

    let second_content = std::fs::read_to_string(output_dir.join("ItemTable.script")).unwrap();
    assert_eq!(first_content, second_content);
}

#[test]
fn test_generate_von_data() {
    let table = SheetTable {
        name: "Item".to_string(),
        kind: TableKind::List,
        headers: vec![
            SheetHeader {
                column: 0,
                field_name: "id".to_string(),
                typing: SheetType::Integer(IntegerKind::Integer32),
                comment: String::new(),
                constraint: FieldConstraint::Primary,
                validation_rules: vec![],
                default_value: None,
            },
            SheetHeader {
                column: 1,
                field_name: "name".to_string(),
                typing: SheetType::String,
                comment: String::new(),
                constraint: FieldConstraint::None,
                validation_rules: vec![],
                default_value: None,
            },
        ],
        rows: vec![vec!["1".to_string(), "铁剑".to_string()]],
    };

    let result = gg_sheet::von_codegen::generate_von_data(&table).unwrap();
    assert!(result.contains("ItemTable"));
    assert!(result.contains("_data"));
}

#[test]
fn test_table_kind_explicit_markers() {
    assert_eq!(TableKind::detect("@dict key"), TableKind::Dict);
    assert_eq!(TableKind::detect("@list id"), TableKind::List);
    assert_eq!(TableKind::detect("@enum name"), TableKind::Enumerate);
    assert_eq!(TableKind::detect("@class max_hp"), TableKind::Class);
    assert_eq!(TableKind::detect("@language key"), TableKind::Language);
}

#[test]
fn test_table_kind_implicit_detection() {
    assert_eq!(TableKind::detect("id"), TableKind::List);
    assert_eq!(TableKind::detect("key"), TableKind::Dict);
    assert_eq!(TableKind::detect("enum"), TableKind::Enumerate);
}

#[test]
fn test_extract_field_name_with_marker() {
    use gg_sheet::schema::extract_field_name;
    assert_eq!(extract_field_name("@dict key"), "key");
    assert_eq!(extract_field_name("@list id"), "id");
    assert_eq!(extract_field_name("@enum name"), "name");
    assert_eq!(extract_field_name("@class max_hp"), "max_hp");
    assert_eq!(extract_field_name("@language key"), "key");
}

#[test]
fn test_extract_field_name_without_marker() {
    use gg_sheet::schema::extract_field_name;
    assert_eq!(extract_field_name("id"), "id");
    assert_eq!(extract_field_name("key"), "key");
    assert_eq!(extract_field_name("@name"), "name");
    assert_eq!(extract_field_name("@@id"), "id");
}

#[test]
fn test_merge_tables_type_consistency() {
    use gg_sheet::merge::merge_tables;

    let table1 = SheetTable {
        name: "Item_Weapon".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "id".to_string(),
            typing: SheetType::Integer(IntegerKind::Integer32),
            comment: String::new(),
            constraint: FieldConstraint::Primary,
            validation_rules: vec![],
            default_value: None,
        }],
        rows: vec![vec!["1".to_string()]],
    };

    let table2 = SheetTable {
        name: "Item_Armor".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "id".to_string(),
            typing: SheetType::Integer(IntegerKind::Integer32),
            comment: String::new(),
            constraint: FieldConstraint::Primary,
            validation_rules: vec![],
            default_value: None,
        }],
        rows: vec![vec!["2".to_string()]],
    };

    let result = merge_tables(&[table1, table2]);
    assert!(result.is_ok());

    let merged = result.unwrap();
    let item_table = merged.iter().find(|t| t.name == "Item");
    assert!(item_table.is_some());
    assert_eq!(item_table.unwrap().rows.len(), 2);
}

#[test]
fn test_parse_validation_rules() {
    use gg_sheet::types::ValidationRule;
    assert_eq!(ValidationRule::Min(0.0).to_string(), "min(0)");
    assert_eq!(ValidationRule::Max(100.0).to_string(), "max(100)");
    assert_eq!(ValidationRule::MinLength(1).to_string(), "min_length(1)");
    assert_eq!(ValidationRule::MaxLength(50).to_string(), "max_length(50)");
    assert_eq!(ValidationRule::Pattern("\\d+".to_string()).to_string(), "pattern(\"\\d+\")");
    assert_eq!(ValidationRule::Required.to_string(), "required");
}

#[test]
fn test_sheet_header_with_validation_rules() {
    use gg_sheet::{reader::RawTable, schema::SheetTable, types::ValidationRule};

    let raw = RawTable {
        name: "Test".to_string(),
        path: std::path::PathBuf::from("Test.csv"),
        header_row: vec!["id".to_string(), "name".to_string()],
        type_row: vec!["i32 @min(0) @max(100)".to_string(), "string @required @max_length(50)".to_string()],
        comment_row: vec!["ID".to_string(), "名称".to_string()],
        data_rows: vec![],
    };

    let table = SheetTable::from_raw(raw).unwrap();
    assert_eq!(table.headers[0].validation_rules.len(), 2);
    assert_eq!(table.headers[0].validation_rules[0], ValidationRule::Min(0.0));
    assert_eq!(table.headers[0].validation_rules[1], ValidationRule::Max(100.0));
    assert_eq!(table.headers[1].validation_rules.len(), 2);
    assert_eq!(table.headers[1].validation_rules[0], ValidationRule::Required);
    assert_eq!(table.headers[1].validation_rules[1], ValidationRule::MaxLength(50));
}

#[test]
fn test_sheet_header_with_default_value() {
    use gg_sheet::{reader::RawTable, schema::SheetTable};

    let raw = RawTable {
        name: "Test".to_string(),
        path: std::path::PathBuf::from("Test.csv"),
        header_row: vec!["level".to_string()],
        type_row: vec!["i32 = 1".to_string()],
        comment_row: vec!["等级".to_string()],
        data_rows: vec![],
    };

    let table = SheetTable::from_raw(raw).unwrap();
    assert_eq!(table.headers[0].default_value, Some("1".to_string()));
}

#[test]
fn test_validation_rule_min_max() {
    use gg_sheet::{
        schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
        types::{IntegerKind, SheetType, ValidationRule},
        validate::validate_table,
    };

    let table = SheetTable {
        name: "Test".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "level".to_string(),
            typing: SheetType::Integer(IntegerKind::Integer32),
            comment: String::new(),
            constraint: FieldConstraint::None,
            validation_rules: vec![ValidationRule::Min(1.0), ValidationRule::Max(100.0)],
            default_value: None,
        }],
        rows: vec![vec!["50".to_string()], vec!["0".to_string()], vec!["101".to_string()]],
    };

    let report = validate_table(&table, None);
    assert_eq!(report.error_count(), 2);
}

#[test]
fn test_validation_rule_required() {
    use gg_sheet::{
        schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
        types::{SheetType, ValidationRule},
        validate::validate_table,
    };

    let table = SheetTable {
        name: "Test".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "name".to_string(),
            typing: SheetType::String,
            comment: String::new(),
            constraint: FieldConstraint::None,
            validation_rules: vec![ValidationRule::Required],
            default_value: None,
        }],
        rows: vec![vec!["hello".to_string()], vec!["".to_string()]],
    };

    let report = validate_table(&table, None);
    assert_eq!(report.error_count(), 1);
}

#[test]
fn test_validation_rule_min_max_length() {
    use gg_sheet::{
        schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
        types::{SheetType, ValidationRule},
        validate::validate_table,
    };

    let table = SheetTable {
        name: "Test".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "name".to_string(),
            typing: SheetType::String,
            comment: String::new(),
            constraint: FieldConstraint::None,
            validation_rules: vec![ValidationRule::MinLength(2), ValidationRule::MaxLength(10)],
            default_value: None,
        }],
        rows: vec![vec!["ok".to_string()], vec!["a".to_string()], vec!["very long name".to_string()]],
    };

    let report = validate_table(&table, None);
    assert_eq!(report.error_count(), 2);
}

#[test]
fn test_validation_rule_pattern() {
    use gg_sheet::{
        schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
        types::{SheetType, ValidationRule},
        validate::validate_table,
    };

    let table = SheetTable {
        name: "Test".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "code".to_string(),
            typing: SheetType::String,
            comment: String::new(),
            constraint: FieldConstraint::None,
            validation_rules: vec![ValidationRule::Pattern(r"^\d{4}$".to_string())],
            default_value: None,
        }],
        rows: vec![vec!["1234".to_string()], vec!["abc".to_string()]],
    };

    let report = validate_table(&table, None);
    assert_eq!(report.error_count(), 1);
}

#[test]
fn test_cross_table_reference_valid() {
    use gg_sheet::{
        schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
        types::{IntegerKind, SheetType},
        validate::validate_tables_with_references,
    };

    let item_table = SheetTable {
        name: "Item".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "id".to_string(),
            typing: SheetType::Integer(IntegerKind::Integer32),
            comment: String::new(),
            constraint: FieldConstraint::Primary,
            validation_rules: vec![],
            default_value: None,
        }],
        rows: vec![vec!["1".to_string()], vec!["2".to_string()], vec!["3".to_string()]],
    };

    let equip_table = SheetTable {
        name: "Equipment".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "item_id".to_string(),
            typing: SheetType::Reference("Item".to_string()),
            comment: String::new(),
            constraint: FieldConstraint::None,
            validation_rules: vec![],
            default_value: None,
        }],
        rows: vec![vec!["1".to_string()], vec!["2".to_string()]],
    };

    let report = validate_tables_with_references(&[item_table, equip_table]);
    assert!(report.is_valid(), "Expected no reference integrity errors, got: {:?}", report.errors);
}

#[test]
fn test_cross_table_reference_invalid() {
    use gg_sheet::{
        schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
        types::{IntegerKind, SheetType},
        validate::{ValidationError, validate_tables_with_references},
    };

    let item_table = SheetTable {
        name: "Item".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "id".to_string(),
            typing: SheetType::Integer(IntegerKind::Integer32),
            comment: String::new(),
            constraint: FieldConstraint::Primary,
            validation_rules: vec![],
            default_value: None,
        }],
        rows: vec![vec!["1".to_string()], vec!["2".to_string()]],
    };

    let equip_table = SheetTable {
        name: "Equipment".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "item_id".to_string(),
            typing: SheetType::Reference("Item".to_string()),
            comment: String::new(),
            constraint: FieldConstraint::None,
            validation_rules: vec![],
            default_value: None,
        }],
        rows: vec![vec!["1".to_string()], vec!["999".to_string()]],
    };

    let report = validate_tables_with_references(&[item_table, equip_table]);
    assert!(!report.is_valid());

    let ref_errors: Vec<&ValidationError> =
        report.errors.iter().filter(|e| matches!(e, ValidationError::ReferenceIntegrity { .. })).collect();
    assert_eq!(ref_errors.len(), 1);
}

#[test]
fn test_cross_table_reference_missing_table() {
    use gg_sheet::{
        schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
        types::SheetType,
        validate::{ValidationError, validate_tables_with_references},
    };

    let equip_table = SheetTable {
        name: "Equipment".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "item_id".to_string(),
            typing: SheetType::Reference("NonExistent".to_string()),
            comment: String::new(),
            constraint: FieldConstraint::None,
            validation_rules: vec![],
            default_value: None,
        }],
        rows: vec![vec!["1".to_string()]],
    };

    let report = validate_tables_with_references(&[equip_table]);
    assert!(!report.is_valid());

    let ref_errors: Vec<&ValidationError> =
        report.errors.iter().filter(|e| matches!(e, ValidationError::ReferenceIntegrity { .. })).collect();
    assert_eq!(ref_errors.len(), 1);
    assert!(ref_errors[0].to_string().contains("NonExistent"));
}

#[test]
fn test_dependency_graph() {
    use gg_sheet::{
        dependency::SheetDependencyGraph,
        schema::{FieldConstraint, SheetHeader, SheetTable, TableKind},
        types::{IntegerKind, SheetType},
    };
    use std::collections::HashSet;

    let item_table = SheetTable {
        name: "Item".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "id".to_string(),
            typing: SheetType::Integer(IntegerKind::Integer32),
            comment: String::new(),
            constraint: FieldConstraint::Primary,
            validation_rules: vec![],
            default_value: None,
        }],
        rows: vec![],
    };

    let equip_table = SheetTable {
        name: "Equipment".to_string(),
        kind: TableKind::List,
        headers: vec![SheetHeader {
            column: 0,
            field_name: "item_id".to_string(),
            typing: SheetType::Reference("Item".to_string()),
            comment: String::new(),
            constraint: FieldConstraint::None,
            validation_rules: vec![],
            default_value: None,
        }],
        rows: vec![],
    };

    let graph = SheetDependencyGraph::build_from_tables(&[item_table, equip_table]);

    let deps = graph.dependencies_of("Equipment");
    assert!(deps.contains("Item"));

    let dependents = graph.dependents_of("Item");
    assert!(dependents.contains("Equipment"));

    let affected = graph.affected_tables(&HashSet::from(["Item".to_string()]));
    assert!(affected.contains("Equipment"));
}

#[test]
fn test_cache_save_load() {
    use gg_sheet::{cache::SheetCache, dependency::SheetDependencyGraph};

    let temp_dir = tempfile::tempdir().unwrap();

    let mut cache = SheetCache::new();
    cache.file_hashes.insert(std::path::PathBuf::from("test.csv"), 12345);
    cache.dependency_graph = SheetDependencyGraph::build_from_tables(&[]);

    cache.save(temp_dir.path()).unwrap();

    let loaded = SheetCache::load(temp_dir.path());
    assert_eq!(loaded.file_hashes.get(&std::path::PathBuf::from("test.csv")), Some(&12345));
}

#[test]
fn test_cache_binary_save_load() {
    use gg_sheet::{cache::SheetCache, dependency::SheetDependencyGraph};

    let temp_dir = tempfile::tempdir().unwrap();

    let mut cache = SheetCache::new();
    cache.file_hashes.insert(std::path::PathBuf::from("test.csv"), 12345);
    cache.file_hashes.insert(std::path::PathBuf::from("data/item.csv"), 67890);
    cache.dependency_graph = SheetDependencyGraph::build_from_tables(&[]);

    cache.save(temp_dir.path()).unwrap();

    let cache_bytes = std::fs::read(temp_dir.path().join(".sheet-cache")).unwrap();
    assert!(cache_bytes.len() >= 16);
    assert_eq!(&cache_bytes[..8], b"GGSHEET\0");

    let loaded = SheetCache::load(temp_dir.path());
    assert_eq!(loaded.file_hashes.get(&std::path::PathBuf::from("test.csv")), Some(&12345));
    assert_eq!(loaded.file_hashes.get(&std::path::PathBuf::from("data/item.csv")), Some(&67890));
}

#[test]
fn test_cache_crc32_validation() {
    use gg_sheet::{cache::SheetCache, dependency::SheetDependencyGraph};

    let temp_dir = tempfile::tempdir().unwrap();

    let mut cache = SheetCache::new();
    cache.file_hashes.insert(std::path::PathBuf::from("test.csv"), 12345);
    cache.dependency_graph = SheetDependencyGraph::build_from_tables(&[]);

    cache.save(temp_dir.path()).unwrap();

    let cache_path = temp_dir.path().join(".sheet-cache");
    let mut data = std::fs::read(&cache_path).unwrap();
    let corrupt_offset = data.len() / 2;
    data[corrupt_offset] ^= 0xFF;
    std::fs::write(&cache_path, &data).unwrap();

    let loaded = SheetCache::load(temp_dir.path());
    assert!(loaded.file_hashes.is_empty());
}

#[test]
fn test_cache_version_mismatch() {
    use gg_sheet::{cache::SheetCache, dependency::SheetDependencyGraph};

    let temp_dir = tempfile::tempdir().unwrap();

    let mut cache = SheetCache::new();
    cache.file_hashes.insert(std::path::PathBuf::from("test.csv"), 12345);
    cache.dependency_graph = SheetDependencyGraph::build_from_tables(&[]);

    cache.save(temp_dir.path()).unwrap();

    let cache_path = temp_dir.path().join(".sheet-cache");
    let mut data = std::fs::read(&cache_path).unwrap();
    let wrong_version: u32 = 999;
    data[8..12].copy_from_slice(&wrong_version.to_le_bytes());
    std::fs::write(&cache_path, &data).unwrap();

    let loaded = SheetCache::load(temp_dir.path());
    assert!(loaded.file_hashes.is_empty());
}

#[test]
fn test_cache_legacy_format_compat() {
    use gg_sheet::cache::SheetCache;

    let temp_dir = tempfile::tempdir().unwrap();

    let legacy_content = "[hashes]\ntest.csv:12345\ndata/item.csv:67890\n[dependencies]\nEquipment:Item\n";
    let cache_path = temp_dir.path().join(".sheet-cache");
    std::fs::write(&cache_path, legacy_content).unwrap();

    let loaded = SheetCache::load(temp_dir.path());
    assert_eq!(loaded.file_hashes.get(&std::path::PathBuf::from("test.csv")), Some(&12345));
    assert_eq!(loaded.file_hashes.get(&std::path::PathBuf::from("data/item.csv")), Some(&67890));
    assert!(loaded.dependency_graph.dependencies_of("Equipment").contains("Item"));
}

#[test]
fn test_incremental_compile() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sheet_dir = temp_dir.path().join("sheet");
    let output_dir = temp_dir.path().join("output");
    std::fs::create_dir_all(&sheet_dir).unwrap();

    let csv_content = "id,name\ni32,string\n物品ID,名称\n1,铁剑\n";
    std::fs::write(sheet_dir.join("Item.csv"), csv_content).unwrap();

    let mut compiler = SheetCompiler::new(&sheet_dir, &output_dir);
    compiler.compile().unwrap();

    let first_script = std::fs::read_to_string(output_dir.join("ItemTable.script")).unwrap();

    compiler.incremental_compile().unwrap();

    let second_script = std::fs::read_to_string(output_dir.join("ItemTable.script")).unwrap();
    assert_eq!(first_script, second_script);

    assert!(output_dir.join(".sheet-cache").exists());
}

#[test]
fn test_sheet_transformer() {
    use gg_compiler::{
        artifact::ArtifactSet,
        context::{BuildConfig, BuildContext},
        transformer::Transformer,
    };
    use gg_sheet::transformer::SheetTransformer;

    let temp_dir = tempfile::tempdir().unwrap();
    let sheet_dir = temp_dir.path().join("sheet");
    let output_dir = temp_dir.path().join("output");
    std::fs::create_dir_all(&sheet_dir).unwrap();

    let csv_content = "id,name\ni32,string\n物品ID,名称\n1,铁剑\n";
    std::fs::write(sheet_dir.join("Item.csv"), csv_content).unwrap();

    let transformer = SheetTransformer::new(&sheet_dir, &output_dir);
    assert_eq!(transformer.name(), "sheet");

    let input_keys = transformer.input_keys();
    assert_eq!(input_keys.len(), 1);
    assert_eq!(input_keys[0].type_name, "sheet_source");

    let output_keys = transformer.output_keys();
    assert_eq!(output_keys.len(), 2);

    let mut context = BuildContext::new(BuildConfig::new());
    let result = transformer.transform(&ArtifactSet::new(), &mut context);
    assert!(result.is_ok());

    let artifacts = result.unwrap();
    assert!(!artifacts.is_empty());
}

#[test]
fn test_parse_value_color_hex6() {
    let result = SheetValue::parse_from_str("#FF0000", &SheetType::Color).unwrap();
    assert_eq!(result, SheetValue::Color { r: 255, g: 0, b: 0, a: 255 });
}

#[test]
fn test_parse_value_color_hex8() {
    let result = SheetValue::parse_from_str("#FF000080", &SheetType::Color).unwrap();
    assert_eq!(result, SheetValue::Color { r: 255, g: 0, b: 0, a: 128 });
}

#[test]
fn test_parse_value_color_rgb() {
    let result = SheetValue::parse_from_str("rgb(255, 0, 0)", &SheetType::Color).unwrap();
    assert_eq!(result, SheetValue::Color { r: 255, g: 0, b: 0, a: 255 });
}

#[test]
fn test_parse_value_color_rgba() {
    let result = SheetValue::parse_from_str("rgba(255, 0, 0, 128)", &SheetType::Color).unwrap();
    assert_eq!(result, SheetValue::Color { r: 255, g: 0, b: 0, a: 128 });
}

#[test]
fn test_parse_value_color_invalid() {
    let result = SheetValue::parse_from_str("invalid", &SheetType::Color);
    assert!(result.is_err());
}

#[test]
fn test_parse_value_color_empty() {
    let result = SheetValue::parse_from_str("", &SheetType::Color).unwrap();
    assert_eq!(result, SheetValue::Color { r: 0, g: 0, b: 0, a: 255 });
}

#[test]
fn test_format_color_field_value() {
    let result = codegen::format_field_value("#FF0000", &SheetType::Color);
    assert_eq!(result, "Color(\"#FF0000FF\")");
}

#[test]
fn test_format_color_von_value() {
    let result = von_codegen::format_von_value("#FF0000", &SheetType::Color);
    assert_eq!(result, "Color { r: 255, g: 0, b: 0, a: 255 }");
}

#[test]
fn test_parse_value_vec2() {
    let result = SheetValue::parse_from_str("(1.0, 2.0)", &SheetType::Vector(VectorKind::Vec2)).unwrap();
    assert_eq!(result, SheetValue::Vector { components: vec![1.0, 2.0] });
}

#[test]
fn test_parse_value_vec3() {
    let result = SheetValue::parse_from_str("(1.0, 2.0, 3.0)", &SheetType::Vector(VectorKind::Vec3)).unwrap();
    assert_eq!(result, SheetValue::Vector { components: vec![1.0, 2.0, 3.0] });
}

#[test]
fn test_parse_value_vec4() {
    let result = SheetValue::parse_from_str("(1.0, 2.0, 3.0, 4.0)", &SheetType::Vector(VectorKind::Vec4)).unwrap();
    assert_eq!(result, SheetValue::Vector { components: vec![1.0, 2.0, 3.0, 4.0] });
}

#[test]
fn test_parse_value_vec_mismatch() {
    let result = SheetValue::parse_from_str("(1.0, 2.0)", &SheetType::Vector(VectorKind::Vec3));
    assert!(result.is_err());
}

#[test]
fn test_parse_value_vec_empty() {
    let result = SheetValue::parse_from_str("", &SheetType::Vector(VectorKind::Vec3)).unwrap();
    assert_eq!(result, SheetValue::Vector { components: vec![0.0, 0.0, 0.0] });
}

#[test]
fn test_format_vector_field_value() {
    let result = codegen::format_field_value("(1.0, 2.0, 3.0)", &SheetType::Vector(VectorKind::Vec3));
    assert_eq!(result, "Vec3(1.0, 2.0, 3.0)");
}

#[test]
fn test_format_vector_von_value() {
    let result = von_codegen::format_von_value("(1.0, 2.0, 3.0)", &SheetType::Vector(VectorKind::Vec3));
    assert_eq!(result, "[1.0, 2.0, 3.0]");
}

#[test]
fn test_parse_value_map_string_int() {
    let map_type =
        SheetType::Map { key: Box::new(SheetType::String), value: Box::new(SheetType::Integer(IntegerKind::Integer32)) };
    let result = SheetValue::parse_from_str("{hp:100,mp:50}", &map_type).unwrap();
    match result {
        SheetValue::Map(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "hp");
            assert_eq!(entries[0].1, SheetValue::Integer32(100));
            assert_eq!(entries[1].0, "mp");
            assert_eq!(entries[1].1, SheetValue::Integer32(50));
        }
        _ => panic!("Expected Map value"),
    }
}

#[test]
fn test_parse_value_map_string_string() {
    let map_type = SheetType::Map { key: Box::new(SheetType::String), value: Box::new(SheetType::String) };
    let result = SheetValue::parse_from_str("{name:\"Hero\"}", &map_type).unwrap();
    match result {
        SheetValue::Map(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "name");
            assert_eq!(entries[0].1, SheetValue::String("Hero".to_string()));
        }
        _ => panic!("Expected Map value"),
    }
}

#[test]
fn test_parse_value_map_empty() {
    let map_type =
        SheetType::Map { key: Box::new(SheetType::String), value: Box::new(SheetType::Integer(IntegerKind::Integer32)) };
    let result = SheetValue::parse_from_str("", &map_type).unwrap();
    assert_eq!(result, SheetValue::Map(Vec::new()));

    let result = SheetValue::parse_from_str("{}", &map_type).unwrap();
    assert_eq!(result, SheetValue::Map(Vec::new()));
}

#[test]
fn test_format_map_field_value() {
    let map_type =
        SheetType::Map { key: Box::new(SheetType::String), value: Box::new(SheetType::Integer(IntegerKind::Integer32)) };
    let result = codegen::format_field_value("{hp:100,mp:50}", &map_type);
    assert_eq!(result, "Map<UTF8Text, i32> { \"hp\": 100, \"mp\": 50 }");
}

#[test]
fn test_format_map_von_value() {
    let map_type =
        SheetType::Map { key: Box::new(SheetType::String), value: Box::new(SheetType::Integer(IntegerKind::Integer32)) };
    let result = von_codegen::format_von_value("{hp:100,mp:50}", &map_type);
    assert_eq!(result, "{ \"hp\": 100, \"mp\": 50 }");
}
