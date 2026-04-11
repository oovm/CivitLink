//! 属性描述符模块
//!
//! 提供属性类型、属性约束、属性描述符、组件描述符和描述符注册表，
//! 用于描述组件的属性结构信息，驱动检查器面板的动态属性编辑。
//! 支持从反射注册表自动生成组件描述符。

use gg_reflection::ReflectionRegistry;

/// 属性类型枚举
///
/// 定义属性支持的数据类型，用于驱动编辑器选择合适的属性编辑器。
#[derive(Debug, Clone)]
pub enum PropertyType {
    /// 字符串类型
    String,
    /// 整数类型
    Int,
    /// 浮点数类型
    Float,
    /// 布尔类型
    Bool,
    /// 枚举类型，包含所有可选值
    Enum(Vec<String>),
    /// 颜色类型
    Color,
    /// 资源路径类型，包含资源扩展名过滤
    AssetPath(String),
    /// 二维向量类型
    Vec2,
    /// 自定义类型，包含类型标识符
    Custom(String),
    /// 数组类型，元素类型由内部 PropertyType 指定
    Array(Box<PropertyType>),
    /// 映射类型，键值类型分别指定
    Map {
        /// 键类型
        key_type: Box<PropertyType>,
        /// 值类型
        value_type: Box<PropertyType>,
    },
    /// 三维向量类型
    Vec3,
    /// 四维向量类型
    Vec4,
    /// 矩形类型，包含 x、y、w、h 四个分量
    Rect,
    /// 实体引用类型，引用另一个实体
    EntityRef,
    /// 结构体类型，包含结构体名称和字段列表
    Struct {
        /// 结构体名称
        name: String,
        /// 字段描述符列表
        fields: Vec<PropertyDescriptor>,
    },
}

/// 属性约束
///
/// 为属性值提供验证和 UI 编辑约束，如范围限制、步进值和最大长度。
#[derive(Debug, Clone, Default)]
pub struct PropertyConstraints {
    /// 最小值
    pub min_value: Option<f64>,
    /// 最大值
    pub max_value: Option<f64>,
    /// 步进值
    pub step: Option<f64>,
    /// 最大长度
    pub max_length: Option<usize>,
}

/// 属性描述符
///
/// 描述单个属性的类型信息、显示名称和约束条件，
/// 用于驱动检查器面板自动生成对应的属性编辑器。
#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    /// 属性名称
    pub name: String,
    /// 显示名称
    pub display_name: String,
    /// 属性类型
    pub property_type: PropertyType,
    /// 默认值
    pub default_value: Option<String>,
    /// 属性约束
    pub constraints: Option<PropertyConstraints>,
}

/// 组件描述符
///
/// 描述一个组件类型的所有属性信息，用于驱动检查器面板
/// 自动生成该组件的完整属性编辑界面。
#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    /// 类型名称
    pub type_name: String,
    /// 显示名称
    pub display_name: String,
    /// 属性列表
    pub properties: Vec<PropertyDescriptor>,
}

/// 描述符注册表
///
/// 管理所有组件描述符的注册和查询，为检查器面板提供
/// 按类型名称查找组件属性结构信息的能力。
/// 支持从反射注册表自动生成组件描述符。
#[derive(Debug, Clone, Default)]
pub struct DescriptorRegistry {
    /// 已注册的组件描述符列表
    descriptors: Vec<ComponentDescriptor>,
}

impl DescriptorRegistry {
    /// 创建空的描述符注册表
    pub fn new() -> Self {
        Self { descriptors: Vec::new() }
    }

    /// 注册组件描述符
    ///
    /// 将组件描述符添加到注册表中，若同名描述符已存在则替换。
    pub fn register_component(&mut self, descriptor: ComponentDescriptor) {
        if let Some(existing) = self.descriptors.iter_mut().find(|d| d.type_name == descriptor.type_name) {
            *existing = descriptor;
        }
        else {
            self.descriptors.push(descriptor);
        }
    }

    /// 获取组件描述符
    ///
    /// 根据类型名称查找已注册的组件描述符。
    pub fn get_component(&self, type_name: &str) -> Option<&ComponentDescriptor> {
        self.descriptors.iter().find(|d| d.type_name == type_name)
    }

    /// 获取所有组件描述符
    pub fn component_descriptors(&self) -> &[ComponentDescriptor] {
        &self.descriptors
    }

    /// 从反射注册表自动生成组件描述符
    ///
    /// 遍历反射注册表中所有已注册的类型，为每个类型生成组件描述符。
    /// 属性类型根据类型名称自动推断：
    /// - "String" 或 "alloc::string::String" → PropertyType::String
    /// - "i32"、"i64"、"u32"、"u64" → PropertyType::Int
    /// - "f32"、"f64" → PropertyType::Float
    /// - "bool" → PropertyType::Bool
    /// - 其他 → PropertyType::Custom(type_name)
    pub fn generate_from_reflection(reflection_registry: &ReflectionRegistry) -> Vec<ComponentDescriptor> {
        reflection_registry
            .get()
            .values()
            .map(|registration| {
                let type_info = registration.type_info();
                ComponentDescriptor {
                    type_name: type_info.type_name.to_string(),
                    display_name: type_info.short_name.clone(),
                    properties: vec![PropertyDescriptor {
                        name: "value".to_string(),
                        display_name: "值".to_string(),
                        property_type: infer_property_type(type_info.type_name),
                        default_value: None,
                        constraints: None,
                    }],
                }
            })
            .collect()
    }
}

/// 根据类型名称推断属性类型
///
/// 将 Rust 类型名称映射为 `PropertyType` 枚举值，
/// 用于从反射信息自动生成属性描述符。
/// 支持推断基础类型和常见复合类型（Vec2、Vec3、Vec4、Rect、EntityRef）。
fn infer_property_type(type_name: &str) -> PropertyType {
    match type_name {
        "String" | "alloc::string::String" => PropertyType::String,
        "i32" | "i64" | "u32" | "u64" => PropertyType::Int,
        "f32" | "f64" => PropertyType::Float,
        "bool" => PropertyType::Bool,
        _ => {
            let short = type_name.split("::").last().unwrap_or(type_name);
            match short {
                "Vec2" => PropertyType::Vec2,
                "Vec3" => PropertyType::Vec3,
                "Vec4" => PropertyType::Vec4,
                "Rect" => PropertyType::Rect,
                name if name.contains("EntityRef") || name.contains("Entity") => PropertyType::EntityRef,
                _ => PropertyType::Custom(type_name.to_string()),
            }
        }
    }
}
