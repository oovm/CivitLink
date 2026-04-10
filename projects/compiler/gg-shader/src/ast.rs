//! gs 语言类型化 AST 定义
//!
//! 定义 GG Shader (gs) 语言的所有 AST 节点类型，
//! 包含完整的类型信息，用于解析器输出和 naga IR 转换器输入。

/// gs 语言标量类型
#[derive(Debug, Clone, PartialEq)]
pub enum GslScalarType {
    /// 布尔类型
    Bool,
    /// 无符号 32 位整数
    U32,
    /// 有符号 32 位整数
    I32,
    /// 32 位浮点数
    F32,
}

/// gs 语言向量维度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GslVectorSize {
    /// 二维向量
    Bi,
    /// 三维向量
    Tri,
    /// 四维向量
    Quad,
}

/// gs 语言类型
#[derive(Debug, Clone, PartialEq)]
pub enum GslType {
    /// 标量类型
    Scalar(GslScalarType),
    /// 向量类型，包含维度和标量类型
    Vector {
        /// 向量维度
        size: GslVectorSize,
        /// 标量类型
        scalar: GslScalarType,
    },
    /// 矩阵类型，包含列数、行数和标量类型
    Matrix {
        /// 列数
        columns: GslVectorSize,
        /// 行数
        rows: GslVectorSize,
        /// 标量类型
        scalar: GslScalarType,
    },
    /// 2D 纹理类型
    Texture2D,
    /// 3D 纹理类型
    Texture3D,
    /// 立方体纹理类型
    TextureCube,
    /// 采样器类型
    Sampler,
    /// 缓冲区类型，包含元素类型
    Buffer(Box<GslType>),
    /// 结构体类型，包含字段列表
    Struct {
        /// 结构体字段列表
        fields: Vec<GslStructField>,
    },
}

/// gs 语言结构体字段
#[derive(Debug, Clone, PartialEq)]
pub struct GslStructField {
    /// 字段名称
    pub name: String,
    /// 字段类型
    pub ty: GslType,
    /// 字段装饰器列表
    pub decorators: Vec<GslDecorator>,
}

/// gs 语言装饰器
#[derive(Debug, Clone, PartialEq)]
pub enum GslDecorator {
    /// 位置装饰器，指定输入/输出位置
    Location(u32),
    /// 内建变量装饰器
    Builtin(GslBuiltinKind),
    /// 资源组装饰器
    Group(u32),
    /// 资源绑定装饰器
    Binding(u32),
    /// 范围装饰器，指定属性范围
    Range(f32, f32),
}

/// gs 语言内建变量类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GslBuiltinKind {
    /// 裁剪空间位置
    Position,
    /// 全局调用 ID（计算着色器）
    GlobalInvocationId,
    /// 顶点索引
    VertexIndex,
    /// 实例索引
    InstanceIndex,
}

/// gs 语言二元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GslBinaryOp {
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
    /// 等于
    Eq,
    /// 不等于
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
}

/// gs 语言一元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GslUnaryOp {
    /// 取负
    Neg,
    /// 逻辑非
    Not,
}

/// gs 语言表达式
#[derive(Debug, Clone, PartialEq)]
pub enum GslExpr {
    /// 字面量：整数
    LiteralInt(i64),
    /// 字面量：浮点数
    LiteralFloat(f64),
    /// 字面量：布尔值
    LiteralBool(bool),
    /// 变量引用
    Variable(String),
    /// 二元运算
    Binary {
        /// 左操作数
        left: Box<GslExpr>,
        /// 运算符
        op: GslBinaryOp,
        /// 右操作数
        right: Box<GslExpr>,
    },
    /// 一元运算
    Unary {
        /// 运算符
        op: GslUnaryOp,
        /// 操作数
        operand: Box<GslExpr>,
    },
    /// 函数调用
    Call {
        /// 函数名称（支持命名空间路径，如 `utils::calculate`）
        func: String,
        /// 参数列表
        args: Vec<GslExpr>,
    },
    /// 向量构造
    VectorConstruct {
        /// 元素类型
        scalar: GslScalarType,
        /// 维度
        size: GslVectorSize,
        /// 构造参数
        args: Vec<GslExpr>,
    },
    /// 矩阵构造
    MatrixConstruct {
        /// 标量类型
        scalar: GslScalarType,
        /// 列数
        columns: GslVectorSize,
        /// 行数
        rows: GslVectorSize,
        /// 构造参数
        args: Vec<GslExpr>,
    },
    /// 成员访问
    Member {
        /// 对象表达式
        object: Box<GslExpr>,
        /// 成员名称
        member: String,
    },
    /// 索引访问
    Index {
        /// 对象表达式
        object: Box<GslExpr>,
        /// 索引表达式
        index: Box<GslExpr>,
    },
    /// 纹理采样
    TextureSample {
        /// 纹理表达式
        texture: Box<GslExpr>,
        /// 采样器表达式
        sampler: Box<GslExpr>,
        /// 坐标表达式
        coords: Box<GslExpr>,
    },
}

