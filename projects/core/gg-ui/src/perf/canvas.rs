use std::{
    collections::HashMap,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

/// Canvas 脏标记位标志
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasDirtyFlag(u32);

impl CanvasDirtyFlag {
    /// 顶点数据脏
    pub const VERTICES: CanvasDirtyFlag = CanvasDirtyFlag(1 << 0);
    /// 材质数据脏
    pub const MATERIAL: CanvasDirtyFlag = CanvasDirtyFlag(1 << 1);
    /// 布局数据脏
    pub const LAYOUT: CanvasDirtyFlag = CanvasDirtyFlag(1 << 2);
    /// 所有标记
    pub const ALL: CanvasDirtyFlag = CanvasDirtyFlag(Self::VERTICES.0 | Self::MATERIAL.0 | Self::LAYOUT.0);

    /// 判断是否包含指定脏标记
    pub fn contains(self, other: CanvasDirtyFlag) -> bool {
        self.0 & other.0 != 0
    }

    /// 判断是否没有任何脏标记
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// 获取内部位值
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl Default for CanvasDirtyFlag {
    fn default() -> Self {
        CanvasDirtyFlag::ALL
    }
}

impl BitOr for CanvasDirtyFlag {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        CanvasDirtyFlag(self.0 | rhs.0)
    }
}

impl BitOrAssign for CanvasDirtyFlag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for CanvasDirtyFlag {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        CanvasDirtyFlag(self.0 & rhs.0)
    }
}

impl BitAndAssign for CanvasDirtyFlag {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for CanvasDirtyFlag {
    type Output = Self;

    fn not(self) -> Self::Output {
        CanvasDirtyFlag(!self.0 & CanvasDirtyFlag::ALL.0)
    }
}

/// Canvas 渲染模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasRenderMode {
    /// 屏幕空间
    ScreenSpace,
    /// 世界空间
    WorldSpace,
    /// 相机空间
    CameraSpace,
}

/// Canvas 中的 UI 元素
#[derive(Debug, Clone)]
pub struct CanvasElement {
    /// 元素 ID
    pub id: String,
    /// 材质 ID
    pub material_id: u64,
    /// 纹理 ID
    pub texture_id: u64,
    /// 顶点数据
    pub vertex_data: Vec<f32>,
    /// 索引数据
    pub index_data: Vec<u32>,
    /// 脏标记
    pub dirty_flags: CanvasDirtyFlag,
    /// 渲染排序
    pub sort_order: i32,
}

/// 批处理合并结果
#[derive(Debug, Clone)]
pub struct CanvasBatch {
    /// 合并后的材质 ID
    pub material_id: u64,
    /// 合并后的纹理 ID
    pub texture_id: u64,
    /// 合并后的顶点数据
    pub merged_vertex_data: Vec<f32>,
    /// 合并后的索引数据
    pub merged_index_data: Vec<u32>,
    /// 包含的元素数量
    pub element_count: usize,
    /// Draw Call 数量
    pub draw_call_count: u32,
}

/// 负责每帧收集和合并 Canvas 下的 UI 网格
pub struct CanvasRebuilder {
    /// Canvas ID
    pub canvas_id: String,
    /// 渲染模式
    pub render_mode: CanvasRenderMode,
    /// Canvas 中的所有元素
    pub elements: Vec<CanvasElement>,
    /// 当前帧的批处理结果
    pub batches: Vec<CanvasBatch>,
    /// 是否为静态 Canvas（静态 Canvas 跳过每帧重建）
    pub is_static: bool,
    /// 是否有脏元素
    pub has_dirty_elements: bool,
}

impl CanvasRebuilder {
    /// 创建新的 Canvas 重建器
    pub fn new(canvas_id: &str, render_mode: CanvasRenderMode) -> Self {
        Self {
            canvas_id: canvas_id.to_string(),
            render_mode,
            elements: Vec::new(),
            batches: Vec::new(),
            is_static: false,
            has_dirty_elements: true,
        }
    }

