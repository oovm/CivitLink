use gg_core::{GError, GErrorKind, GResult};
use gg_ir::IrModule;
use gg_script::compiler::ValkyrieCompiler;
use oak_core::{Builder, SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

fn compile_source(source: &str) -> GResult<IrModule> {
    let language = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&language);
    let source_text = SourceText::new(source);
    let mut cache = oak_core::parser::ParseSession::<ValkyrieLanguage>::default();
    let diagnostics = builder.build(&source_text, &[], &mut cache);

    match diagnostics.result {
        Ok(root) => {
            let compiler = ValkyrieCompiler::new("test");
            compiler.compile(&root, "test")
        }
        Err(e) => Err(GError { kind: GErrorKind::Runtime, message: format!("Parse error: {}", e) }),
    }
}

#[test]
fn test_compile_micro() {
    let source = r#"
        micro init() {
            let entity1 = spawn_entity()
            add_component(entity1, "Position")
            set_field(entity1, "Position", "x", 0.0)
            set_field(entity1, "Position", "y", 0.0)
            print("Hello from script!")
        }
    "#;

    let module = compile_source(source).unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "init");
    assert_eq!(module.functions[0].param_count, 0);
}

#[test]
fn test_compile_two_micros() {
    let source = r#"
        micro init() {
            print("init")
        }

        micro update() {
            print("update")
        }
    "#;

    let module = compile_source(source).unwrap();
    assert_eq!(module.functions.len(), 2);
    assert_eq!(module.functions[0].name, "init");
    assert_eq!(module.functions[1].name, "update");
}

#[test]
fn test_compile_let_and_arithmetic() {
    let source = r#"
        micro calc() {
            let x = 10
            let y = x + 5
            let z = x * y
        }
    "#;

    let module = compile_source(source).unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "calc");
    assert!(module.functions[0].instructions.len() > 0);
}

#[test]
fn test_compile_if_else() {
    let source = r#"
        micro check() {
            if (true) {
                print("yes")
            } else {
                print("no")
            }
        }
    "#;

    let module = compile_source(source).unwrap();
    assert_eq!(module.functions.len(), 1);
}

#[test]
fn test_compile_micro_with_params() {
    let source = r#"
        micro greet(name) {
            print(name)
        }
    "#;

    let module = compile_source(source).unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].param_count, 1);
}

#[test]
fn test_compile_loop() {
    let source = r#"
        micro counter() {
            let i = 0
            loop (i < 10) {
                let i = i + 1
            }
        }
    "#;

    let module = compile_source(source).unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "counter");
}

#[test]
fn test_compile_namespace() {
    let source = r#"
        namespace Game {
            micro start() {
                print("started")
            }
        }
    "#;

    let module = compile_source(source).unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "start");
}
