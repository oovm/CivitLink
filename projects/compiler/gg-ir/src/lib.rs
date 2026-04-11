#![warn(missing_docs)]

//! GG 引擎 IR 模块
//! 提供中间表示、指令集定义和优化 Pass 基础设施

use std::io::{Cursor, Read, Write};

use gg_core::{GError, GErrorKind, GResult};

/// IR 二进制格式的魔数 "GGIR"
const IR_MAGIC: u32 = 0x47474952;
/// IR 二进制格式的版本号
const IR_VERSION: u32 = 1;

/// 目标平台
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TargetPlatform {
    /// 桌面平台
    Desktop,
    /// 移动平台
    Mobile,
    /// Web 平台
    Web,
    /// 服务器平台
    Server,
    /// 所有平台
    All,
}

impl TargetPlatform {
    /// 从字符串解析目标平台
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "desktop" => Some(TargetPlatform::Desktop),
            "mobile" => Some(TargetPlatform::Mobile),
            "web" => Some(TargetPlatform::Web),
            "server" => Some(TargetPlatform::Server),
            "all" => Some(TargetPlatform::All),
            _ => None,
        }
    }

    /// 将目标平台转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetPlatform::Desktop => "desktop",
            TargetPlatform::Mobile => "mobile",
            TargetPlatform::Web => "web",
            TargetPlatform::Server => "server",
            TargetPlatform::All => "all",
        }
    }
}

/// 入口点
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryPoint {
    /// 入口点名称
    pub name: String,
    /// 目标平台
    pub target: TargetPlatform,
    /// 优先级
    pub priority: u32,
}

/// IR 值类型
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    /// 尾调用优化，替换尾位置的 Call
    TailCall(usize),
    /// 从函数返回
    Return,

    /// 创建实体，返回实体 ID
    SpawnEntity,
    /// 销毁实体
    DespawnEntity,

    /// 添加组件（组件类型名字符串池索引）
    AddComponent(usize),
    /// 获取组件（组件类型名字符串池索引）
    GetComponent(usize),
    /// 设置组件（组件类型名字符串池索引）
    SetComponent(usize),

    /// 调用宿主函数（函数名字符串池索引，参数数量）
    HostCall(usize, usize),

    /// 弹出栈顶
    Pop,
    /// 复制栈顶
    Dup,

    /// 从对象获取字段值（字段名字符串池索引）
    GetField(usize),
    /// 设置对象的字段值（字段名字符串池索引）
    SetField(usize),
    /// 按索引获取元素（栈: [容器, 索引] -> [值]）
    GetIndex,
    /// 按索引设置元素（栈: [容器, 索引, 值] -> []）
    SetIndex,
    /// 创建具有 field_count 个字段的新对象
    NewObject(usize),
    /// 创建具有 element_count 个元素的新列表
    NewList(usize),
    /// 创建具有 pair_count 个键值对的新映射
    NewMap(usize),
    /// 将 count 个字符串拼接为一个
    StringConcat(usize),
}

/// IR 函数
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IrFunction {
    /// 函数名称
    pub name: String,
    /// 参数数量
    pub param_count: usize,
    /// 局部变量数量
    pub local_count: usize,
    /// 局部变量名称列表，用于调试
    #[serde(default)]
    pub local_names: Vec<String>,
    /// 指令序列
    pub instructions: Vec<OpCode>,
    /// 是否为入口函数
    #[serde(default)]
    pub is_entry: bool,
    /// 目标平台
    #[serde(default)]
    pub target: Option<TargetPlatform>,
}

/// IR 模块
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IrModule {
    /// 模块名称
    pub name: String,
    /// 常量池
    pub constants: Vec<IrValue>,
    /// 字符串池
    pub string_pool: Vec<String>,
    /// 函数列表
    pub functions: Vec<IrFunction>,
    /// 入口点列表
    #[serde(default)]
    pub entry_points: Vec<EntryPoint>,
    /// 目标平台
    #[serde(default)]
    pub target_platform: Option<TargetPlatform>,
}

