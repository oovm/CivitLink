//! 属性编辑器模块
//!
//! 提供属性编辑器组件 trait、编辑器工厂 trait、编辑器注册表，
//! 以及内置的属性类型编辑器工厂实现。

use crate::descriptor::PropertyType;

/// 属性编辑器组件 trait
///
/// 定义单个属性编辑器的值读写接口，
/// 用于在检查器面板中编辑属性值。
pub trait PropertyEditorWidget {
    /// 设置编辑器当前值
    fn set_value(&mut self, value: &str);

    /// 获取编辑器当前值
    fn get_value(&self) -> String;

    /// 值是否已被修改
    fn is_modified(&self) -> bool;
}

/// 属性编辑器工厂 trait
///
/// 定义属性编辑器的创建接口，根据属性类型判断能否编辑并创建对应的编辑器组件。
pub trait PropertyEditorFactory {
    /// 判断此工厂是否能编辑指定属性类型
    fn can_edit(&self, property_type: &PropertyType) -> bool;

    /// 创建属性编辑器组件
    fn create_editor(&self) -> Box<dyn PropertyEditorWidget>;
}

/// 属性编辑器注册表
///
/// 管理所有属性编辑器工厂的注册和查询，
/// 根据属性类型查找合适的工厂创建编辑器组件。
pub struct PropertyEditorRegistry {
    /// 已注册的编辑器工厂列表
    factories: Vec<Box<dyn PropertyEditorFactory>>,
}

impl PropertyEditorRegistry {
    /// 创建空的属性编辑器注册表
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
        }
    }

    /// 注册编辑器工厂
    pub fn register_factory(&mut self, factory: Box<dyn PropertyEditorFactory>) {
        self.factories.push(factory);
    }

    /// 根据属性类型创建编辑器组件
    ///
    /// 遍历已注册的工厂，返回第一个能编辑指定属性类型的工厂所创建的编辑器。
    pub fn create_editor(&self, property_type: &PropertyType) -> Option<Box<dyn PropertyEditorWidget>> {
        self.factories
            .iter()
            .find(|f| f.can_edit(property_type))
            .map(|f| f.create_editor())
    }
}

impl Default for PropertyEditorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 字符串编辑器组件
///
/// 存储字符串值的简单编辑器组件，用于编辑 `PropertyType::String` 类型属性。
pub struct StringEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl StringEditorWidget {
    /// 创建新的字符串编辑器组件
    pub fn new() -> Self {
        Self {
            value: String::new(),
            initial_value: String::new(),
        }
    }
}

impl Default for StringEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for StringEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value.is_empty() {
            self.initial_value = value.to_string();
        }
        self.value = value.to_string();
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }

    fn is_modified(&self) -> bool {
        self.value != self.initial_value
    }
}

/// 字符串编辑器工厂
///
/// 为 `PropertyType::String` 类型属性创建 `StringEditorWidget`。
pub struct StringEditorFactory;

impl PropertyEditorFactory for StringEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::String)
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(StringEditorWidget::new())
    }
}

/// 数值编辑器组件
///
/// 存储数值字符串的简单编辑器组件，用于编辑 `PropertyType::Int` 和 `PropertyType::Float` 类型属性。
pub struct NumericEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl NumericEditorWidget {
    /// 创建新的数值编辑器组件
    pub fn new() -> Self {
        Self {
            value: String::new(),
            initial_value: String::new(),
        }
    }
}

impl Default for NumericEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for NumericEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value.is_empty() {
            self.initial_value = value.to_string();
        }
        self.value = value.to_string();
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }

    fn is_modified(&self) -> bool {
        self.value != self.initial_value
    }
}

/// 数值编辑器工厂
///
/// 为 `PropertyType::Int` 和 `PropertyType::Float` 类型属性创建 `NumericEditorWidget`。
pub struct NumericEditorFactory;

impl PropertyEditorFactory for NumericEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Int | PropertyType::Float)
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(NumericEditorWidget::new())
    }
}

/// 布尔编辑器组件
///
/// 存储布尔值的简单编辑器组件，用于编辑 `PropertyType::Bool` 类型属性。
pub struct BoolEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl BoolEditorWidget {
    /// 创建新的布尔编辑器组件
    pub fn new() -> Self {
        Self {
            value: String::new(),
            initial_value: String::new(),
        }
    }
}

impl Default for BoolEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for BoolEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value.is_empty() {
            self.initial_value = value.to_string();
        }
        self.value = value.to_string();
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }

    fn is_modified(&self) -> bool {
        self.value != self.initial_value
    }
}

/// 布尔编辑器工厂
///
/// 为 `PropertyType::Bool` 类型属性创建 `BoolEditorWidget`。
pub struct BoolEditorFactory;

impl PropertyEditorFactory for BoolEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Bool)
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(BoolEditorWidget::new())
    }
}

/// 枚举编辑器组件
///
/// 存储枚举值的简单编辑器组件，用于编辑 `PropertyType::Enum` 类型属性。
pub struct EnumEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl EnumEditorWidget {
    /// 创建新的枚举编辑器组件
    pub fn new() -> Self {
        Self {
            value: String::new(),
            initial_value: String::new(),
        }
    }
}

impl Default for EnumEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for EnumEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value.is_empty() {
            self.initial_value = value.to_string();
        }
        self.value = value.to_string();
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }

    fn is_modified(&self) -> bool {
        self.value != self.initial_value
    }
}

/// 枚举编辑器工厂
///
/// 为 `PropertyType::Enum` 类型属性创建 `EnumEditorWidget`。
pub struct EnumEditorFactory;

impl PropertyEditorFactory for EnumEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Enum(_))
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(EnumEditorWidget::new())
    }
}

/// 颜色编辑器组件
///
/// 存储颜色值的简单编辑器组件，用于编辑 `PropertyType::Color` 类型属性。
pub struct ColorEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl ColorEditorWidget {
    /// 创建新的颜色编辑器组件
    pub fn new() -> Self {
        Self {
            value: String::new(),
            initial_value: String::new(),
        }
    }
}

impl Default for ColorEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for ColorEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value.is_empty() {
            self.initial_value = value.to_string();
        }
        self.value = value.to_string();
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }

    fn is_modified(&self) -> bool {
        self.value != self.initial_value
    }
}

/// 颜色编辑器工厂
///
/// 为 `PropertyType::Color` 类型属性创建 `ColorEditorWidget`。
pub struct ColorEditorFactory;

impl PropertyEditorFactory for ColorEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Color)
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(ColorEditorWidget::new())
    }
}

/// 资源路径编辑器组件
///
/// 存储资源路径的简单编辑器组件，用于编辑 `PropertyType::AssetPath` 类型属性。
pub struct AssetPathEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl AssetPathEditorWidget {
    /// 创建新的资源路径编辑器组件
    pub fn new() -> Self {
        Self {
            value: String::new(),
            initial_value: String::new(),
        }
    }
}

impl Default for AssetPathEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for AssetPathEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value.is_empty() {
            self.initial_value = value.to_string();
        }
        self.value = value.to_string();
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }

    fn is_modified(&self) -> bool {
        self.value != self.initial_value
    }
}

/// 资源路径编辑器工厂
///
/// 为 `PropertyType::AssetPath` 类型属性创建 `AssetPathEditorWidget`。
pub struct AssetPathEditorFactory;

impl PropertyEditorFactory for AssetPathEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::AssetPath(_))
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(AssetPathEditorWidget::new())
    }
}