    /// 添加 UI 元素
    pub fn add_element(&mut self, element: CanvasElement) {
        if !element.dirty_flags.is_empty() {
            self.has_dirty_elements = true;
        }
        self.elements.push(element);
    }

    /// 标记元素顶点为脏
    pub fn set_vertices_dirty(&mut self, element_id: &str) {
        if let Some(element) = self.elements.iter_mut().find(|e| e.id == element_id) {
            element.dirty_flags = element.dirty_flags | CanvasDirtyFlag::VERTICES;
            self.has_dirty_elements = true;
        }
    }

    /// 标记元素材质为脏
    pub fn set_material_dirty(&mut self, element_id: &str) {
        if let Some(element) = self.elements.iter_mut().find(|e| e.id == element_id) {
            element.dirty_flags = element.dirty_flags | CanvasDirtyFlag::MATERIAL;
            self.has_dirty_elements = true;
        }
    }

    /// 执行合并-重建：收集脏元素，按材质/纹理分组合并网格，生成 Draw Call 队列。
    /// 如果是静态 Canvas 且无脏元素，跳过重建
    pub fn rebuild(&mut self) {
        if self.is_static && !self.has_dirty_elements {
            return;
        }

        self.batches.clear();

        let mut group_map: HashMap<(u64, u64), Vec<&CanvasElement>> = HashMap::new();

        for element in &self.elements {
            let key = (element.material_id, element.texture_id);
            group_map.entry(key).or_default().push(element);
        }

        for ((material_id, texture_id), group_elements) in &group_map {
            let mut merged_vertex_data = Vec::new();
            let mut merged_index_data = Vec::new();
            let mut element_count = 0usize;
            let mut vertex_offset = 0u32;

            let mut sorted_elements: Vec<&&CanvasElement> = group_elements.iter().collect();
            sorted_elements.sort_by_key(|e| e.sort_order);

            for element in sorted_elements {
                merged_vertex_data.extend_from_slice(&element.vertex_data);
                for &index in &element.index_data {
                    merged_index_data.push(index + vertex_offset);
                }
                vertex_offset += (element.vertex_data.len() / 3) as u32;
                element_count += 1;
            }

            self.batches.push(CanvasBatch {
                material_id: *material_id,
                texture_id: *texture_id,
                merged_vertex_data,
                merged_index_data,
                element_count,
                draw_call_count: 1,
            });
        }

        for element in &mut self.elements {
            element.dirty_flags = CanvasDirtyFlag::default() & !CanvasDirtyFlag::ALL;
        }
        self.has_dirty_elements = false;
    }

    /// 获取当前帧的批处理结果
    pub fn get_batches(&self) -> &[CanvasBatch] {
        &self.batches
    }

    /// 设置 Canvas 是否为静态
    pub fn set_static(&mut self, is_static: bool) {
        self.is_static = is_static;
    }

    /// 动静分离：将指定元素分离到新的动态 Canvas，返回新的 CanvasRebuilder
    pub fn separate_dynamic_elements(&mut self, dynamic_element_ids: &[&str]) -> CanvasRebuilder {
        let mut dynamic_rebuilder = CanvasRebuilder::new(&format!("{}_dynamic", self.canvas_id), self.render_mode);

        let dynamic_id_set: std::collections::HashSet<&str> = dynamic_element_ids.iter().copied().collect();

        let mut remaining = Vec::new();
        for element in self.elements.drain(..) {
            if dynamic_id_set.contains(element.id.as_str()) {
                dynamic_rebuilder.add_element(element);
            }
            else {
                remaining.push(element);
            }
        }

        self.elements = remaining;
        self.has_dirty_elements = self.elements.iter().any(|e| !e.dirty_flags.is_empty());

        dynamic_rebuilder
    }
}

/// 批处理合并结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchMergeResult {
    /// 成功合并
    Merged,
    /// 材质不同，无法合并
    MaterialDifferent,
    /// 纹理不同，无法合并
    TextureDifferent,
    /// 渲染顺序被中断
    SortOrderInterrupted,
}

