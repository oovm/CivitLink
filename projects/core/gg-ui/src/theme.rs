use std::collections::HashMap;

use gg_render::Color;

/// 阴影令牌
#[derive(Debug, Clone)]
pub struct ShadowToken {
    /// 阴影水平偏移
    pub offset_x: f32,
    /// 阴影垂直偏移
    pub offset_y: f32,
    /// 模糊半径
    pub blur: f32,
    /// 阴影颜色
    pub color: Color,
}

/// 主题标识
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThemeId {
    /// 亮色主题
    Light,
    /// 暗色主题
    Dark,
    /// 自定义主题
    Custom(String),
}

/// 主题令牌访问 trait
pub trait ThemeTokens {
    /// 获取颜色令牌
    fn color(&self, name: &str) -> Option<&Color>;
    /// 获取间距令牌
    fn spacing(&self, name: &str) -> Option<f32>;
    /// 获取字体大小令牌
    fn font_size(&self, name: &str) -> Option<f32>;
    /// 获取圆角令牌
    fn border_radius(&self, name: &str) -> Option<f32>;
    /// 获取阴影令牌
    fn shadow(&self, name: &str) -> Option<&ShadowToken>;
}

/// 主题定义
#[derive(Debug, Clone)]
pub struct Theme {
    /// 主题标识
    pub id: ThemeId,
    /// 主题名称
    pub name: String,
    /// 颜色令牌
    pub colors: HashMap<String, Color>,
    /// 间距令牌
    pub spacing: HashMap<String, f32>,
    /// 字体大小令牌
    pub font_sizes: HashMap<String, f32>,
    /// 圆角令牌
    pub border_radii: HashMap<String, f32>,
    /// 阴影令牌
    pub shadows: HashMap<String, ShadowToken>,
}

impl Theme {
    /// 创建新主题
    pub fn new(id: ThemeId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            colors: HashMap::new(),
            spacing: HashMap::new(),
            font_sizes: HashMap::new(),
            border_radii: HashMap::new(),
            shadows: HashMap::new(),
        }
    }

    /// 添加颜色令牌
    pub fn with_color(mut self, name: impl Into<String>, color: Color) -> Self {
        self.colors.insert(name.into(), color);
        self
    }

    /// 添加间距令牌
    pub fn with_spacing(mut self, name: impl Into<String>, value: f32) -> Self {
        self.spacing.insert(name.into(), value);
        self
    }

    /// 添加字体大小令牌
    pub fn with_font_size(mut self, name: impl Into<String>, value: f32) -> Self {
        self.font_sizes.insert(name.into(), value);
        self
    }

    /// 添加圆角令牌
    pub fn with_border_radius(mut self, name: impl Into<String>, value: f32) -> Self {
        self.border_radii.insert(name.into(), value);
        self
    }

    /// 添加阴影令牌
    pub fn with_shadow(mut self, name: impl Into<String>, shadow: ShadowToken) -> Self {
        self.shadows.insert(name.into(), shadow);
        self
    }
}

impl ThemeTokens for Theme {
    fn color(&self, name: &str) -> Option<&Color> {
        self.colors.get(name)
    }

    fn spacing(&self, name: &str) -> Option<f32> {
        self.spacing.get(name).copied()
    }

    fn font_size(&self, name: &str) -> Option<f32> {
        self.font_sizes.get(name).copied()
    }

    fn border_radius(&self, name: &str) -> Option<f32> {
        self.border_radii.get(name).copied()
    }

    fn shadow(&self, name: &str) -> Option<&ShadowToken> {
        self.shadows.get(name)
    }
}

/// 主题注册表
pub struct ThemeRegistry {
    /// 主题映射
    themes: HashMap<ThemeId, Theme>,
    /// 当前活跃主题 ID
    active_id: ThemeId,
    /// 主题变更信号
    change_signal: crate::reactive::Signal<ThemeId>,
}

impl ThemeRegistry {
    /// 创建注册表并注册默认主题
    pub fn new(default_theme: Theme) -> Self {
        let id = default_theme.id.clone();
        let mut themes = HashMap::new();
        themes.insert(id.clone(), default_theme);
        Self { themes, active_id: id.clone(), change_signal: crate::reactive::Signal::new(id) }
    }

    /// 注册主题
    pub fn register(&mut self, theme: Theme) {
        self.themes.insert(theme.id.clone(), theme);
    }

    /// 获取主题
    pub fn get(&self, id: &ThemeId) -> Option<&Theme> {
        self.themes.get(id)
    }

    /// 设置当前活跃主题（如果主题存在）
    pub fn set_active(&mut self, id: &ThemeId) {
        if self.themes.contains_key(id) {
            self.active_id = id.clone();
            self.change_signal.set(id.clone());
        }
    }

    /// 获取当前活跃主题
    pub fn active(&self) -> &Theme {
        self.themes.get(&self.active_id).expect("active theme must exist in registry")
    }

    /// 获取当前活跃主题 ID
    pub fn active_id(&self) -> &ThemeId {
        &self.active_id
    }

    /// 获取主题变更信号
    pub fn on_theme_change(&self) -> &crate::reactive::Signal<ThemeId> {
        &self.change_signal
    }
}
