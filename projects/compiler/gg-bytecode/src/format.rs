use std::collections::HashMap;

use gg_core::GResult;

/// 字节码魔数头 "GGBC"
pub const MAGIC: u32 = 0x47474243;
/// 字节码格式版本号
pub const VERSION: u32 = 3;

/// 字节码值类型，与 IrValue 对应但面向字节码
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BytecodeValue {
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
    /// 有序列表
    List(Vec<BytecodeValue>),
    /// 有序键值对对象
    Object(Vec<(String, BytecodeValue)>),
    /// 映射
    Map(Vec<(BytecodeValue, BytecodeValue)>),
}

/// 字节码操作码（紧凑的一字节表示）
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BytecodeOpCode {
    /// 从常量池加载常量
    LoadConst = 0x01,
    /// 从整数常量池加载
    LoadConstInt = 0x0A,
    /// 从浮点常量池加载
    LoadConstFloat = 0x0B,
    /// 从字符串常量池加载
    LoadConstString = 0x0C,
    /// 从实体常量池加载
    LoadConstEntity = 0x0D,
    /// 从布尔常量池加载
    LoadConstBool = 0x0E,
    /// 加载 null
    LoadNull = 0x02,
    /// 加载 true
    LoadTrue = 0x03,
    /// 加载 false
    LoadFalse = 0x04,
    /// 加载局部变量
    LoadLocal = 0x05,
    /// 存储局部变量
    StoreLocal = 0x06,
    /// 加法
    Add = 0x10,
    /// 减法
    Sub = 0x11,
    /// 乘法
    Mul = 0x12,
    /// 除法
    Div = 0x13,
    /// 取模
    Mod = 0x15,
    /// 取负
    Neg = 0x14,
    /// 相等
    Eq = 0x20,
    /// 不等
    Ne = 0x21,
    /// 小于
    Lt = 0x22,
    /// 小于等于
    Le = 0x23,
    /// 大于
    Gt = 0x24,
    /// 大于等于
    Ge = 0x25,
    /// 逻辑与
    And = 0x30,
    /// 逻辑或
    Or = 0x31,
    /// 逻辑非
    Not = 0x32,
    /// 无条件跳转
    Jump = 0x40,
    /// 为假时跳转
    JumpIfFalse = 0x41,
    /// 为真时跳转
    JumpIfTrue = 0x42,
    /// 调用函数
    Call = 0x50,
    /// 返回
    Return = 0x51,
    /// 尾调用优化
    TailCall = 0x52,
    /// 创建实体
    SpawnEntity = 0x60,
    /// 销毁实体
    DespawnEntity = 0x61,
    /// 添加组件
    AddComponent = 0x62,
    /// 获取组件
    GetComponent = 0x63,
    /// 设置组件
    SetComponent = 0x64,
    /// 调用宿主函数
    HostCall = 0x70,
    /// 获取字段
    GetField = 0x82,
    /// 设置字段
    SetField = 0x83,
    /// 按索引获取
    GetIndex = 0x84,
    /// 按索引设置
    SetIndex = 0x85,
    /// 创建新对象
    NewObject = 0x86,
    /// 创建新列表
    NewList = 0x87,
    /// 创建新映射
    NewMap = 0x88,
    /// 字符串拼接
    StringConcat = 0x89,
    /// 弹出栈顶
    Pop = 0x80,
    /// 复制栈顶
    Dup = 0x81,
}

