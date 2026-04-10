use gg_bytecode::{
    format::{BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeValue},
    interpreter::{BytecodeInterpreter, InterpretResult},
    host::Host,
};

/// 测试用宿主实现
struct TestHost {
    /// 已创建的实体列表
    entities: Vec<u64>,
    /// 操作日志
    log: Vec<String>,
}

impl TestHost {
    /// 创建新的测试宿主
    fn new() -> Self {
        Self { entities: Vec::new(), log: Vec::new() }
    }
}

impl Host for TestHost {
    fn spawn_entity(&mut self) -> u64 {
        let id = self.entities.len() as u64;
        self.entities.push(id);
        id
    }

    fn despawn_entity(&mut self, _entity_id: u64) {}

    fn add_component(&mut self, entity_id: u64, component_type: &str, value: BytecodeValue) {
        self.log.push(format!("add_component({}, {}, {:?})", entity_id, component_type, value));
    }

    fn get_component_field(&mut self, _entity_id: u64, _component_type: &str, _field: &str) -> Option<BytecodeValue> {
        None
    }

    fn set_component_field(&mut self, entity_id: u64, component_type: &str, field: &str, value: BytecodeValue) {
        self.log.push(format!("set_component_field({}, {}, {}, {:?})", entity_id, component_type, field, value));
    }

    fn call_host_function(&mut self, name: &str, args: Vec<BytecodeValue>) -> Option<BytecodeValue> {
        self.log.push(format!("call_host_function({}, {:?})", name, args));
        match name {
            "print" => None,
            "spawn_entity" => {
                let id = self.entities.len() as u64;
                self.entities.push(id);
                Some(BytecodeValue::Entity(id))
            }
            _ => None,
        }
    }
}

/// 测试基本算术：返回 42 的简单函数
#[test]
fn test_basic_arithmetic_return_42() {
    let module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![BytecodeValue::Int(42)],
        string_pool: vec![],
        functions: vec![BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![BytecodeInstruction::LoadConst { index: 0 }, BytecodeInstruction::Return],
        }],
    };

    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();
    let result = interpreter.execute(&module, "main", &mut host);

    match result {
        InterpretResult::Return(BytecodeValue::Int(42)) => {}
        other => panic!("期望返回 Int(42)，实际: {:?}", other),
    }
}

/// 测试函数调用：一个函数调用另一个函数
#[test]
fn test_function_call() {
    let module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![BytecodeValue::Int(10), BytecodeValue::String("helper".to_string())],
        string_pool: vec![],
        functions: vec![
            BytecodeFunction {
                name: "helper".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![BytecodeInstruction::LoadConst { index: 0 }, BytecodeInstruction::Return],
            },
            BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![
                    BytecodeInstruction::LoadConst { index: 1 },
                    BytecodeInstruction::Call { arg_count: 0 },
                    BytecodeInstruction::Return,
                ],
            },
        ],
    };

    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();
    let result = interpreter.execute(&module, "main", &mut host);

    match result {
        InterpretResult::Return(BytecodeValue::Int(10)) => {}
        other => panic!("期望返回 Int(10)，实际: {:?}", other),
    }
}

/// 测试条件执行：if/else 分支
#[test]
fn test_conditional_execution() {
    let module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![BytecodeValue::Int(1), BytecodeValue::Int(2)],
        string_pool: vec![],
        functions: vec![BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![
                BytecodeInstruction::LoadTrue,
                BytecodeInstruction::JumpIfFalse { address: 4 },
                BytecodeInstruction::LoadConst { index: 0 },
                BytecodeInstruction::Jump { address: 5 },
                BytecodeInstruction::LoadConst { index: 1 },
                BytecodeInstruction::Return,
            ],
        }],
    };

    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();
    let result = interpreter.execute(&module, "main", &mut host);

    match result {
        InterpretResult::Return(BytecodeValue::Int(1)) => {}
        other => panic!("期望返回 Int(1)（true 分支），实际: {:?}", other),
    }
}

/// 测试宿主交互：spawn_entity、add_component、print
#[test]
fn test_host_interaction() {
    let module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![BytecodeValue::String("Position".to_string())],
        string_pool: vec!["Position".to_string(), "print".to_string()],
        functions: vec![BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 1,
            instructions: vec![
                BytecodeInstruction::SpawnEntity,
                BytecodeInstruction::StoreLocal { index: 0 },
                BytecodeInstruction::LoadLocal { index: 0 },
                BytecodeInstruction::LoadConst { index: 0 },
                BytecodeInstruction::AddComponent { type_name_index: 0 },
                BytecodeInstruction::HostCall { name_index: 1, arg_count: 0 },
                BytecodeInstruction::Pop,
                BytecodeInstruction::Return,
            ],
        }],
    };

    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();
    let result = interpreter.execute(&module, "main", &mut host);

    assert!(matches!(result, InterpretResult::Return(_)));
    assert_eq!(host.entities.len(), 1);
    assert!(host.log.iter().any(|l| l.starts_with("add_component")));
}
