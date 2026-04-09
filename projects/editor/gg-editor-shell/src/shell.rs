//! 编辑器壳程序

use crate::panel::{EditorPanel, PanelContext, PanelData};
use gg_core::GResult;
use gg_ecs::World;

/// 编辑器壳程序
pub struct EditorShell {
    /// 已注册的面板列表
    panels: Vec<Box<dyn EditorPanel>>,
    /// 面板共享数据
    panel_data: PanelData,
    /// 是否运行中
    is_running: bool,
}

impl EditorShell {
    /// 创建新的编辑器壳程序
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            panel_data: PanelData::new(),
            is_running: false,
        }
    }

    /// 注册面板
    pub fn register_panel(&mut self, panel: Box<dyn EditorPanel>) {
        self.panels.push(panel);
    }

    /// 设置 World
    pub fn set_world(&mut self, world: &mut World) {
        self.panel_data.set_world(world);
    }

    /// 执行一帧
    pub fn tick(&mut self) -> GResult<()> {
        for panel in &mut self.panels {
            if panel.is_visible() {
                let mut context = PanelContext {
                    panel_data: &mut self.panel_data,
                };
                panel.render(&mut context)?;
            }
        }
        Ok(())
    }

    /// 运行主循环
    pub fn run(&mut self) -> GResult<()> {
        self.is_running = true;
        while self.is_running {
            self.tick()?;
        }
        Ok(())
    }

    /// 关闭
    pub fn shutdown(&mut self) {
        self.is_running = false;
    }
}

impl Default for EditorShell {
    fn default() -> Self {
        Self::new()
    }
}
