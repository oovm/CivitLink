use gg_script::type_checker::{DiagnosticSeverity, TypeChecker, TypeInfo};
use oak_core::{Builder, SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

fn parse_and_check(source: &str) -> Vec<gg_script::type_checker::TypeDiagnostic> {
    let language = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&language);
    let source_text = SourceText::new(source);
    let mut cache = oak_core::parser::ParseSession::<ValkyrieLanguage>::default();
    let diagnostics = builder.build(&source_text, &[], &mut cache);

    match diagnostics.result {
        Ok(root) => {
            let mut type_checker = TypeChecker::new();
            type_checker.check_root(&root)
        }
        Err(e) => panic!("Parse error: {}", e),
    }
}

#[test]
fn test_bool_literal_type() {
    let source = r#"
        micro test() {
            let x = true
            let y = false
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for bool literals, got: {:?}", diags);
}

#[test]
fn test_string_literal_type() {
    let source = r#"
        micro test() {
            let x = "hello"
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for string literals, got: {:?}", diags);
}

#[test]
fn test_integer_literal_inferred_as_int() {
    let source = r#"
        micro test() {
            let x = 42
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for integer literals, got: {:?}", diags);
}

#[test]
fn test_float_literal_inferred_as_float() {
    let source = r#"
        micro test() {
            let x = 3.14
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for float literals, got: {:?}", diags);
}

#[test]
fn test_arithmetic_binary_int() {
    let source = r#"
        micro test() {
            let x = 10
            let y = 5
            let z = x + y
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for int arithmetic, got: {:?}", diags);
}

#[test]
fn test_arithmetic_binary_float() {
    let source = r#"
        micro test() {
            let x = 1.5
            let y = 2.5
            let z = x + y
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for float arithmetic, got: {:?}", diags);
}

#[test]
fn test_comparison_returns_bool() {
    let source = r#"
        micro test() {
            let x = 10
            let y = 5
            let z = x > y
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for comparison, got: {:?}", diags);
}

#[test]
fn test_logical_operators_require_bool() {
    let source = r#"
        micro test() {
            let x = true
            let y = false
            let z = x && y
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for logical ops on bools, got: {:?}", diags);
}

#[test]
fn test_logical_operator_with_non_bool() {
    let source = r#"
        micro test() {
            let x = 10
            let y = true
            let z = x && y
        }
    "#;

    let diags = parse_and_check(source);
    assert!(!diags.is_empty(), "Expected diagnostics for logical op with non-bool");
    let has_warning = diags.iter().any(|d| d.severity == DiagnosticSeverity::Warning && d.message.contains("Logical operator"));
    assert!(has_warning, "Expected logical operator warning");
}

#[test]
fn test_unary_negate_numeric() {
    let source = r#"
        micro test() {
            let x = 10
            let y = -x
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for negating int, got: {:?}", diags);
}

#[test]
fn test_unary_not_bool() {
    let source = r#"
        micro test() {
            let x = true
            let y = !x
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for NOT on bool, got: {:?}", diags);
}

#[test]
fn test_unary_not_non_bool() {
    let source = r#"
        micro test() {
            let x = 10
            let y = !x
        }
    "#;

    let diags = parse_and_check(source);
    assert!(!diags.is_empty(), "Expected diagnostics for NOT on non-bool");
    let has_warning = diags.iter().any(|d| d.severity == DiagnosticSeverity::Warning && d.message.contains("Logical NOT"));
    assert!(has_warning, "Expected logical NOT warning");
}

#[test]
fn test_if_condition_must_be_bool() {
    let source = r#"
        micro test() {
            if (10) {
                print("oops")
            }
        }
    "#;

    let diags = parse_and_check(source);
    assert!(!diags.is_empty(), "Expected diagnostics for non-bool if condition");
    let has_error = diags.iter().any(|d| d.severity == DiagnosticSeverity::Error && d.message.contains("If condition"));
    assert!(has_error, "Expected if condition error");
}

#[test]
fn test_if_condition_bool_ok() {
    let source = r#"
        micro test() {
            if (true) {
                print("ok")
            }
        }
    "#;

    let diags = parse_and_check(source);
    let if_errors: Vec<_> = diags.iter().filter(|d| d.message.contains("If condition")).collect();
    assert!(if_errors.is_empty(), "Expected no if condition errors for bool, got: {:?}", if_errors);
}

#[test]
fn test_function_signature_checking() {
    let source = r#"
        micro add(a, b) {
            return a + b
        }

        micro test() {
            let result = add(1, 2)
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "Expected no diagnostics for correct function call, got: {:?}", diags);
}

#[test]
fn test_function_wrong_arg_count() {
    let source = r#"
        micro add(a, b) {
            return a + b
        }

        micro test() {
            let result = add(1)
        }
    "#;

    let diags = parse_and_check(source);
    assert!(!diags.is_empty(), "Expected diagnostics for wrong argument count");
    let has_error = diags.iter().any(|d| d.severity == DiagnosticSeverity::Error && d.message.contains("arguments"));
    assert!(has_error, "Expected argument count error");
}

#[test]
fn test_gradual_typing_no_annotations_no_errors() {
    let source = r#"
        micro test() {
            let x = something_unknown
            let y = x + other_thing
        }
    "#;

    let diags = parse_and_check(source);
    let type_mismatch_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(type_mismatch_errors.is_empty(), "Gradual typing: no type errors for unknown variables, got: {:?}", type_mismatch_errors);
}

#[test]
fn test_gradual_typing_unknown_no_binary_error() {
    let source = r#"
        micro test() {
            let x = unknown_var
            let y = 10
            let z = x + y
        }
    "#;

    let diags = parse_and_check(source);
    let binary_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Cannot apply operator"))
        .collect();
    assert!(binary_errors.is_empty(), "Gradual typing: no binary op errors with Unknown, got: {:?}", binary_errors);
}

#[test]
fn test_type_mismatch_comparison() {
    let source = r#"
        micro test() {
            let x = 10
            let y = true
            let z = x > y
        }
    "#;

    let diags = parse_and_check(source);
    assert!(!diags.is_empty(), "Expected diagnostics for comparison between different types");
    let has_warning = diags.iter().any(|d| d.message.contains("Comparison between different types"));
    assert!(has_warning, "Expected comparison type mismatch warning");
}

#[test]
fn test_return_type_mismatch() {
    let source = r#"
        micro test() {
            return 10
        }
    "#;

    let diags = parse_and_check(source);
    assert!(diags.is_empty(), "No return type annotation means no return type error, got: {:?}", diags);
}

#[test]
fn test_namespace_items_checked() {
    let source = r#"
        namespace Game {
            micro start() {
                if (10) {
                    print("bad")
                }
            }
        }
    "#;

    let diags = parse_and_check(source);
    assert!(!diags.is_empty(), "Expected diagnostics for namespace inner items");
    let has_error = diags.iter().any(|d| d.message.contains("If condition"));
    assert!(has_error, "Expected if condition error in namespace");
}

#[test]
fn test_type_info_display() {
    assert_eq!(TypeInfo::Int.to_string(), "Int");
    assert_eq!(TypeInfo::Float.to_string(), "Float");
    assert_eq!(TypeInfo::Bool.to_string(), "Bool");
    assert_eq!(TypeInfo::String.to_string(), "String");
    assert_eq!(TypeInfo::Null.to_string(), "Null");
    assert_eq!(TypeInfo::Unknown.to_string(), "Unknown");

    let func_type = TypeInfo::Function {
        param_types: vec![TypeInfo::Int, TypeInfo::Int],
        return_type: Box::new(TypeInfo::Bool),
    };
    assert_eq!(func_type.to_string(), "(Int, Int) -> Bool");
}

#[test]
fn test_type_info_equality() {
    assert_eq!(TypeInfo::Int, TypeInfo::Int);
    assert_ne!(TypeInfo::Int, TypeInfo::Float);
    assert_ne!(TypeInfo::Unknown, TypeInfo::Int);

    let func1 = TypeInfo::Function {
        param_types: vec![TypeInfo::Int],
        return_type: Box::new(TypeInfo::Bool),
    };
    let func2 = TypeInfo::Function {
        param_types: vec![TypeInfo::Int],
        return_type: Box::new(TypeInfo::Bool),
    };
    assert_eq!(func1, func2);
}

#[test]
fn test_diagnostic_severity_display() {
    assert_eq!(DiagnosticSeverity::Warning.to_string(), "warning");
    assert_eq!(DiagnosticSeverity::Error.to_string(), "error");
}

#[test]
fn test_loop_condition_must_be_bool() {
    let source = r#"
        micro test() {
            let i = 0
            loop (10) {
                let i = i + 1
            }
        }
    "#;

    let diags = parse_and_check(source);
    assert!(!diags.is_empty(), "Expected diagnostics for non-bool loop condition");
    let has_error = diags.iter().any(|d| d.message.contains("Loop condition"));
    assert!(has_error, "Expected loop condition error");
}
