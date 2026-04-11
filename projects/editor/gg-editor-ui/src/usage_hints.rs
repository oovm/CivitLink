//! UsageHints GPU 驱动变换模块
//!
//! 提供 UsageHints 机制，允许将特定属性变化（位移、颜色、透明度、缩放）
//! 通过 GPU uniform 传入着色器执行变换，避免 CPU 侧重新生成顶点数据

use std::collections::HashMap;

/// 单个 UsageHint 标志，指示哪种属性变化由 GPU 处理
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageHint {
    /// 位移变化通过 uniform 传入着色器，GPU 执行变换
    TransformOffset,
    /// 颜色变化通过 uniform 传入着色器，GPU 执行颜色混合
    ColorTint,
    /// 透明度变化通过 uniform 传入着色器
    Opacity,
    /// 缩放变化通过 uniform 传入着色器
    ScaleTransform,
}

impl UsageHint {
    /// 获取对应位标志值
    fn bit(self) -> u8 {
        match self {
            UsageHint::TransformOffset => 1,
            UsageHint::ColorTint => 2,
            UsageHint::Opacity => 4,
            UsageHint::ScaleTransform => 8,
        }
    }
}

/// UsageHint 集合，基于 u8 位标志实现
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsageHints {
    /// 内部位标志
    flags: u8,
}

impl UsageHints {
    /// 无任何 hint
    pub const NONE: UsageHints = UsageHints { flags: 0 };
    /// TransformOffset hint 标志
    pub const TRANSFORM_OFFSET: UsageHints = UsageHints { flags: 1 };
    /// ColorTint hint 标志
    pub const COLOR_TINT: UsageHints = UsageHints { flags: 2 };
    /// Opacity hint 标志
    pub const OPACITY: UsageHints = UsageHints { flags: 4 };
    /// ScaleTransform hint 标志
    pub const SCALE_TRANSFORM: UsageHints = UsageHints { flags: 8 };

    /// 判断是否包含指定 hint
    pub fn contains(&self, hint: UsageHint) -> bool {
        self.flags & hint.bit() != 0
    }

    /// 插入指定 hint
    pub fn insert(&mut self, hint: UsageHint) {
        self.flags |= hint.bit();
    }

    /// 移除指定 hint
    pub fn remove(&mut self, hint: UsageHint) {
        self.flags &= !hint.bit();
    }

    /// 判断是否没有任何 hint
    pub fn is_empty(&self) -> bool {
        self.flags == 0
    }
}

impl Default for UsageHints {
    fn default() -> Self {
        UsageHints::NONE
    }
}

/// GPU 变换 uniform 数据，传递给着色器执行 GPU 侧变换
#[derive(Debug, Clone, Copy)]
pub struct GpuTransformUniform {
    /// X 位移
    pub offset_x: f32,
    /// Y 位移
    pub offset_y: f32,
    /// X 缩放
    pub scale_x: f32,
    /// Y 缩放
    pub scale_y: f32,
    /// 颜色调色 (RGBA)
    pub color_tint: [f32; 4],
    /// 透明度
    pub opacity: f32,
}

impl Default for GpuTransformUniform {
    fn default() -> Self {
        GpuTransformUniform {
            offset_x: 0.0,
            offset_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            color_tint: [1.0, 1.0, 1.0, 1.0],
            opacity: 1.0,
        }
    }
}

/// 管理所有元素的 UsageHints 和 GPU uniform 数据
#[derive(Debug, Clone)]
pub struct UsageHintsManager {
    /// 元素 ID 到 UsageHints 的映射
    hints: HashMap<String, UsageHints>,
    /// 元素 ID 到 GPU uniform 数据的映射
    uniforms: HashMap<String, GpuTransformUniform>,
}

impl UsageHintsManager {
    /// 创建新的 UsageHintsManager
    pub fn new() -> Self {
        UsageHintsManager {
            hints: HashMap::new(),
            uniforms: HashMap::new(),
        }
    }

