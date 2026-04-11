//! GG Widget 编译器 prelude 模块
//! 导出最常用的 Widget 编译类型

pub use crate::{
    artifact::{DependencyType, WidgetArtifact, WidgetDependency, WidgetMeta},
    compiler::WidgetTransformer,
    error::{WidgetError, WidgetResult},
    parser::{WidgetFile, WidgetParser},
    registry::{ComponentRegistry, ComponentSchema, EventSchema, PropertySchema, PropertyType},
    style::StyleIr,
    template::TemplateIr,
};
