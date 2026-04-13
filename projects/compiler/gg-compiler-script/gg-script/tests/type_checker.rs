use std::collections::HashMap;

use gg_script::type_checker::{DiagnosticSeverity, FunctionSignature, TypeChecker, TypeDiagnostic, TypeEnvironment, TypeInfo};
use oak_valkyrie::{ast::*, lexer::token_type::ValkyrieTokenType};

fn make_identifier(name: &str) -> Identifier {
    Identifier { name: name.to_string(), span: Default::default() }
}

fn make_name_path(name: &str) -> NamePath {
    NamePath { parts: vec![make_identifier(name)], span: Default::default() }
}

fn make_int_literal(value: i64) -> TermExpression {
    TermExpression::StringLiteral(StringLiteral {
        segments: vec![StringSegment::Text(Box::new(TextSegment { content: value.to_string(), span: Default::default() }))],
        quote_count: 0,
        prefix: None,
        span: Default::default(),
    })
}

fn make_float_literal(value: f64) -> TermExpression {
    TermExpression::StringLiteral(StringLiteral {
        segments: vec![StringSegment::Text(Box::new(TextSegment { content: value.to_string(), span: Default::default() }))],
        quote_count: 0,
        prefix: None,
        span: Default::default(),
    })
}

fn make_bool_literal(value: bool) -> TermExpression {
    TermExpression::Bool { value, span: Default::default() }
}

fn make_string_literal(value: &str) -> TermExpression {
    TermExpression::StringLiteral(StringLiteral {
        segments: vec![StringSegment::Text(Box::new(TextSegment { content: value.to_string(), span: Default::default() }))],
        quote_count: 1,
        prefix: None,
        span: Default::default(),
    })
}

fn make_var_expr(name: &str) -> TermExpression {
    TermExpression::NamePath(Box::new(make_name_path(name)))
}

fn make_binary_expr(lhs: TermExpression, op: ValkyrieTokenType, rhs: TermExpression) -> TermExpression {
    TermExpression::Binary(Box::new(TermBinaryNode { lhs, operator: op, rhs, span: Default::default() }))
}

fn make_unary_expr(op: ValkyrieTokenType, base: TermExpression) -> TermExpression {
    TermExpression::Unary(Box::new(TermUnaryNode { operator: op, base, span: Default::default() }))
}

fn make_apply_call(callee: TermExpression, args: Vec<TermExpression>) -> TermExpression {
    TermExpression::ApplyCall { callee: Box::new(callee), args, span: Default::default() }
}

fn make_if_expr(condition: TermExpression, then_block: Block, else_block: Option<Block>) -> TermExpression {
    TermExpression::If {
        pattern: None,
        condition: Box::new(condition),
        then_branch: then_block,
        else_branch: else_block,
        span: Default::default(),
    }
}

fn make_return_expr(value: Option<TermExpression>) -> TermExpression {
    TermExpression::Return(Box::new(Return { base: value, span: Default::default() }))
}

fn make_let_stmt(name: &str, expr: TermExpression) -> Statement {
    Statement::Let(Let {
        is_mutable: false,
        pattern: Pattern::Variable(Box::new(VariablePattern { name: make_identifier(name), span: Default::default() })),
        ty: None,
        expr,
        annotations: Vec::new(),
        span: Default::default(),
    })
}

fn make_expr_stmt(expr: TermExpression) -> Statement {
    Statement::ExprStmt(ExprStmt { annotations: Vec::new(), expr, semi: false, span: Default::default() })
}

fn make_block(statements: Vec<Statement>) -> Block {
    Block { statements, span: Default::default() }
}

fn make_micro(name: &str, params: Vec<Param>, body: Block) -> MicroDeclaration {
    MicroDeclaration {
        name: make_identifier(name),
        generics: Vec::new(),
        annotations: Vec::new(),
        params,
        return_type: None,
        body,
        span: Default::default(),
        is_abstract: false,
        is_final: false,
    }
}

fn make_param(name: &str) -> Param {
    Param { name: make_identifier(name), ty: None, default: None, span: Default::default() }
}

fn make_typed_param(name: &str, type_name: &str) -> Param {
    Param {
        name: make_identifier(name),
        ty: Some(TypeExpression::Namepath(Box::new(make_name_path(type_name)))),
        default: None,
        span: Default::default(),
    }
}

fn make_micro_item(name: &str, params: Vec<Param>, body: Block) -> StatementNode {
    StatementNode::Micro(Box::new(make_micro(name, params, body)))
}

fn make_root(items: Vec<StatementNode>) -> ValkyrieRoot {
    ValkyrieRoot { items }
}

#[test]
fn test_bool_literal_inference() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&make_bool_literal(true));
    assert_eq!(ty, TypeInfo::Bool);
}

#[test]
fn test_string_literal_inference() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&make_string_literal("hello"));
    assert_eq!(ty, TypeInfo::String);
}