impl BytecodeOpCode {
    /// 从字节值转换为操作码
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(BytecodeOpCode::LoadConst),
            0x0A => Some(BytecodeOpCode::LoadConstInt),
            0x0B => Some(BytecodeOpCode::LoadConstFloat),
            0x0C => Some(BytecodeOpCode::LoadConstString),
            0x0D => Some(BytecodeOpCode::LoadConstEntity),
            0x0E => Some(BytecodeOpCode::LoadConstBool),
            0x02 => Some(BytecodeOpCode::LoadNull),
            0x03 => Some(BytecodeOpCode::LoadTrue),
            0x04 => Some(BytecodeOpCode::LoadFalse),
            0x05 => Some(BytecodeOpCode::LoadLocal),
            0x06 => Some(BytecodeOpCode::StoreLocal),
            0x10 => Some(BytecodeOpCode::Add),
            0x11 => Some(BytecodeOpCode::Sub),
            0x12 => Some(BytecodeOpCode::Mul),
            0x13 => Some(BytecodeOpCode::Div),
            0x15 => Some(BytecodeOpCode::Mod),
            0x14 => Some(BytecodeOpCode::Neg),
            0x20 => Some(BytecodeOpCode::Eq),
            0x21 => Some(BytecodeOpCode::Ne),
            0x22 => Some(BytecodeOpCode::Lt),
            0x23 => Some(BytecodeOpCode::Le),
            0x24 => Some(BytecodeOpCode::Gt),
            0x25 => Some(BytecodeOpCode::Ge),
            0x30 => Some(BytecodeOpCode::And),
            0x31 => Some(BytecodeOpCode::Or),
            0x32 => Some(BytecodeOpCode::Not),
            0x40 => Some(BytecodeOpCode::Jump),
            0x41 => Some(BytecodeOpCode::JumpIfFalse),
            0x42 => Some(BytecodeOpCode::JumpIfTrue),
            0x50 => Some(BytecodeOpCode::Call),
            0x51 => Some(BytecodeOpCode::Return),
            0x52 => Some(BytecodeOpCode::TailCall),
            0x60 => Some(BytecodeOpCode::SpawnEntity),
            0x61 => Some(BytecodeOpCode::DespawnEntity),
            0x62 => Some(BytecodeOpCode::AddComponent),
            0x63 => Some(BytecodeOpCode::GetComponent),
            0x64 => Some(BytecodeOpCode::SetComponent),
            0x70 => Some(BytecodeOpCode::HostCall),
            0x80 => Some(BytecodeOpCode::Pop),
            0x81 => Some(BytecodeOpCode::Dup),
            0x82 => Some(BytecodeOpCode::GetField),
            0x83 => Some(BytecodeOpCode::SetField),
            0x84 => Some(BytecodeOpCode::GetIndex),
            0x85 => Some(BytecodeOpCode::SetIndex),
            0x86 => Some(BytecodeOpCode::NewObject),
            0x87 => Some(BytecodeOpCode::NewList),
            0x88 => Some(BytecodeOpCode::NewMap),
            0x89 => Some(BytecodeOpCode::StringConcat),
            _ => None,
        }
    }

    /// 转换为字节值
    pub fn as_byte(&self) -> u8 {
        *self as u8
    }
}

