//! 属性控件生成模块
//!
//! 根据属性类型生成对应的 UI 控件描述，
//! 用于驱动检查器面板的属性编辑控件创建。

use crate::descriptor::{PropertyConstraints, PropertyType};

/// 根据属性类型和约束创建属性控件描述
///
/// 返回控件类型的字符串标识，用于驱动 UI 控件的创建。
/// 约束参数保留用于未来扩展，当前未使用。
pub fn create_property_control(property_type: &PropertyType, _constraints: Option<&PropertyConstraints>) -> String {
    match property_type {
        PropertyType::String => "text_input".to_string(),
        PropertyType::Int => "numeric_slider".to_string(),
        PropertyType::Float => "numeric_slider".to_string(),
        PropertyType::Bool => "toggle_switch".to_string(),
        PropertyType::Enum(_) => "dropdown_select".to_string(),
        PropertyType::Color => "color_picker".to_string(),
        PropertyType::AssetPath(_) => "asset_selector".to_string(),
        PropertyType::Vec2 => "vec2_input".to_string(),
        PropertyType::Custom(_) => "custom_editor".to_string(),
    }
}
