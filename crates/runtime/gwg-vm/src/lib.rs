//! GWG Engine 虚拟机
//!
//! 提供执行中间表示的虚拟机。

use std::collections::HashMap;

use gwg_types::prelude::*;
use gwg_ecs::prelude::*;
use gwg_ir::prelude::*;

/// 虚拟机错误
#[derive(thiserror::Error, Debug)]
pub enum VmError {
    /// 无效的指令
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(String),
    
    /// 无效的操作数
    #[error("Invalid operand: {0}")]
    InvalidOperand(String),
    
    /// 类型不匹配
    #[error("Type mismatch")]
    TypeMismatch,
    
    /// 除零错误
    #[error("Division by zero")]
    DivisionByZero,
    
    /// 无效的标签
    #[error("Invalid label: {0}")]
    InvalidLabel(usize),
    
    /// 无效的函数
    #[error("Invalid function: {0}")]
    InvalidFunction(usize),
    
    /// 无效的类型 ID
    #[error("Invalid type ID: {0}")]
    InvalidTypeId(usize),
    
    /// 栈溢出
    #[error("Stack overflow")]
    StackOverflow,
    
    /// 栈下溢
    #[error("Stack underflow")]
    StackUnderflow,
    
    /// 实体不存在
    #[error("Entity not found: {0:?}")]
    EntityNotFound(Entity),
    
    /// 组件不存在
    #[error("Component not found")]
    ComponentNotFound,
    
    /// 资源不存在
    #[error("Resource not found")]
    ResourceNotFound,
}

/// VM 结果
pub type VmResult<T> = Result<T, VmError>;

/// 调用帧
#[derive(Clone, Debug)]
struct CallFrame {
    /// 返回地址
    return_address: usize,
    /// 函数索引
    function: usize,
    /// 基指针
    base_pointer: usize,
}

/// 虚拟机
pub struct VirtualMachine {
    /// ECS 世界
    world: World,
    /// 加载的模块
    modules: HashMap<String, Module>,
    /// 值栈
    stack: Vec<Value>,
    /// 调用栈
    call_stack: Vec<CallFrame>,
    /// 当前模块
    current_module: Option<String>,
    /// 当前函数
    current_function: Option<usize>,
    /// 程序计数器
    pc: usize,
    /// 基指针
    bp: usize,
    /// 寄存器
    registers: Vec<Value>,
    /// 局部变量
    locals: Vec<Value>,
    /// 查询状态
    query_state: Option<QueryState>,
}

impl VirtualMachine {
    /// 创建新的虚拟机
    pub fn new() -> Self {
        Self {
            world: World::new(),
            modules: HashMap::new(),
            stack: Vec::with_capacity(256),
            call_stack: Vec::with_capacity(32),
            current_module: None,
            current_function: None,
            pc: 0,
            bp: 0,
            registers: vec![Value::Unit; 16],
            locals: Vec::new(),
            query_state: None,
        }
    }

    /// 加载模块
    pub fn load_module(&mut self, module: Module) {
        self.modules.insert(module.name.clone(), module);
    }

    /// 获取 ECS 世界
    pub fn world(&self) -> &World {
        &self.world
    }

    /// 获取 ECS 世界可变引用
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// 调用函数
    pub fn call(&mut self, module_name: &str, func_name: &str, args: Vec<Value>) -> VmResult<Value> {
        let module = self.modules.get(module_name).ok_or_else(|| {
            VmError::InvalidInstruction(format!("Module not found: {}", module_name))
        })?;

        let func_idx = module.functions.iter()
            .position(|f| f.name == func_name)
            .ok_or_else(|| {
                VmError::InvalidInstruction(format!("Function not found: {}", func_name))
            })?;

        let func = &module.functions[func_idx];
        if args.len() != func.arity {
            return Err(VmError::InvalidInstruction(
                format!("Function expects {} arguments, got {}", func.arity, args.len())
            ));
        }

        self.current_module = Some(module_name.to_string());
        self.current_function = Some(func_idx);
        self.pc = 0;
        self.bp = self.stack.len();
        self.locals.resize(func.locals, Value::Unit);

        for arg in args {
            self.stack.push(arg);
        }

        self.run()
    }