/// 字节码指令，包含操作码和操作数
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BytecodeInstruction {
    /// 从常量池加载常量
    LoadConst {
        /// 常量池索引
        index: u32,
    },
    /// 从整数常量池加载
    LoadConstInt {
        /// 整数常量池索引
        index: u32,
    },
    /// 从浮点常量池加载
    LoadConstFloat {
        /// 浮点常量池索引
        index: u32,
    },
    /// 从字符串常量池加载
    LoadConstString {
        /// 字符串常量池索引
        index: u32,
    },
    /// 从实体常量池加载
    LoadConstEntity {
        /// 实体常量池索引
        index: u32,
    },
    /// 从布尔常量池加载
    LoadConstBool {
        /// 布尔常量池索引
        index: u32,
    },
    /// 加载 null
    LoadNull,
    /// 加载 true
    LoadTrue,
    /// 加载 false
    LoadFalse,
    /// 加载局部变量
    LoadLocal {
        /// 局部变量索引
        index: u32,
    },
    /// 存储局部变量
    StoreLocal {
        /// 局部变量索引
        index: u32,
    },
    /// 加法
    Add,
    /// 减法
    Sub,
    /// 乘法
    Mul,
    /// 除法
    Div,
    /// 取模
    Mod,
    /// 取负
    Neg,
    /// 相等
    Eq,
    /// 不等
    Ne,
    /// 小于
    Lt,
    /// 小于等于
    Le,
    /// 大于
    Gt,
    /// 大于等于
    Ge,
    /// 逻辑与
    And,
    /// 逻辑或
    Or,
    /// 逻辑非
    Not,
    /// 无条件跳转
    Jump {
        /// 目标地址
        address: u32,
    },
    /// 为假时跳转
    JumpIfFalse {
        /// 目标地址
        address: u32,
    },
    /// 为真时跳转
    JumpIfTrue {
        /// 目标地址
        address: u32,
    },
    /// 调用函数
    Call {
        /// 参数数量
        arg_count: u32,
    },
    /// 返回
    Return,
    /// 尾调用，复用当前栈帧进行调用优化
    TailCall(u32),
    /// 创建实体
    SpawnEntity,
    /// 销毁实体
    DespawnEntity,
    /// 添加组件
    AddComponent {
        /// 组件类型名字符串池索引
        type_name_index: u32,
    },
    /// 获取组件
    GetComponent {
        /// 组件类型名字符串池索引
        type_name_index: u32,
    },
    /// 设置组件
    SetComponent {
        /// 组件类型名字符串池索引
        type_name_index: u32,
    },
    /// 调用宿主函数
    HostCall {
        /// 函数名字符串池索引
        name_index: u32,
        /// 参数数量
        arg_count: u32,
    },
    /// 弹出栈顶
    Pop,
    /// 复制栈顶
    Dup,
    /// 获取对象字段
    GetField {
        /// 字段名字符串池索引
        name_index: u32,
    },
    /// 设置对象字段
    SetField {
        /// 字段名字符串池索引
        name_index: u32,
    },
    /// 按索引获取元素
    GetIndex,
    /// 按索引设置元素
    SetIndex,
    /// 创建新对象
    NewObject {
        /// 字段数量
        field_count: u32,
    },
    /// 创建新列表
    NewList {
        /// 元素数量
        element_count: u32,
    },
    /// 创建新映射
    NewMap {
        /// 键值对数量
        pair_count: u32,
    },
    /// 字符串拼接
    StringConcat {
        /// 拼接数量
        count: u32,
    },
}

impl BytecodeInstruction {
    /// 获取指令的操作码
    pub fn opcode(&self) -> BytecodeOpCode {
        match self {
            BytecodeInstruction::LoadConst { .. } => BytecodeOpCode::LoadConst,
            BytecodeInstruction::LoadConstInt { .. } => BytecodeOpCode::LoadConstInt,
            BytecodeInstruction::LoadConstFloat { .. } => BytecodeOpCode::LoadConstFloat,
            BytecodeInstruction::LoadConstString { .. } => BytecodeOpCode::LoadConstString,
            BytecodeInstruction::LoadConstEntity { .. } => BytecodeOpCode::LoadConstEntity,
            BytecodeInstruction::LoadConstBool { .. } => BytecodeOpCode::LoadConstBool,
            BytecodeInstruction::LoadNull => BytecodeOpCode::LoadNull,
            BytecodeInstruction::LoadTrue => BytecodeOpCode::LoadTrue,
            BytecodeInstruction::LoadFalse => BytecodeOpCode::LoadFalse,
            BytecodeInstruction::LoadLocal { .. } => BytecodeOpCode::LoadLocal,
            BytecodeInstruction::StoreLocal { .. } => BytecodeOpCode::StoreLocal,
            BytecodeInstruction::Add => BytecodeOpCode::Add,
            BytecodeInstruction::Sub => BytecodeOpCode::Sub,
            BytecodeInstruction::Mul => BytecodeOpCode::Mul,
            BytecodeInstruction::Div => BytecodeOpCode::Div,
            BytecodeInstruction::Mod => BytecodeOpCode::Mod,
            BytecodeInstruction::Neg => BytecodeOpCode::Neg,
            BytecodeInstruction::Eq => BytecodeOpCode::Eq,
            BytecodeInstruction::Ne => BytecodeOpCode::Ne,
            BytecodeInstruction::Lt => BytecodeOpCode::Lt,
            BytecodeInstruction::Le => BytecodeOpCode::Le,
            BytecodeInstruction::Gt => BytecodeOpCode::Gt,
            BytecodeInstruction::Ge => BytecodeOpCode::Ge,
            BytecodeInstruction::And => BytecodeOpCode::And,
            BytecodeInstruction::Or => BytecodeOpCode::Or,
            BytecodeInstruction::Not => BytecodeOpCode::Not,
            BytecodeInstruction::Jump { .. } => BytecodeOpCode::Jump,
            BytecodeInstruction::JumpIfFalse { .. } => BytecodeOpCode::JumpIfFalse,
            BytecodeInstruction::JumpIfTrue { .. } => BytecodeOpCode::JumpIfTrue,
            BytecodeInstruction::Call { .. } => BytecodeOpCode::Call,
            BytecodeInstruction::Return => BytecodeOpCode::Return,
            BytecodeInstruction::TailCall(_) => BytecodeOpCode::TailCall,
            BytecodeInstruction::SpawnEntity => BytecodeOpCode::SpawnEntity,
            BytecodeInstruction::DespawnEntity => BytecodeOpCode::DespawnEntity,
            BytecodeInstruction::AddComponent { .. } => BytecodeOpCode::AddComponent,
            BytecodeInstruction::GetComponent { .. } => BytecodeOpCode::GetComponent,
            BytecodeInstruction::SetComponent { .. } => BytecodeOpCode::SetComponent,
            BytecodeInstruction::HostCall { .. } => BytecodeOpCode::HostCall,
            BytecodeInstruction::Pop => BytecodeOpCode::Pop,
            BytecodeInstruction::Dup => BytecodeOpCode::Dup,
            BytecodeInstruction::GetField { .. } => BytecodeOpCode::GetField,
            BytecodeInstruction::SetField { .. } => BytecodeOpCode::SetField,
            BytecodeInstruction::GetIndex => BytecodeOpCode::GetIndex,
            BytecodeInstruction::SetIndex => BytecodeOpCode::SetIndex,
            BytecodeInstruction::NewObject { .. } => BytecodeOpCode::NewObject,
            BytecodeInstruction::NewList { .. } => BytecodeOpCode::NewList,
            BytecodeInstruction::NewMap { .. } => BytecodeOpCode::NewMap,
            BytecodeInstruction::StringConcat { .. } => BytecodeOpCode::StringConcat,
        }
    }