#[test]
fn test_int_literal_inference() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&make_int_literal(42));
    assert_eq!(ty, TypeInfo::Int);
}

#[test]
fn test_float_literal_inference() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&make_float_literal(3.14));
    assert_eq!(ty, TypeInfo::Float);
}

#[test]
fn test_variable_lookup() {
    let mut checker = TypeChecker::new();
    checker.env.insert_variable("x".to_string(), TypeInfo::Int);
    let ty = checker.infer_expr(&make_var_expr("x"));
    assert_eq!(ty, TypeInfo::Int);
}

#[test]
fn test_unknown_variable_returns_unknown() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&make_var_expr("unknown"));
    assert_eq!(ty, TypeInfo::Unknown);
}

#[test]
fn test_int_addition() {
    let mut checker = TypeChecker::new();
    let expr = make_binary_expr(make_int_literal(10), ValkyrieTokenType::Plus, make_int_literal(5));
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Int);
}

#[test]
fn test_float_addition() {
    let mut checker = TypeChecker::new();
    let expr = make_binary_expr(make_float_literal(1.5), ValkyrieTokenType::Plus, make_float_literal(2.5));
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Float);
}

fn make_type_expr(type_name: &str) -> TypeExpression {
    TypeExpression::Namepath(Box::new(make_name_path(type_name)))
}

fn make_field(name: &str, type_name: &str) -> FieldDeclaration {
    FieldDeclaration {
        name: make_identifier(name),
        ty: make_type_expr(type_name),
        default: None,
        annotations: Vec::new(),
        span: Default::default(),
    }
}

fn make_method(name: &str, params: Vec<Param>, return_type: Option<TypeExpression>, body: Option<Block>) -> MethodDeclaration {
    MethodDeclaration {
        name: make_identifier(name),
        generics: Vec::new(),
        params,
        return_type,
        body,
        annotations: Vec::new(),
        span: Default::default(),
    }
}

#[test]
fn test_check_class_registers_type() {
    let mut checker = TypeChecker::new();
    let class = ClassDeclaration {
        annotations: Vec::new(),
        name: make_identifier("Player"),
        generics: Vec::new(),
        parents: Vec::new(),
        fields: vec![make_field("health", "i32"), make_field("name", "String")],
        methods: Vec::new(),
        span: Default::default(),
    };
    checker.check_class(&class);
    assert_eq!(checker.env.lookup_type("Player"), Some(&TypeInfo::Object("Player".to_string())));
}

#[test]
fn test_check_class_registers_fields_during_method_check() {
    let mut checker = TypeChecker::new();
    let class = ClassDeclaration {
        annotations: Vec::new(),
        name: make_identifier("Player"),
        generics: Vec::new(),
        parents: Vec::new(),
        fields: vec![make_field("health", "i32")],
        methods: vec![make_method(
            "get_health",
            vec![],
            Some(make_type_expr("i32")),
            Some(make_block(vec![make_expr_stmt(make_var_expr("self.health"))])),
        )],
        span: Default::default(),
    };
    checker.check_class(&class);
    assert_eq!(checker.env.lookup_type("Player"), Some(&TypeInfo::Object("Player".to_string())));
    let sig = checker.env.lookup_function("get_health");
    assert!(sig.is_some());
}

#[test]
fn test_check_class_registers_methods() {
    let mut checker = TypeChecker::new();
    let class = ClassDeclaration {
        annotations: Vec::new(),
        name: make_identifier("Player"),
        generics: Vec::new(),
        parents: Vec::new(),
        fields: Vec::new(),
        methods: vec![make_method("take_damage", vec![make_typed_param("amount", "i32")], None, Some(make_block(vec![])))],
        span: Default::default(),
    };
    checker.check_class(&class);
    let sig = checker.env.lookup_function("take_damage");
    assert!(sig.is_some());
    let sig = sig.unwrap();
    assert_eq!(sig.param_types.len(), 1);
    assert_eq!(sig.param_types[0], TypeInfo::Int);
}

#[test]
fn test_check_trait_registers_type() {
    let mut checker = TypeChecker::new();
    let trait_decl = Trait {
        name: make_identifier("Serializable"),
        generics: Vec::new(),
        methods: Vec::new(),
        associated_types: Vec::new(),
        annotations: Vec::new(),
        span: Default::default(),
    };
    checker.check_trait(&trait_decl);
    assert_eq!(checker.env.lookup_type("Serializable"), Some(&TypeInfo::Trait("Serializable".to_string())));
}