/// 单个 Canvas 的批处理信息
#[derive(Debug, Clone)]
pub struct CanvasBatchInfo {
    /// Canvas ID
    pub canvas_id: String,
    /// 总 Draw Call 数量
    pub total_draw_calls: u32,
    /// 总批次数
    pub total_batches: u32,
    /// 总元素数量
    pub total_elements: usize,
    /// 每个元素的合并结果
    pub merge_results: Vec<(String, BatchMergeResult)>,
    /// 是否为静态 Canvas
    pub is_static: bool,
}

/// 动静分离建议
#[derive(Debug, Clone)]
pub struct DynamicSeparationSuggestion {
    /// 建议分离的元素 ID
    pub element_id: String,
    /// 当前所在 Canvas ID
    pub parent_canvas_id: String,
    /// 分离原因
    pub reason: String,
    /// 脏标记频率（0.0~1.0）
    pub dirty_frequency: f32,
}

/// 批处理调试器
pub struct CanvasBatchDebugger {
    /// 所有 Canvas 的批处理信息
    pub canvas_infos: Vec<CanvasBatchInfo>,
    /// 动静分离建议
    pub suggestions: Vec<DynamicSeparationSuggestion>,
    /// 元素 ID → 最近 N 帧的脏标记历史
    pub dirty_history: HashMap<String, Vec<bool>>,
    /// 历史窗口大小（默认 60 帧）
    pub history_window: usize,
}

impl CanvasBatchDebugger {
    /// 创建新的批处理调试器
    pub fn new() -> Self {
        Self { canvas_infos: Vec::new(), suggestions: Vec::new(), dirty_history: HashMap::new(), history_window: 60 }
    }

    /// 记录一帧的批处理信息，接收所有 Canvas 的 CanvasRebuilder 引用
    pub fn record_frame(&mut self, rebuilders: &[&CanvasRebuilder]) {
        self.canvas_infos.clear();

        for rebuilder in rebuilders {
            let total_draw_calls: u32 = rebuilder.batches.iter().map(|b| b.draw_call_count).sum();
            let total_batches = rebuilder.batches.len() as u32;
            let total_elements = rebuilder.elements.len();
            let merge_results = Self::compute_merge_results(rebuilder);

            for element in &rebuilder.elements {
                let is_dirty = !element.dirty_flags.is_empty();
                let history = self.dirty_history.entry(element.id.clone()).or_default();
                history.push(is_dirty);
                if history.len() > self.history_window {
                    history.remove(0);
                }
            }

            self.canvas_infos.push(CanvasBatchInfo {
                canvas_id: rebuilder.canvas_id.clone(),
                total_draw_calls,
                total_batches,
                total_elements,
                merge_results,
                is_static: rebuilder.is_static,
            });
        }
    }

    /// 分析所有 Canvas 的批处理数据，生成动静分离建议。
    /// 规则：如果元素在最近 N 帧中脏标记频率超过 50%，建议分离到独立 Canvas
    pub fn analyze(&self) -> Vec<DynamicSeparationSuggestion> {
        let mut result = Vec::new();

        for canvas_info in &self.canvas_infos {
            for (element_id, _) in &canvas_info.merge_results {
                if let Some(history) = self.dirty_history.get(element_id) {
                    if history.is_empty() {
                        continue;
                    }
                    let dirty_count = history.iter().filter(|&&d| d).count();
                    let frequency = dirty_count as f32 / history.len() as f32;
                    if frequency > 0.5 {
                        result.push(DynamicSeparationSuggestion {
                            element_id: element_id.clone(),
                            parent_canvas_id: canvas_info.canvas_id.clone(),
                            reason: format!("元素脏标记频率 {:.1}% 超过阈值 50%，建议分离到独立动态 Canvas", frequency * 100.0),
                            dirty_frequency: frequency,
                        });
                    }
                }
            }
        }

        result
    }

