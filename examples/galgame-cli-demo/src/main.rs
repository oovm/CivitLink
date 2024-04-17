//! Galgame 命令行演示
//! 使用 gg-runtime-core 框架和 Galgame 插件演示核心逻辑

use gg_core::{GResult, GError, GErrorKind, plugin::Plugin};
use gg_runtime_core::Runtime;
use gg_galgame_schema::components::*;
use gg_galgame_schema::resources::*;
use gg_plugin_dialogue::plugin::DialoguePlugin;
use gg_plugin_portrait::plugin::PortraitPlugin;
use gg_plugin_scene_transition::plugin::SceneTransitionPlugin;
use gg_plugin_save::plugin::SavePlugin;
use std::sync::Arc;
use std::io::{self, BufRead};

/// 演示入口
fn main() -> GResult<()> {
    println!("========================================");
    println!("  GG Galgame 引擎 - 命令行演示");
    println!("========================================");
    println!();

    // 创建运行时
    let mut runtime = Runtime::new()?;

    // 注册 Galgame 插件
    println!("正在注册插件...");
    runtime.register_plugin(Arc::new(DialoguePlugin))?;
    runtime.register_plugin(Arc::new(PortraitPlugin))?;
    runtime.register_plugin(Arc::new(SceneTransitionPlugin))?;
    runtime.register_plugin(Arc::new(SavePlugin))?;
    println!("插件注册完成！");
    println!();

    // 创建示例对话节点
    println!("正在创建示例剧情...");
    create_sample_dialogue(&mut runtime)?;
    println!("示例剧情创建完成！");
    println!();

    // 开始演示
    println!("剧情演示开始：");
    println!("----------------------------------------");
    println!();

    // 简单的命令行交互循环
    run_interactive_loop(&mut runtime)?;

    println!();
    println!("----------------------------------------");
    println!("演示结束！");
    Ok(())
}

/// 创建示例对话节点
fn create_sample_dialogue(runtime: &mut Runtime) -> GResult<()> {
    let world = runtime.host_mut().world_mut();

    // 节点 1: 开场
    let node1 = world.spawn();
    world.add_component(node1, DialogueNode {
        id: "start".to_string(),
        speaker_id: None,
        text: "这是一个晴朗的早晨，阳光透过窗帘洒进房间...".to_string(),
        commands: vec![
            DialogueCommand::ChangeBackground {
                asset_path: "bg/room.png".to_string(),
                transition: TransitionType::Fade { duration_secs: 1.0 },
            },
        ],
        choices: vec![],
        next_node_id: Some("node2".to_string()),
    })?;

    // 节点 2: 女主角登场
    let node2 = world.spawn();
    world.add_component(node2, DialogueNode {
        id: "node2".to_string(),
        speaker_id: Some("heroine".to_string()),
        text: "早上好！睡得好吗？".to_string(),
        commands: vec![
            DialogueCommand::ShowPortrait {
                character_id: "heroine".to_string(),
                expression: "smile".to_string(),
                position: PortraitPosition::Center,
                transition: TransitionType::Fade { duration_secs: 0.5 },
            },
            DialogueCommand::PlayBgm {
                asset_path: "bgm/morning.mp3".to_string(),
                volume: 0.8,
                fade_in_secs: 1.0,
            },
        ],
        choices: vec![
            Choice {
                text: "早上好！".to_string(),
                next_node_id: "node3_good".to_string(),
                condition: None,
            },
            Choice {
                text: "...你是谁？".to_string(),
                next_node_id: "node3_who".to_string(),
                condition: None,
            },
        ],
        next_node_id: None,
    })?;

    // 节点 3a: 好结局分支
    let node3_good = world.spawn();
    world.add_component(node3_good, DialogueNode {
        id: "node3_good".to_string(),
        speaker_id: Some("heroine".to_string()),
        text: "嗯，早上好！今天天气真好呢，要不要一起去散步？".to_string(),
        commands: vec![
            DialogueCommand::SetVariable {
                name: "affection_heroine".to_string(),
                value: VariableValue::Integer(10),
            },
        ],
        choices: vec![],
        next_node_id: Some("node4".to_string()),
    })?;

    // 节点 3b: 疑问分支
    let node3_who = world.spawn();
    world.add_component(node3_who, DialogueNode {
        id: "node3_who".to_string(),
        speaker_id: Some("heroine".to_string()),
        text: "哎？你不记得我了吗？我是你的青梅竹马啊...".to_string(),
        commands: vec![
            DialogueCommand::ShowPortrait {
                character_id: "heroine".to_string(),
                expression: "sad".to_string(),
                position: PortraitPosition::Center,
                transition: TransitionType::CrossDissolve { duration_secs: 0.3 },
            },
        ],
        choices: vec![],
        next_node_id: Some("node4".to_string()),
    })?;

    // 节点 4: 结尾
    let node4 = world.spawn();
    world.add_component(node4, DialogueNode {
        id: "node4".to_string(),
        speaker_id: None,
        text: "故事才刚刚开始...（演示结束）".to_string(),
        commands: vec![],
        choices: vec![],
        next_node_id: None,
    })?;

    // 创建角色定义
    let character = world.spawn();
    world.add_component(character, CharacterDef {
        id: "heroine".to_string(),
        name: "小樱".to_string(),
        default_portrait_path: Some("portrait/heroine_smile.png".to_string()),
        expression_map: {
            let mut map = std::collections::HashMap::new();
            map.insert("smile".to_string(), "portrait/heroine_smile.png".to_string());
            map.insert("sad".to_string(), "portrait/heroine_sad.png".to_string());
            map
        },
        default_position: PortraitPosition::Center,
        color: Some([1.0, 0.8, 0.8, 1.0]),
    })?;

    // 初始化游戏变量
    let game_vars = GameVariables {
        variables: {
            let mut vars = std::collections::HashMap::new();
            vars.insert("affection_heroine".to_string(), VariableValue::Integer(0));
            vars
        },
    };
    // 注意：在实际项目中，需要将 GameVariables 添加为 ECS 资源或组件

    Ok(())
}