#[test]
fn test_check_trait_registers_method_signatures() {
    let mut checker = TypeChecker::new();
    let trait_decl = Trait {
        name: make_identifier("Drawable"),
        generics: Vec::new(),
        methods: vec![make_method("draw", vec![make_typed_param("ctx", "String")], Some(make_type_expr("Bool")), None)],
        associated_types: Vec::new(),
        annotations: Vec::new(),
        span: Default::default(),
    };
    checker.check_trait(&trait_decl);
    let sig = checker.env.lookup_function("Drawable_draw");
    assert!(sig.is_some());
    let sig = sig.unwrap();
    assert_eq!(sig.param_types.len(), 1);
    assert_eq!(sig.param_types[0], TypeInfo::String);
    assert_eq!(sig.return_type, TypeInfo::Bool);
}

#[test]
fn test_check_enums_registers_type() {
    let mut checker = TypeChecker::new();
    let enums = Enums {
        kind: EnumsKind::Enum,
        name: make_identifier("Color"),
        generics: Vec::new(),
        variants: Vec::new(),
        annotations: Vec::new(),
        span: Default::default(),
    };
    checker.check_enums(&enums);
    assert_eq!(checker.env.lookup_type("Color"), Some(&TypeInfo::Object("Color".to_string())));
}

#[test]
fn test_check_component_registers_type() {
    let mut checker = TypeChecker::new();
    let component = ComponentDeclaration {
        annotations: Vec::new(),
        name: make_identifier("Position"),
        fields: vec![make_field("x", "f32"), make_field("y", "f32")],
        events: Vec::new(),
        span: Default::default(),
    };
    checker.check_component(&component);
    assert_eq!(checker.env.lookup_type("Position"), Some(&TypeInfo::Component("Position".to_string())));
}

#[test]
fn test_check_structure_registers_type() {
    let mut checker = TypeChecker::new();
    let structure = StructureDeclaration {
        name: make_identifier("Vec2"),
        generics: Vec::new(),
        parents: Vec::new(),
        fields: vec![make_field("x", "f32"), make_field("y", "f32")],
        annotations: Vec::new(),
        span: Default::default(),
    };
    checker.check_structure(&structure);
    assert_eq!(checker.env.lookup_type("Vec2"), Some(&TypeInfo::Object("Vec2".to_string())));
}

#[test]
fn test_check_singleton_registers_type() {
    let mut checker = TypeChecker::new();
    let singleton = SingletonDeclaration {
        name: make_identifier("GameConfig"),
        generics: Vec::new(),
        parents: Vec::new(),
        fields: Vec::new(),
        methods: Vec::new(),
        annotations: Vec::new(),
        span: Default::default(),
    };
    checker.check_singleton(&singleton);
    assert_eq!(checker.env.lookup_type("GameConfig"), Some(&TypeInfo::Object("GameConfig".to_string())));
}

#[test]
fn test_check_flags_registers_type() {
    let mut checker = TypeChecker::new();
    let flags =
        Flags { name: make_identifier("Permissions"), variants: Vec::new(), annotations: Vec::new(), span: Default::default() };
    checker.check_flags(&flags);
    assert_eq!(checker.env.lookup_type("Permissions"), Some(&TypeInfo::Object("Permissions".to_string())));
}

#[test]
fn test_check_system_checks_methods() {
    let mut checker = TypeChecker::new();
    let system = SystemDeclaration {
        annotations: Vec::new(),
        name: make_identifier("MovementSystem"),
        methods: vec![make_method("execute", vec![make_typed_param("world", "String")], None, Some(make_block(vec![])))],
        span: Default::default(),
    };
    checker.check_system(&system);
    let sig = checker.env.lookup_function("execute");
    assert!(sig.is_some());
    assert_eq!(sig.unwrap().param_types.len(), 1);
}

#[test]
fn test_check_root_with_class() {
    let mut checker = TypeChecker::new();
    let root = make_root(vec![StatementNode::Class(Box::new(ClassDeclaration {
        annotations: Vec::new(),
        name: make_identifier("Enemy"),
        generics: Vec::new(),
        parents: Vec::new(),
        fields: vec![make_field("hp", "i32")],
        methods: Vec::new(),
        span: Default::default(),
    }))]);
    let diags = checker.check_root(&root);
    assert!(diags.is_empty());
    assert_eq!(checker.env.lookup_type("Enemy"), Some(&TypeInfo::Object("Enemy".to_string())));
}

#[test]
fn test_check_root_with_enums_and_component() {
    let mut checker = TypeChecker::new();
    let root = make_root(vec![
        StatementNode::Enums(Box::new(Enums {
            kind: EnumsKind::Enum,
            name: make_identifier("Direction"),
            generics: Vec::new(),
            variants: Vec::new(),
            annotations: Vec::new(),
            span: Default::default(),
        })),
        StatementNode::Component(Box::new(ComponentDeclaration {
            annotations: Vec::new(),
            name: make_identifier("Velocity"),
            fields: vec![make_field("speed", "f32")],
            events: Vec::new(),
            span: Default::default(),
        })),
    ]);
    let diags = checker.check_root(&root);
    assert!(diags.is_empty());
    assert_eq!(checker.env.lookup_type("Direction"), Some(&TypeInfo::Object("Direction".to_string())));
    assert_eq!(checker.env.lookup_type("Velocity"), Some(&TypeInfo::Component("Velocity".to_string())));
}