    /// 运行虚拟机
    fn run(&mut self) -> VmResult<Value> {
        let module_name = self.current_module.clone().ok_or_else(|| {
            VmError::InvalidInstruction("No module loaded".to_string())
        })?;
        
        let module = self.modules.get(&module_name).ok_or_else(|| {
            VmError::InvalidInstruction(format!("Module not found: {}", module_name))
        })?;
        
        let func_idx = self.current_function.ok_or_else(|| {
            VmError::InvalidInstruction("No function selected".to_string())
        })?;
        
        let func = &module.functions[func_idx];

        while self.pc < func.instructions.len() {
            let instr = &func.instructions[self.pc];
            self.execute_instruction(module, func, instr)?;
            self.pc += 1;
        }

        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    /// 执行单个指令
    fn execute_instruction(
        &mut self,
        module: &Module,
        func: &Function,
        instr: &Instruction,
    ) -> VmResult<()> {
        match instr.opcode {
            OpCode::Nop => {}
            
            OpCode::LoadConst => {
                let idx = self.get_usize_operand(&instr.operands, 0)?;
                let value = module.constants.get(idx).ok_or_else(|| {
                    VmError::InvalidOperand(format!("Constant index out of bounds: {}", idx))
                })?;
                self.stack.push(value.clone());
            }
            
            OpCode::LoadVar => {
                let idx = self.get_usize_operand(&instr.operands, 0)?;
                let value = self.locals.get(idx).ok_or_else(|| {
                    VmError::InvalidOperand(format!("Local index out of bounds: {}", idx))
                })?;
                self.stack.push(value.clone());
            }
            
            OpCode::StoreVar => {
                let idx = self.get_usize_operand(&instr.operands, 0)?;
                let value = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                if idx < self.locals.len() {
                    self.locals[idx] = value;
                }
            }
            
            OpCode::Add => {
                let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let result = self.add_values(a, b)?;
                self.stack.push(result);
            }
            
            OpCode::Sub => {
                let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let result = self.sub_values(a, b)?;
                self.stack.push(result);
            }
            
            OpCode::Mul => {
                let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let result = self.mul_values(a, b)?;
                self.stack.push(result);
            }
            
            OpCode::Div => {
                let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let result = self.div_values(a, b)?;
                self.stack.push(result);
            }
            
            OpCode::Eq => {
                let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let result = Value::Bool(a == b);
                self.stack.push(result);
            }
            
            OpCode::Neq => {
                let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let result = Value::Bool(a != b);
                self.stack.push(result);
            }
            
            OpCode::Jmp => {
                let label_idx = self.get_usize_operand(&instr.operands, 0)?;
                let target = func.labels.get(label_idx).ok_or_else(|| {
                    VmError::InvalidLabel(label_idx)
                })?;
                self.pc = *target - 1;
            }
            
            OpCode::JmpIfTrue => {
                let label_idx = self.get_usize_operand(&instr.operands, 0)?;
                let cond = self.pop_bool()?;
                if cond {
                    let target = func.labels.get(label_idx).ok_or_else(|| {
                        VmError::InvalidLabel(label_idx)
                    })?;
                    self.pc = *target - 1;
                }
            }
            
            OpCode::JmpIfFalse => {
                let label_idx = self.get_usize_operand(&instr.operands, 0)?;
                let cond = self.pop_bool()?;
                if !cond {
                    let target = func.labels.get(label_idx).ok_or_else(|| {
                        VmError::InvalidLabel(label_idx)
                    })?;
                    self.pc = *target - 1;
                }
            }
            
            OpCode::Call => {
                let func_idx = self.get_usize_operand(&instr.operands, 0)?;
                let called_func = module.functions.get(func_idx).ok_or_else(|| {
                    VmError::InvalidFunction(func_idx)
                })?;
                
                let frame = CallFrame {
                    return_address: self.pc,
                    function: self.current_function.unwrap(),
                    base_pointer: self.bp,
                };
                
                self.call_stack.push(frame);
                self.current_function = Some(func_idx);
                self.pc = 0;
                self.bp = self.stack.len() - called_func.arity;
                self.locals.resize(called_func.locals, Value::Unit);
            }
            
            OpCode::Ret => {
                let result = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                
                let frame = self.call_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.current_function = Some(frame.function);
                self.pc = frame.return_address;
                self.bp = frame.base_pointer;
                self.stack.truncate(self.bp);
                self.stack.push(result);
            }
            
            OpCode::CreateEntity => {
                let entity = self.world.spawn_empty();
                self.stack.push(Value::Entity(entity));
            }
            
            OpCode::DestroyEntity => {
                let entity = self.pop_entity()?;
                self.world.despawn(entity);
            }
            
            OpCode::QueryStart => {
                // TODO: 实现查询开始
                self.stack.push(Value::Unit);
            }
            
            OpCode::QueryNext => {
                // TODO: 实现查询下一个
                self.stack.push(Value::Bool(false));
            }
            
            OpCode::QueryEnd => {
                self.query_state = None;
            }
            
            OpCode::InsertResource => {
                // TODO: 实现资源插入
            }
            
            OpCode::GetResource => {
                // TODO: 实现资源获取
                self.stack.push(Value::Unit);
            }
            
            OpCode::SetResource => {
                // TODO: 实现资源设置
            }
            
            OpCode::RemoveResource => {
                // TODO: 实现资源移除
            }
        }
        Ok(())
    }

    /// 获取 usize 类型的操作数
    fn get_usize_operand(&self, operands: &[Operand], idx: usize) -> VmResult<usize> {
        match operands.get(idx) {
            Some(Operand::Int(i)) => Ok(*i as usize),
            Some(Operand::Register(r)) => Ok(*r),
            Some(Operand::Variable(v)) => Ok(*v),
            Some(Operand::Label(l)) => Ok(*l),
            Some(Operand::Function(f)) => Ok(*f),
            Some(Operand::TypeId(t)) => Ok(*t),
            _ => Err(VmError::InvalidOperand("Expected usize operand".to_string())),
        }
    }

    /// 弹出布尔值
    fn pop_bool(&mut self) -> VmResult<bool> {
        match self.stack.pop() {
            Some(Value::Bool(b)) => Ok(b),
            _ => Err(VmError::TypeMismatch),
        }
    }

    /// 弹出整数值
    fn pop_int(&mut self) -> VmResult<i64> {
        match self.stack.pop() {
            Some(Value::Int(i)) => Ok(i),
            _ => Err(VmError::TypeMismatch),
        }
    }

    /// 弹出浮点数值
    fn pop_float(&mut self) -> VmResult<f64> {
        match self.stack.pop() {
            Some(Value::Float(f)) => Ok(f),
            _ => Err(VmError::TypeMismatch),
        }
    }

    /// 弹出实体值
    fn pop_entity(&mut self) -> VmResult<Entity> {
        match self.stack.pop() {
            Some(Value::Entity(e)) => Ok(e),
            _ => Err(VmError::TypeMismatch),
        }
    }

    /// 相加两个值
    fn add_values(&self, a: Value, b: Value) -> VmResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
            (Value::String(x), Value::String(y)) => Ok(Value::String(format!("{}{}", x, y))),
            _ => Err(VmError::TypeMismatch),
        }
    }

    /// 相减两个值
    fn sub_values(&self, a: Value, b: Value) -> VmResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
            _ => Err(VmError::TypeMismatch),
        }
    }

    /// 相乘两个值
    fn mul_values(&self, a: Value, b: Value) -> VmResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
            _ => Err(VmError::TypeMismatch),
        }
    }

    /// 相除两个值
    fn div_values(&self, a: Value, b: Value) -> VmResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 {
                    return Err(VmError::DivisionByZero);
                }
                Ok(Value::Int(x / y))
            }
            (Value::Float(x), Value::Float(y)) => {
                if y == 0.0 {
                    return Err(VmError::DivisionByZero);
                }
                Ok(Value::Float(x / y))
            }
            _ => Err(VmError::TypeMismatch),
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

pub mod prelude {
    //! 虚拟机的预导入模块

    pub use super::{VirtualMachine, VmError, VmResult};
    pub use gwg_types::prelude::*;
    pub use gwg_ecs::prelude::*;
    pub use gwg_ir::prelude::*;
}
