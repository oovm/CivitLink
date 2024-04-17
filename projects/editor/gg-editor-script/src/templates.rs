//! 剧本模板生成

use gg_galgame_schema::components::{Choice, DialogueNode};

/// 剧本模板枚举
#[derive(Debug, Clone, Copy)]
pub enum ScriptTemplate {
    /// 日常对话模板
    DailyConversation,
    /// 告白场景模板
    ConfessionScene,
    /// 战斗叙事模板
    BattleNarration,
}

/// 生成日常对话模板节点
pub fn generate_daily_conversation() -> Vec<DialogueNode> {
    vec![
        DialogueNode {
            id: "daily_start".to_string(),
            speaker_id: None,
            text: "又是平凡的一天...".to_string(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: Some("daily_greeting".to_string()),
        },
        DialogueNode {
            id: "daily_greeting".to_string(),
            speaker_id: Some("heroine".to_string()),
            text: "早上好！今天天气真不错呢。".to_string(),
            commands: Vec::new(),
            choices: vec![
                Choice {
                    text: "是啊，要不要一起散步？".to_string(), next_node_id: "daily_walk".to_string(), condition: None
                },
                Choice { text: "嗯...我还有点事".to_string(), next_node_id: "daily_busy".to_string(), condition: None },
            ],
            next_node_id: None,
        },
        DialogueNode {
            id: "daily_walk".to_string(),
            speaker_id: Some("heroine".to_string()),
            text: "太好了！那我们走吧！".to_string(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: None,
        },
        DialogueNode {
            id: "daily_busy".to_string(),
            speaker_id: Some("heroine".to_string()),
            text: "这样啊...那下次吧。".to_string(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: None,
        },
    ]
}

/// 生成告白场景模板节点
pub fn generate_confession_scene() -> Vec<DialogueNode> {
    vec![
        DialogueNode {
            id: "confession_start".to_string(),
            speaker_id: None,
            text: "夕阳染红了天边，空气中弥漫着紧张的气氛...".to_string(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: Some("confession_hesitate".to_string()),
        },
        DialogueNode {
            id: "confession_hesitate".to_string(),
            speaker_id: None,
            text: "心跳加速，话到嘴边却说不出口...".to_string(),
            commands: Vec::new(),
            choices: vec![
                Choice {
                    text: "鼓起勇气告白".to_string(), next_node_id: "confession_accept".to_string(), condition: None
                },
                Choice {
                    text: "还是算了吧...".to_string(), next_node_id: "confession_retreat".to_string(), condition: None
                },
            ],
            next_node_id: None,
        },
        DialogueNode {
            id: "confession_accept".to_string(),
            speaker_id: Some("heroine".to_string()),
            text: "我...我也一直喜欢你！".to_string(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: None,
        },
        DialogueNode {
            id: "confession_retreat".to_string(),
            speaker_id: None,
            text: "话又咽了回去，也许下次...".to_string(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: None,
        },
    ]
}

/// 生成战斗叙事模板节点
pub fn generate_battle_narration() -> Vec<DialogueNode> {
    vec![
        DialogueNode {
            id: "battle_start".to_string(),
            speaker_id: None,
            text: "敌人出现了！战斗一触即发！".to_string(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: Some("battle_action".to_string()),
        },
        DialogueNode {
            id: "battle_action".to_string(),
            speaker_id: None,
            text: "必须做出决断！".to_string(),
            commands: Vec::new(),
            choices: vec![
                Choice { text: "全力进攻".to_string(), next_node_id: "battle_attack".to_string(), condition: None },
                Choice { text: "防守反击".to_string(), next_node_id: "battle_defend".to_string(), condition: None },
            ],
            next_node_id: None,
        },
        DialogueNode {
            id: "battle_attack".to_string(),
            speaker_id: None,
            text: "集中全力的一击！胜负在此一举！".to_string(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: None,
        },
        DialogueNode {
            id: "battle_defend".to_string(),
            speaker_id: None,
            text: "稳住阵脚，等待对手露出破绽...".to_string(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: None,
        },
    ]
}