#[test]
fn test_int_float_addition() {
    let mut checker = TypeChecker::new();
    let expr = make_binary_expr(make_int_literal(10), ValkyrieTokenType::Plus, make_float_literal(2.5));
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Float);
}

#[test]
fn test_comparison_returns_bool() {
    let mut checker = TypeChecker::new();
    let expr = make_binary_expr(make_int_literal(10), ValkyrieTokenType::GreaterThan, make_int_literal(5));
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Bool);
}

#[test]
fn test_logical_and_returns_bool() {
    let mut checker = TypeChecker::new();
    let expr = make_binary_expr(make_bool_literal(true), ValkyrieTokenType::AndAnd, make_bool_literal(false));
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Bool);
}

#[test]
fn test_logical_and_with_non_bool_warns() {
    let mut checker = TypeChecker::new();
    let expr = make_binary_expr(make_int_literal(10), ValkyrieTokenType::AndAnd, make_bool_literal(true));
    checker.infer_expr(&expr);
    let diags = &checker.diagnostics;
    assert!(!diags.is_empty());
    assert!(diags.iter().any(|d| d.message.contains("Logical operator") && d.severity == DiagnosticSeverity::Warning));
}

#[test]
fn test_unary_negate_int() {
    let mut checker = TypeChecker::new();
    let expr = make_unary_expr(ValkyrieTokenType::Minus, make_int_literal(10));
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Int);
}

#[test]
fn test_unary_negate_float() {
    let mut checker = TypeChecker::new();
    let expr = make_unary_expr(ValkyrieTokenType::Minus, make_float_literal(3.14));
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Float);
}

#[test]
fn test_unary_not_bool() {
    let mut checker = TypeChecker::new();
    let expr = make_unary_expr(ValkyrieTokenType::Bang, make_bool_literal(true));
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Bool);
}

#[test]
fn test_unary_not_non_bool_warns() {
    let mut checker = TypeChecker::new();
    let expr = make_unary_expr(ValkyrieTokenType::Bang, make_int_literal(10));
    checker.infer_expr(&expr);
    let diags = &checker.diagnostics;
    assert!(!diags.is_empty());
    assert!(diags.iter().any(|d| d.message.contains("Logical NOT") && d.severity == DiagnosticSeverity::Warning));
}

#[test]
fn test_if_condition_must_be_bool() {
    let mut checker = TypeChecker::new();
    let expr = make_if_expr(make_int_literal(10), make_block(vec![make_expr_stmt(make_bool_literal(true))]), None);
    checker.infer_expr(&expr);
    let diags = &checker.diagnostics;
    assert!(diags.iter().any(|d| d.message.contains("If condition") && d.severity == DiagnosticSeverity::Error));
}

#[test]
fn test_if_condition_bool_ok() {
    let mut checker = TypeChecker::new();
    let expr = make_if_expr(make_bool_literal(true), make_block(vec![make_expr_stmt(make_bool_literal(true))]), None);
    checker.infer_expr(&expr);
    let if_errors: Vec<_> = checker.diagnostics.iter().filter(|d| d.message.contains("If condition")).collect();
    assert!(if_errors.is_empty());
}

#[test]
fn test_function_call_correct_args() {
    let mut checker = TypeChecker::new();
    checker.env.insert_function(FunctionSignature {
        name: "add".to_string(),
        param_types: vec![TypeInfo::Int, TypeInfo::Int],
        return_type: TypeInfo::Int,
    });
    let expr = make_apply_call(make_var_expr("add"), vec![make_int_literal(1), make_int_literal(2)]);
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Int);
}

#[test]
fn test_function_call_wrong_arg_count() {
    let mut checker = TypeChecker::new();
    checker.env.insert_function(FunctionSignature {
        name: "add".to_string(),
        param_types: vec![TypeInfo::Int, TypeInfo::Int],
        return_type: TypeInfo::Int,
    });
    let expr = make_apply_call(make_var_expr("add"), vec![make_int_literal(1)]);
    checker.infer_expr(&expr);
    assert!(checker.diagnostics.iter().any(|d| d.message.contains("arguments") && d.severity == DiagnosticSeverity::Error));
}

#[test]
fn test_function_call_wrong_arg_type() {
    let mut checker = TypeChecker::new();
    checker.env.insert_function(FunctionSignature {
        name: "add".to_string(),
        param_types: vec![TypeInfo::Int, TypeInfo::Int],
        return_type: TypeInfo::Int,
    });
    let expr = make_apply_call(make_var_expr("add"), vec![make_int_literal(1), make_bool_literal(true)]);
    checker.infer_expr(&expr);
    assert!(checker.diagnostics.iter().any(|d| d.message.contains("expects type") && d.severity == DiagnosticSeverity::Error));
}

