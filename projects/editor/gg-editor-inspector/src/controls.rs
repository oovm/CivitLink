//! 属性控件生成模块
//!
//! 根据属性类型和约束在 UiTree 中创建对应的 UI 控件节点，
//! 用于驱动检查器面板的属性编辑控件渲染。

use gg_render::Color;
use gg_ui::{FlexDirection, FontStyle, LayoutStyle, SizeValue, Style, UiNodeData, UiNodeId, UiTree};

use crate::descriptor::{PropertyConstraints, PropertyDescriptor, PropertyType};

/// 创建属性行容器样式
///
/// 返回一个 Row 方向、带间距和内边距的容器样式，
/// 用于包裹属性标签和编辑控件。
fn row_container_style() -> Style {
    Style::new().with_layout(
        LayoutStyle::new()
            .with_direction(FlexDirection::Row)
            .with_gap(8.0)
            .with_padding(4.0)
            .with_align_items(gg_ui::FlexAlign::Center),
    )
}

/// 创建属性标签样式
///
/// 返回带字体样式的标签样式，使用白色字体、固定宽度。
fn label_style() -> Style {
    Style::new()
        .with_layout(LayoutStyle::new().with_width(SizeValue::Px(100.0)).with_min_width(SizeValue::Px(80.0)))
        .with_font(FontStyle::new().with_size(13.0).with_color(Color::WHITE))
}

/// 创建输入控件容器样式
///
/// 返回用于包裹输入控件的弹性填充样式。
fn input_style() -> Style {
    Style::new().with_layout(LayoutStyle::new().with_width(SizeValue::Px(160.0)).with_min_width(SizeValue::Px(60.0)))
}

/// 创建值显示样式
///
/// 返回用于显示当前数值的文本样式。
fn value_display_style() -> Style {
    Style::new()
        .with_layout(LayoutStyle::new().with_width(SizeValue::Px(50.0)).with_min_width(SizeValue::Px(40.0)))
        .with_font(FontStyle::new().with_size(12.0).with_color(Color::WHITE))
}

/// 创建颜色预览样式
///
/// 返回带背景色和圆角的颜色预览方块样式。
fn color_preview_style() -> Style {
    Style::new()
        .with_layout(LayoutStyle::new().with_width(SizeValue::Px(24.0)).with_height(SizeValue::Px(24.0)))
        .with_corner_radius(4.0)
        .with_background_color(Color::WHITE)
}

/// 根据属性类型、名称和约束在 UiTree 中创建属性控件节点
///
/// 为每种属性类型创建包含标签和对应编辑控件的容器节点，
/// 并将所有子节点添加到容器中，返回容器节点 ID。
///
/// # 参数
///
/// - `property_type` - 属性类型，决定创建哪种控件
/// - `property_name` - 属性显示名称，用于标签文本
/// - `constraints` - 属性约束，用于数值范围等
/// - `ui_tree` - UI 树，控件节点将创建在其中
pub fn create_property_control(
    property_type: &PropertyType,
    property_name: &str,
    constraints: Option<&PropertyConstraints>,
    ui_tree: &mut UiTree,
) -> UiNodeId {
    match property_type {
        PropertyType::String => create_string_control(property_name, ui_tree),
        PropertyType::Int => create_numeric_control(property_name, constraints, ui_tree),
        PropertyType::Float => create_numeric_control(property_name, constraints, ui_tree),
        PropertyType::Bool => create_bool_control(property_name, ui_tree),
        PropertyType::Enum(variants) => create_enum_control(property_name, variants, ui_tree),
        PropertyType::Color => create_color_control(property_name, ui_tree),
        PropertyType::AssetPath(ext) => create_asset_path_control(property_name, ext, ui_tree),
        PropertyType::Vec2 => create_vec2_control(property_name, ui_tree),
        PropertyType::Vec3 => create_vec3_control(property_name, ui_tree),
        PropertyType::Vec4 => create_vec4_control(property_name, ui_tree),
        PropertyType::Rect => create_rect_control(property_name, ui_tree),
        PropertyType::EntityRef => create_entity_ref_control(property_name, ui_tree),
        PropertyType::Struct { name, fields } => create_struct_control(property_name, name, fields, ui_tree),
        PropertyType::Array(_) => create_array_control(property_name, ui_tree),
        PropertyType::Map { .. } => create_map_control(property_name, ui_tree),
        PropertyType::Custom(type_name) => create_custom_control(property_name, type_name, ui_tree),
    }
}

