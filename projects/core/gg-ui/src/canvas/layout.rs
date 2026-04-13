//! 游戏UI布局系统模块
//!
//! 实现游戏UI的布局功能，包括自动布局、锚点系统等

use super::components::RectTransform;
use gg_ecs::{Entity, World};

/// 布局组类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutGroupType {
    /// 水平布局
    Horizontal,
    /// 垂直布局
    Vertical,
    /// 网格布局
    Grid,
}

/// 布局组组件
#[derive(Debug, Clone)]
pub struct LayoutGroup {
    /// 布局类型
    pub layout_type: LayoutGroupType,
    /// 子元素间距
    pub spacing: f32,
    /// 内边距
    pub padding: [f32; 4],
    /// 子元素对齐方式
    pub child_alignment: Alignment,
    /// 是否控制子元素大小
    pub control_child_size: bool,
    /// 是否使用子元素的布局属性
    pub use_child_layout_properties: bool,
}

/// 对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// 左上
    UpperLeft,
    /// 中上
    UpperCenter,
    /// 右上
    UpperRight,
    /// 左中
    MiddleLeft,
    /// 中中
    MiddleCenter,
    /// 右中
    MiddleRight,
    /// 左下
    LowerLeft,
    /// 中下
    LowerCenter,
    /// 右下
    LowerRight,
}

impl Default for LayoutGroup {
    fn default() -> Self {
        Self {
            layout_type: LayoutGroupType::Horizontal,
            spacing: 0.0,
            padding: [0.0, 0.0, 0.0, 0.0],
            child_alignment: Alignment::UpperLeft,
            control_child_size: false,
            use_child_layout_properties: false,
        }
    }
}

/// 水平布局组
#[derive(Debug, Clone)]
pub struct HorizontalLayoutGroup {
    /// 子元素间距
    pub spacing: f32,
    /// 内边距
    pub padding: [f32; 4],
    /// 子元素对齐方式
    pub child_alignment: Alignment,
    /// 是否控制子元素大小
    pub control_child_size: bool,
    /// 是否使用子元素的布局属性
    pub use_child_layout_properties: bool,
    /// 子元素宽度
    pub child_width: f32,
    /// 子元素高度
    pub child_height: f32,
}

impl Default for HorizontalLayoutGroup {
    fn default() -> Self {
        Self {
            spacing: 0.0,
            padding: [0.0, 0.0, 0.0, 0.0],
            child_alignment: Alignment::UpperLeft,
            control_child_size: false,
            use_child_layout_properties: false,
            child_width: 100.0,
            child_height: 50.0,
        }
    }
}

/// 垂直布局组
#[derive(Debug, Clone)]
pub struct VerticalLayoutGroup {
    /// 子元素间距
    pub spacing: f32,
    /// 内边距
    pub padding: [f32; 4],
    /// 子元素对齐方式
    pub child_alignment: Alignment,
    /// 是否控制子元素大小
    pub control_child_size: bool,
    /// 是否使用子元素的布局属性
    pub use_child_layout_properties: bool,
    /// 子元素宽度
    pub child_width: f32,
    /// 子元素高度
    pub child_height: f32,
}

impl Default for VerticalLayoutGroup {
    fn default() -> Self {
        Self {
            spacing: 0.0,
            padding: [0.0, 0.0, 0.0, 0.0],
            child_alignment: Alignment::UpperLeft,
            control_child_size: false,
            use_child_layout_properties: false,
            child_width: 100.0,
            child_height: 50.0,
        }
    }
}

/// 网格布局组
#[derive(Debug, Clone)]
pub struct GridLayoutGroup {
    /// 单元格大小
    pub cell_size: [f32; 2],
    /// 单元格间距
    pub spacing: [f32; 2],
    /// 内边距
    pub padding: [f32; 4],
    /// 子元素对齐方式
    pub child_alignment: Alignment,
    /// 约束计数
    pub constraint_count: u32,
    /// 约束类型
    pub constraint_type: GridConstraintType,
    /// 启动轴
    pub start_axis: GridStartAxis,
    /// 是否控制子元素大小
    pub control_child_size: bool,
    /// 是否使用子元素的布局属性
    pub use_child_layout_properties: bool,
}

/// 网格约束类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridConstraintType {
    /// 固定列数
    FixedColumnCount,
    /// 固定行数
    FixedRowCount,
}

/// 网格启动轴
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridStartAxis {
    /// 水平
    Horizontal,
    /// 垂直
    Vertical,
}