    /// 设置元素的 UsageHints
    pub fn set_hints(&mut self, element_id: &str, hints: UsageHints) {
        self.hints.insert(element_id.to_string(), hints);
        if !self.uniforms.contains_key(element_id) {
            self.uniforms.insert(element_id.to_string(), GpuTransformUniform::default());
        }
    }

    /// 获取元素的 UsageHints
    pub fn get_hints(&self, element_id: &str) -> Option<&UsageHints> {
        self.hints.get(element_id)
    }

    /// 更新位移 uniform，仅当元素有 TransformOffset hint 时生效
    pub fn update_transform_offset(&mut self, element_id: &str, offset_x: f32, offset_y: f32) {
        if let Some(hints) = self.hints.get(element_id) {
            if hints.contains(UsageHint::TransformOffset) {
                if let Some(uniform) = self.uniforms.get_mut(element_id) {
                    uniform.offset_x = offset_x;
                    uniform.offset_y = offset_y;
                }
            }
        }
    }

    /// 更新颜色 uniform，仅当元素有 ColorTint hint 时生效
    pub fn update_color_tint(&mut self, element_id: &str, color: [f32; 4]) {
        if let Some(hints) = self.hints.get(element_id) {
            if hints.contains(UsageHint::ColorTint) {
                if let Some(uniform) = self.uniforms.get_mut(element_id) {
                    uniform.color_tint = color;
                }
            }
        }
    }

    /// 更新透明度 uniform，仅当元素有 Opacity hint 时生效
    pub fn update_opacity(&mut self, element_id: &str, opacity: f32) {
        if let Some(hints) = self.hints.get(element_id) {
            if hints.contains(UsageHint::Opacity) {
                if let Some(uniform) = self.uniforms.get_mut(element_id) {
                    uniform.opacity = opacity;
                }
            }
        }
    }

    /// 更新缩放 uniform，仅当元素有 ScaleTransform hint 时生效
    pub fn update_scale(&mut self, element_id: &str, scale_x: f32, scale_y: f32) {
        if let Some(hints) = self.hints.get(element_id) {
            if hints.contains(UsageHint::ScaleTransform) {
                if let Some(uniform) = self.uniforms.get_mut(element_id) {
                    uniform.scale_x = scale_x;
                    uniform.scale_y = scale_y;
                }
            }
        }
    }

    /// 获取元素的 GPU uniform 数据
    pub fn get_uniforms(&self, element_id: &str) -> Option<&GpuTransformUniform> {
        self.uniforms.get(element_id)
    }

    /// 判断元素是否需要 CPU 侧顶点数据更新
    ///
    /// 如果元素的脏标记仅涉及 UsageHints 覆盖的属性，则返回 false（由 GPU 处理）。
    /// 映射关系：
    /// - TransformOffset / ScaleTransform → 覆盖 TRANSFORM 脏标记
    /// - ColorTint / Opacity → 覆盖 STYLE 脏标记
    /// - LAYOUT / CONTENT 脏标记始终需要 CPU 更新
    pub fn needs_vertex_update(&self, element_id: &str, dirty_flag: &crate::DirtyFlag) -> bool {
        if dirty_flag.is_empty() {
            return false;
        }

        let hints = match self.hints.get(element_id) {
            Some(h) => h,
            None => return true,
        };

        if hints.is_empty() {
            return true;
        }

        if dirty_flag.contains(crate::DirtyFlag::LAYOUT) {
            return true;
        }

        if dirty_flag.contains(crate::DirtyFlag::CONTENT) {
            return true;
        }

        if dirty_flag.contains(crate::DirtyFlag::STYLE) {
            if !hints.contains(UsageHint::ColorTint) && !hints.contains(UsageHint::Opacity) {
                return true;
            }
        }

        if dirty_flag.contains(crate::DirtyFlag::TRANSFORM) {
            if !hints.contains(UsageHint::TransformOffset) && !hints.contains(UsageHint::ScaleTransform) {
                return true;
            }
        }

        false
    }
}

impl Default for UsageHintsManager {
    fn default() -> Self {
        Self::new()
    }
}
