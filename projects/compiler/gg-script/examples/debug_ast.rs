use oak_core::{Builder, SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

fn main() {
    let language = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&language);

    let sources = vec![
        ("simple micro", "micro init() {\n    print(\"hello\")\n}"),
        ("two micros", "micro init() {\n    print(\"init\")\n}\n\nmicro update() {\n    print(\"update\")\n}"),
        ("let", "micro calc() {\n    let x = 10\n}"),
        ("if else", "micro check() {\n    if (true) {\n        print(\"yes\")\n    } else {\n        print(\"no\")\n    }\n}"),
    ];

    for (name, source) in sources {
        println!("=== {} ===", name);
        let source_text = SourceText::new(source);
        let mut cache = oak_core::parser::ParseSession::<ValkyrieLanguage>::default();
        let diagnostics = builder.build(&source_text, &[], &mut cache);

        println!("Diagnostics: {:?}", diagnostics.diagnostics);
        match diagnostics.result {
            Ok(root) => {
                println!("Root items count: {}", root.items.len());
                for (i, item) in root.items.iter().enumerate() {
                    let kind = match item {
                        oak_valkyrie::ast::Item::Micro(_) => "Micro",
                        oak_valkyrie::ast::Item::Namespace(_) => "Namespace",
                        oak_valkyrie::ast::Item::Statement(_) => "Statement",
                        oak_valkyrie::ast::Item::Class(_) => "Class",
                        oak_valkyrie::ast::Item::Using(_) => "Using",
                        _ => "Other",
                    };
                    println!("  Item {}: {}", i, kind);
                    if let oak_valkyrie::ast::Item::Micro(m) = item {
                        println!("    Micro name: {}", m.name.name);
                        println!("    Params: {}", m.params.len());
                        println!("    Body statements: {}", m.body.statements.len());
                    }
                    if let oak_valkyrie::ast::Item::Statement(s) = item {
                        match s {
                            oak_valkyrie::ast::Statement::Let { pattern, .. } => {
                                println!("    Let statement");
                                if let oak_valkyrie::ast::Pattern::Variable { name, .. } = pattern {
                                    println!("    Variable name: {}", name.name);
                                }
                            }
                            oak_valkyrie::ast::Statement::ExprStmt { .. } => {
                                println!("    Expr statement");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        println!();
    }
}