impl IrModule {
    /// 创建新的 IR 模块
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            constants: Vec::new(),
            string_pool: Vec::new(),
            functions: Vec::new(),
            entry_points: Vec::new(),
            target_platform: None,
        }
    }

    /// 添加常量到常量池，返回索引
    pub fn add_constant(&mut self, value: IrValue) -> usize {
        let index = self.constants.len();
        self.constants.push(value);
        index
    }

    /// 查找常量池中已有的常量，若存在则返回索引
    pub fn find_constant(&self, value: &IrValue) -> Option<usize> {
        self.constants.iter().position(|c| c == value)
    }

    /// 添加常量到常量池，若已存在则复用索引
    pub fn add_or_get_constant(&mut self, value: IrValue) -> usize {
        if let Some(idx) = self.find_constant(&value) {
            return idx;
        }
        self.add_constant(value)
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

    /// 添加入口点
    pub fn add_entry_point(&mut self, entry_point: EntryPoint) {
        self.entry_points.push(entry_point);
    }

    /// 按名称查找入口点
    pub fn find_entry_point(&self, name: &str) -> Option<&EntryPoint> {
        self.entry_points.iter().find(|e| e.name == name)
    }

    /// 根据索引获取字符串池中的字符串
    pub fn get_string(&self, idx: usize) -> &str {
        self.string_pool.get(idx).map(|s| s.as_str()).unwrap_or("")
    }

    /// 添加字符串到字符串池，若已存在则返回已有索引
    pub fn add_or_get_string(&mut self, s: String) -> usize {
        if let Some(idx) = self.string_pool.iter().position(|existing| *existing == s) {
            return idx;
        }
        let idx = self.string_pool.len();
        self.string_pool.push(s);
        idx
    }

    /// 将 IR 模块序列化为优化的二进制格式
    pub fn serialize_to_binary(&self) -> GResult<Vec<u8>> {
        let mut buf = Cursor::new(Vec::new());

        buf.write_all(&IR_MAGIC.to_le_bytes())?;
        buf.write_all(&IR_VERSION.to_le_bytes())?;

        Self::write_string(&mut buf, &self.name)?;

        let mut int_constants: Vec<i64> = Vec::new();
        let mut float_constants: Vec<f64> = Vec::new();
        let mut string_constants: Vec<String> = Vec::new();
        let mut bool_constants: Vec<bool> = Vec::new();
        let mut entity_constants: Vec<u64> = Vec::new();
        let mut null_count: u32 = 0;
        let mut index_map: Vec<usize> = Vec::new();

        for value in &self.constants {
            match value {
                IrValue::Int(v) => {
                    index_map.push(int_constants.len());
                    int_constants.push(*v);
                }
                IrValue::Float(v) => {
                    index_map.push(int_constants.len() + float_constants.len());
                    float_constants.push(*v);
                }
                IrValue::String(v) => {
                    index_map.push(int_constants.len() + float_constants.len() + string_constants.len());
                    string_constants.push(v.clone());
                }
                IrValue::Bool(v) => {
                    index_map.push(int_constants.len() + float_constants.len() + string_constants.len() + bool_constants.len());
                    bool_constants.push(*v);
                }
                IrValue::Entity(v) => {
                    index_map.push(
                        int_constants.len()
                            + float_constants.len()
                            + string_constants.len()
                            + bool_constants.len()
                            + entity_constants.len(),
                    );
                    entity_constants.push(*v);
                }
                IrValue::Null => {
                    index_map.push(
                        int_constants.len()
                            + float_constants.len()
                            + string_constants.len()
                            + bool_constants.len()
                            + entity_constants.len()
                            + null_count as usize,
                    );
                    null_count += 1;
                }
            }
        }

        buf.write_all(&(int_constants.len() as u32).to_le_bytes())?;
        for v in &int_constants {
            buf.write_all(&v.to_le_bytes())?;
        }

        buf.write_all(&(float_constants.len() as u32).to_le_bytes())?;
        for v in &float_constants {
            buf.write_all(&v.to_le_bytes())?;
        }

        buf.write_all(&(string_constants.len() as u32).to_le_bytes())?;
        for s in &string_constants {
            Self::write_string(&mut buf, s)?;
        }

        buf.write_all(&(bool_constants.len() as u32).to_le_bytes())?;
        for v in &bool_constants {
            buf.write_all(&[*v as u8])?;
        }

        buf.write_all(&(entity_constants.len() as u32).to_le_bytes())?;
        for v in &entity_constants {
            buf.write_all(&v.to_le_bytes())?;
        }

        buf.write_all(&null_count.to_le_bytes())?;

        buf.write_all(&(self.string_pool.len() as u32).to_le_bytes())?;
        for s in &self.string_pool {
            Self::write_string(&mut buf, s)?;
        }

        buf.write_all(&(self.functions.len() as u32).to_le_bytes())?;
        for func in &self.functions {
            Self::write_string(&mut buf, &func.name)?;
            buf.write_all(&(func.param_count as u32).to_le_bytes())?;
            buf.write_all(&(func.local_count as u32).to_le_bytes())?;

            buf.write_all(&(func.local_names.len() as u32).to_le_bytes())?;
            for name in &func.local_names {
                Self::write_string(&mut buf, name)?;
            }

            buf.write_all(&[if func.is_entry { 1u8 } else { 0u8 }])?;
            Self::write_target_platform_opt(&mut buf, &func.target)?;

            buf.write_all(&(func.instructions.len() as u32).to_le_bytes())?;
            for op in &func.instructions {
                Self::write_opcode(&mut buf, op, &index_map)?;
            }
        }

        buf.write_all(&(self.entry_points.len() as u32).to_le_bytes())?;
        for entry in &self.entry_points {
            Self::write_string(&mut buf, &entry.name)?;
            Self::write_target_platform_val(&mut buf, &entry.target)?;
            buf.write_all(&entry.priority.to_le_bytes())?;
        }

        Self::write_target_platform_opt(&mut buf, &self.target_platform)?;

        Ok(buf.into_inner())
    }

    /// 从二进制格式反序列化 IR 模块
    pub fn deserialize_from_binary(data: &[u8]) -> GResult<Self> {
        let mut buf = Cursor::new(data);

        let magic = Self::read_u32(&mut buf)?;
        if magic != IR_MAGIC {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("无效的 IR 魔数: 期望 0x{:08X}, 实际 0x{:08X}", IR_MAGIC, magic),
            });
        }

        let version = Self::read_u32(&mut buf)?;
        if version != IR_VERSION {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("不支持的 IR 版本: 期望 {}, 实际 {}", IR_VERSION, version),
            });
        }

        let name = Self::read_string(&mut buf)?;

        let int_count = Self::read_u32(&mut buf)? as usize;
        let mut int_constants = Vec::with_capacity(int_count);
        for _ in 0..int_count {
            int_constants.push(Self::read_i64(&mut buf)?);
        }

        let float_count = Self::read_u32(&mut buf)? as usize;
        let mut float_constants = Vec::with_capacity(float_count);
        for _ in 0..float_count {
            float_constants.push(Self::read_f64(&mut buf)?);
        }

        let string_count = Self::read_u32(&mut buf)? as usize;
        let mut string_constants = Vec::with_capacity(string_count);
        for _ in 0..string_count {
            string_constants.push(Self::read_string(&mut buf)?);
        }

        let bool_count = Self::read_u32(&mut buf)? as usize;
        let mut bool_constants = Vec::with_capacity(bool_count);
        for _ in 0..bool_count {
            bool_constants.push(Self::read_u8_from(&mut buf)? != 0);
        }

        let entity_count = Self::read_u32(&mut buf)? as usize;
        let mut entity_constants = Vec::with_capacity(entity_count);
        for _ in 0..entity_count {
            entity_constants.push(Self::read_u64(&mut buf)?);
        }

        let null_count = Self::read_u32(&mut buf)? as usize;

        let mut constants = Vec::new();
        for v in int_constants {
            constants.push(IrValue::Int(v));
        }
        for v in float_constants {
            constants.push(IrValue::Float(v));
        }
        for v in string_constants {
            constants.push(IrValue::String(v));
        }
        for v in bool_constants {
            constants.push(IrValue::Bool(v));
        }
        for v in entity_constants {
            constants.push(IrValue::Entity(v));
        }
        for _ in 0..null_count {
            constants.push(IrValue::Null);
        }

        let sp_count = Self::read_u32(&mut buf)? as usize;
        let mut string_pool = Vec::with_capacity(sp_count);
        for _ in 0..sp_count {
            string_pool.push(Self::read_string(&mut buf)?);
        }

        let func_count = Self::read_u32(&mut buf)? as usize;
        let mut functions = Vec::with_capacity(func_count);
        for _ in 0..func_count {
            let func_name = Self::read_string(&mut buf)?;
            let param_count = Self::read_u32(&mut buf)? as usize;
            let local_count = Self::read_u32(&mut buf)? as usize;

            let ln_count = Self::read_u32(&mut buf)? as usize;
            let mut local_names = Vec::with_capacity(ln_count);
            for _ in 0..ln_count {
                local_names.push(Self::read_string(&mut buf)?);
            }

            let is_entry = Self::read_u8_from(&mut buf)? != 0;
            let target = Self::read_target_platform_opt(&mut buf)?;

            let inst_count = Self::read_u32(&mut buf)? as usize;
            let mut instructions = Vec::with_capacity(inst_count);
            for _ in 0..inst_count {
                instructions.push(Self::read_opcode(&mut buf)?);
            }

            functions.push(IrFunction {
                name: func_name,
                param_count,
                local_count,
                local_names,
                instructions,
                is_entry,
                target,
            });
        }

        let entry_count = Self::read_u32(&mut buf)? as usize;
        let mut entry_points = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            let entry_name = Self::read_string(&mut buf)?;
            let entry_target = Self::read_target_platform_val(&mut buf)?;
            let priority = Self::read_u32(&mut buf)?;
            entry_points.push(EntryPoint { name: entry_name, target: entry_target, priority });
        }

        let target_platform = Self::read_target_platform_opt(&mut buf)?;

        Ok(Self { name, constants, string_pool, functions, entry_points, target_platform })
    }

    fn write_string(buf: &mut Cursor<Vec<u8>>, s: &str) -> GResult<()> {
        buf.write_all(&(s.len() as u32).to_le_bytes())?;
        buf.write_all(s.as_bytes())?;
        Ok(())
    }

    fn read_string(buf: &mut Cursor<&[u8]>) -> GResult<String> {
        let len = Self::read_u32(buf)? as usize;
        let mut bytes = vec![0u8; len];
        buf.read_exact(&mut bytes)?;
        String::from_utf8(bytes)
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("字符串解码失败: {}", e) })
    }

    fn read_u8_from(buf: &mut Cursor<&[u8]>) -> GResult<u8> {
        let mut bytes = [0u8; 1];
        buf.read_exact(&mut bytes)?;
        Ok(bytes[0])
    }

    fn read_u32(buf: &mut Cursor<&[u8]>) -> GResult<u32> {
        let mut bytes = [0u8; 4];
        buf.read_exact(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i64(buf: &mut Cursor<&[u8]>) -> GResult<i64> {
        let mut bytes = [0u8; 8];
        buf.read_exact(&mut bytes)?;
        Ok(i64::from_le_bytes(bytes))
    }

    fn read_f64(buf: &mut Cursor<&[u8]>) -> GResult<f64> {
        let mut bytes = [0u8; 8];
        buf.read_exact(&mut bytes)?;
        Ok(f64::from_le_bytes(bytes))
    }

    fn read_u64(buf: &mut Cursor<&[u8]>) -> GResult<u64> {
        let mut bytes = [0u8; 8];
        buf.read_exact(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn target_platform_to_u8(target: &Option<TargetPlatform>) -> u8 {
        match target {
            None => 0,
            Some(TargetPlatform::Desktop) => 1,
            Some(TargetPlatform::Mobile) => 2,
            Some(TargetPlatform::Web) => 3,
            Some(TargetPlatform::Server) => 4,
            Some(TargetPlatform::All) => 5,
        }
    }

    fn u8_to_target_platform(v: u8) -> Option<TargetPlatform> {
        match v {
            0 => None,
            1 => Some(TargetPlatform::Desktop),
            2 => Some(TargetPlatform::Mobile),
            3 => Some(TargetPlatform::Web),
            4 => Some(TargetPlatform::Server),
            5 => Some(TargetPlatform::All),
            _ => None,
        }
    }

    fn write_target_platform_opt(buf: &mut Cursor<Vec<u8>>, target: &Option<TargetPlatform>) -> GResult<()> {
        buf.write_all(&[Self::target_platform_to_u8(target)])?;
        Ok(())
    }

    fn write_target_platform_val(buf: &mut Cursor<Vec<u8>>, target: &TargetPlatform) -> GResult<()> {
        buf.write_all(&[Self::target_platform_to_u8(&Some(*target))])?;
        Ok(())
    }

    fn read_target_platform_opt(buf: &mut Cursor<&[u8]>) -> GResult<Option<TargetPlatform>> {
        let v = Self::read_u8_from(buf)?;
        Ok(Self::u8_to_target_platform(v))
    }

    fn read_target_platform_val(buf: &mut Cursor<&[u8]>) -> GResult<TargetPlatform> {
        let v = Self::read_u8_from(buf)?;
        Self::u8_to_target_platform(v)
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: format!("无效的目标平台值: {}", v) })
    }

    fn opcode_to_u8(op: &OpCode) -> u8 {
        match op {
            OpCode::LoadConst(_) => 0,
            OpCode::LoadNull => 1,
            OpCode::LoadTrue => 2,
            OpCode::LoadFalse => 3,
            OpCode::LoadLocal(_) => 4,
            OpCode::StoreLocal(_) => 5,
            OpCode::Add => 6,
            OpCode::Sub => 7,
            OpCode::Mul => 8,
            OpCode::Div => 9,
            OpCode::Mod => 10,
            OpCode::Neg => 11,
            OpCode::Eq => 12,
            OpCode::Ne => 13,
            OpCode::Lt => 14,
            OpCode::Le => 15,
            OpCode::Gt => 16,
            OpCode::Ge => 17,
            OpCode::And => 18,
            OpCode::Or => 19,
            OpCode::Not => 20,
            OpCode::Jump(_) => 21,
            OpCode::JumpIfFalse(_) => 22,
            OpCode::JumpIfTrue(_) => 23,
            OpCode::Call(_) => 24,
            OpCode::Return => 25,
            OpCode::TailCall(_) => 26,
            OpCode::SpawnEntity => 27,
            OpCode::DespawnEntity => 28,
            OpCode::AddComponent(_) => 29,
            OpCode::GetComponent(_) => 30,
            OpCode::SetComponent(_) => 31,
            OpCode::HostCall(_, _) => 32,
            OpCode::Pop => 33,
            OpCode::Dup => 34,
            OpCode::GetField(_) => 35,
            OpCode::SetField(_) => 36,
            OpCode::GetIndex => 37,
            OpCode::SetIndex => 38,
            OpCode::NewObject(_) => 39,
            OpCode::NewList(_) => 40,
            OpCode::NewMap(_) => 41,
            OpCode::StringConcat(_) => 42,
        }
    }

    fn write_opcode(buf: &mut Cursor<Vec<u8>>, op: &OpCode, index_map: &[usize]) -> GResult<()> {
        buf.write_all(&[Self::opcode_to_u8(op)])?;
        match op {
            OpCode::LoadConst(idx) => {
                let new_idx = index_map
                    .get(*idx)
                    .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: format!("常量索引越界: {}", idx) })?;
                buf.write_all(&(*new_idx as u32).to_le_bytes())?;
            }
            OpCode::LoadLocal(idx)
            | OpCode::StoreLocal(idx)
            | OpCode::Jump(idx)
            | OpCode::JumpIfFalse(idx)
            | OpCode::JumpIfTrue(idx)
            | OpCode::Call(idx)
            | OpCode::TailCall(idx)
            | OpCode::AddComponent(idx)
            | OpCode::GetComponent(idx)
            | OpCode::SetComponent(idx)
            | OpCode::GetField(idx)
            | OpCode::SetField(idx)
            | OpCode::NewObject(idx)
            | OpCode::NewList(idx)
            | OpCode::NewMap(idx)
            | OpCode::StringConcat(idx) => {
                buf.write_all(&(*idx as u32).to_le_bytes())?;
            }
            OpCode::HostCall(name_idx, arg_count) => {
                buf.write_all(&(*name_idx as u32).to_le_bytes())?;
                buf.write_all(&(*arg_count as u32).to_le_bytes())?;
            }
            _ => {}
        }
        Ok(())
    }

    fn read_opcode(buf: &mut Cursor<&[u8]>) -> GResult<OpCode> {
        let opcode_byte = Self::read_u8_from(buf)?;
        match opcode_byte {
            0 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::LoadConst(idx))
            }
            1 => Ok(OpCode::LoadNull),
            2 => Ok(OpCode::LoadTrue),
            3 => Ok(OpCode::LoadFalse),
            4 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::LoadLocal(idx))
            }
            5 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::StoreLocal(idx))
            }
            6 => Ok(OpCode::Add),
            7 => Ok(OpCode::Sub),
            8 => Ok(OpCode::Mul),
            9 => Ok(OpCode::Div),
            10 => Ok(OpCode::Mod),
            11 => Ok(OpCode::Neg),
            12 => Ok(OpCode::Eq),
            13 => Ok(OpCode::Ne),
            14 => Ok(OpCode::Lt),
            15 => Ok(OpCode::Le),
            16 => Ok(OpCode::Gt),
            17 => Ok(OpCode::Ge),
            18 => Ok(OpCode::And),
            19 => Ok(OpCode::Or),
            20 => Ok(OpCode::Not),
            21 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::Jump(idx))
            }
            22 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::JumpIfFalse(idx))
            }
            23 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::JumpIfTrue(idx))
            }
            24 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::Call(idx))
            }
            25 => Ok(OpCode::Return),
            26 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::TailCall(idx))
            }
            27 => Ok(OpCode::SpawnEntity),
            28 => Ok(OpCode::DespawnEntity),
            29 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::AddComponent(idx))
            }
            30 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::GetComponent(idx))
            }
            31 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::SetComponent(idx))
            }
            32 => {
                let name_idx = Self::read_u32(buf)? as usize;
                let arg_count = Self::read_u32(buf)? as usize;
                Ok(OpCode::HostCall(name_idx, arg_count))
            }
            33 => Ok(OpCode::Pop),
            34 => Ok(OpCode::Dup),
            35 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::GetField(idx))
            }
            36 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::SetField(idx))
            }
            37 => Ok(OpCode::GetIndex),
            38 => Ok(OpCode::SetIndex),
            39 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::NewObject(idx))
            }
            40 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::NewList(idx))
            }
            41 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::NewMap(idx))
            }
            42 => {
                let idx = Self::read_u32(buf)? as usize;
                Ok(OpCode::StringConcat(idx))
            }
            _ => Err(GError { kind: GErrorKind::Runtime, message: format!("未知的操作码: {}", opcode_byte) }),
        }
    }
}

