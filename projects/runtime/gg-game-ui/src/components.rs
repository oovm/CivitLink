//! 游戏UI组件模块
//! 
//! 实现游戏UI的核心组件，如Canvas、RectTransform、Graphic等

use gg_ecs::Component;
use gg_render::Color;

/// Canvas 渲染模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasRenderMode {
    /// 屏幕空间覆盖
    ScreenSpaceOverlay,
    /// 屏幕空间相机
    ScreenSpaceCamera,
    /// 世界空间
    WorldSpace,
}

/// Canvas 组件，管理UI元素的渲染
#[derive(Debug, Clone, Component)]
pub struct Canvas {
    /// 渲染模式
    pub render_mode: CanvasRenderMode,
    /// 是否像素完美
    pub pixel_perfect: bool,
    /// 排序顺序
    pub sorting_order: i32,
    /// 目标显示
    pub target_display: i32,
    /// 世界相机（仅在 ScreenSpaceCamera 和 WorldSpace 模式下使用）
    pub world_camera: Option<gg_ecs::Entity>,
    /// 平面距离（仅在 ScreenSpaceCamera 模式下使用）
    pub plane_distance: f32,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            render_mode: CanvasRenderMode::ScreenSpaceOverlay,
            pixel_perfect: false,
            sorting_order: 0,
            target_display: 0,
            world_camera: None,
            plane_distance: 1.0,
        }
    }
}

/// RectTransform 组件，管理UI元素的位置、旋转、缩放
#[derive(Debug, Clone, Component)]
pub struct RectTransform {
    /// 锚点最小值
    pub anchor_min: [f32; 2],
    /// 锚点最大值
    pub anchor_max: [f32; 2],
    /// 锚点位置
    pub anchored_position: [f32; 2],
    /// 尺寸增量
    pub size_delta: [f32; 2],
    /// 轴心点
    pub pivot: [f32; 2],
    /// 旋转
    pub rotation: [f32; 4], // quaternion
    /// 本地缩放
    pub local_scale: [f32; 3],
    /// 父元素
    pub parent: Option<gg_ecs::Entity>,
    /// 子元素
    pub children: Vec<gg_ecs::Entity>,
}

impl Default for RectTransform {
    fn default() -> Self {
        Self {
            anchor_min: [0.0, 0.0],
            anchor_max: [1.0, 1.0],
            anchored_position: [0.0, 0.0],
            size_delta: [0.0, 0.0],
            pivot: [0.5, 0.5],
            rotation: [0.0, 0.0, 0.0, 1.0],
            local_scale: [1.0, 1.0, 1.0],
            parent: None,
            children: Vec::new(),
        }
    }
}

/// CanvasRenderer 组件，负责UI元素的渲染
#[derive(Debug, Clone, Component)]
pub struct CanvasRenderer {
    /// 是否裁剪透明网格
    pub cull_transparent_mesh: bool,
    /// 是否有网格
    pub has_mesh: bool,
    /// 材质
    pub material: Option<gg_ecs::Entity>,
}

impl Default for CanvasRenderer {
    fn default() -> Self {
        Self {
            cull_transparent_mesh: true,
            has_mesh: false,
            material: None,
        }
    }
}

/// Graphic 组件基类，所有可渲染的UI元素都继承自此类
#[derive(Debug, Clone, Component)]
pub struct Graphic {
    /// 颜色
    pub color: Color,
    /// 是否接收射线
    pub raycast_target: bool,
    /// 材质
    pub material: Option<gg_ecs::Entity>,
    /// 纹理
    pub texture: Option<gg_ecs::Entity>,
    /// 是否可见
    pub visible: bool,
}

impl Default for Graphic {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            raycast_target: true,
            material: None,
            texture: None,
            visible: true,
        }
    }
}

/// Image 组件，用于显示图片
#[derive(Debug, Clone, Component)]
pub struct Image {
    /// 图片源
    pub sprite: Option<gg_ecs::Entity>,
    /// 填充方式
    pub fill_method: ImageFillMethod,
    /// 填充原点
    pub fill_origin: u32,
    /// 填充量
    pub fill_amount: f32,
    /// 是否保持纵横比
    pub preserve_aspect: bool,
}

/// 图片填充方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFillMethod {
    /// 不填充
    None,
    /// 水平填充
    Horizontal,
    /// 垂直填充
    Vertical,
    /// 径向填充
    Radial360,
    /// 径向填充（180度）
    Radial180,
    /// 径向填充（90度）
    Radial90,
    /// 径向填充（30度）
    Radial30,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            sprite: None,
            fill_method: ImageFillMethod::None,
            fill_origin: 0,
            fill_amount: 1.0,
            preserve_aspect: false,
        }
    }
}

/// Text 组件，用于显示文本
#[derive(Debug, Clone, Component)]
pub struct Text {
    /// 文本内容
    pub text: String,
    /// 字体
    pub font: Option<gg_ecs::Entity>,
    /// 字体大小
    pub font_size: f32,
    /// 字体样式
    pub font_style: FontStyle,
    /// 对齐方式
    pub alignment: TextAlignment,
    /// 水平溢出
    pub horizontal_overflow: TextOverflow,
    /// 垂直溢出
    pub vertical_overflow: TextOverflow,
    /// 行间距
    pub line_spacing: f32,
    /// 是否支持富文本
    pub rich_text: bool,
}