    /// 获取指令的操作数，无操作数的指令返回 None
    pub fn operand(&self) -> Option<u32> {
        match self {
            BytecodeInstruction::LoadConst { index } => Some(*index),
            BytecodeInstruction::LoadConstInt { index } => Some(*index),
            BytecodeInstruction::LoadConstFloat { index } => Some(*index),
            BytecodeInstruction::LoadConstString { index } => Some(*index),
            BytecodeInstruction::LoadConstEntity { index } => Some(*index),
            BytecodeInstruction::LoadConstBool { index } => Some(*index),
            BytecodeInstruction::LoadLocal { index } => Some(*index),
            BytecodeInstruction::StoreLocal { index } => Some(*index),
            BytecodeInstruction::Jump { address } => Some(*address),
            BytecodeInstruction::JumpIfFalse { address } => Some(*address),
            BytecodeInstruction::JumpIfTrue { address } => Some(*address),
            BytecodeInstruction::Call { arg_count } => Some(*arg_count),
            BytecodeInstruction::TailCall(arity) => Some(*arity),
            BytecodeInstruction::AddComponent { type_name_index } => Some(*type_name_index),
            BytecodeInstruction::GetComponent { type_name_index } => Some(*type_name_index),
            BytecodeInstruction::SetComponent { type_name_index } => Some(*type_name_index),
            BytecodeInstruction::HostCall { name_index, arg_count: _ } => Some(*name_index),
            BytecodeInstruction::GetField { name_index } => Some(*name_index),
            BytecodeInstruction::SetField { name_index } => Some(*name_index),
            BytecodeInstruction::NewObject { field_count } => Some(*field_count),
            BytecodeInstruction::NewList { element_count } => Some(*element_count),
            BytecodeInstruction::NewMap { pair_count } => Some(*pair_count),
            BytecodeInstruction::StringConcat { count } => Some(*count),
            BytecodeInstruction::LoadNull
            | BytecodeInstruction::LoadTrue
            | BytecodeInstruction::LoadFalse
            | BytecodeInstruction::Add
            | BytecodeInstruction::Sub
            | BytecodeInstruction::Mul
            | BytecodeInstruction::Div
            | BytecodeInstruction::Mod
            | BytecodeInstruction::Neg
            | BytecodeInstruction::Eq
            | BytecodeInstruction::Ne
            | BytecodeInstruction::Lt
            | BytecodeInstruction::Le
            | BytecodeInstruction::Gt
            | BytecodeInstruction::Ge
            | BytecodeInstruction::And
            | BytecodeInstruction::Or
            | BytecodeInstruction::Not
            | BytecodeInstruction::Return
            | BytecodeInstruction::SpawnEntity
            | BytecodeInstruction::DespawnEntity
            | BytecodeInstruction::Pop
            | BytecodeInstruction::Dup
            | BytecodeInstruction::GetIndex
            | BytecodeInstruction::SetIndex => None,
        }
    }
}