/// 公共子表达式消除优化 Pass
pub mod common_subexpr;
/// 常量折叠优化 Pass
pub mod constant_fold;
/// 死代码消除优化 Pass
pub mod dead_code;
/// 逃逸分析 Pass
pub mod escape_analysis;
/// 内联展开优化 Pass
pub mod inline_expand;
/// 循环不变量外提优化 Pass
pub mod loop_invariant;
/// 循环展开优化 Pass
pub mod loop_unroll;
/// IR 优化 Pass 基础设施
pub mod pass;
/// 尾调用优化 Pass
pub mod tail_call;

/// 创建默认的 IR 优化器，包含常量折叠、死代码消除、内联展开、公共子表达式消除、尾调用优化、循环不变量外提和循环展开 Pass
pub fn default_optimizer() -> pass::IrOptimizer {
    let mut optimizer = pass::IrOptimizer::new();
    optimizer.add_pass(Box::new(constant_fold::ConstantFoldPass));
    optimizer.add_pass(Box::new(dead_code::DeadCodeElimPass));
    optimizer.add_pass(Box::new(inline_expand::InlineExpansionPass::new()));
    optimizer.add_pass(Box::new(common_subexpr::CommonSubexprElimPass::new()));
    optimizer.add_pass(Box::new(tail_call::TailCallOptPass));
    optimizer.add_pass(Box::new(loop_invariant::LoopInvariantCodeMotionPass));
    optimizer.add_pass(Box::new(loop_unroll::LoopUnrollingPass::new()));
    optimizer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_platform_from_str() {
        assert_eq!(TargetPlatform::from_str("desktop"), Some(TargetPlatform::Desktop));
        assert_eq!(TargetPlatform::from_str("mobile"), Some(TargetPlatform::Mobile));
        assert_eq!(TargetPlatform::from_str("web"), Some(TargetPlatform::Web));
        assert_eq!(TargetPlatform::from_str("server"), Some(TargetPlatform::Server));
        assert_eq!(TargetPlatform::from_str("all"), Some(TargetPlatform::All));
        assert_eq!(TargetPlatform::from_str("unknown"), None);
    }

    #[test]
    fn test_target_platform_as_str() {
        assert_eq!(TargetPlatform::Desktop.as_str(), "desktop");
        assert_eq!(TargetPlatform::Mobile.as_str(), "mobile");
        assert_eq!(TargetPlatform::Web.as_str(), "web");
        assert_eq!(TargetPlatform::Server.as_str(), "server");
        assert_eq!(TargetPlatform::All.as_str(), "all");
    }

    #[test]
    fn test_target_platform_round_trip() {
        for platform in
            [TargetPlatform::Desktop, TargetPlatform::Mobile, TargetPlatform::Web, TargetPlatform::Server, TargetPlatform::All]
        {
            let s = platform.as_str();
            assert_eq!(TargetPlatform::from_str(s), Some(platform));
        }
    }

    #[test]
    fn test_entry_point_creation() {
        let entry = EntryPoint { name: "main".to_string(), target: TargetPlatform::Desktop, priority: 10 };
        assert_eq!(entry.name, "main");
        assert_eq!(entry.target, TargetPlatform::Desktop);
        assert_eq!(entry.priority, 10);
    }

    #[test]
    fn test_ir_module_add_entry_point() {
        let mut module = IrModule::new("test_module");
        assert!(module.entry_points.is_empty());

        module.add_entry_point(EntryPoint { name: "main".to_string(), target: TargetPlatform::Desktop, priority: 0 });
        module.add_entry_point(EntryPoint { name: "game_loop".to_string(), target: TargetPlatform::Mobile, priority: 5 });

        assert_eq!(module.entry_points.len(), 2);
    }

    #[test]
    fn test_ir_module_find_entry_point() {
        let mut module = IrModule::new("test_module");
        module.add_entry_point(EntryPoint { name: "main".to_string(), target: TargetPlatform::Desktop, priority: 0 });
        module.add_entry_point(EntryPoint { name: "game_loop".to_string(), target: TargetPlatform::Mobile, priority: 5 });

        let found = module.find_entry_point("main");
        assert!(found.is_some());
        let entry = found.unwrap();
        assert_eq!(entry.name, "main");
        assert_eq!(entry.target, TargetPlatform::Desktop);
        assert_eq!(entry.priority, 0);

        assert!(module.find_entry_point("nonexistent").is_none());
    }

    #[test]
    fn test_ir_module_target_platform() {
        let mut module = IrModule::new("test_module");
        assert!(module.target_platform.is_none());

        module.target_platform = Some(TargetPlatform::Web);
        assert_eq!(module.target_platform, Some(TargetPlatform::Web));
    }

    #[test]
    fn test_ir_function_entry_fields() {
        let func = IrFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            local_names: vec![],
            instructions: vec![],
            is_entry: true,
            target: Some(TargetPlatform::Server),
        };
        assert!(func.is_entry);
        assert_eq!(func.target, Some(TargetPlatform::Server));
    }

    #[test]
    fn test_ir_function_default_entry_fields() {
        let func = IrFunction {
            name: "helper".to_string(),
            param_count: 1,
            local_count: 1,
            local_names: vec![],
            instructions: vec![],
            is_entry: false,
            target: None,
        };
        assert!(!func.is_entry);
        assert!(func.target.is_none());
    }

    #[test]
    fn test_target_platform_equality() {
        assert_eq!(TargetPlatform::Desktop, TargetPlatform::Desktop);
        assert_ne!(TargetPlatform::Desktop, TargetPlatform::Mobile);
    }

    #[test]
    fn test_target_platform_copy() {
        let a = TargetPlatform::Web;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_ir_function_local_names() {
        let func = IrFunction {
            name: "test_func".to_string(),
            param_count: 2,
            local_count: 3,
            local_names: vec!["self".to_string(), "arg1".to_string(), "result".to_string()],
            instructions: vec![OpCode::Return],
            is_entry: false,
            target: None,
        };
        assert_eq!(func.local_names.len(), 3);
        assert_eq!(func.local_names[0], "self");
        assert_eq!(func.local_names[1], "arg1");
        assert_eq!(func.local_names[2], "result");
    }

    #[test]
    fn test_opcode_tail_call() {
        let op = OpCode::TailCall(3);
        assert_eq!(op, OpCode::TailCall(3));
        assert_ne!(op, OpCode::Call(3));

        let op2 = OpCode::TailCall(0);
        assert_eq!(op2, OpCode::TailCall(0));
    }

    #[test]
    fn test_serialize_deserialize_round_trip() {
        let mut module = IrModule::new("test_round_trip");
        module.add_constant(IrValue::Int(42));
        module.add_constant(IrValue::Float(3.14));
        module.add_constant(IrValue::String("hello".to_string()));
        module.add_constant(IrValue::Bool(true));
        module.add_constant(IrValue::Entity(100));
        module.add_constant(IrValue::Null);

        module.add_or_get_string("Position".to_string());
        module.add_or_get_string("Velocity".to_string());

        module.add_function(IrFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 2,
            local_names: vec!["x".to_string(), "y".to_string()],
            instructions: vec![
                OpCode::LoadConst(0),
                OpCode::LoadConst(1),
                OpCode::Add,
                OpCode::StoreLocal(0),
                OpCode::LoadLocal(0),
                OpCode::TailCall(1),
                OpCode::Return,
            ],
            is_entry: true,
            target: Some(TargetPlatform::Desktop),
        });

        module.add_entry_point(EntryPoint { name: "main".to_string(), target: TargetPlatform::Desktop, priority: 0 });

        module.target_platform = Some(TargetPlatform::All);

        let binary = module.serialize_to_binary().expect("序列化失败");
        let restored = IrModule::deserialize_from_binary(&binary).expect("反序列化失败");

        assert_eq!(restored.name, "test_round_trip");

        assert_eq!(restored.constants.len(), 6);
        assert_eq!(restored.constants[0], IrValue::Int(42));
        assert_eq!(restored.constants[1], IrValue::Float(3.14));
        assert_eq!(restored.constants[2], IrValue::String("hello".to_string()));
        assert_eq!(restored.constants[3], IrValue::Bool(true));
        assert_eq!(restored.constants[4], IrValue::Entity(100));
        assert_eq!(restored.constants[5], IrValue::Null);

        assert_eq!(restored.string_pool.len(), 2);
        assert_eq!(restored.string_pool[0], "Position");
        assert_eq!(restored.string_pool[1], "Velocity");

        assert_eq!(restored.functions.len(), 1);
        let func = &restored.functions[0];
        assert_eq!(func.name, "main");
        assert_eq!(func.param_count, 0);
        assert_eq!(func.local_count, 2);
        assert_eq!(func.local_names, vec!["x".to_string(), "y".to_string()]);
        assert!(func.is_entry);
        assert_eq!(func.target, Some(TargetPlatform::Desktop));
        assert_eq!(func.instructions.len(), 7);

        assert_eq!(restored.entry_points.len(), 1);
        assert_eq!(restored.entry_points[0].name, "main");
        assert_eq!(restored.entry_points[0].target, TargetPlatform::Desktop);
        assert_eq!(restored.entry_points[0].priority, 0);

        assert_eq!(restored.target_platform, Some(TargetPlatform::All));
    }

    #[test]
    fn test_serialize_deserialize_binary_stability() {
        let mut module = IrModule::new("stability_test");
        module.add_constant(IrValue::Int(10));
        module.add_constant(IrValue::Bool(false));

        module.add_function(IrFunction {
            name: "compute".to_string(),
            param_count: 1,
            local_count: 1,
            local_names: vec!["input".to_string()],
            instructions: vec![OpCode::LoadConst(0), OpCode::LoadLocal(0), OpCode::Add, OpCode::Return],
            is_entry: false,
            target: None,
        });

        let binary1 = module.serialize_to_binary().expect("第一次序列化失败");
        let binary2 = module.serialize_to_binary().expect("第二次序列化失败");
        assert_eq!(binary1, binary2, "相同模块的序列化结果应稳定一致");
    }

    #[test]
    fn test_serialize_deserialize_empty_module() {
        let module = IrModule::new("empty");
        let binary = module.serialize_to_binary().expect("空模块序列化失败");
        let restored = IrModule::deserialize_from_binary(&binary).expect("空模块反序列化失败");

        assert_eq!(restored.name, "empty");
        assert!(restored.constants.is_empty());
        assert!(restored.string_pool.is_empty());
        assert!(restored.functions.is_empty());
        assert!(restored.entry_points.is_empty());
        assert!(restored.target_platform.is_none());
    }

    #[test]
    fn test_serialize_invalid_magic() {
        let data = vec![0u8; 8];
        let result = IrModule::deserialize_from_binary(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_invalid_version() {
        let mut data = vec![];
        data.extend_from_slice(&0x47474952u32.to_le_bytes());
        data.extend_from_slice(&999u32.to_le_bytes());
        let result = IrModule::deserialize_from_binary(&data);
        assert!(result.is_err());
    }
}
