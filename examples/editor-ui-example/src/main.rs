//! 编辑器 UI 示例
//! 
//! 展示如何使用编辑器 UI 系统创建编辑器界面

use std::path::Path;

use gg_editor_ui::{GuiRuntime, components::*};
use gg_ecs::World;
use gg_error::GResult;

fn main() -> GResult<()> {
    // 创建世界
    let mut world = World::new();
    
    // 创建 GUI 运行时
    let mut gui_runtime = GuiRuntime::new();
    
    // 初始化 GUI 运行时
    gui_runtime.initialize(&mut world)?;
    
    // 加载 .vx 文件
    let ui_entity = gui_runtime.load_vx_file(Path::new("examples/editor-ui-example/ui/main.vx"))?;
    
    println!("Editor UI example started. Press Ctrl+C to exit.");
    
    // 模拟主循环
    let mut frame_count = 0;
    loop {
        // 更新 GUI
        gui_runtime.update(&mut world)?;
        
        // 渲染 GUI
        // 注意：实际渲染需要实现 Renderer 特质
        // gui_runtime.render(&world, &mut renderer)?;
        
        // 处理事件
        // 注意：实际事件处理需要从窗口系统获取事件
        // if let Some(event) = get_event() {
        //     gui_runtime.handle_event(&mut world, &event)?;
        // }
        
        // 每 60 帧打印一次
        frame_count += 1;
        if frame_count % 60 == 0 {
            println!("Frame: {}", frame_count);
        }
        
        // 模拟帧率
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
