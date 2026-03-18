//! GWG Engine 中间表示
//!
//! 提供游戏逻辑的中间表示层。

pub use gwg_types::prelude::*;
pub use gwg_ecs::prelude::*;

/// IR 操作码
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    /// 空操作
    Nop,
    /// 加载常量
    LoadConst,
    /// 加载变量
    LoadVar,
    /// 存储变量
    StoreVar,
    /// 加法
    Add,
    /// 减法
    Sub,
    /// 乘法
    Mul,
    /// 除法
    Div,
    /// 等于
    Eq,
    /// 不等于
    Neq,
    /// 小于
    Lt,
    /// 小于等于
    Le,
    /// 大于
    Gt,
    /// 大于等于
    Ge,
    /// 跳转
    Jmp,
    /// 条件跳转（真）
    JmpIfTrue,
    /// 条件跳转（假）
    JmpIfFalse,
    /// 调用函数
    Call,
    /// 从函数返回
    Ret,
    /// 创建实体
    CreateEntity,
    /// 销毁实体
    DestroyEntity,
    /// 添加组件
    AddComponent,
    /// 移除组件
    RemoveComponent,
    /// 获取组件
    GetComponent,
    /// 设置组件
    SetComponent,
    /// 查询开始
    QueryStart,
    /// 查询下一个
    QueryNext,
    /// 查询结束
    QueryEnd,
    /// 插入资源
    InsertResource,
    /// 获取资源
    GetResource,
    /// 设置资源
    SetResource,
    /// 移除资源
    RemoveResource,
}

/// IR 指令
#[derive(Clone, Debug)]
pub struct Instruction {
    /// 操作码
    pub opcode: OpCode,
    /// 操作数
    pub operands: Vec<Operand>,
}

impl Instruction {
    /// 创建新的指令
    pub fn new(opcode: OpCode, operands: Vec<Operand>) -> Self {
        Self { opcode, operands }
    }
}

/// 操作数
#[derive(Clone, Debug)]
pub enum Operand {
    /// 整数
    Int(i64),
    /// 浮点数
    Float(f64),
    /// 布尔值
    Bool(bool),
    /// 字符串
    String(String),
    /// 实体
    Entity(Entity),
    /// 寄存器索引
    Register(usize),
    /// 变量索引
    Variable(usize),
    /// 标签索引
    Label(usize),
    /// 函数索引
    Function(usize),
    /// 类型 ID 索引
    TypeId(usize),
}

/// IR 值
#[derive(Clone, Debug)]
pub enum Value {
    /// 空值
    Unit,
    /// 整数
    Int(i64),
    /// 浮点数
    Float(f64),
    /// 布尔值
    Bool(bool),
    /// 字符串
    String(String),
    /// 实体
    Entity(Entity),
}

/// IR 函数
#[derive(Clone, Debug)]
pub struct Function {
    /// 函数名
    pub name: String,
    /// 参数数量
    pub arity: usize,
    /// 局部变量数量
    pub locals: usize,
    /// 指令列表
    pub instructions: Vec<Instruction>,
    /// 标签位置
    pub labels: Vec<usize>,
}

impl Function {
    /// 创建新的函数
    pub fn new(name: String, arity: usize, locals: usize) -> Self {
        Self {
            name,
            arity,
            locals,
            instructions: Vec::new(),
            labels: Vec::new(),
        }
    }

    /// 添加指令
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    /// 添加标签
    pub fn add_label(&mut self) -> usize {
        let label = self.labels.len();
        self.labels.push(self.instructions.len());
        label
    }
}

/// IR 模块
#[derive(Clone, Debug)]
pub struct Module {
    /// 模块名
    pub name: String,
    /// 类型 ID 表
    pub type_ids: Vec<TypeId>,
    /// 函数表
    pub functions: Vec<Function>,
    /// 常量池
    pub constants: Vec<Value>,
}

impl Module {
    /// 创建新的模块
    pub fn new(name: String) -> Self {
        Self {
            name,
            type_ids: Vec::new(),
            functions: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// 添加类型 ID
    pub fn add_type_id(&mut self, type_id: TypeId) -> usize {
        let index = self.type_ids.len();
        self.type_ids.push(type_id);
        index
    }

    /// 添加函数
    pub fn add_function(&mut self, func: Function) -> usize {
        let index = self.functions.len();
        self.functions.push(func);
        index
    }

    /// 添加常量
    pub fn add_constant(&mut self, value: Value) -> usize {
        let index = self.constants.len();
        self.constants.push(value);
        index
    }
}

/// IR 构建器
pub struct ModuleBuilder {
    module: Module,
    current_function: Option<usize>,
}

impl ModuleBuilder {
    /// 创建新的模块构建器
    pub fn new(name: String) -> Self {
        Self {
            module: Module::new(name),
            current_function: None,
        }
    }

    /// 添加类型 ID
    pub fn add_type_id(&mut self, type_id: TypeId) -> usize {
        self.module.add_type_id(type_id)
    }

    /// 开始函数
    pub fn begin_function(&mut self, name: String, arity: usize, locals: usize) {
        let func = Function::new(name, arity, locals);
        let index = self.module.add_function(func);
        self.current_function = Some(index);
    }

    /// 结束函数
    pub fn end_function(&mut self) {
        self.current_function = None;
    }

    /// 添加指令
    pub fn emit(&mut self, opcode: OpCode, operands: Vec<Operand>) {
        if let Some(func_idx) = self.current_function {
            let instr = Instruction::new(opcode, operands);
            self.module.functions[func_idx].add_instruction(instr);
        }
    }

    /// 添加标签
    pub fn label(&mut self) -> usize {
        if let Some(func_idx) = self.current_function {
            self.module.functions[func_idx].add_label()
        } else {
            0
        }
    }

    /// 添加常量
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.module.add_constant(value)
    }

    /// 构建模块
    pub fn build(self) -> Module {
        self.module
    }
}

pub mod prelude {
    //! IR 的预导入模块

    pub use super::{
        Function, Instruction, Module, ModuleBuilder, OpCode, Operand, Value,
    };
    pub use gwg_types::prelude::*;
    pub use gwg_ecs::prelude::*;
}
