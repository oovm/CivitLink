use gg_ecs::prelude::*;
use super::super::components::dialogue::*;

pub fn dialogue_system(
    mut query: Query<(&mut Dialogue, Option<&Choice>)>,
    mut scene_query: Query<&mut Scene>,
) {
    for (mut dialogue, choice) in query.iter_mut() {
        // 处理对话显示逻辑
        // 这里应该与 UI 系统集成
        println!("{}: {}", dialogue.speaker, dialogue.text);
        
        if let Some(choice) = choice {
            println!("选择项:");
            for (i, (text, _)) in choice.options.iter().enumerate() {
                println!("{}: {}", i + 1, text);
            }
        }
    }
}

pub fn choice_system(
    mut query: Query<(&mut Choice, &mut Dialogue)>,
) {
    for (mut choice, mut dialogue) in query.iter_mut() {
        // 处理选择逻辑
        // 这里应该与输入系统集成
    }
}

pub fn scene_system(
    query: Query<&Scene>,
) {
    for scene in query.iter() {
        // 处理场景切换逻辑
        println!("当前场景: {}", scene.name);
    }
}