impl Default for GridLayoutGroup {
    fn default() -> Self {
        Self {
            cell_size: [100.0, 100.0],
            spacing: [10.0, 10.0],
            padding: [10.0, 10.0, 10.0, 10.0],
            child_alignment: Alignment::UpperLeft,
            constraint_count: 2,
            constraint_type: GridConstraintType::FixedColumnCount,
            start_axis: GridStartAxis::Horizontal,
            control_child_size: true,
            use_child_layout_properties: false,
        }
    }
}

/// 内容大小适配组件
#[derive(Debug, Clone)]
pub struct ContentSizeFitter {
    /// 水平适配模式
    pub horizontal_fit: ContentSizeFitMode,
    /// 垂直适配模式
    pub vertical_fit: ContentSizeFitMode,
}

/// 内容大小适配模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentSizeFitMode {
    /// 不受限制
    Unconstrained,
    /// 最小大小
    MinSize,
    /// 首选大小
    PreferredSize,
}

impl Default for ContentSizeFitter {
    fn default() -> Self {
        Self { horizontal_fit: ContentSizeFitMode::Unconstrained, vertical_fit: ContentSizeFitMode::Unconstrained }
    }
}

/// 宽高比适配组件
#[derive(Debug, Clone)]
pub struct AspectRatioFitter {
    /// 宽高比
    pub aspect_ratio: f32,
    /// 适配模式
    pub aspect_mode: AspectMode,
}

/// 宽高比适配模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectMode {
    /// 无
    None,
    /// 宽度控制高度
    WidthControlsHeight,
    /// 高度控制宽度
    HeightControlsWidth,
    /// 符合父元素
    FitInParent,
    /// 填充父元素
    EnvelopeParent,
}

impl Default for AspectRatioFitter {
    fn default() -> Self {
        Self { aspect_ratio: 1.0, aspect_mode: AspectMode::None }
    }
}

/// 布局引擎
pub struct LayoutEngine {
    /// 是否需要重建布局
    pub needs_rebuild: bool,
}

impl LayoutEngine {
    /// 创建新的布局引擎
    pub fn new() -> Self {
        Self { needs_rebuild: true }
    }

    /// 计算布局
    pub fn compute(&mut self, world: &mut World) {
        self.needs_rebuild = false;

        self.process_layout_groups(world);

        self.process_content_size_fitters(world);

        self.process_aspect_ratio_fitters(world);
    }

    /// 处理布局组
    fn process_layout_groups(&self, world: &mut World) {
        let horizontal_layouts: Vec<(Entity, HorizontalLayoutGroup)> = world
            .entities()
            .into_iter()
            .filter_map(|entity| world.get_component::<HorizontalLayoutGroup>(entity).cloned().map(|l| (entity, l)))
            .collect();

        for (entity, layout) in horizontal_layouts {
            self.process_horizontal_layout(entity, &layout, world);
        }

        let vertical_layouts: Vec<(Entity, VerticalLayoutGroup)> = world
            .entities()
            .into_iter()
            .filter_map(|entity| world.get_component::<VerticalLayoutGroup>(entity).cloned().map(|l| (entity, l)))
            .collect();

        for (entity, layout) in vertical_layouts {
            self.process_vertical_layout(entity, &layout, world);
        }

        let grid_layouts: Vec<(Entity, GridLayoutGroup)> = world
            .entities()
            .into_iter()
            .filter_map(|entity| world.get_component::<GridLayoutGroup>(entity).cloned().map(|l| (entity, l)))
            .collect();

        for (entity, layout) in grid_layouts {
            self.process_grid_layout(entity, &layout, world);
        }
    }

    /// 处理水平布局
    fn process_horizontal_layout(&self, entity: Entity, layout: &HorizontalLayoutGroup, world: &mut World) {
        let children = world.get_component::<RectTransform>(entity).map(|rt| rt.children.clone());

        let Some(children) = children
        else {
            return;
        };

        let mut current_x = layout.padding[3];
        let y = layout.padding[0];

        for child in children {
            if let Some(child_rect) = world.get_component_mut::<RectTransform>(child) {
                child_rect.anchored_position[0] = current_x;
                child_rect.anchored_position[1] = y;

                if layout.control_child_size {
                    child_rect.size_delta[0] = layout.child_width;
                    child_rect.size_delta[1] = layout.child_height;
                }

                current_x += layout.child_width + layout.spacing;
            }
        }
    }

