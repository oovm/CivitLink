use gg_bytecode::{format::BytecodeValue, host::Host};
use gg_script::{ScriptCompiler, ScriptLoader};
use gg_vm::{Vm, VmResult};

#[test]
fn test_script_compiler_basic() {
    let source = r#"
        micro init() {
            let entity1 = spawn_entity()
            add_component(entity1, "Position")
            set_field(entity1, "Position", "x", 0.0)
            set_field(entity1, "Position", "y", 0.0)
            print("Hello from script!")
        }
    "#;

    let compiler = ScriptCompiler::new();
    let module = compiler.compile(source, "test").unwrap();

    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "init");
    assert_eq!(module.functions[0].param_count, 0);
}

#[test]
fn test_script_compiler_two_micros() {
    let source = r#"
        micro init() {
            print("init")
        }

        micro update() {
            print("update")
        }
    "#;

    let compiler = ScriptCompiler::new();
    let module = compiler.compile(source, "test").unwrap();

    assert_eq!(module.functions.len(), 2);
    assert_eq!(module.functions[0].name, "init");
    assert_eq!(module.functions[1].name, "update");
}

#[test]
fn test_script_compiler_let_and_arithmetic() {
    let source = r#"
        micro calc() {
            let x = 10
            let y = x + 5
            let z = x * y
        }
    "#;

    let compiler = ScriptCompiler::new();
    let module = compiler.compile(source, "test").unwrap();

    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "calc");
    assert!(!module.functions[0].instructions.is_empty());
}

#[test]
fn test_script_compiler_if_else() {
    let source = r#"
        micro check() {
            if (true) {
                print("yes")
            } else {
                print("no")
            }
        }
    "#;

    let compiler = ScriptCompiler::new();
    let module = compiler.compile(source, "test").unwrap();

    assert_eq!(module.functions.len(), 1);
}

#[test]
fn test_script_compiler_micro_with_params() {
    let source = r#"
        micro greet(name) {
            print(name)
        }
    "#;

    let compiler = ScriptCompiler::new();
    let module = compiler.compile(source, "test").unwrap();

    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].param_count, 1);
}

#[test]
fn test_script_loader_load_string() {
    let loader = ScriptLoader::new();
    let source = r#"
        micro main() {
            print("hello")
        }
    "#;

    let module = loader.load_string(source, "test").unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "main");
}

#[test]
fn test_script_compiler_no_optimize() {
    let source = r#"
        micro main() {
            print("hello")
        }
    "#;

    let compiler = ScriptCompiler::no_optimize();
    let module = compiler.compile(source, "test").unwrap();

    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "main");
}

#[test]
fn test_script_compiler_compile_to_ir() {
    let source = r#"
        micro main() {
            print("hello")
        }
    "#;

    let compiler = ScriptCompiler::new();
    let ir_module = compiler.compile_to_ir(source, "test").unwrap();

    assert_eq!(ir_module.functions.len(), 1);
    assert_eq!(ir_module.functions[0].name, "main");
    assert!(!ir_module.functions[0].instructions.is_empty());
}

// 端到端测试：源码 → IR → VM 执行
#[test]
fn test_end_to_end_pipeline_arithmetic() {
    struct TestHost {
        log: Vec<String>,
    }

    impl TestHost {
        fn new() -> Self {
            Self { log: Vec::new() }
        }
    }

    impl Host for TestHost {
        fn spawn_entity(&mut self) -> u64 {
            0
        }
        fn despawn_entity(&mut self, _entity_id: u64) {}
        fn add_component(&mut self, _entity_id: u64, _component_type: &str, _value: BytecodeValue) {}
        fn get_component_field(&mut self, _entity_id: u64, _component_type: &str, _field: &str) -> Option<BytecodeValue> {
            None
        }
        fn set_component_field(&mut self, _entity_id: u64, _component_type: &str, _field: &str, _value: BytecodeValue) {}
        fn call_host_function(&mut self, name: &str, args: Vec<BytecodeValue>) -> Option<BytecodeValue> {
            self.log.push(format!("call_host_function({}, {:?})", name, args));
            None
        }
    }

    let source = r#"
        micro calc() {
            let x = 10
            let y = x + 5
            print(y)
        }
    "#;

    let compiler = ScriptCompiler::new();
    let module = compiler.compile_to_ir(source, "test").unwrap();

    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "calc");

    let mut vm = Vm::new();
    let mut host = TestHost::new();
    let result = vm.execute_ir(&module, "calc", &mut host);

    match &result {
        VmResult::Error { message, .. } => panic!("VM execution error: {}", message),
        _ => {}
    }
    assert!(matches!(result, VmResult::Ok | VmResult::Return(_)));
    assert!(host.log.iter().any(|l| l.contains("call_host_function(print")));
}
