//! Schema IR 中间表示类型模块
//! 定义 Schema DSL 编译过程中的所有中间表示类型

use serde::{Deserialize, Serialize};

/// Schema 根 IR，包含命名空间下所有定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaIr {
    /// 命名空间名称
    pub namespace: String,
    /// 模型定义列表
    pub models: Vec<ModelIr>,
    /// 枚举定义列表
    pub enums: Vec<EnumIr>,
    /// 消息定义列表
    pub messages: Vec<MessageIr>,
    /// 服务定义列表
    pub services: Vec<ServiceIr>,
    /// 数据库配置列表
    pub schema_configs: Vec<SchemaConfigIr>,
}

/// Schema 数据库配置 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfigIr {
    /// 配置名称
    pub name: String,
    /// 数据库方言
    pub dialect: String,
    /// 连接字符串
    pub connection: String,
    /// 连接池大小
    pub pool_size: Option<u32>,
}

/// 模型定义 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelIr {
    /// 模型名称
    pub name: String,
    /// 数据库表名
    pub table_name: String,
    /// 字段列表
    pub fields: Vec<FieldIr>,
    /// 注解列表
    pub annotations: Vec<AnnotationIr>,
}

/// 枚举定义 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumIr {
    /// 枚举名称
    pub name: String,
    /// 变体列表
    pub variants: Vec<EnumVariantIr>,
}

/// 枚举变体 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariantIr {
    /// 变体名称
    pub name: String,
    /// 变体值
    pub value: i32,
}

/// 消息定义 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageIr {
    /// 消息名称
    pub name: String,
    /// 字段列表
    pub fields: Vec<FieldIr>,
}

/// 服务定义 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIr {
    /// 服务名称
    pub name: String,
    /// 方法列表
    pub methods: Vec<ServiceMethodIr>,
}

/// 服务方法 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMethodIr {
    /// 方法名称
    pub name: String,
    /// 请求类型名称
    pub request_type: String,
    /// 响应类型名称
    pub response_type: String,
    /// RPC 方法类型
    pub method_type: RpcMethodType,
    /// 注解列表
    pub annotations: Vec<AnnotationIr>,
}

/// RPC 方法类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcMethodType {
    /// 一元调用
    Unary,
    /// 服务端流
    ServerStream,
    /// 客户端流
    ClientStream,
    /// 双向流
    Bidirectional,
}

/// 字段定义 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldIr {
    /// 字段名称
    pub name: String,
    /// 字段类型
    pub field_type: FieldTypeIr,
    /// 默认值
    pub default_value: Option<String>,
    /// 注解列表
    pub annotations: Vec<AnnotationIr>,
    /// 是否可选
    pub is_optional: bool,
}

/// 字段类型 IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldTypeIr {
    /// 8 位有符号整数
    I8,
    /// 16 位有符号整数
    I16,
    /// 32 位有符号整数
    I32,
    /// 64 位有符号整数
    I64,
    /// 8 位无符号整数
    U8,
    /// 16 位无符号整数
    U16,
    /// 32 位无符号整数
    U32,
    /// 64 位无符号整数
    U64,
    /// 32 位浮点数
    F32,
    /// 64 位浮点数
    F64,
    /// 布尔类型
    Bool,
    /// 字符串类型
    String,
    /// 字节数组类型
    Bytes,
    /// 日期时间类型
    Datetime,
    /// UUID 类型
    Uuid,
    /// 数组类型
    Array(Box<FieldTypeIr>),
    /// 引用数组类型
    RefArray(Box<FieldTypeIr>),
    /// 映射类型
    Map(Box<FieldTypeIr>, Box<FieldTypeIr>),
    /// 自定义类型
    Custom(String),
    /// 可选类型
    Optional(Box<FieldTypeIr>),
}

/// 注解 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationIr {
    /// 注解名称
    pub name: String,
    /// 注解参数列表
    pub arguments: Vec<AnnotationArgIr>,
}

/// 注解参数 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationArgIr {
    /// 参数名称，位置参数时为 None
    pub name: Option<String>,
    /// 参数值
    pub value: String,
}
