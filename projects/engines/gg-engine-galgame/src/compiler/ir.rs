//! Galgame 编译器中间表示类型定义

use serde::{Deserialize, Serialize};

/// Galgame 编译器根 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalgameIr {
    /// 前置元数据
    pub front_matter: FrontMatterIr,
    /// 导入列表
    pub imports: Vec<String>,
    /// 对话节点列表
    pub dialogues: Vec<DialogueIr>,
}

/// 前置元数据 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontMatterIr {
    /// 标题
    pub title: Option<String>,
    /// 作者
    pub author: Option<String>,
    /// 版本
    pub version: Option<String>,
    /// 角色定义列表
    pub characters: Vec<CharacterDefIr>,
    /// 初始变量值
    pub variables: std::collections::HashMap<String, VariableValueIr>,
}

impl Default for FrontMatterIr {
    fn default() -> Self {
        Self { title: None, author: None, version: None, characters: Vec::new(), variables: std::collections::HashMap::new() }
    }
}

/// 角色定义 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDefIr {
    /// 角色唯一标识
    pub id: String,
    /// 角色显示名称
    pub name: String,
    /// 默认立绘资源路径
    pub default_portrait_path: Option<String>,
    /// 角色名字颜色（RGBA）
    pub color: Option<[f32; 4]>,
}

/// 变量值 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableValueIr {
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 字符串值
    String(String),
}

/// 对话节点 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueIr {
    /// 节点唯一标识
    pub id: String,
    /// 说话角色 ID
    pub speaker_id: Option<String>,
    /// 对话文本
    pub text: String,
    /// 内联命令列表
    pub commands: Vec<CommandIr>,
    /// 选项列表
    pub choices: Vec<ChoiceIr>,
    /// 无选项时的跳转目标节点 ID
    pub next_node_id: Option<String>,
}

/// 对话内联命令 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandIr {
    /// 切换场景
    ChangeScene {
        /// 场景名称
        name: String,
        /// 背景资源路径
        bg: String,
    },
    /// 显示角色立绘
    ShowCharacter {
        /// 角色名称
        name: String,
        /// 立绘资源标识
        sprite: String,
        /// 显示位置
        position: String,
    },
    /// 播放背景音乐
    PlayBgm {
        /// 音频资源路径
        path: String,
    },
    /// 播放音效
    PlaySe {
        /// 音效资源路径
        path: String,
    },
    /// 淡入效果
    FadeIn {
        /// 持续时间（秒）
        duration: f32,
    },
    /// 淡出效果
    FadeOut {
        /// 持续时间（秒）
        duration: f32,
    },
    /// 设置变量
    SetVariable {
        /// 变量名
        name: String,
        /// 变量值
        value: String,
    },
    /// 调用宿主函数
    CallFunction {
        /// 函数名
        name: String,
        /// 参数列表
        args: Vec<String>,
    },
    /// 跳转到标签
    Goto {
        /// 目标标签名
        target: String,
    },
}

/// 选项 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceIr {
    /// 选项文本
    pub text: String,
    /// 跳转目标节点 ID
    pub next_node_id: String,
    /// 显示条件表达式
    pub condition: Option<String>,
}

/// 条件分支 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionIr {
    /// 条件表达式
    pub condition: String,
    /// 条件为真时的对话列表
    pub true_branch: Vec<DialogueIr>,
    /// 条件为假时的对话列表
    pub false_branch: Vec<DialogueIr>,
}

/// 循环 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopIr {
    /// 循环次数
    pub times: u32,
    /// 循环体对话列表
    pub body: Vec<DialogueIr>,
}

/// 编译后的对话数据库
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueDB {
    /// 所有节点 ID 集合
    pub all_node_ids: std::collections::HashSet<String>,
    /// 文件名到序列的映射
    pub sequences: std::collections::HashMap<String, StorySequence>,
}

impl DialogueDB {
    /// 创建空的对话数据库
    pub fn new() -> Self {
        Self { all_node_ids: std::collections::HashSet::new(), sequences: std::collections::HashMap::new() }
    }
}

impl Default for DialogueDB {
    fn default() -> Self {
        Self::new()
    }
}

/// 有序的对话节点序列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySequence {
    /// 序列中的对话节点列表
    pub nodes: Vec<DialogueIr>,
}
