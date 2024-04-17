use std::collections::HashMap;

use gg_render::Color;
use gg_ui::{Signal, Theme, ThemeId, ThemeRegistry, ThemeTokens};

/// 编辑器专用主题，扩展基础 Theme
#[derive(Debug, Clone)]
pub struct EditorTheme {
    /// 基础主题
    pub base: Theme,
    /// 面板背景色
    pub panel_background: Color,
    /// 工具栏背景色
    pub toolbar_background: Color,
    /// 检查器背景色
    pub inspector_background: Color,
    /// 层级视图背景色
    pub hierarchy_background: Color,
    /// 选中项颜色
    pub selection_color: Color,
    /// 悬停颜色
    pub hover_color: Color,
    /// 激活颜色
    pub active_color: Color,
    /// 禁用颜色
    pub disabled_color: Color,
    /// 强调色
    pub accent_color: Color,
}

impl EditorTheme {
    /// 创建编辑器主题，使用基础主题的默认颜色
    pub fn new(base: Theme) -> Self {
        let panel_background = base.color("background").copied().unwrap_or(Color::WHITE);
        let toolbar_background = base.color("surface").copied().unwrap_or(Color::new(0.94, 0.94, 0.94, 1.0));
        let selection_color = base.color("primary").copied().unwrap_or(Color::new(0.0, 0.47, 0.83, 1.0));
        let accent_color = base.color("primary").copied().unwrap_or(Color::new(0.0, 0.47, 0.83, 1.0));

        Self {
            base,
            panel_background,
            toolbar_background,
            inspector_background: panel_background,
            hierarchy_background: toolbar_background,
            selection_color,
            hover_color: Color::new(0.9, 0.9, 0.9, 1.0),
            active_color: Color::new(0.8, 0.8, 0.8, 1.0),
            disabled_color: Color::new(0.63, 0.63, 0.63, 1.0),
            accent_color,
        }
    }

    /// 设置面板背景色
    pub fn with_panel_background(mut self, color: Color) -> Self {
        self.panel_background = color;
        self
    }

    /// 设置工具栏背景色
    pub fn with_toolbar_background(mut self, color: Color) -> Self {
        self.toolbar_background = color;
        self
    }

    /// 设置检查器背景色
    pub fn with_inspector_background(mut self, color: Color) -> Self {
        self.inspector_background = color;
        self
    }

    /// 设置层级视图背景色
    pub fn with_hierarchy_background(mut self, color: Color) -> Self {
        self.hierarchy_background = color;
        self
    }

    /// 设置选中项颜色
    pub fn with_selection_color(mut self, color: Color) -> Self {
        self.selection_color = color;
        self
    }

    /// 设置悬停颜色
    pub fn with_hover_color(mut self, color: Color) -> Self {
        self.hover_color = color;
        self
    }

    /// 设置激活颜色
    pub fn with_active_color(mut self, color: Color) -> Self {
        self.active_color = color;
        self
    }

    /// 设置禁用颜色
    pub fn with_disabled_color(mut self, color: Color) -> Self {
        self.disabled_color = color;
        self
    }

    /// 设置强调色
    pub fn with_accent_color(mut self, color: Color) -> Self {
        self.accent_color = color;
        self
    }

    /// 获取基础主题
    pub fn base(&self) -> &Theme {
        &self.base
    }
}

impl ThemeTokens for EditorTheme {
    fn color(&self, name: &str) -> Option<&Color> {
        self.base.color(name)
    }

    fn spacing(&self, name: &str) -> Option<f32> {
        self.base.spacing(name)
    }

    fn font_size(&self, name: &str) -> Option<f32> {
        self.base.font_size(name)
    }

    fn border_radius(&self, name: &str) -> Option<f32> {
        self.base.border_radius(name)
    }

    fn shadow(&self, name: &str) -> Option<&gg_ui::ShadowToken> {
        self.base.shadow(name)
    }
}

/// 编辑器主题提供者
pub struct EditorThemeProvider {
    /// 主题注册表
    registry: ThemeRegistry,
    /// 编辑器主题映射
    editor_themes: HashMap<ThemeId, EditorTheme>,
}