/// 创建字符串属性控件
///
/// 创建包含标签和文本输入框的 Row 容器。
/// 文本输入框使用 `Custom { kind: "text_input" }` 节点类型。
fn create_string_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("string_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("string_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let input_id = ui_tree.create_node("string_input", input_style(), UiNodeData::Custom { kind: "text_input".to_string() });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, input_id);

    container_id
}

/// 创建数值属性控件
///
/// 创建包含标签、数值滑块和值显示的 Row 容器。
/// 数值滑块使用 `Custom { kind: "numeric_slider:{min}:{max}:{step}" }` 节点类型，
/// 约束参数编码在 kind 字符串中。
fn create_numeric_control(property_name: &str, constraints: Option<&PropertyConstraints>, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("numeric_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("numeric_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let kind = format_numeric_slider_kind(constraints);
    let slider_id = ui_tree.create_node("numeric_slider", input_style(), UiNodeData::Custom { kind });

    let value_id = ui_tree.create_node("numeric_value", value_display_style(), UiNodeData::Text { content: String::new() });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, slider_id);
    ui_tree.add_child(container_id, value_id);

    container_id
}

/// 格式化数值滑块的 kind 字符串
///
/// 将约束参数编码为 `numeric_slider:{min}:{max}:{step}` 格式，
/// 未指定的约束使用默认值（min=0, max=100, step=1）。
fn format_numeric_slider_kind(constraints: Option<&PropertyConstraints>) -> String {
    let c = constraints;
    let min = c.and_then(|c| c.min_value).unwrap_or(0.0);
    let max = c.and_then(|c| c.max_value).unwrap_or(100.0);
    let step = c.and_then(|c| c.step).unwrap_or(1.0);
    format!("numeric_slider:{min}:{max}:{step}")
}

/// 创建布尔属性控件
///
/// 创建包含标签和开关的 Row 容器。
/// 开关使用 `Custom { kind: "toggle_switch" }` 节点类型。
fn create_bool_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("bool_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("bool_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let toggle_id = ui_tree.create_node("bool_toggle", input_style(), UiNodeData::Custom { kind: "toggle_switch".to_string() });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, toggle_id);

    container_id
}

/// 创建枚举属性控件
///
/// 创建包含标签和下拉选择器的 Row 容器。
/// 下拉选择器使用 `Custom { kind: "dropdown_select:variant1,variant2,..." }` 节点类型，
/// 变体列表编码在 kind 字符串中。
fn create_enum_control(property_name: &str, variants: &[String], ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("enum_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("enum_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let kind = format!("dropdown_select:{}", variants.join(","));
    let dropdown_id = ui_tree.create_node("enum_dropdown", input_style(), UiNodeData::Custom { kind });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, dropdown_id);

    container_id
}

/// 创建颜色属性控件
///
/// 创建包含标签、颜色预览和颜色输入的 Row 容器。
/// 颜色预览使用 `Custom { kind: "color_preview" }` 节点类型，
/// 颜色输入使用 `Custom { kind: "color_input" }` 节点类型。
fn create_color_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("color_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("color_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let preview_id =
        ui_tree.create_node("color_preview", color_preview_style(), UiNodeData::Custom { kind: "color_preview".to_string() });

    let input_id = ui_tree.create_node("color_input", input_style(), UiNodeData::Custom { kind: "color_input".to_string() });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, preview_id);
    ui_tree.add_child(container_id, input_id);

    container_id
}

