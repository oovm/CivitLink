use gg_script::type_checker::{
    DiagnosticSeverity, FunctionSignature, TypeChecker, TypeDiagnostic, TypeEnvironment, TypeInfo,
};
use oak_valkyrie::ast::*;
use oak_valkyrie::lexer::token_type::ValkyrieTokenType;

fn make_identifier(name: &str) -> Identifier {
    Identifier { name: name.to_string(), span: Default::default() }
}

fn make_name_path(name: &str) -> NamePath {
    NamePath { parts: vec![make_identifier(name)], span: Default::default() }
}

fn make_int_literal(value: i64) -> TermExpression {
    TermExpression::StringLiteral(StringLiteral {
        segments: vec![StringSegment::Text(Box::new(TextSegment {
            content: value.to_string(),
            span: Default::default(),
        }))],
        quote_count: 0,
        prefix: None,
        span: Default::default(),
    })
}

fn make_float_literal(value: f64) -> TermExpression {
    TermExpression::StringLiteral(StringLiteral {
        segments: vec![StringSegment::Text(Box::new(TextSegment {
            content: value.to_string(),
            span: Default::default(),
        }))],
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
        segments: vec![StringSegment::Text(Box::new(TextSegment {
            content: value.to_string(),
            span: Default::default(),
        }))],
        quote_count: 1,
        prefix: None,
        span: Default::default(),
    })
}

fn make_var_expr(name: &str) -> TermExpression {
    TermExpression::NamePath(Box::new(make_name_path(name)))
}

fn make_binary_expr(lhs: TermExpression, op: ValkyrieTokenType, rhs: TermExpression) -> TermExpression {
    TermExpression::Binary(Box::new(TermBinaryNode {
        lhs,
        operator: op,
        rhs,
        span: Default::default(),
    }))
}

fn make_unary_expr(op: ValkyrieTokenType, base: TermExpression) -> TermExpression {
    TermExpression::Unary(Box::new(TermUnaryNode {
        operator: op,
        base,
        span: Default::default(),
    }))
}

fn make_apply_call(callee: TermExpression, args: Vec<TermExpression>) -> TermExpression {
    TermExpression::ApplyCall {
        callee: Box::new(callee),
        args,
        span: Default::default(),
    }
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
    TermExpression::Return(Box::new(Return {
        base: value,
        span: Default::default(),
    }))
}

fn make_let_stmt(name: &str, expr: TermExpression) -> Statement {
    Statement::Let(Let {
        is_mutable: false,
        pattern: Pattern::Variable(Box::new(VariablePattern {
            name: make_identifier(name),
            span: Default::default(),
        })),
        ty: None,
        expr,
        annotations: Vec::new(),
        span: Default::default(),
    })
}

fn make_expr_stmt(expr: TermExpression) -> Statement {
    Statement::ExprStmt(ExprStmt {
        annotations: Vec::new(),
        expr,
        semi: false,
        span: Default::default(),
    })
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
    Param {
        name: make_identifier(name),
        ty: None,
        default: None,
        span: Default::default(),
    }
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
    let expr = make_if_expr(
        make_int_literal(10),
        make_block(vec![make_expr_stmt(make_bool_literal(true))]),
        None,
    );
    checker.infer_expr(&expr);
    let diags = &checker.diagnostics;
    assert!(diags.iter().any(|d| d.message.contains("If condition") && d.severity == DiagnosticSeverity::Error));
}

#[test]
fn test_if_condition_bool_ok() {
    let mut checker = TypeChecker::new();
    let expr = make_if_expr(
        make_bool_literal(true),
        make_block(vec![make_expr_stmt(make_bool_literal(true))]),
        None,
    );
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
    assert!(checker.diagnostics.is_empty());
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
        make_block(vec![
            make_expr_stmt(make_binary_expr(make_var_expr("x"), ValkyrieTokenType::Plus, make_int_literal(1))),
        ]),
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
        make_block(vec![
            make_expr_stmt(make_if_expr(
                make_int_literal(10),
                make_block(vec![make_expr_stmt(make_bool_literal(true))]),
                None,
            )),
        ]),
    );
    let diags = checker.check_micro(&micro);
    assert!(diags.iter().any(|d| d.message.contains("If condition")));
}

#[test]
fn test_check_root_with_micro() {
    let mut checker = TypeChecker::new();
    let root = make_root(vec![
        make_micro_item(
            "test",
            vec![make_typed_param("x", "i32")],
            make_block(vec![
                make_expr_stmt(make_binary_expr(make_var_expr("x"), ValkyrieTokenType::Plus, make_int_literal(1))),
            ]),
        ),
    ]);
    let diags = checker.check_root(&root);
    assert!(diags.is_empty(), "Expected no diagnostics, got: {:?}", diags);
}

#[test]
fn test_return_type_mismatch() {
    let mut checker = TypeChecker::new();
    let mut micro = make_micro(
        "test",
        vec![],
        make_block(vec![
            make_expr_stmt(make_return_expr(Some(make_bool_literal(true)))),
        ]),
    );
    micro.return_type = Some(TypeExpression::Namepath(Box::new(make_name_path("i32"))));
    let diags = checker.check_micro(&micro);
    assert!(diags.iter().any(|d| d.message.contains("Return type mismatch")));
}

#[test]
fn test_return_no_annotation_no_error() {
    let mut checker = TypeChecker::new();
    let micro = make_micro(
        "test",
        vec![],
        make_block(vec![
            make_expr_stmt(make_return_expr(Some(make_int_literal(42)))),
        ]),
    );
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
    };
    assert_eq!(diag.to_string(), "[error] Type mismatch (10-20)");
}

#[test]
fn test_paren_expr() {
    let mut checker = TypeChecker::new();
    let expr = TermExpression::Paren {
        expr: Box::new(make_int_literal(42)),
        span: Default::default(),
    };
    let ty = checker.infer_expr(&expr);
    assert_eq!(ty, TypeInfo::Int);
}

#[test]
fn test_null_literal() {
    let mut checker = TypeChecker::new();
    let expr = TermExpression::StringLiteral(StringLiteral {
        segments: vec![StringSegment::Text(Box::new(TextSegment {
            content: "null".to_string(),
            span: Default::default(),
        }))],
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
    assert_eq!(ty, TypeInfo::Function {
        param_types: vec![TypeInfo::Int],
        return_type: Box::new(TypeInfo::Bool),
    });
}

#[test]
fn test_let_stmt_type_inference() {
    let mut checker = TypeChecker::new();
    let micro = make_micro(
        "test",
        vec![],
        make_block(vec![
            make_let_stmt("x", make_int_literal(42)),
            make_expr_stmt(make_var_expr("x")),
        ]),
    );
    let diags = checker.check_micro(&micro);
    assert!(diags.is_empty(), "Expected no diagnostics, got: {:?}", diags);
}

#[test]
fn test_namespace_items_checked() {
    let mut checker = TypeChecker::new();
    let root = make_root(vec![
        StatementNode::Namespace(Box::new(NamespaceDeclaration {
            name: make_name_path("Game"),
            items: vec![
                StatementNode::Micro(Box::new(make_micro(
                    "start",
                    vec![],
                    make_block(vec![
                        make_expr_stmt(make_if_expr(
                            make_int_literal(10),
                            make_block(vec![make_expr_stmt(make_bool_literal(true))]),
                            None,
                        )),
                    ]),
                ))),
            ],
            annotations: Vec::new(),
            span: Default::default(),
        })),
    ]);
    let diags = checker.check_root(&root);
    assert!(diags.iter().any(|d| d.message.contains("If condition")));
}