/// 字节码函数
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BytecodeFunction {
    /// 函数名称
    pub name: String,
    /// 参数数量
    pub param_count: u32,
    /// 局部变量数量
    pub local_count: u32,
    /// 指令序列
    pub instructions: Vec<BytecodeInstruction>,
    /// 局部变量名称列表，用于调试
    #[serde(default)]
    pub local_names: Vec<String>,
}

/// 字节码入口点
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BytecodeEntryPoint {
    /// 入口点名称
    pub name: String,
    /// 目标平台
    pub target: String,
    /// 优先级
    pub priority: u32,
}

/// 字节码模块，反序列化后的完整字节码表示
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BytecodeModule {
    /// 模块名称
    pub name: String,
    /// 格式版本号
    pub version: u32,
    /// 常量池
    pub constants: Vec<BytecodeValue>,
    /// 整数常量池
    pub int_constants: Vec<i64>,
    /// 浮点常量池
    pub float_constants: Vec<f64>,
    /// 字符串常量池
    pub string_constants: Vec<String>,
    /// 实体常量池
    pub entity_constants: Vec<u64>,
    /// 布尔常量池
    pub bool_constants: Vec<bool>,
    /// 字符串池（用于组件类型名、宿主函数名等）
    pub string_pool: Vec<String>,
    /// 函数列表
    pub functions: Vec<BytecodeFunction>,
    /// 调试信息（可选）
    pub debug_info: Option<crate::debug_info::DebugInfo>,
    /// 入口点列表
    pub entry_points: Vec<BytecodeEntryPoint>,
    /// 运行时函数索引缓存，映射函数名到函数列表中的索引
    #[serde(skip)]
    pub function_index: HashMap<String, usize>,
}

impl BytecodeModule {
    /// 创建新的字节码模块
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: VERSION,
            constants: Vec::new(),
            int_constants: Vec::new(),
            float_constants: Vec::new(),
            string_constants: Vec::new(),
            entity_constants: Vec::new(),
            bool_constants: Vec::new(),
            string_pool: Vec::new(),
            functions: Vec::new(),
            debug_info: None,
            entry_points: Vec::new(),
            function_index: HashMap::new(),
        }
    }

    /// 构建函数名到索引的映射缓存
    pub fn build_function_index(&mut self) {
        self.function_index.clear();
        for (i, func) in self.functions.iter().enumerate() {
            self.function_index.insert(func.name.clone(), i);
        }
    }

    /// 添加常量到常量池，返回索引
    pub fn add_constant(&mut self, value: BytecodeValue) -> u32 {
        let index = self.constants.len() as u32;
        self.constants.push(value);
        index
    }

    /// 添加整数常量，返回索引
    pub fn add_int_constant(&mut self, value: i64) -> u32 {
        let index = self.int_constants.len() as u32;
        self.int_constants.push(value);
        index
    }

    /// 添加浮点常量，返回索引
    pub fn add_float_constant(&mut self, value: f64) -> u32 {
        let index = self.float_constants.len() as u32;
        self.float_constants.push(value);
        index
    }

    /// 添加字符串常量，返回索引
    pub fn add_string_constant(&mut self, value: String) -> u32 {
        if let Some(idx) = self.string_constants.iter().position(|s| *s == value) {
            return idx as u32;
        }
        let index = self.string_constants.len() as u32;
        self.string_constants.push(value);
        index
    }

    /// 添加实体常量，返回索引
    pub fn add_entity_constant(&mut self, value: u64) -> u32 {
        let index = self.entity_constants.len() as u32;
        self.entity_constants.push(value);
        index
    }

    /// 添加布尔常量，返回索引
    pub fn add_bool_constant(&mut self, value: bool) -> u32 {
        let index = self.bool_constants.len() as u32;
        self.bool_constants.push(value);
        index
    }

    /// 添加字符串到字符串池，返回索引（去重）
    pub fn add_string(&mut self, s: &str) -> u32 {
        if let Some(idx) = self.string_pool.iter().position(|existing| existing == s) {
            return idx as u32;
        }
        let index = self.string_pool.len() as u32;
        self.string_pool.push(s.to_string());
        index
    }

    /// 添加函数
    pub fn add_function(&mut self, function: BytecodeFunction) {
        self.functions.push(function);
    }

    /// 按名称查找函数
    pub fn find_function(&self, name: &str) -> Option<&BytecodeFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// 按名称查找可变引用
    pub fn find_function_mut(&mut self, name: &str) -> Option<&mut BytecodeFunction> {
        self.functions.iter_mut().find(|f| f.name == name)
    }
}

