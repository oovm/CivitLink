#![warn(missing_docs)]

//! GG 编辑器属性检查器模块
//! 提供基于属性描述符的动态属性编辑框架

pub mod binding;
pub mod controls;
pub mod descriptor;
pub mod editor;
pub mod panel;

pub use binding::{EcsPropertyBinding, PropertyBinding, PropertyStore, ReflectionPropertyBinding, SetPropertyCommand};
pub use controls::create_property_control;
pub use descriptor::{ComponentDescriptor, DescriptorRegistry, PropertyConstraints, PropertyDescriptor, PropertyType};
pub use editor::{
    ArrayEditorFactory, ArrayEditorWidget, AssetPathEditorFactory, AssetPathEditorWidget, BoolEditorFactory, BoolEditorWidget,
    ColorEditorFactory, ColorEditorWidget, EntityRefEditorFactory, EntityRefEditorWidget, EnumEditorFactory, EnumEditorWidget,
    MapEditorFactory, MapEditorWidget, NumericEditorFactory, NumericEditorWidget, PropertyEditorFactory,
    PropertyEditorRegistry, PropertyEditorWidget, RectEditorFactory, RectEditorWidget, StringEditorFactory, StringEditorWidget,
    StructEditorFactory, StructEditorWidget, Vec2EditorFactory, Vec2EditorWidget, Vec3EditorFactory, Vec3EditorWidget,
    Vec4EditorFactory, Vec4EditorWidget,
};
pub use panel::{ActivePropertyEntry, InspectorPanel};