#[test]
fn test_gradual_typing_unknown_no_error() {
    let mut checker = TypeChecker::new();
    let expr = make_binary_expr(make_var_expr("unknown"), ValkyrieTokenType::Plus, make_int_literal(10));
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Unknown);
    let type_mismatch_diags: Vec<_> =
        checker.diagnostics.iter().filter(|d| d.message.contains("Cannot apply operator")).collect();
    assert!(type_mismatch_diags.is_empty());
}

#[test]
fn test_comparison_different_types_warns() {
    let mut checker = TypeChecker::new();
    let expr = make_binary_expr(make_int_literal(10), ValkyrieTokenType::GreaterThan, make_bool_literal(true));
    checker.infer_expr(&expr);
    assert!(checker.diagnostics.iter().any(|d| d.message.contains("Comparison between different types")));
}

#[test]
fn test_check_micro_function() {
    let mut checker = TypeChecker::new();
    let micro = make_micro(
        "test",
        vec![make_typed_param("x", "i32")],
        make_block(vec![make_expr_stmt(make_binary_expr(make_var_expr("x"), ValkyrieTokenType::Plus, make_int_literal(1)))]),
    );
    let diags = checker.check_micro(&micro);
    assert!(diags.is_empty(), "Expected no diagnostics, got: {:?}", diags);
}

#[test]
fn test_check_micro_if_condition_error() {
    let mut checker = TypeChecker::new();
    let micro = make_micro(
        "test",
        vec![],
        make_block(vec![make_expr_stmt(make_if_expr(
            make_int_literal(10),
            make_block(vec![make_expr_stmt(make_bool_literal(true))]),
            None,
        ))]),
    );
    let diags = checker.check_micro(&micro);
    assert!(diags.iter().any(|d| d.message.contains("If condition")));
}

#[test]
fn test_check_root_with_micro() {
    let mut checker = TypeChecker::new();
    let root = make_root(vec![make_micro_item(
        "test",
        vec![make_typed_param("x", "i32")],
        make_block(vec![make_expr_stmt(make_binary_expr(make_var_expr("x"), ValkyrieTokenType::Plus, make_int_literal(1)))]),
    )]);
    let diags = checker.check_root(&root);
    assert!(diags.is_empty(), "Expected no diagnostics, got: {:?}", diags);
}

#[test]
fn test_return_type_mismatch() {
    let mut checker = TypeChecker::new();
    let mut micro =
        make_micro("test", vec![], make_block(vec![make_expr_stmt(make_return_expr(Some(make_bool_literal(true))))]));
    micro.return_type = Some(TypeExpression::Namepath(Box::new(make_name_path("i32"))));
    let diags = checker.check_micro(&micro);
    assert!(diags.iter().any(|d| d.message.contains("Return type mismatch")));
}

#[test]
fn test_return_no_annotation_no_error() {
    let mut checker = TypeChecker::new();
    let micro = make_micro("test", vec![], make_block(vec![make_expr_stmt(make_return_expr(Some(make_int_literal(42))))]));
    let diags = checker.check_micro(&micro);
    assert!(diags.is_empty(), "Expected no diagnostics without return type annotation, got: {:?}", diags);
}

#[test]
fn test_type_info_display() {
    assert_eq!(TypeInfo::Int.to_string(), "Int");
    assert_eq!(TypeInfo::Float.to_string(), "Float");
    assert_eq!(TypeInfo::Bool.to_string(), "Bool");
    assert_eq!(TypeInfo::String.to_string(), "String");
    assert_eq!(TypeInfo::Null.to_string(), "Null");
    assert_eq!(TypeInfo::Unknown.to_string(), "Unknown");

    let func_type =
        TypeInfo::Function { param_types: vec![TypeInfo::Int, TypeInfo::Int], return_type: Box::new(TypeInfo::Bool) };
    assert_eq!(func_type.to_string(), "(Int, Int) -> Bool");
}

#[test]
fn test_type_info_equality() {
    assert_eq!(TypeInfo::Int, TypeInfo::Int);
    assert_ne!(TypeInfo::Int, TypeInfo::Float);
    assert_ne!(TypeInfo::Unknown, TypeInfo::Int);

    let func1 = TypeInfo::Function { param_types: vec![TypeInfo::Int], return_type: Box::new(TypeInfo::Bool) };
    let func2 = TypeInfo::Function { param_types: vec![TypeInfo::Int], return_type: Box::new(TypeInfo::Bool) };
    assert_eq!(func1, func2);
}