/// 二进制写入辅助工具
pub(crate) struct BinaryWriter {
    buffer: Vec<u8>,
}

impl BinaryWriter {
    /// 创建新的二进制写入器
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// 写入 u8
    pub fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    /// 写入 u32（小端序）
    pub fn write_u32(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// 写入 i64（小端序）
    pub fn write_i64(&mut self, value: i64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// 写入 f64（小端序）
    pub fn write_f64(&mut self, value: f64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// 写入 u64（小端序）
    pub fn write_u64(&mut self, value: u64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// 写入长度前缀的字节数组
    pub fn write_bytes(&mut self, data: &[u8]) {
        self.write_u32(data.len() as u32);
        self.buffer.extend_from_slice(data);
    }

    /// 写入长度前缀的字符串
    pub fn write_string(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    /// 获取写入结果
    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }
}

impl std::io::Write for BinaryWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// 二进制读取辅助工具
pub(crate) struct BinaryReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> BinaryReader<'a> {
    /// 创建新的二进制读取器
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// 获取当前读取位置
    pub fn position(&self) -> usize {
        self.position
    }

    /// 获取剩余字节数
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    /// 读取 u8
    pub fn read_u8(&mut self) -> GResult<u8> {
        if self.position >= self.data.len() {
            return Err(gg_core::GError {
                kind: gg_core::GErrorKind::Runtime, message: "读取 u8 时数据不足".to_string()
            });
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    /// 读取 u32（小端序）
    pub fn read_u32(&mut self) -> GResult<u32> {
        if self.position + 4 > self.data.len() {
            return Err(gg_core::GError {
                kind: gg_core::GErrorKind::Runtime, message: "读取 u32 时数据不足".to_string()
            });
        }
        let bytes: [u8; 4] = self.data[self.position..self.position + 4].try_into().unwrap();
        self.position += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    /// 读取 i64（小端序）
    pub fn read_i64(&mut self) -> GResult<i64> {
        if self.position + 8 > self.data.len() {
            return Err(gg_core::GError {
                kind: gg_core::GErrorKind::Runtime, message: "读取 i64 时数据不足".to_string()
            });
        }
        let bytes: [u8; 8] = self.data[self.position..self.position + 8].try_into().unwrap();
        self.position += 8;
        Ok(i64::from_le_bytes(bytes))
    }

    /// 读取 f64（小端序）
    pub fn read_f64(&mut self) -> GResult<f64> {
        if self.position + 8 > self.data.len() {
            return Err(gg_core::GError {
                kind: gg_core::GErrorKind::Runtime, message: "读取 f64 时数据不足".to_string()
            });
        }
        let bytes: [u8; 8] = self.data[self.position..self.position + 8].try_into().unwrap();
        self.position += 8;
        Ok(f64::from_le_bytes(bytes))
    }

    /// 读取 u64（小端序）
    pub fn read_u64(&mut self) -> GResult<u64> {
        if self.position + 8 > self.data.len() {
            return Err(gg_core::GError {
                kind: gg_core::GErrorKind::Runtime, message: "读取 u64 时数据不足".to_string()
            });
        }
        let bytes: [u8; 8] = self.data[self.position..self.position + 8].try_into().unwrap();
        self.position += 8;
        Ok(u64::from_le_bytes(bytes))
    }

    /// 读取长度前缀的字节数组
    pub fn read_bytes(&mut self) -> GResult<&'a [u8]> {
        let len = self.read_u32()? as usize;
        if self.position + len > self.data.len() {
            return Err(gg_core::GError {
                kind: gg_core::GErrorKind::Runtime,
                message: format!("读取字节数组时数据不足: 需要 {} 字节", len),
            });
        }
        let result = &self.data[self.position..self.position + len];
        self.position += len;
        Ok(result)
    }

    /// 读取长度前缀的字符串
    pub fn read_string(&mut self) -> GResult<String> {
        let bytes = self.read_bytes()?;
        String::from_utf8(bytes.to_vec())
            .map_err(|e| gg_core::GError {
                kind: gg_core::GErrorKind::Runtime, message: format!("字符串解码失败: {}", e)
            })
    }
}

impl<'a> std::io::Read for BinaryReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let available = self.data.len().saturating_sub(self.position);
        let to_read = std::cmp::min(available, buf.len());
        buf[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
        self.position += to_read;
        Ok(to_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_function_with_local_names() {
        let func = BytecodeFunction {
            name: "test_func".to_string(),
            param_count: 2,
            local_count: 4,
            instructions: vec![BytecodeInstruction::LoadNull, BytecodeInstruction::Return],
            local_names: vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()],
        };
        assert_eq!(func.name, "test_func");
        assert_eq!(func.local_names.len(), 4);
        assert_eq!(func.local_names[0], "a");
        assert_eq!(func.local_names[3], "d");
    }

    #[test]
    fn test_bytecode_function_local_names_default() {
        let func = BytecodeFunction {
            name: "f".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![],
            local_names: vec![],
        };
        assert!(func.local_names.is_empty());
    }

    #[test]
    fn test_build_function_index() {
        let mut module = BytecodeModule::new("test_module");
        module.add_function(BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![],
            local_names: vec![],
        });
        module.add_function(BytecodeFunction {
            name: "helper".to_string(),
            param_count: 1,
            local_count: 1,
            instructions: vec![],
            local_names: vec!["x".to_string()],
        });

        assert!(module.function_index.is_empty());

        module.build_function_index();

        assert_eq!(module.function_index.get("main"), Some(&0));
        assert_eq!(module.function_index.get("helper"), Some(&1));
        assert_eq!(module.function_index.get("nonexistent"), None);
    }

    #[test]
    fn test_tail_call_opcode_value() {
        assert_eq!(BytecodeOpCode::TailCall as u8, 0x52);
        assert_eq!(BytecodeOpCode::from_byte(0x52), Some(BytecodeOpCode::TailCall));
    }

    #[test]
    fn test_tail_call_instruction_opcode() {
        let inst = BytecodeInstruction::TailCall(3);
        assert_eq!(inst.opcode(), BytecodeOpCode::TailCall);
    }

    #[test]
    fn test_tail_call_instruction_operand() {
        let inst = BytecodeInstruction::TailCall(5);
        assert_eq!(inst.operand(), Some(5));
    }

    #[test]
    fn test_operand_no_operand_instructions() {
        assert_eq!(BytecodeInstruction::LoadNull.operand(), None);
        assert_eq!(BytecodeInstruction::Return.operand(), None);
        assert_eq!(BytecodeInstruction::Add.operand(), None);
    }

    #[test]
    fn test_operand_with_operand_instructions() {
        assert_eq!(BytecodeInstruction::LoadLocal { index: 42 }.operand(), Some(42));
        assert_eq!(BytecodeInstruction::Call { arg_count: 3 }.operand(), Some(3));
        assert_eq!(BytecodeInstruction::Jump { address: 100 }.operand(), Some(100));
    }
}
