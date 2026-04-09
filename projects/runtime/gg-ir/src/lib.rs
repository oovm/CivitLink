#![warn(missing_docs)]

/// GG 引擎 IR 模块
/// 提供中间表示和指令集定义

/// IR 值类型
#[derive(Debug, Clone, PartialEq)]
pub enum IrValue {
    /// 整数
    Int(i64),
    /// 浮点数
    Float(f64),
    /// 布尔值
    Bool(bool),
    /// 字符串值
    String(String),
    /// 实体 ID
    Entity(u64),
    /// 空值
    Null,
}

/// IR 操作码
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    /// 从常量池加载常量到栈顶
    LoadConst(usize),
    /// 加载 null 到栈顶
    LoadNull,
    /// 加载 true 到栈顶
    LoadTrue,
    /// 加载 false 到栈顶
    LoadFalse,

    /// 加载局部变量到栈顶
    LoadLocal(usize),
    /// 将栈顶存储到局部变量
    StoreLocal(usize),

    /// 栈顶两个值相加
    Add,
    /// 栈顶两个值相减
    Sub,
    /// 栈顶两个值相乘
    Mul,
    /// 栈顶两个值相除
    Div,
    /// 栈顶两个值取模
    Mod,
    /// 栈顶值取负
    Neg,

    /// 相等比较
    Eq,
    /// 不等比较
    Ne,
    /// 小于比较
    Lt,
    /// 小于等于比较
    Le,
    /// 大于比较
    Gt,
    /// 大于等于比较
    Ge,

    /// 逻辑与
    And,
    /// 逻辑或
    Or,
    /// 逻辑非
    Not,

    /// 无条件跳转
    Jump(usize),
    /// 如果栈顶为 false 则跳转
    JumpIfFalse(usize),
    /// 如果栈顶为 true 则跳转
    JumpIfTrue(usize),

    /// 调用函数（参数数量）
    Call(usize),
    /// 从函数返回
    Return,

    /// 创建实体，返回实体 ID
    SpawnEntity,
    /// 销毁实体
    DespawnEntity,

    /// 添加组件（组件类型名）
    AddComponent(String),
    /// 获取组件（组件类型名）
    GetComponent(String),
    /// 设置组件（组件类型名）
    SetComponent(String),

    /// 调用宿主函数（函数名，参数数量）
    HostCall(String, usize),

    /// 弹出栈顶
    Pop,
    /// 复制栈顶
    Dup,
}

/// IR 函数
#[derive(Debug, Clone)]
pub struct IrFunction {
    /// 函数名称
    pub name: String,
    /// 参数数量
    pub param_count: usize,
    /// 局部变量数量
    pub local_count: usize,
    /// 指令序列
    pub instructions: Vec<OpCode>,
}

/// IR 模块
#[derive(Debug, Clone)]
pub struct IrModule {
    /// 模块名称
    pub name: String,
    /// 常量池
    pub constants: Vec<IrValue>,
    /// 函数列表
    pub functions: Vec<IrFunction>,
}

impl IrModule {
    /// 创建新的 IR 模块
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            constants: Vec::new(),
            functions: Vec::new(),
        }
    }

    /// 添加常量到常量池，返回索引
    pub fn add_constant(&mut self, value: IrValue) -> usize {
        let index = self.constants.len();
        self.constants.push(value);
        index
    }

    /// 添加函数
    pub fn add_function(&mut self, function: IrFunction) {
        self.functions.push(function);
    }

    /// 按名称查找函数
    pub fn find_function(&self, name: &str) -> Option<&IrFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// 按名称查找可变引用
    pub fn find_function_mut(&mut self, name: &str) -> Option<&mut IrFunction> {
        self.functions.iter_mut().find(|f| f.name == name)
    }
}