    /// 获取指定 Canvas 的批处理信息
    pub fn get_canvas_info(&self, canvas_id: &str) -> Option<&CanvasBatchInfo> {
        self.canvas_infos.iter().find(|info| info.canvas_id == canvas_id)
    }

    /// 生成调试报告，包含每个 Canvas 的 Draw Call 数量、批处理合并情况、未合并原因、动静分离建议
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Canvas Batch Debug Report ===\n\n");

        for canvas_info in &self.canvas_infos {
            report.push_str(&format!("Canvas: {}\n", canvas_info.canvas_id));
            report.push_str(&format!("  Static: {}\n", if canvas_info.is_static { "Yes" } else { "No" }));
            report.push_str(&format!("  Draw Calls: {}\n", canvas_info.total_draw_calls));
            report.push_str(&format!("  Batches: {}\n", canvas_info.total_batches));
            report.push_str(&format!("  Elements: {}\n", canvas_info.total_elements));

            report.push_str("  Merge Results:\n");
            for (element_id, merge_result) in &canvas_info.merge_results {
                let label = match merge_result {
                    BatchMergeResult::Merged => "Merged",
                    BatchMergeResult::MaterialDifferent => "Material Different",
                    BatchMergeResult::TextureDifferent => "Texture Different",
                    BatchMergeResult::SortOrderInterrupted => "Sort Order Interrupted",
                };
                report.push_str(&format!("    {} -> {}\n", element_id, label));
            }

            report.push('\n');
        }

        let suggestions = self.analyze();
        if suggestions.is_empty() {
            report.push_str("No dynamic separation suggestions.\n");
        }
        else {
            report.push_str("=== Dynamic Separation Suggestions ===\n\n");
            for suggestion in &suggestions {
                report.push_str(&format!("Element: {}\n", suggestion.element_id));
                report.push_str(&format!("  Canvas: {}\n", suggestion.parent_canvas_id));
                report.push_str(&format!("  Reason: {}\n", suggestion.reason));
                report.push_str(&format!("  Dirty Frequency: {:.1}%\n", suggestion.dirty_frequency * 100.0));
                report.push('\n');
            }
        }

        report
    }

    /// 计算每个元素的合并结果
    fn compute_merge_results(rebuilder: &CanvasRebuilder) -> Vec<(String, BatchMergeResult)> {
        let mut results = Vec::new();
        let elements = &rebuilder.elements;

        for element in elements {
            let element_key = (element.material_id, element.texture_id);
            let same_group_count = elements.iter().filter(|e| (e.material_id, e.texture_id) == element_key).count();

            if same_group_count > 1 {
                let group_sort_orders: Vec<i32> =
                    elements.iter().filter(|e| (e.material_id, e.texture_id) == element_key).map(|e| e.sort_order).collect();

                let min_sort = *group_sort_orders.iter().min().unwrap_or(&element.sort_order);
                let max_sort = *group_sort_orders.iter().max().unwrap_or(&element.sort_order);

                let has_interrupt = elements
                    .iter()
                    .filter(|e| (e.material_id, e.texture_id) != element_key)
                    .any(|other| other.sort_order >= min_sort && other.sort_order <= max_sort);

                if has_interrupt {
                    results.push((element.id.clone(), BatchMergeResult::SortOrderInterrupted));
                }
                else {
                    results.push((element.id.clone(), BatchMergeResult::Merged));
                }
            }
            else {
                let has_same_material_diff_texture =
                    elements.iter().any(|e| e.material_id == element.material_id && e.texture_id != element.texture_id);
                let has_diff_material = elements.iter().any(|e| e.material_id != element.material_id);

                if has_same_material_diff_texture {
                    results.push((element.id.clone(), BatchMergeResult::TextureDifferent));
                }
                else if has_diff_material {
                    results.push((element.id.clone(), BatchMergeResult::MaterialDifferent));
                }
                else {
                    results.push((element.id.clone(), BatchMergeResult::Merged));
                }
            }
        }

        results
    }
}

impl Default for CanvasBatchDebugger {
    fn default() -> Self {
        Self::new()
    }
}