/// gs 语言语句
#[derive(Debug, Clone, PartialEq)]
pub enum GslStmt {
    /// 不可变变量声明
    Let {
        /// 变量名称
        name: String,
        /// 变量类型（可省略，由推断确定）
        ty: Option<GslType>,
        /// 初始值表达式
        value: GslExpr,
    },
    /// 可变变量声明
    LetMut {
        /// 变量名称
        name: String,
        /// 变量类型（可省略，由推断确定）
        ty: Option<GslType>,
        /// 初始值表达式
        value: GslExpr,
    },
    /// 赋值语句
    Assign {
        /// 目标表达式
        target: GslExpr,
        /// 值表达式
        value: GslExpr,
    },
    /// 返回语句
    Return {
        /// 返回值表达式（可省略）
        value: Option<GslExpr>,
    },
    /// if 语句
    If {
        /// 条件表达式
        condition: GslExpr,
        /// then 分支语句
        then_block: Vec<GslStmt>,
        /// else 分支语句（可省略）
        else_block: Option<Vec<GslStmt>>,
    },
    /// 表达式语句
    Expr {
        /// 表达式
        expr: GslExpr,
    },
}

/// gs 语言函数参数
#[derive(Debug, Clone, PartialEq)]
pub struct GslParam {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub ty: GslType,
    /// 参数装饰器列表
    pub decorators: Vec<GslDecorator>,
}

/// gs 语言函数类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GslFunctionKind {
    /// 顶点着色器
    Vertex,
    /// 片段着色器
    Fragment,
    /// 计算着色器
    Compute,
    /// 微型函数（可复用的辅助函数）
    Micro,
}

/// gs 语言函数定义
#[derive(Debug, Clone, PartialEq)]
pub struct GslFunction {
    /// 函数类型
    pub kind: GslFunctionKind,
    /// 函数名称
    pub name: String,
    /// 参数列表
    pub params: Vec<GslParam>,
    /// 返回类型（可省略）
    pub return_type: Option<GslType>,
    /// 返回类型装饰器
    pub return_decorators: Vec<GslDecorator>,
    /// 函数体语句
    pub body: Vec<GslStmt>,
}

/// gs 语言 uniform 声明
#[derive(Debug, Clone, PartialEq)]
pub struct GslUniform {
    /// uniform 名称
    pub name: String,
    /// uniform 类型
    pub ty: GslType,
    /// 装饰器列表（@group, @binding）
    pub decorators: Vec<GslDecorator>,
}

/// gs 语言剔除模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GslCullMode {
    /// 背面剔除
    Back,
    /// 正面剔除
    Front,
    /// 不剔除
    None,
}

/// gs 语言混合模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GslBlendMode {
    /// 不透明
    Opaque,
    /// Alpha 混合
    Alpha,
    /// 加法混合
    Additive,
}

/// gs 语言渲染状态
#[derive(Debug, Clone, PartialEq)]
pub struct GslRenderStates {
    /// 剔除模式
    pub cull_mode: GslCullMode,
    /// 混合模式
    pub blend_mode: GslBlendMode,
    /// 是否启用深度测试
    pub depth_test: bool,
    /// 是否启用深度写入
    pub depth_write: bool,
    /// 是否以线框模式渲染
    pub wireframe: bool,
}

/// gs 语言着色器属性
#[derive(Debug, Clone, PartialEq)]
pub struct GslProperty {
    /// 属性名称
    pub name: String,
    /// 属性类型
    pub ty: String,
    /// 默认值
    pub default_value: Option<String>,
    /// 范围装饰器
    pub range: Option<(f32, f32)>,
}

/// gs 语言回退策略
#[derive(Debug, Clone, PartialEq)]
pub struct GslFallback {
    /// 回退条件
    pub when: String,
    /// 回退目标着色器名称
    pub shader: String,
}

/// gs 语言着色器块
#[derive(Debug, Clone, PartialEq)]
pub struct GslShaderBlock {
    /// 着色器名称
    pub name: String,
    /// 着色器类型（如 PBR、Phong、Unlit、Compute）
    pub kind: String,
    /// 着色器属性列表
    pub properties: Vec<GslProperty>,
    /// 渲染队列
    pub render_queue: Option<String>,
    /// 渲染状态
    pub render_states: Option<GslRenderStates>,
    /// 着色器函数列表
    pub functions: Vec<GslFunction>,
    /// Uniforms 列表
    pub uniforms: Vec<GslUniform>,
    /// 回退策略
    pub fallback: Option<GslFallback>,
}

/// gs 语言命名空间声明
#[derive(Debug, Clone, PartialEq)]
pub struct GslNamespace {
    /// 命名空间路径（如 `my_shaders::common`）
    pub path: String,
    /// 命名空间内的函数列表
    pub functions: Vec<GslFunction>,
}

/// gs 语言 using 声明
#[derive(Debug, Clone, PartialEq)]
pub struct GslUsing {
    /// 引入路径
    pub path: String,
}

/// gs 语言文件解析结果
#[derive(Debug, Clone, PartialEq)]
pub struct GslShaderFile {
    /// 着色器块列表
    pub shaders: Vec<GslShaderBlock>,
    /// 命名空间列表
    pub namespaces: Vec<GslNamespace>,
    /// using 声明列表
    pub usings: Vec<GslUsing>,
    /// 独立 micro 函数列表
    pub micro_functions: Vec<GslFunction>,
}
