//! gs AST → naga IR 转换器的类型定义

use naga::{Scalar as NagaScalar, ScalarKind};

use rustc_hash::FxHashSet;

/// 命名表达式映射类型（表达式句柄 → 名称）
pub type NamedExpressions = naga::FastIndexMap<naga::Handle<naga::Expression>, String>;

/// 变体条件编译启用的关键字集合
pub type EnabledKeywords = FxHashSet<String>;

/// gs 着色器属性信息
///
/// 表示 shader 块中的一个属性声明，如 `let albedo: texture = "white"`。
#[derive(Debug, Clone)]
pub struct GsProperty {
    /// 属性名称
    pub name: String,
    /// 属性类型名称（如 "texture"、"f32"、"color"、"vector"）
    pub type_name: String,
    /// 属性默认值
    pub default_value: Option<String>,
}

/// gs 着色器入口点信息
///
/// 表示一个带有 `@vertex`/`@fragment`/`@compute` 注解的 micro 函数。
#[derive(Debug, Clone)]
pub struct GsEntryPoint {
    /// 入口点名称
    pub name: String,
    /// 着色器阶段
    pub stage: naga::ShaderStage,
    /// 原始 micro 声明
    pub micro: oak_valkyrie::ast::MicroDeclaration,
}

/// gs uniform 字段信息
///
/// 表示 uniform 结构体中的一个字段。
#[derive(Debug, Clone)]
pub struct GsUniformField {
    /// 字段名称
    pub name: String,
    /// 字段类型名称（如 "mat44"、"vec3"、"vec4"、"f32"、"tex2"）
    pub type_name: String,
}

/// gs 着色器渲染状态信息
///
/// 表示 render_states 块中的键值对。
#[derive(Debug, Clone)]
pub struct GsRenderState {
    /// 状态名称
    pub name: String,
    /// 状态值
    pub value: String,
}

/// 类型大小和对齐信息
pub struct TypeSizeAlign {
    /// 类型大小（字节）
    pub size: u32,
    /// 类型对齐（字节）
    pub align: u32,
}

/// 类型布局信息（std140/std430 对齐规则计算结果）
pub struct TypeLayoutInfo {
    /// 结构体总大小（字节）
    pub size: u32,
    /// 结构体最大对齐（字节）
    pub align: u32,
    /// 每个字段的偏移量
    pub field_offsets: Vec<u32>,
}

/// 计算 std140 布局
///
/// 按照 std140 对齐规则计算结构体字段的偏移量和总大小。
/// std140 规则中，数组步长和结构体大小都对齐到 vec4（16 字节）。
pub fn calc_std140_layout(fields: &[(String, String)], type_size_align_fn: impl Fn(&str) -> TypeSizeAlign) -> TypeLayoutInfo {
    let mut offset = 0u32;
    let mut max_align = 1u32;
    let mut field_offsets = Vec::with_capacity(fields.len());

    for (_name, type_name) in fields {
        let sa = type_size_align_fn(type_name);
        let align = sa.align.max(1);
        max_align = max_align.max(align);
        offset = align_offset(offset, align);
        field_offsets.push(offset);
        offset += sa.size;
    }

    let struct_align = max_align.max(16);
    offset = align_offset(offset, struct_align);

    TypeLayoutInfo { size: offset, align: struct_align, field_offsets }
}

/// 计算 std430 布局
///
/// 按照 std430 对齐规则计算结构体字段的偏移量和总大小。
/// std430 规则中，数组步长和结构体大小对齐到元素/结构体自身的对齐（非 vec4）。
pub fn calc_std430_layout(fields: &[(String, String)], type_size_align_fn: impl Fn(&str) -> TypeSizeAlign) -> TypeLayoutInfo {
    let mut offset = 0u32;
    let mut max_align = 1u32;
    let mut field_offsets = Vec::with_capacity(fields.len());

    for (_name, type_name) in fields {
        let sa = type_size_align_fn(type_name);
        let align = sa.align.max(1);
        max_align = max_align.max(align);
        offset = align_offset(offset, align);
        field_offsets.push(offset);
        offset += sa.size;
    }

    let struct_align = max_align.max(1);
    offset = align_offset(offset, struct_align);

    TypeLayoutInfo { size: offset, align: struct_align, field_offsets }
}

/// 对齐偏移量
pub fn align_offset(offset: u32, align: u32) -> u32 {
    if align == 0 {
        return offset;
    }
    (offset + align - 1) / align * align
}

/// f32 标量常量
pub const F32_SCALAR: NagaScalar = NagaScalar { kind: ScalarKind::Float, width: 4 };
/// i32 标量常量
pub const I32_SCALAR: NagaScalar = NagaScalar { kind: ScalarKind::Sint, width: 4 };
/// u32 标量常量
pub const U32_SCALAR: NagaScalar = NagaScalar { kind: ScalarKind::Uint, width: 4 };
/// bool 标量常量
pub const BOOL_SCALAR: NagaScalar = NagaScalar { kind: ScalarKind::Bool, width: 1 };