#[test]
fn test_diagnostic_severity_display() {
    assert_eq!(DiagnosticSeverity::Warning.to_string(), "warning");
    assert_eq!(DiagnosticSeverity::Error.to_string(), "error");
}

#[test]
fn test_type_environment() {
    let mut env = TypeEnvironment::new();
    assert!(env.lookup_variable("x").is_none());
    env.insert_variable("x".to_string(), TypeInfo::Int);
    assert_eq!(env.lookup_variable("x"), Some(&TypeInfo::Int));

    assert!(env.lookup_function("add").is_none());
    env.insert_function(FunctionSignature {
        name: "add".to_string(),
        param_types: vec![TypeInfo::Int, TypeInfo::Int],
        return_type: TypeInfo::Int,
    });
    assert!(env.lookup_function("add").is_some());
}

#[test]
fn test_type_diagnostic_display() {
    let diag = TypeDiagnostic {
        message: "Type mismatch".to_string(),
        severity: DiagnosticSeverity::Error,
        span_start: 10,
        span_end: 20,
        suggestion: None,
    };
    assert_eq!(diag.to_string(), "[error] Type mismatch (10-20)");
}

#[test]
fn test_type_diagnostic_display_with_suggestion() {
    let diag = TypeDiagnostic {
        message: "Type mismatch".to_string(),
        severity: DiagnosticSeverity::Error,
        span_start: 10,
        span_end: 20,
        suggestion: Some("Consider changing the type annotation to Int".to_string()),
    };
    assert_eq!(diag.to_string(), "[error] Type mismatch (10-20)\n  Suggestion: Consider changing the type annotation to Int");
}

#[test]
fn test_type_diagnostic_suggestion_field() {
    let diag = TypeDiagnostic {
        message: "test".to_string(),
        severity: DiagnosticSeverity::Warning,
        span_start: 0,
        span_end: 5,
        suggestion: Some("fix it".to_string()),
    };
    assert_eq!(diag.suggestion, Some("fix it".to_string()));

    let diag_no_suggestion = TypeDiagnostic {
        message: "test".to_string(),
        severity: DiagnosticSeverity::Warning,
        span_start: 0,
        span_end: 5,
        suggestion: None,
    };
    assert_eq!(diag_no_suggestion.suggestion, None);
}

#[test]
fn test_levenshtein_distance() {
    assert_eq!(TypeChecker::levenshtein_distance("", ""), 0);
    assert_eq!(TypeChecker::levenshtein_distance("abc", "abc"), 0);
    assert_eq!(TypeChecker::levenshtein_distance("abc", "abd"), 1);
    assert_eq!(TypeChecker::levenshtein_distance("kitten", "sitting"), 3);
    assert_eq!(TypeChecker::levenshtein_distance("score", "scor"), 1);
    assert_eq!(TypeChecker::levenshtein_distance("player", "playr"), 1);
}

#[test]
fn test_find_similar_names() {
    let candidates = vec!["score".to_string(), "scor".to_string(), "player".to_string(), "health".to_string()];
    let similar = TypeChecker::find_similar_names("scor", &candidates, 3);
    assert!(similar.contains(&"score".to_string()));
    assert!(!similar.contains(&"scor".to_string()));

    let similar2 = TypeChecker::find_similar_names("xyz", &candidates, 3);
    assert!(similar2.is_empty());
}

#[test]
fn test_find_similar_names_max_results() {
    let candidates = vec!["abc".to_string(), "abd".to_string(), "abe".to_string(), "abf".to_string()];
    let similar = TypeChecker::find_similar_names("abx", &candidates, 2);
    assert!(similar.len() <= 2);
}

#[test]
fn test_undefined_variable_diagnostic() {
    let mut checker = TypeChecker::new();
    checker.env.insert_variable("score".to_string(), TypeInfo::Int);
    let ty = checker.infer_expr(&make_var_expr("scor"));
    assert_eq!(ty, TypeInfo::Unknown);
    assert!(checker.diagnostics.iter().any(|d| d.message.contains("Undefined variable")));
}

#[test]
fn test_undefined_variable_suggestion() {
    let mut checker = TypeChecker::new();
    checker.env.insert_variable("score".to_string(), TypeInfo::Int);
    checker.env.insert_variable("player".to_string(), TypeInfo::Object("Player".to_string()));
    let ty = checker.infer_expr(&make_var_expr("scor"));
    assert_eq!(ty, TypeInfo::Unknown);
    let diag = checker.diagnostics.iter().find(|d| d.message.contains("Undefined variable: scor")).unwrap();
    assert!(diag.suggestion.is_some());
    let suggestion = diag.suggestion.as_ref().unwrap();
    assert!(suggestion.contains("score"));
}

#[test]
fn test_defined_variable_no_diagnostic() {
    let mut checker = TypeChecker::new();
    checker.env.insert_variable("x".to_string(), TypeInfo::Int);
    let ty = checker.infer_expr(&make_var_expr("x"));
    assert_eq!(ty, TypeInfo::Int);
    assert!(checker.diagnostics.is_empty());
}