impl EditorThemeProvider {
    /// 创建提供者并注册默认主题
    pub fn new(default_theme: EditorTheme) -> Self {
        let id = default_theme.base.id.clone();
        let registry = ThemeRegistry::new(default_theme.base.clone());
        let mut editor_themes = HashMap::new();
        editor_themes.insert(id, default_theme);
        Self { registry, editor_themes }
    }

    /// 注册编辑器主题
    pub fn register(&mut self, theme: EditorTheme) {
        let id = theme.base.id.clone();
        self.registry.register(theme.base.clone());
        self.editor_themes.insert(id, theme);
    }

    /// 获取当前主题
    pub fn current_theme(&self) -> &EditorTheme {
        let active_id = self.registry.active_id();
        self.editor_themes.get(active_id).expect("current editor theme must exist in provider")
    }

    /// 设置当前主题
    pub fn set_theme(&mut self, id: &ThemeId) {
        self.registry.set_active(id);
    }

    /// 获取主题变更信号
    pub fn on_theme_change(&self) -> &Signal<ThemeId> {
        self.registry.on_theme_change()
    }
}

/// 返回预设的编辑器亮色主题
pub fn default_light_theme() -> EditorTheme {
    let base = Theme::new(ThemeId::Light, "Light")
        .with_color("primary", Color::new(0.0, 0.47, 0.83, 1.0))
        .with_color("background", Color::new(1.0, 1.0, 1.0, 1.0))
        .with_color("surface", Color::new(0.94, 0.94, 0.94, 1.0))
        .with_color("text-primary", Color::new(0.1, 0.1, 0.1, 1.0))
        .with_color("border", Color::new(0.83, 0.83, 0.83, 1.0));

    EditorTheme::new(base)
        .with_panel_background(Color::new(1.0, 1.0, 1.0, 1.0))
        .with_toolbar_background(Color::new(0.94, 0.94, 0.94, 1.0))
        .with_inspector_background(Color::new(0.98, 0.98, 0.98, 1.0))
        .with_hierarchy_background(Color::new(0.96, 0.96, 0.96, 1.0))
        .with_selection_color(Color::new(0.0, 0.47, 0.83, 1.0))
        .with_hover_color(Color::new(0.9, 0.9, 0.9, 1.0))
        .with_active_color(Color::new(0.8, 0.8, 0.8, 1.0))
        .with_disabled_color(Color::new(0.63, 0.63, 0.63, 1.0))
        .with_accent_color(Color::new(0.0, 0.47, 0.83, 1.0))
}

/// 返回预设的编辑器暗色主题
pub fn default_dark_theme() -> EditorTheme {
    let base = Theme::new(ThemeId::Dark, "Dark")
        .with_color("primary", Color::new(0.0, 0.47, 0.83, 1.0))
        .with_color("background", Color::new(0.12, 0.12, 0.12, 1.0))
        .with_color("surface", Color::new(0.18, 0.18, 0.18, 1.0))
        .with_color("text-primary", Color::new(0.88, 0.88, 0.88, 1.0))
        .with_color("border", Color::new(0.28, 0.28, 0.28, 1.0));

    EditorTheme::new(base)
        .with_panel_background(Color::new(0.17, 0.17, 0.17, 1.0))
        .with_toolbar_background(Color::new(0.2, 0.2, 0.2, 1.0))
        .with_inspector_background(Color::new(0.15, 0.15, 0.15, 1.0))
        .with_hierarchy_background(Color::new(0.18, 0.18, 0.18, 1.0))
        .with_selection_color(Color::new(0.04, 0.28, 0.44, 1.0))
        .with_hover_color(Color::new(0.16, 0.18, 0.18, 1.0))
        .with_active_color(Color::new(0.24, 0.24, 0.24, 1.0))
        .with_disabled_color(Color::new(0.35, 0.35, 0.35, 1.0))
        .with_accent_color(Color::new(0.0, 0.47, 0.83, 1.0))
}