    /// 处理垂直布局
    fn process_vertical_layout(&self, entity: Entity, layout: &VerticalLayoutGroup, world: &mut World) {
        let children = world.get_component::<RectTransform>(entity).map(|rt| rt.children.clone());

        let Some(children) = children
        else {
            return;
        };

        let x = layout.padding[3];
        let mut current_y = layout.padding[0];

        for child in children {
            if let Some(child_rect) = world.get_component_mut::<RectTransform>(child) {
                child_rect.anchored_position[0] = x;
                child_rect.anchored_position[1] = current_y;

                if layout.control_child_size {
                    child_rect.size_delta[0] = layout.child_width;
                    child_rect.size_delta[1] = layout.child_height;
                }

                current_y += layout.child_height + layout.spacing;
            }
        }
    }

    /// 处理网格布局
    fn process_grid_layout(&self, entity: Entity, layout: &GridLayoutGroup, world: &mut World) {
        let children = world.get_component::<RectTransform>(entity).map(|rt| rt.children.clone());

        let Some(children) = children
        else {
            return;
        };

        let start_x = layout.padding[3];
        let start_y = layout.padding[0];

        let mut x = start_x;
        let mut y = start_y;
        let mut index = 0;

        for child in children {
            if let Some(child_rect) = world.get_component_mut::<RectTransform>(child) {
                child_rect.anchored_position[0] = x;
                child_rect.anchored_position[1] = y;

                if layout.control_child_size {
                    child_rect.size_delta[0] = layout.cell_size[0];
                    child_rect.size_delta[1] = layout.cell_size[1];
                }

                index += 1;
                if layout.start_axis == GridStartAxis::Horizontal {
                    if layout.constraint_type == GridConstraintType::FixedColumnCount && index % layout.constraint_count == 0 {
                        x = start_x;
                        y += layout.cell_size[1] + layout.spacing[1];
                    }
                    else {
                        x += layout.cell_size[0] + layout.spacing[0];
                    }
                }
                else {
                    if layout.constraint_type == GridConstraintType::FixedRowCount && index % layout.constraint_count == 0 {
                        y = start_y;
                        x += layout.cell_size[0] + layout.spacing[0];
                    }
                    else {
                        y += layout.cell_size[1] + layout.spacing[1];
                    }
                }
            }
        }
    }

    /// 处理内容大小适配
    fn process_content_size_fitters(&self, world: &mut World) {
        let fitter_entities: Vec<(Entity, ContentSizeFitter)> = world
            .entities()
            .into_iter()
            .filter_map(|entity| world.get_component::<ContentSizeFitter>(entity).cloned().map(|f| (entity, f)))
            .collect();

        for (entity, content_size_fitter) in fitter_entities {
            if let Some(_rect_transform) = world.get_component_mut::<RectTransform>(entity) {
                if content_size_fitter.horizontal_fit != ContentSizeFitMode::Unconstrained {}
                if content_size_fitter.vertical_fit != ContentSizeFitMode::Unconstrained {}
            }
        }
    }

    /// 处理宽高比适配
    fn process_aspect_ratio_fitters(&self, world: &mut World) {
        let fitter_entities: Vec<(Entity, AspectRatioFitter)> = world
            .entities()
            .into_iter()
            .filter_map(|entity| world.get_component::<AspectRatioFitter>(entity).cloned().map(|f| (entity, f)))
            .collect();

        for (entity, aspect_ratio_fitter) in fitter_entities {
            if aspect_ratio_fitter.aspect_mode != AspectMode::None {
                if let Some(rect_transform) = world.get_component_mut::<RectTransform>(entity) {
                    match aspect_ratio_fitter.aspect_mode {
                        AspectMode::WidthControlsHeight => {
                            rect_transform.size_delta[1] = rect_transform.size_delta[0] / aspect_ratio_fitter.aspect_ratio;
                        }
                        AspectMode::HeightControlsWidth => {
                            rect_transform.size_delta[0] = rect_transform.size_delta[1] * aspect_ratio_fitter.aspect_ratio;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// 标记需要重建布局
    pub fn mark_dirty(&mut self) {
        self.needs_rebuild = true;
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 布局系统
pub struct LayoutSystem {
    /// 布局引擎
    layout_engine: LayoutEngine,
}

impl LayoutSystem {
    /// 创建新的布局系统
    pub fn new() -> Self {
        Self { layout_engine: LayoutEngine::new() }
    }
}

impl gg_ecs::System for LayoutSystem {
    fn name(&self) -> &str {
        "LayoutSystem"
    }

    fn execute(&mut self, world: &mut World) -> gg_error::GResult<()> {
        self.layout_engine.compute(world);
        Ok(())
    }
}