/// 创建资源路径属性控件
///
/// 创建包含标签和资源选择器的 Row 容器。
/// 资源选择器使用 `Custom { kind: "asset_selector:{ext}" }` 节点类型，
/// 扩展名过滤编码在 kind 字符串中。
fn create_asset_path_control(property_name: &str, ext: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("asset_path_control", row_container_style(), UiNodeData::Container);

    let label_id =
        ui_tree.create_node("asset_path_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let kind = format!("asset_selector:{ext}");
    let selector_id = ui_tree.create_node("asset_path_selector", input_style(), UiNodeData::Custom { kind });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, selector_id);

    container_id
}

/// 创建二维向量属性控件
///
/// 创建包含标签和 X/Y 输入框的 Row 容器。
fn create_vec2_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("vec2_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("vec2_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let x_input_id =
        ui_tree.create_node("vec2_x_input", input_style(), UiNodeData::Custom { kind: "vec2_x_input".to_string() });

    let y_input_id =
        ui_tree.create_node("vec2_y_input", input_style(), UiNodeData::Custom { kind: "vec2_y_input".to_string() });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, x_input_id);
    ui_tree.add_child(container_id, y_input_id);

    container_id
}

/// 创建自定义类型属性控件
///
/// 创建包含标签和自定义编辑器的 Row 容器。
/// 自定义编辑器使用 `Custom { kind: "custom_editor:{type_name}" }` 节点类型，
/// 类型名称编码在 kind 字符串中。
fn create_custom_control(property_name: &str, type_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("custom_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("custom_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let kind = format!("custom_editor:{type_name}");
    let editor_id = ui_tree.create_node("custom_editor", input_style(), UiNodeData::Custom { kind });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, editor_id);

    container_id
}

/// 创建数组属性控件
///
/// 创建包含标签、元素列表和添加按钮的 Column 容器。
fn create_array_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("array_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("array_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let add_btn_id =
        ui_tree.create_node("array_add_btn", input_style(), UiNodeData::Custom { kind: "array_add_button".to_string() });

    let list_id = ui_tree.create_node(
        "array_list",
        Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0)),
        UiNodeData::Custom { kind: "array_list".to_string() },
    );

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, add_btn_id);
    ui_tree.add_child(container_id, list_id);

    container_id
}

/// 创建映射属性控件
///
/// 创建包含标签、键值对列表和添加按钮的 Column 容器。
fn create_map_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("map_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("map_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let add_btn_id =
        ui_tree.create_node("map_add_btn", input_style(), UiNodeData::Custom { kind: "map_add_button".to_string() });

    let list_id = ui_tree.create_node(
        "map_list",
        Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0)),
        UiNodeData::Custom { kind: "map_list".to_string() },
    );

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, add_btn_id);
    ui_tree.add_child(container_id, list_id);

    container_id
}

/// 创建三维向量属性控件
///
/// 创建包含标签和 X、Y、Z 三个数值输入框的 Row 容器。
pub fn create_vec3_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("vec3_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("vec3_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let x_input_id =
        ui_tree.create_node("vec3_x_input", input_style(), UiNodeData::Custom { kind: "vec3_x_input".to_string() });

    let y_input_id =
        ui_tree.create_node("vec3_y_input", input_style(), UiNodeData::Custom { kind: "vec3_y_input".to_string() });

    let z_input_id =
        ui_tree.create_node("vec3_z_input", input_style(), UiNodeData::Custom { kind: "vec3_z_input".to_string() });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, x_input_id);
    ui_tree.add_child(container_id, y_input_id);
    ui_tree.add_child(container_id, z_input_id);

    container_id
}

/// 创建四维向量属性控件
///
/// 创建包含标签和 X、Y、Z、W 四个数值输入框的 Row 容器。
pub fn create_vec4_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("vec4_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("vec4_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let x_input_id =
        ui_tree.create_node("vec4_x_input", input_style(), UiNodeData::Custom { kind: "vec4_x_input".to_string() });

    let y_input_id =
        ui_tree.create_node("vec4_y_input", input_style(), UiNodeData::Custom { kind: "vec4_y_input".to_string() });

    let z_input_id =
        ui_tree.create_node("vec4_z_input", input_style(), UiNodeData::Custom { kind: "vec4_z_input".to_string() });

    let w_input_id =
        ui_tree.create_node("vec4_w_input", input_style(), UiNodeData::Custom { kind: "vec4_w_input".to_string() });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, x_input_id);
    ui_tree.add_child(container_id, y_input_id);
    ui_tree.add_child(container_id, z_input_id);
    ui_tree.add_child(container_id, w_input_id);

    container_id
}

