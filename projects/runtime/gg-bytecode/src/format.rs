use gg_core::GResult;

/// 字节码魔数头 "GGBC"
pub const MAGIC: u32 = 0x47474243;
/// 字节码格式版本号
pub const VERSION: u32 = 1;

/// 字节码值类型，与 IrValue 对应但面向字节码
#[derive(Debug, Clone, PartialEq)]
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
}

/// 字节码操作码（紧凑的一字节表示）
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BytecodeOpCode {
    /// 从常量池加载常量
    LoadConst = 0x01,
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
            0x60 => Some(BytecodeOpCode::SpawnEntity),
            0x61 => Some(BytecodeOpCode::DespawnEntity),
            0x62 => Some(BytecodeOpCode::AddComponent),
            0x63 => Some(BytecodeOpCode::GetComponent),
            0x64 => Some(BytecodeOpCode::SetComponent),
            0x70 => Some(BytecodeOpCode::HostCall),
            0x80 => Some(BytecodeOpCode::Pop),
            0x81 => Some(BytecodeOpCode::Dup),
            _ => None,
        }
    }

    /// 转换为字节值
    pub fn as_byte(&self) -> u8 {
        *self as u8
    }
}

/// 字节码指令，包含操作码和操作数
#[derive(Debug, Clone)]
pub enum BytecodeInstruction {
    /// 从常量池加载常量
    LoadConst {
        /// 常量池索引
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
}

impl BytecodeInstruction {
    /// 获取指令的操作码
    pub fn opcode(&self) -> BytecodeOpCode {
        match self {
            BytecodeInstruction::LoadConst { .. } => BytecodeOpCode::LoadConst,
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
            BytecodeInstruction::SpawnEntity => BytecodeOpCode::SpawnEntity,
            BytecodeInstruction::DespawnEntity => BytecodeOpCode::DespawnEntity,
            BytecodeInstruction::AddComponent { .. } => BytecodeOpCode::AddComponent,
            BytecodeInstruction::GetComponent { .. } => BytecodeOpCode::GetComponent,
            BytecodeInstruction::SetComponent { .. } => BytecodeOpCode::SetComponent,
            BytecodeInstruction::HostCall { .. } => BytecodeOpCode::HostCall,
            BytecodeInstruction::Pop => BytecodeOpCode::Pop,
            BytecodeInstruction::Dup => BytecodeOpCode::Dup,
        }
    }
}

/// 字节码函数
#[derive(Debug, Clone)]
pub struct BytecodeFunction {
    /// 函数名称
    pub name: String,
    /// 参数数量
    pub param_count: u32,
    /// 局部变量数量
    pub local_count: u32,
    /// 指令序列
    pub instructions: Vec<BytecodeInstruction>,
}

/// 字节码模块，反序列化后的完整字节码表示
#[derive(Debug, Clone)]
pub struct BytecodeModule {
    /// 模块名称
    pub name: String,
    /// 格式版本号
    pub version: u32,
    /// 常量池
    pub constants: Vec<BytecodeValue>,
    /// 字符串池（用于组件类型名、宿主函数名等）
    pub string_pool: Vec<String>,
    /// 函数列表
    pub functions: Vec<BytecodeFunction>,
}

impl BytecodeModule {
    /// 创建新的字节码模块
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: VERSION,
            constants: Vec::new(),
            string_pool: Vec::new(),
            functions: Vec::new(),
        }
    }

    /// 添加常量到常量池，返回索引
    pub fn add_constant(&mut self, value: BytecodeValue) -> u32 {
        let index = self.constants.len() as u32;
        self.constants.push(value);
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
        Self {
            buffer: Vec::new(),
        }
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
                kind: gg_core::GErrorKind::Runtime,
                message: "读取 u8 时数据不足".to_string(),
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
                kind: gg_core::GErrorKind::Runtime,
                message: "读取 u32 时数据不足".to_string(),
            });
        }
        let bytes: [u8; 4] = self.data[self.position..self.position + 4]
            .try_into()
            .unwrap();
        self.position += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    /// 读取 i64（小端序）
    pub fn read_i64(&mut self) -> GResult<i64> {
        if self.position + 8 > self.data.len() {
            return Err(gg_core::GError {
                kind: gg_core::GErrorKind::Runtime,
                message: "读取 i64 时数据不足".to_string(),
            });
        }
        let bytes: [u8; 8] = self.data[self.position..self.position + 8]
            .try_into()
            .unwrap();
        self.position += 8;
        Ok(i64::from_le_bytes(bytes))
    }

    /// 读取 f64（小端序）
    pub fn read_f64(&mut self) -> GResult<f64> {
        if self.position + 8 > self.data.len() {
            return Err(gg_core::GError {
                kind: gg_core::GErrorKind::Runtime,
                message: "读取 f64 时数据不足".to_string(),
            });
        }
        let bytes: [u8; 8] = self.data[self.position..self.position + 8]
            .try_into()
            .unwrap();
        self.position += 8;
        Ok(f64::from_le_bytes(bytes))
    }

    /// 读取 u64（小端序）
    pub fn read_u64(&mut self) -> GResult<u64> {
        if self.position + 8 > self.data.len() {
            return Err(gg_core::GError {
                kind: gg_core::GErrorKind::Runtime,
                message: "读取 u64 时数据不足".to_string(),
            });
        }
        let bytes: [u8; 8] = self.data[self.position..self.position + 8]
            .try_into()
            .unwrap();
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
        String::from_utf8(bytes.to_vec()).map_err(|e| gg_core::GError {
            kind: gg_core::GErrorKind::Runtime,
            message: format!("字符串解码失败: {}", e),
        })
    }
}
