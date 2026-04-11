use gg_lsp::semantic::{DiagnosticSeverity, SemanticAnalyzer, Symbol, SymbolKind, SymbolTable};

#[test]
fn test_symbol_table_construction() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "\
namespace Game
micro main(x: int, y: float)
    let score: int = 0
    let mut level = 1
";

    let result = analyzer.analyze(source);

    assert!(result.symbol_table.get("Game").is_some());
    assert!(result.symbol_table.get("main").is_some());
    assert!(result.symbol_table.get("x").is_some());
    assert!(result.symbol_table.get("y").is_some());
    assert!(result.symbol_table.get("score").is_some());
    assert!(result.symbol_table.get("level").is_some());

    let game = result.symbol_table.get("Game").unwrap();
    assert_eq!(game.kind, SymbolKind::Namespace);

    let main_fn = result.symbol_table.get("main").unwrap();
    assert_eq!(main_fn.kind, SymbolKind::Function);
    assert_eq!(main_fn.type_info, Some("(x, y)".to_string()));

    let x_param = result.symbol_table.get("x").unwrap();
    assert_eq!(x_param.kind, SymbolKind::Parameter);

    let score = result.symbol_table.get("score").unwrap();
    assert_eq!(score.kind, SymbolKind::Variable);
    assert_eq!(score.type_info, Some("int".to_string()));

    let level = result.symbol_table.get("level").unwrap();
    assert_eq!(level.kind, SymbolKind::Variable);
    assert_eq!(level.type_info, None);
}

#[test]
fn test_definition_lookup() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "\
namespace Game
micro update(dt: float)
    let delta = dt
";

    analyzer.analyze(source);

    let game = analyzer.get_definition("Game").unwrap();
    assert_eq!(game.kind, SymbolKind::Namespace);
    assert_eq!(game.line, 0);

    let update = analyzer.get_definition("update").unwrap();
    assert_eq!(update.kind, SymbolKind::Function);

    let dt = analyzer.get_definition("dt").unwrap();
    assert_eq!(dt.kind, SymbolKind::Parameter);

    let delta = analyzer.get_definition("delta").unwrap();
    assert_eq!(delta.kind, SymbolKind::Variable);

    assert!(analyzer.get_definition("nonexistent").is_none());
}

#[test]
fn test_hover_info_variable() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "let score: int = 0\n";
    analyzer.analyze(source);

    let hover = analyzer.get_hover_info(0, 4, source).unwrap();
    assert!(hover.contents.contains("variable"));
    assert!(hover.contents.contains("score"));
    assert!(hover.contents.contains("int"));
    assert!(hover.range.is_some());
}

#[test]
fn test_hover_info_function() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "micro main(x: int)\n";
    analyzer.analyze(source);

    let hover = analyzer.get_hover_info(0, 6, source).unwrap();
    assert!(hover.contents.contains("function"));
    assert!(hover.contents.contains("main"));
}

#[test]
fn test_hover_info_namespace() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "namespace Game\n";
    analyzer.analyze(source);

    let hover = analyzer.get_hover_info(0, 10, source).unwrap();
    assert!(hover.contents.contains("namespace"));
    assert!(hover.contents.contains("Game"));
}

#[test]
fn test_hover_info_parameter() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "micro main(x: int)\n";
    analyzer.analyze(source);

    let hover = analyzer.get_hover_info(0, 11, source).unwrap();
    assert!(hover.contents.contains("parameter"));
    assert!(hover.contents.contains("x"));
}

#[test]
fn test_hover_info_nonexistent() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "let score = 0\n";
    analyzer.analyze(source);

    let result = analyzer.get_hover_info(0, 0, source);
    assert!(result.is_none());
}

#[test]
fn test_diagnostic_collection_undefined() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "\
let x = 1
let y = unknown_var
";

    let result = analyzer.analyze(source);

    let undef_diags: Vec<_> =
        result.diagnostics.iter().filter(|d| d.message.contains("undefined") && d.message.contains("unknown_var")).collect();
    assert!(!undef_diags.is_empty());
    assert_eq!(undef_diags[0].severity, DiagnosticSeverity::Warning);
}

#[test]
fn test_diagnostic_no_false_positive_on_keywords() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "\
namespace Game
micro main()
    let x = 1
    if x > 0
        return x
";

    let result = analyzer.analyze(source);

    let keyword_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("undefined")
                && (d.message.contains("if")
                    || d.message.contains("return")
                    || d.message.contains("namespace")
                    || d.message.contains("micro"))
        })
        .collect();
    assert!(keyword_diags.is_empty());
}

#[test]
fn test_diagnostic_no_false_positive_on_builtins() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "print(\"hello\")\n";

    let result = analyzer.analyze(source);

    let builtin_diags: Vec<_> =
        result.diagnostics.iter().filter(|d| d.message.contains("undefined") && d.message.contains("print")).collect();
    assert!(builtin_diags.is_empty());
}

#[test]
fn test_symbol_table_get_all() {
    let mut table = SymbolTable::new();
    table.insert(Symbol { name: "x".to_string(), kind: SymbolKind::Variable, line: 0, column: 4, type_info: None });
    table.insert(Symbol { name: "x".to_string(), kind: SymbolKind::Parameter, line: 2, column: 11, type_info: None });

    let all = table.get_all("x");
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].kind, SymbolKind::Variable);
    assert_eq!(all[1].kind, SymbolKind::Parameter);
}

#[test]
fn test_empty_source() {
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze("");

    assert!(result.symbol_table.symbols.is_empty());
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_comment_lines_skipped() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "\
// let fake = 1
/* namespace Fake */
let real = 2
";

    let result = analyzer.analyze(source);

    assert!(result.symbol_table.get("fake").is_none());
    assert!(result.symbol_table.get("Fake").is_none());
    assert!(result.symbol_table.get("real").is_some());
}

#[test]
fn test_const_declaration() {
    let mut analyzer = SemanticAnalyzer::new();
    let source = "const MAX: int = 100\n";

    let result = analyzer.analyze(source);

    let sym = result.symbol_table.get("MAX").unwrap();
    assert_eq!(sym.kind, SymbolKind::Variable);
    assert_eq!(sym.type_info, Some("int".to_string()));
}

#[test]
fn test_symbol_kind_display_name() {
    assert_eq!(SymbolKind::Variable.display_name(), "variable");
    assert_eq!(SymbolKind::Function.display_name(), "function");
    assert_eq!(SymbolKind::Namespace.display_name(), "namespace");
    assert_eq!(SymbolKind::Parameter.display_name(), "parameter");
}