#[test]
fn test_paren_expr() {
    let mut checker = TypeChecker::new();
    let expr = TermExpression::Paren { expr: Box::new(make_int_literal(42)), span: Default::default() };
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Int);
}

#[test]
fn test_null_literal() {
    let mut checker = TypeChecker::new();
    let expr = TermExpression::StringLiteral(StringLiteral {
        segments: vec![StringSegment::Text(Box::new(TextSegment { content: "null".to_string(), span: Default::default() }))],
        quote_count: 0,
        prefix: None,
        span: Default::default(),
    });
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Null);
}

#[test]
fn test_micro_lambda_type() {
    let mut checker = TypeChecker::new();
    let expr = TermExpression::Micro(AnonymousMicro {
        params: vec![make_typed_param("x", "i32")],
        return_type: Some(TypeExpression::Namepath(Box::new(make_name_path("Bool")))),
        body: make_block(vec![]),
        span: Default::default(),
    });
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Function { param_types: vec![TypeInfo::Int], return_type: Box::new(TypeInfo::Bool) });
}

#[test]
fn test_let_stmt_type_inference() {
    let mut checker = TypeChecker::new();
    let micro = make_micro(
        "test",
        vec![],
        make_block(vec![make_let_stmt("x", make_int_literal(42)), make_expr_stmt(make_var_expr("x"))]),
    );
    let diags = checker.check_micro(&micro);
    assert!(diags.is_empty(), "Expected no diagnostics, got: {:?}", diags);
}

#[test]
fn test_namespace_items_checked() {
    let mut checker = TypeChecker::new();
    let root = make_root(vec![StatementNode::Namespace(Box::new(NamespaceDeclaration {
        name: make_name_path("Game"),
        items: vec![StatementNode::Micro(Box::new(make_micro(
            "start",
            vec![],
            make_block(vec![make_expr_stmt(make_if_expr(
                make_int_literal(10),
                make_block(vec![make_expr_stmt(make_bool_literal(true))]),
                None,
            ))]),
        )))],
        annotations: Vec::new(),
        span: Default::default(),
    }))]);
    let diags = checker.check_root(&root);
    assert!(diags.iter().any(|d| d.message.contains("If condition")));
}

#[test]
fn test_array_type_display() {
    let ty = TypeInfo::Array(Box::new(TypeInfo::Int));
    assert_eq!(ty.to_string(), "Array<Int>");
}

#[test]
fn test_map_type_display() {
    let ty = TypeInfo::Map(Box::new(TypeInfo::String), Box::new(TypeInfo::Int));
    assert_eq!(ty.to_string(), "Map<String, Int>");
}

#[test]
fn test_object_type_display() {
    let ty = TypeInfo::Object("Player".to_string());
    assert_eq!(ty.to_string(), "Player");
}

#[test]
fn test_trait_type_display() {
    let ty = TypeInfo::Trait("Serializable".to_string());
    assert_eq!(ty.to_string(), "Serializable");
}

#[test]
fn test_component_type_display() {
    let ty = TypeInfo::Component("Position".to_string());
    assert_eq!(ty.to_string(), "Position");
}

#[test]
fn test_type_environment_register_and_lookup_type() {
    let mut env = TypeEnvironment::new();
    assert!(env.lookup_type("Player").is_none());
    env.register_type("Player".to_string(), TypeInfo::Object("Player".to_string()));
    assert_eq!(env.lookup_type("Player"), Some(&TypeInfo::Object("Player".to_string())));
}

#[test]
fn test_index_on_array_returns_element_type() {
    let mut checker = TypeChecker::new();
    checker.env.insert_variable("arr".to_string(), TypeInfo::Array(Box::new(TypeInfo::Int)));
    let expr = TermExpression::Index {
        receiver: Box::new(make_var_expr("arr")),
        index: Box::new(make_int_literal(0)),
        span: Default::default(),
    };
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Int);
}

#[test]
fn test_index_on_map_returns_value_type() {
    let mut checker = TypeChecker::new();
    checker.env.insert_variable("dict".to_string(), TypeInfo::Map(Box::new(TypeInfo::String), Box::new(TypeInfo::Float)));
    let expr = TermExpression::Index {
        receiver: Box::new(make_var_expr("dict")),
        index: Box::new(make_string_literal("key")),
        span: Default::default(),
    };
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Float);
}

