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
    /// 按类型名称注册的编辑器工厂
    typed_factories: std::collections::HashMap<String, Box<dyn PropertyEditorFactory>>,
}

impl PropertyEditorRegistry {
    /// 创建空的属性编辑器注册表
    pub fn new() -> Self {
        Self { factories: Vec::new(), typed_factories: std::collections::HashMap::new() }
    }

    /// 注册编辑器工厂
    pub fn register_factory(&mut self, factory: Box<dyn PropertyEditorFactory>) {
        self.factories.push(factory);
    }

    /// 按类型名称注册自定义编辑器工厂
    ///
    /// 当 Inspector 遇到 `PropertyType::Custom(type_name)` 类型属性时，
    /// 优先使用按类型名称注册的专用工厂创建编辑器。
    pub fn register_factory_for_type(&mut self, type_name: String, factory: Box<dyn PropertyEditorFactory>) {
        self.typed_factories.insert(type_name, factory);
    }

    /// 根据属性类型创建编辑器组件
    ///
    /// 遍历已注册的工厂，返回第一个能编辑指定属性类型的工厂所创建的编辑器。
    pub fn create_editor(&self, property_type: &PropertyType) -> Option<Box<dyn PropertyEditorWidget>> {
        if let PropertyType::Custom(type_name) = property_type {
            if let Some(factory) = self.typed_factories.get(type_name) {
                return Some(factory.create_editor());
            }
        }
        self.factories.iter().find(|f| f.can_edit(property_type)).map(|f| f.create_editor())
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
        Self { value: String::new(), initial_value: String::new() }
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
        Self { value: String::new(), initial_value: String::new() }
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
        Self { value: String::new(), initial_value: String::new() }
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
        Self { value: String::new(), initial_value: String::new() }
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
        Self { value: String::new(), initial_value: String::new() }
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
        Self { value: String::new(), initial_value: String::new() }
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

/// 二维向量编辑器组件
///
/// 存储二维向量值的编辑器组件，用于编辑 `PropertyType::Vec2` 类型属性。
/// 值格式为 "x,y" 字符串。
pub struct Vec2EditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl Vec2EditorWidget {
    /// 创建新的二维向量编辑器组件
    pub fn new() -> Self {
        Self { value: "0,0".to_string(), initial_value: "0,0".to_string() }
    }
}

impl Default for Vec2EditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for Vec2EditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value == "0,0" {
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

/// 二维向量编辑器工厂
///
/// 为 `PropertyType::Vec2` 类型属性创建 `Vec2EditorWidget`。
pub struct Vec2EditorFactory;

impl PropertyEditorFactory for Vec2EditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Vec2)
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(Vec2EditorWidget::new())
    }
}

/// 数组编辑器组件
///
/// 存储数组值的编辑器组件，用于编辑 `PropertyType::Array` 类型属性。
/// 值格式为 JSON 数组字符串。
pub struct ArrayEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl ArrayEditorWidget {
    /// 创建新的数组编辑器组件
    pub fn new() -> Self {
        Self { value: "[]".to_string(), initial_value: "[]".to_string() }
    }
}

impl Default for ArrayEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for ArrayEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value == "[]" {
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

/// 数组编辑器工厂
///
/// 为 `PropertyType::Array` 类型属性创建 `ArrayEditorWidget`。
pub struct ArrayEditorFactory;

impl PropertyEditorFactory for ArrayEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Array(_))
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(ArrayEditorWidget::new())
    }
}

/// 映射编辑器组件
///
/// 存储映射值的编辑器组件，用于编辑 `PropertyType::Map` 类型属性。
/// 值格式为 JSON 对象字符串。
pub struct MapEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl MapEditorWidget {
    /// 创建新的映射编辑器组件
    pub fn new() -> Self {
        Self { value: "{}".to_string(), initial_value: "{}".to_string() }
    }
}

impl Default for MapEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for MapEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value == "{}" {
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

/// 映射编辑器工厂
///
/// 为 `PropertyType::Map` 类型属性创建 `MapEditorWidget`。
pub struct MapEditorFactory;

impl PropertyEditorFactory for MapEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Map { .. })
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(MapEditorWidget::new())
    }
}