/// 运行交互循环
fn run_interactive_loop(runtime: &mut Runtime) -> GResult<()> {
    let stdin = io::stdin();
    let mut current_node_id = Some("start".to_string());

    while let Some(node_id) = current_node_id.take() {
        // 查找对话节点
        let world = runtime.host_mut().world();
        let mut found_node: Option<&DialogueNode> = None;

        for &entity in world.entities() {
            if let Some(node) = world.get_component::<DialogueNode>(entity) {
                if node.id == node_id {
                    found_node = Some(node);
                    break;
                }
            }
        }

        let Some(node) = found_node else {
            eprintln!("错误：找不到节点 {}", node_id);
            break;
        };

        // 显示说话者
        if let Some(speaker_id) = &node.speaker_id {
            // 查找角色名
            let mut speaker_name = speaker_id.clone();
            for &entity in world.entities() {
                if let Some(char_def) = world.get_component::<CharacterDef>(entity) {
                    if char_def.id == *speaker_id {
                        speaker_name = char_def.name.clone();
                        break;
                    }
                }
            }
            println!("【{}】", speaker_name);
        }

        // 显示对话文本
        println!("{}", node.text);
        println!();

        // 显示命令执行（简化版）
        if !node.commands.is_empty() {
            println!("[执行命令]");
            for cmd in &node.commands {
                match cmd {
                    DialogueCommand::PlayBgm { asset_path, .. } => {
                        println!("  → 播放 BGM: {}", asset_path);
                    }
                    DialogueCommand::ShowPortrait { character_id, expression, position, .. } => {
                        let pos_str = match position {
                            PortraitPosition::Left => "左",
                            PortraitPosition::Center => "中",
                            PortraitPosition::Right => "右",
                            PortraitPosition::Custom { x, y } => &format!("自定义({}, {})", x, y),
                        };
                        println!("  → 显示立绘: {} ({}, {})", character_id, expression, pos_str);
                    }
                    DialogueCommand::ChangeBackground { asset_path, .. } => {
                        println!("  → 切换背景: {}", asset_path);
                    }
                    DialogueCommand::SetVariable { name, value } => {
                        println!("  → 设置变量: {} = {:?}", name, value);
                    }
                    _ => {
                        println!("  → 其他命令: {:?}", cmd);
                    }
                }
            }
            println!();
        }

        // 处理选项
        if !node.choices.is_empty() {
            println!("请选择：");
            for (i, choice) in node.choices.iter().enumerate() {
                println!("  {}. {}", i + 1, choice.text);
            }
            println!();

            // 读取用户输入
            print!("请输入选项编号: ");
            io::Write::flush(&mut io::stdout()).ok();

            let mut line = String::new();
            if let Ok(_) = stdin.lock().read_line(&mut line) {
                if let Ok(choice_idx) = line.trim().parse::<usize>() {
                    if choice_idx >= 1 && choice_idx <= node.choices.len() {
                        let choice = &node.choices[choice_idx - 1];
                        current_node_id = Some(choice.next_node_id.clone());
                        println!();
                        continue;
                    }
                }
            }

            println!("无效输入，退出演示。");
            break;
        }

        // 没有选项，使用 next_node_id
        current_node_id = node.next_node_id.clone();
        if current_node_id.is_some() {
            println!("（按 Enter 继续...）");
            let mut _dummy = String::new();
            stdin.lock().read_line(&mut _dummy).ok();
            println!();
        }
    }

    Ok(())
}