/// 创建矩形属性控件
///
/// 创建包含标签和 X、Y、W、H 四个数值输入框的 Row 容器。
pub fn create_rect_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("rect_control", row_container_style(), UiNodeData::Container);

    let label_id = ui_tree.create_node("rect_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let x_input_id =
        ui_tree.create_node("rect_x_input", input_style(), UiNodeData::Custom { kind: "rect_x_input".to_string() });

    let y_input_id =
        ui_tree.create_node("rect_y_input", input_style(), UiNodeData::Custom { kind: "rect_y_input".to_string() });

    let w_input_id =
        ui_tree.create_node("rect_w_input", input_style(), UiNodeData::Custom { kind: "rect_w_input".to_string() });

    let h_input_id =
        ui_tree.create_node("rect_h_input", input_style(), UiNodeData::Custom { kind: "rect_h_input".to_string() });

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, x_input_id);
    ui_tree.add_child(container_id, y_input_id);
    ui_tree.add_child(container_id, w_input_id);
    ui_tree.add_child(container_id, h_input_id);

    container_id
}

/// 创建实体引用属性控件
///
/// 创建包含标签、实体 ID 文本输入和拖放区域的 Row 容器。
pub fn create_entity_ref_control(property_name: &str, ui_tree: &mut UiTree) -> UiNodeId {
    let container_id = ui_tree.create_node("entity_ref_control", row_container_style(), UiNodeData::Container);

    let label_id =
        ui_tree.create_node("entity_ref_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let input_id =
        ui_tree.create_node("entity_ref_input", input_style(), UiNodeData::Custom { kind: "entity_ref_input".to_string() });

    let drop_zone_id = ui_tree.create_node(
        "entity_ref_drop_zone",
        Style::new()
            .with_layout(LayoutStyle::new().with_width(SizeValue::Px(32.0)).with_height(SizeValue::Px(24.0)))
            .with_corner_radius(4.0)
            .with_background_color(Color::new(0.3, 0.3, 0.35, 1.0)),
        UiNodeData::Custom { kind: "entity_drop_zone".to_string() },
    );

    ui_tree.add_child(container_id, label_id);
    ui_tree.add_child(container_id, input_id);
    ui_tree.add_child(container_id, drop_zone_id);

    container_id
}

/// 创建结构体属性控件
///
/// 创建包含标签和可折叠嵌套属性区域的 Column 容器，
/// 嵌套区域中为每个字段递归创建对应的属性控件。
pub fn create_struct_control(
    property_name: &str,
    struct_name: &str,
    fields: &[PropertyDescriptor],
    ui_tree: &mut UiTree,
) -> UiNodeId {
    let container_id = ui_tree.create_node(
        "struct_control",
        Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(4.0).with_padding(4.0)),
        UiNodeData::Container,
    );

    let header_id = ui_tree.create_node(
        "struct_header",
        row_container_style(),
        UiNodeData::Custom { kind: format!("struct_header:{}", struct_name) },
    );

    let label_id = ui_tree.create_node("struct_label", label_style(), UiNodeData::Text { content: property_name.to_string() });

    let toggle_id =
        ui_tree.create_node("struct_toggle", input_style(), UiNodeData::Custom { kind: "struct_toggle".to_string() });

    ui_tree.add_child(header_id, label_id);
    ui_tree.add_child(header_id, toggle_id);
    ui_tree.add_child(container_id, header_id);

    let fields_id = ui_tree.create_node(
        "struct_fields",
        Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0).with_padding(8.0)),
        UiNodeData::Container,
    );

    for field in fields {
        let field_id = create_property_control(&field.property_type, &field.display_name, field.constraints.as_ref(), ui_tree);
        ui_tree.add_child(fields_id, field_id);
    }

    ui_tree.add_child(container_id, fields_id);

    container_id
}