/// 字体样式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    /// 正常
    Normal,
    /// 粗体
    Bold,
    /// 斜体
    Italic,
    /// 粗斜体
    BoldAndItalic,
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    /// 左上角
    UpperLeft,
    /// 中上
    UpperCenter,
    /// 右上角
    UpperRight,
    /// 左中
    MiddleLeft,
    /// 中中
    MiddleCenter,
    /// 右中
    MiddleRight,
    /// 左下角
    LowerLeft,
    /// 中下
    LowerCenter,
    /// 右下角
    LowerRight,
}

/// 文本溢出方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOverflow {
    /// 截断
    Truncate,
    /// 溢出
    Overflow,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            font: None,
            font_size: 14.0,
            font_style: FontStyle::Normal,
            alignment: TextAlignment::UpperLeft,
            horizontal_overflow: TextOverflow::Truncate,
            vertical_overflow: TextOverflow::Truncate,
            line_spacing: 1.0,
            rich_text: true,
        }
    }
}

/// Button 组件，用于交互
#[derive(Debug, Clone, Component)]
pub struct Button {
    /// 过渡方式
    pub transition: ButtonTransition,
    /// 目标图形
    pub target_graphic: Option<gg_ecs::Entity>,
    /// 高亮颜色
    pub highlight_color: Color,
    /// 按下颜色
    pub pressed_color: Color,
    /// 禁用颜色
    pub disabled_color: Color,
    /// 动画持续时间
    pub color_multiplier: f32,
    /// 动画持续时间
    pub fade_duration: f32,
    /// 是否可交互
    pub interactable: bool,
    /// 点击回调
    pub onclick: Option<Box<dyn Fn() + Send + Sync>>,
}

/// 按钮过渡方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTransition {
    /// 颜色过渡
    ColorTint,
    /// 精灵过渡
    SpriteSwap,
    /// 动画过渡
    Animation,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            transition: ButtonTransition::ColorTint,
            target_graphic: None,
            highlight_color: Color::new(0.9f32, 0.9f32, 0.9f32, 1.0f32),
            pressed_color: Color::new(0.7f32, 0.7f32, 0.7f32, 1.0f32),
            disabled_color: Color::new(0.5f32, 0.5f32, 0.5f32, 0.5f32),
            color_multiplier: 1.0,
            fade_duration: 0.1,
            interactable: true,
            onclick: None,
        }
    }
}

/// InputField 组件，用于文本输入
#[derive(Debug, Clone, Component)]
pub struct InputField {
    /// 文本
    pub text: String,
    /// 占位文本
    pub placeholder: String,
    /// 字符限制
    pub character_limit: u32,
    /// 是否可编辑
    pub interactable: bool,
    /// 内容类型
    pub content_type: InputFieldContentType,
    /// 输入类型
    pub input_type: InputFieldInputType,
    /// 光标颜色
    pub caret_color: Color,
    /// 选择颜色
    pub selection_color: Color,
    /// 文本组件
    pub text_component: Option<gg_ecs::Entity>,
    /// 占位文本组件
    pub placeholder_component: Option<gg_ecs::Entity>,
}

/// 输入字段内容类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFieldContentType {
    /// 标准
    Standard,
    /// 自动更正
    Autocorrected,
    /// 整数
    IntegerNumber,
    /// 小数
    DecimalNumber,
    /// 字母
    Alphanumeric,
    /// 名称
    Name,
    /// 电子邮件
    EmailAddress,
    /// 密码
    Password,
    /// PIN码
    Pin,
    /// 自定义
    Custom,
}

/// 输入字段输入类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFieldInputType {
    /// 标准
    Standard,
    /// 自动更正
    Autocorrected,
    /// 整数
    IntegerNumber,
    /// 小数
    DecimalNumber,
    /// 字母
    Alphanumeric,
    /// 名称
    Name,
    /// 电子邮件
    EmailAddress,
    /// 密码
    Password,
    /// PIN码
    Pin,
    /// 自定义
    Custom,
}

impl Default for InputField {
    fn default() -> Self {
        Self {
            text: String::new(),
            placeholder: String::new(),
            character_limit: 0,
            interactable: true,
            content_type: InputFieldContentType::Standard,
            input_type: InputFieldInputType::Standard,
            caret_color: Color::WHITE,
            selection_color: Color::new(0.2f32, 0.4f32, 0.6f32, 0.8f32),
            text_component: None,
            placeholder_component: None,
        }
    }
}

/// ScrollRect 组件，用于滚动内容
#[derive(Debug, Clone, Component)]
pub struct ScrollRect {
    /// 内容
    pub content: Option<gg_ecs::Entity>,
    /// 水平滚动
    pub horizontal: bool,
    /// 垂直滚动
    pub vertical: bool,
    /// 移动类型
    pub movement_type: ScrollRectMovementType,
    /// 弹性
    pub elasticity: f32,
    /// 惯性
    pub inertia: bool,
    /// 减速度
    pub deceleration_rate: f32,
    /// 滚动灵敏度
    pub scroll_sensitivity: f32,
}

/// 滚动矩形移动类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollRectMovementType {
    /// 不受限制
    Unrestricted,
    /// 弹性
    Elastic,
    /// 钳制
    Clamped,
}

impl Default for ScrollRect {
    fn default() -> Self {
        Self {
            content: None,
            horizontal: false,
            vertical: true,
            movement_type: ScrollRectMovementType::Elastic,
            elasticity: 0.1f32,
            inertia: true,
            deceleration_rate: 0.135f32,
            scroll_sensitivity: 1.0f32,
        }
    }
}
