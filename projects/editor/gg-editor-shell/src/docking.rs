//! 基础 Docking 布局系统

use crate::panel::{EditorPanel, PanelPosition};
use gg_render::{Color, DrawCommand, Rect, RenderContext};

/// 停靠区域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockRegion {
    /// 左侧区域
    Left,
    /// 右侧区域
    Right,
    /// 中央区域
    Center,
    /// 底部区域
    Bottom,
}

/// 区域分隔条
#[derive(Debug, Clone)]
pub struct DockSplit {
    /// 分隔条所属的区域边界
    pub region: DockRegion,
    /// 分隔条的位置（像素偏移）
    pub position: f32,
    /// 最小区域尺寸
    pub min_size: f32,
    /// 是否正在拖拽
    pub is_dragging: bool,
}

/// 区域配置
#[derive(Debug, Clone)]
pub struct DockRegionConfig {
    /// 区域宽度（Left/Right 使用）或高度（Bottom 使用）
    pub size: f32,
    /// 最小尺寸
    pub min_size: f32,
    /// 该区域包含的面板名称列表
    pub panel_names: Vec<String>,
}

/// Docking 布局管理器
#[derive(Debug, Clone)]
pub struct DockingLayout {
    /// 左侧区域配置
    pub left: Option<DockRegionConfig>,
    /// 右侧区域配置
    pub right: Option<DockRegionConfig>,
    /// 底部区域配置
    pub bottom: Option<DockRegionConfig>,
    /// 分隔条列表
    pub splits: Vec<DockSplit>,
}

/// 布局计算结果
#[derive(Debug, Clone, PartialEq)]
pub struct PanelLayout {
    /// 面板名称
    pub name: String,
    /// X 坐标
    pub x: f32,
    /// Y 坐标
    pub y: f32,
    /// 宽度
    pub width: f32,
    /// 高度
    pub height: f32,
}

impl DockingLayout {
    /// 创建默认布局
    ///
    /// 默认布局包含右侧检查器面板（300px）和底部资源浏览器面板（200px）。
    pub fn default_layout() -> Self {
        DockingLayout {
            left: None,
            right: Some(DockRegionConfig {
                size: 300.0,
                min_size: 200.0,
                panel_names: vec!["inspector".to_string()],
            }),
            bottom: Some(DockRegionConfig {
                size: 200.0,
                min_size: 100.0,
                panel_names: vec!["asset_browser".to_string()],
            }),
            splits: vec![
                DockSplit {
                    region: DockRegion::Right,
                    position: 300.0,
                    min_size: 200.0,
                    is_dragging: false,
                },
                DockSplit {
                    region: DockRegion::Bottom,
                    position: 200.0,
                    min_size: 100.0,
                    is_dragging: false,
                },
            ],
        }
    }

    /// 计算所有面板的布局
    ///
    /// 根据当前区域配置和窗口尺寸，为每个面板计算位置和大小。
    ///
    /// # 参数
    ///
    /// - `panels` - 面板列表
    /// - `width` - 可用宽度
    /// - `height` - 可用高度
    pub fn compute(&self, panels: &[Box<dyn EditorPanel>], width: f32, height: f32) -> Vec<PanelLayout> {
        let mut results = Vec::new();

        let right_width = self.right.as_ref().map(|r| r.size).unwrap_or(0.0);
        let bottom_height = self.bottom.as_ref().map(|b| b.size).unwrap_or(0.0);
        let left_width = self.left.as_ref().map(|l| l.size).unwrap_or(0.0);

        for panel in panels {
            let hint = panel.layout_hint();
            let (x, y, w, h) = match hint.position {
                PanelPosition::Left => (0.0, 0.0, left_width, height - bottom_height),
                PanelPosition::Right => (width - right_width, 0.0, right_width, height - bottom_height),
                PanelPosition::Center => (left_width, 0.0, width - left_width - right_width, height - bottom_height),
                PanelPosition::Bottom => (0.0, height - bottom_height, width, bottom_height),
                PanelPosition::Floating => (0.0, 0.0, width, height),
            };
            results.push(PanelLayout { name: panel.name().to_string(), x, y, width: w, height: h });
        }
        results
    }