#[test]
fn test_closure_captures_outer_variable() {
    let mut checker = TypeChecker::new();
    checker.env.insert_variable("x".to_string(), TypeInfo::Int);
    let expr = TermExpression::Micro(AnonymousMicro {
        params: vec![make_typed_param("y", "i32")],
        return_type: Some(TypeExpression::Namepath(Box::new(make_name_path("i32")))),
        body: make_block(vec![make_expr_stmt(make_binary_expr(
            make_var_expr("x"),
            ValkyrieTokenType::Plus,
            make_var_expr("y"),
        ))]),
        span: Default::default(),
    });
    let ty = checker.infer_expr(&expr);
    assert_eq!(
        ty,
        TypeInfo::Closure {
            param_types: vec![TypeInfo::Int],
            return_type: Box::new(TypeInfo::Int),
            captures: vec![("x".to_string(), TypeInfo::Int)],
        }
    );
}

#[test]
fn test_closure_no_captures_returns_function() {
    let mut checker = TypeChecker::new();
    let expr = TermExpression::Micro(AnonymousMicro {
        params: vec![make_typed_param("x", "i32")],
        return_type: Some(TypeExpression::Namepath(Box::new(make_name_path("i32")))),
        body: make_block(vec![make_expr_stmt(make_var_expr("x"))]),
        span: Default::default(),
    });
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Function { param_types: vec![TypeInfo::Int], return_type: Box::new(TypeInfo::Int) });
}

#[test]
fn test_closure_captures_multiple_variables() {
    let mut checker = TypeChecker::new();
    checker.env.insert_variable("a".to_string(), TypeInfo::Int);
    checker.env.insert_variable("b".to_string(), TypeInfo::Float);
    let expr = TermExpression::Micro(AnonymousMicro {
        params: vec![make_typed_param("x", "i32")],
        return_type: None,
        body: make_block(vec![
            make_expr_stmt(make_var_expr("a")),
            make_expr_stmt(make_var_expr("b")),
            make_expr_stmt(make_var_expr("x")),
        ]),
        span: Default::default(),
    });
    let ty = checker.infer_expr(&expr);
    match ty {
        TypeInfo::Closure { captures, .. } => {
            assert_eq!(captures.len(), 2);
            let capture_names: Vec<&str> = captures.iter().map(|(n, _)| n.as_str()).collect();
            assert!(capture_names.contains(&"a"));
            assert!(capture_names.contains(&"b"));
        }
        _ => panic!("Expected Closure type, got {:?}", ty),
    }
}

#[test]
fn test_closure_param_shadows_outer() {
    let mut checker = TypeChecker::new();
    checker.env.insert_variable("x".to_string(), TypeInfo::Float);
    let expr = TermExpression::Micro(AnonymousMicro {
        params: vec![make_typed_param("x", "i32")],
        return_type: Some(TypeExpression::Namepath(Box::new(make_name_path("i32")))),
        body: make_block(vec![make_expr_stmt(make_var_expr("x"))]),
        span: Default::default(),
    });
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Function { param_types: vec![TypeInfo::Int], return_type: Box::new(TypeInfo::Int) });
}

#[test]
fn test_collect_variable_references() {
    let expr = make_binary_expr(make_var_expr("x"), ValkyrieTokenType::Plus, make_var_expr("y"));
    let refs = TypeChecker::collect_variable_references(&expr);
    assert_eq!(refs, vec!["x", "y"]);
}

#[test]
fn test_collect_variable_references_nested() {
    let expr = TermExpression::Micro(AnonymousMicro {
        params: vec![make_typed_param("z", "i32")],
        return_type: None,
        body: make_block(vec![make_expr_stmt(make_binary_expr(
            make_var_expr("x"),
            ValkyrieTokenType::Plus,
            make_var_expr("z"),
        ))]),
        span: Default::default(),
    });
    let refs = TypeChecker::collect_variable_references(&expr);
    assert!(refs.contains(&"x".to_string()));
    assert!(refs.contains(&"z".to_string()));
}

#[test]
fn test_infer_closure_captures() {
    let mut outer_vars = HashMap::new();
    outer_vars.insert("x".to_string(), TypeInfo::Int);
    outer_vars.insert("y".to_string(), TypeInfo::Float);
    let referenced = vec!["x".to_string(), "y".to_string(), "z".to_string()];
    let param_names = vec!["z".to_string()];
    let captures = TypeChecker::infer_closure_captures(&referenced, &outer_vars, &param_names);
    assert_eq!(captures.len(), 2);
    let capture_names: Vec<&str> = captures.iter().map(|(n, _)| n.as_str()).collect();
    assert!(capture_names.contains(&"x"));
    assert!(capture_names.contains(&"y"));
}

#[test]
fn test_infer_closure_captures_dedup() {
    let mut outer_vars = HashMap::new();
    outer_vars.insert("x".to_string(), TypeInfo::Int);
    let referenced = vec!["x".to_string(), "x".to_string(), "x".to_string()];
    let param_names: Vec<String> = vec![];
    let captures = TypeChecker::infer_closure_captures(&referenced, &outer_vars, &param_names);
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0], ("x".to_string(), TypeInfo::Int));
}
