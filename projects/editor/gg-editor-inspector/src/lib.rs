//! GG 编辑器属性检查器模块
//! 提供基于属性描述符的动态属性编辑框架

pub mod binding;
pub mod descriptor;
pub mod editor;
pub mod panel;

pub use binding::{PropertyBinding, SetPropertyCommand};
pub use descriptor::{
    ComponentDescriptor, DescriptorRegistry, PropertyConstraints, PropertyDescriptor, PropertyType,
};
pub use editor::{
    AssetPathEditorFactory, AssetPathEditorWidget, BoolEditorFactory, BoolEditorWidget,
    ColorEditorFactory, ColorEditorWidget, EnumEditorFactory, EnumEditorWidget,
    NumericEditorFactory, NumericEditorWidget, PropertyEditorFactory, PropertyEditorRegistry,
    PropertyEditorWidget, StringEditorFactory, StringEditorWidget,
};
pub use panel::InspectorPanel;