    /// 开始拖拽分隔条
    ///
    /// # 参数
    ///
    /// - `region` - 要拖拽的区域
    /// - `_x` - 鼠标 X 坐标（保留参数）
    /// - `_y` - 鼠标 Y 坐标（保留参数）
    pub fn handle_drag_start(&mut self, region: DockRegion, _x: f32, _y: f32) {
        if let Some(split) = self.splits.iter_mut().find(|s| s.region == region) {
            split.is_dragging = true;
        }
    }

    /// 拖拽移动分隔条
    ///
    /// # 参数
    ///
    /// - `region` - 拖拽的区域
    /// - `delta_x` - X 方向偏移量
    /// - `delta_y` - Y 方向偏移量
    pub fn handle_drag_move(&mut self, region: DockRegion, delta_x: f32, delta_y: f32) {
        if let Some(split) = self.splits.iter_mut().find(|s| s.region == region) {
            if split.is_dragging {
                match region {
                    DockRegion::Left | DockRegion::Right => {
                        let new_pos = split.position + delta_x;
                        if new_pos >= split.min_size {
                            split.position = new_pos;
                            let mut config = match region {
                                DockRegion::Left => self.left.as_mut(),
                                DockRegion::Right => self.right.as_mut(),
                                _ => None,
                            };
                            if let Some(ref mut cfg) = config {
                                cfg.size = split.position;
                            }
                        }
                    }
                    DockRegion::Bottom => {
                        let new_pos = split.position + delta_y;
                        if new_pos >= split.min_size {
                            split.position = new_pos;
                            if let Some(ref mut cfg) = self.bottom.as_mut() {
                                cfg.size = split.position;
                            }
                        }
                    }
                    DockRegion::Center => {}
                }
            }
        }
    }

    /// 结束拖拽分隔条
    ///
    /// # 参数
    ///
    /// - `region` - 拖拽的区域
    pub fn handle_drag_end(&mut self, region: DockRegion) {
        if let Some(split) = self.splits.iter_mut().find(|s| s.region == region) {
            split.is_dragging = false;
        }
    }

    /// 渲染区域分隔条
    ///
    /// 在相邻区域边界绘制可见的分隔条线条。
    ///
    /// # 参数
    ///
    /// - `context` - 渲染上下文
    /// - `width` - 渲染表面宽度
    /// - `height` - 渲染表面高度
    pub fn render_splits(&self, context: &mut RenderContext, width: f32, height: f32) {
        let split_color = Color::new(0.5, 0.5, 0.5, 1.0);
        let split_thickness = 2.0;

        for split in &self.splits {
            let rect = match split.region {
                DockRegion::Left | DockRegion::Right => {
                    let x = match split.region {
                        DockRegion::Left => split.position,
                        DockRegion::Right => width - split.position,
                        _ => 0.0,
                    };
                    Rect::new(x - split_thickness / 2.0, 0.0, split_thickness, height)
                }
                DockRegion::Bottom => {
                    let y = height - split.position;
                    Rect::new(0.0, y - split_thickness / 2.0, width, split_thickness)
                }
                DockRegion::Center => continue,
            };
            context.draw(DrawCommand::Rect {
                rect,
                color: split_color,
                corner_radius: 0.0,
            });
        }
    }

    /// 检测鼠标是否在分隔条区域
    ///
    /// 在分隔条位置 ±4px 范围内视为命中。
    ///
    /// # 参数
    ///
    /// - `x` - 鼠标 X 坐标
    /// - `y` - 鼠标 Y 坐标
    /// - `width` - 渲染表面宽度
    /// - `height` - 渲染表面高度
    pub fn hit_test_split(&self, x: f32, y: f32, width: f32, height: f32) -> Option<DockRegion> {
        const HIT_THRESHOLD: f32 = 4.0;

        for split in &self.splits {
            let is_hit = match split.region {
                DockRegion::Left | DockRegion::Right => {
                    let split_x = match split.region {
                        DockRegion::Left => split.position,
                        DockRegion::Right => width - split.position,
                        _ => 0.0,
                    };
                    (x - split_x).abs() <= HIT_THRESHOLD && y >= 0.0 && y <= height
                }
                DockRegion::Bottom => {
                    let split_y = height - split.position;
                    (y - split_y).abs() <= HIT_THRESHOLD && x >= 0.0 && x <= width
                }
                DockRegion::Center => false,
            };
            if is_hit {
                return Some(split.region);
            }
        }
        None
    }
}
