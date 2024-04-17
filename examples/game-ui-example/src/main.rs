//! 游戏 UI 示例
//! 
//! 展示如何使用游戏 UI 系统创建游戏内界面

use gg_ui::canvas::{Canvas, RectTransform, components::*};
use gg_ecs::World;
use gg_error::GResult;

fn main() -> GResult<()> {
    // 创建世界
    let mut world = World::new();
    
    // 创建 Canvas
    let canvas_entity = world.create_entity();
    world.add_component(canvas_entity, Canvas {
        render_mode: crate::RenderMode::ScreenSpaceOverlay,
        sort_order: 0,
        pixel_perfect: false,
        override_sorting: false,
        sorting_layer_id: 0,
        sorting_order: 0,
    });
    
    // 创建按钮
    let button_entity = world.create_entity();
    world.add_component(button_entity, RectTransform {
        anchor_min: (0.5, 0.5),
        anchor_max: (0.5, 0.5),
        pivot: (0.5, 0.5),
        anchored_position: (0.0, 0.0),
        size_delta: (200.0, 50.0),
        local_scale: (1.0, 1.0, 1.0),
        rotation: crate::Quaternion::identity(),
    });
    world.add_component(button_entity, Image {
        sprite: None,
        color: crate::Color::new(0.0, 0.5, 1.0, 1.0),
        material: None,
        raycast_target: true,
        maskable: true,
    });
    world.add_component(button_entity, Text {
        text: "Click Me".to_string(),
        font: crate::Handle::new(0), // 实际使用时需要加载字体
        font_size: 20.0,
        color: crate::Color::new(1.0, 1.0, 1.0, 1.0),
        alignment: crate::TextAlignment::Center,
        line_spacing: 1.0,
        word_wrap: false,
        raycast_target: false,
    });
    world.add_component(button_entity, Button {
        interactable: true,
        transition: crate::ButtonTransition::ColorTint,
        onclick: Some(Box::new(|world, entity| {
            println!("Button clicked!");
        })),
    });
    
    // 创建文本
    let text_entity = world.create_entity();
    world.add_component(text_entity, RectTransform {
        anchor_min: (0.5, 0.6),
        anchor_max: (0.5, 0.6),
        pivot: (0.5, 0.5),
        anchored_position: (0.0, 0.0),
        size_delta: (300.0, 50.0),
        local_scale: (1.0, 1.0, 1.0),
        rotation: crate::Quaternion::identity(),
    });
    world.add_component(text_entity, Text {
        text: "Welcome to GG Engine!".to_string(),
        font: crate::Handle::new(0), // 实际使用时需要加载字体
        font_size: 24.0,
        color: crate::Color::new(1.0, 1.0, 1.0, 1.0),
        alignment: crate::TextAlignment::Center,
        line_spacing: 1.0,
        word_wrap: false,
        raycast_target: false,
    });
    
    println!("Game UI example started. Press Ctrl+C to exit.");
    
    // 模拟主循环
    let mut frame_count = 0;
    loop {
        // 更新游戏
        world.execute_systems()?;
        
        // 渲染游戏
        // 注意：实际渲染需要实现渲染系统
        // render_game(&world)?;
        
        // 每 60 帧打印一次
        frame_count += 1;
        if frame_count % 60 == 0 {
            println!("Frame: {}", frame_count);
        }
        
        // 模拟帧率
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