/// 三维向量编辑器组件
///
/// 存储三维向量值的编辑器组件，用于编辑 `PropertyType::Vec3` 类型属性。
/// 值格式为 "x,y,z" 字符串。
pub struct Vec3EditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl Vec3EditorWidget {
    /// 创建新的三维向量编辑器组件
    pub fn new() -> Self {
        Self { value: "0,0,0".to_string(), initial_value: "0,0,0".to_string() }
    }
}

impl Default for Vec3EditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for Vec3EditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value == "0,0,0" {
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

/// 三维向量编辑器工厂
///
/// 为 `PropertyType::Vec3` 类型属性创建 `Vec3EditorWidget`。
pub struct Vec3EditorFactory;

impl PropertyEditorFactory for Vec3EditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Vec3)
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(Vec3EditorWidget::new())
    }
}

/// 四维向量编辑器组件
///
/// 存储四维向量值的编辑器组件，用于编辑 `PropertyType::Vec4` 类型属性。
/// 值格式为 "x,y,z,w" 字符串。
pub struct Vec4EditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl Vec4EditorWidget {
    /// 创建新的四维向量编辑器组件
    pub fn new() -> Self {
        Self { value: "0,0,0,0".to_string(), initial_value: "0,0,0,0".to_string() }
    }
}

impl Default for Vec4EditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for Vec4EditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value == "0,0,0,0" {
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

/// 四维向量编辑器工厂
///
/// 为 `PropertyType::Vec4` 类型属性创建 `Vec4EditorWidget`。
pub struct Vec4EditorFactory;

impl PropertyEditorFactory for Vec4EditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Vec4)
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(Vec4EditorWidget::new())
    }
}

/// 矩形编辑器组件
///
/// 存储矩形值的编辑器组件，用于编辑 `PropertyType::Rect` 类型属性。
/// 值格式为 "x,y,w,h" 字符串。
pub struct RectEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl RectEditorWidget {
    /// 创建新的矩形编辑器组件
    pub fn new() -> Self {
        Self { value: "0,0,0,0".to_string(), initial_value: "0,0,0,0".to_string() }
    }
}

impl Default for RectEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for RectEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value == "0,0,0,0" {
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

/// 矩形编辑器工厂
///
/// 为 `PropertyType::Rect` 类型属性创建 `RectEditorWidget`。
pub struct RectEditorFactory;

impl PropertyEditorFactory for RectEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Rect)
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(RectEditorWidget::new())
    }
}

/// 实体引用编辑器组件
///
/// 存储实体引用值的编辑器组件，用于编辑 `PropertyType::EntityRef` 类型属性。
/// 值格式为 "entity:<id>" 或 "none" 字符串。
pub struct EntityRefEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl EntityRefEditorWidget {
    /// 创建新的实体引用编辑器组件
    pub fn new() -> Self {
        Self { value: "none".to_string(), initial_value: "none".to_string() }
    }
}

impl Default for EntityRefEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for EntityRefEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value == "none" {
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

/// 实体引用编辑器工厂
///
/// 为 `PropertyType::EntityRef` 类型属性创建 `EntityRefEditorWidget`。
pub struct EntityRefEditorFactory;

impl PropertyEditorFactory for EntityRefEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::EntityRef)
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(EntityRefEditorWidget::new())
    }
}

/// 结构体编辑器组件
///
/// 存储结构体值的编辑器组件，用于编辑 `PropertyType::Struct` 类型属性。
/// 值格式为 JSON 对象字符串。
pub struct StructEditorWidget {
    /// 当前值
    value: String,
    /// 初始值
    initial_value: String,
}

impl StructEditorWidget {
    /// 创建新的结构体编辑器组件
    pub fn new() -> Self {
        Self { value: "{}".to_string(), initial_value: "{}".to_string() }
    }
}

impl Default for StructEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorWidget for StructEditorWidget {
    fn set_value(&mut self, value: &str) {
        if self.initial_value == "{}" {
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

/// 结构体编辑器工厂
///
/// 为 `PropertyType::Struct` 类型属性创建 `StructEditorWidget`。
pub struct StructEditorFactory;

impl PropertyEditorFactory for StructEditorFactory {
    fn can_edit(&self, property_type: &PropertyType) -> bool {
        matches!(property_type, PropertyType::Struct { .. })
    }

    fn create_editor(&self) -> Box<dyn PropertyEditorWidget> {
        Box::new(StructEditorWidget::new())
    }
}
