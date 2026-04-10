//! GG 对话系统插件核心类型模块
//! 定义对话系统所需的所有 ECS 组件和资源类型

use gg_render::TextureId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 滑动方向枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlideDirection {
    /// 向左
    Left,
    /// 向右
    Right,
    /// 向上
    Up,
    /// 向下
    Down,
}

/// 过渡动画类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionType {
    /// 无动画
    None,
    /// 淡入淡出
    Fade {
        /// 持续时间（秒）
        duration_secs: f32,
    },
    /// 交叉溶解
    CrossDissolve {
        /// 持续时间（秒）
        duration_secs: f32,
    },
    /// 滑动
    Slide {
        /// 持续时间（秒）
        duration_secs: f32,
        /// 滑动方向
        direction: SlideDirection,
    },
}

/// 立绘位置枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortraitPosition {
    /// 左侧
    Left,
    /// 中央
    Center,
    /// 右侧
    Right,
    /// 自定义坐标
    Custom {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
    },
}

/// 变量值枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableValue {
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 字符串
    String(String),
}

/// 选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// 选项文本
    pub text: String,
    /// 跳转目标节点 ID
    pub next_node_id: String,
    /// 显示条件表达式
    pub condition: Option<String>,
}

/// 对话内联命令枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueCommand {
    /// 播放 BGM
    PlayBgm {
        /// 资源路径
        asset_path: String,
        /// 音量
        volume: f32,
        /// 淡入时长（秒）
        fade_in_secs: f32,
    },
    /// 停止 BGM
    StopBgm {
        /// 淡出时长（秒）
        fade_out_secs: f32,
    },
    /// 播放音效
    PlaySe {
        /// 资源路径
        asset_path: String,
        /// 音量
        volume: f32,
    },
    /// 显示立绘
    ShowPortrait {
        /// 角色 ID
        character_id: String,
        /// 表情标签
        expression: String,
        /// 立绘位置
        position: PortraitPosition,
        /// 过渡动画类型
        transition: TransitionType,
    },
    /// 隐藏立绘
    HidePortrait {
        /// 角色 ID
        character_id: String,
        /// 过渡动画类型
        transition: TransitionType,
    },
    /// 切换背景
    ChangeBackground {
        /// 资源路径
        asset_path: String,
        /// 过渡动画类型
        transition: TransitionType,
    },
    /// 设置变量
    SetVariable {
        /// 变量名
        name: String,
        /// 变量值
        value: VariableValue,
    },
    /// 等待
    Wait {
        /// 等待时长（秒）
        duration_secs: f32,
    },
}

/// 对话节点
///
/// 自动实现 `Component` trait（通过 blanket impl）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    /// 节点唯一标识
    pub id: String,
    /// 说话角色 ID（None 表示旁白）
    pub speaker_id: Option<String>,
    /// 对话文本
    pub text: String,
    /// 内联命令列表
    pub commands: Vec<DialogueCommand>,
    /// 选项列表
    pub choices: Vec<Choice>,
    /// 无选项时的跳转目标
    pub next_node_id: Option<String>,
}

/// 角色定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDef {
    /// 角色唯一标识
    pub id: String,
    /// 角色显示名称
    pub name: String,
    /// 默认立绘资源路径
    pub default_portrait_path: Option<String>,
    /// 表情标签到立绘路径的映射
    pub expression_map: HashMap<String, String>,
    /// 默认位置锚点
    pub default_position: PortraitPosition,
    /// 角色名字颜色（RGBA）
    pub color: Option<[f32; 4]>,
}

/// 立绘状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortraitState {
    /// 关联角色 ID
    pub character_id: String,
    /// 当前表情标签
    pub current_expression: String,
    /// 当前位置
    pub position: PortraitPosition,
    /// 缩放比例（默认 1.0）
    pub scale: f32,
    /// 透明度（默认 1.0）
    pub opacity: f32,
    /// 是否正在说话（用于高亮）
    pub is_speaking: bool,
    /// Z 轴排序
    pub z_order: i32,
    /// 立绘纹理标识
    #[serde(skip)]
    pub texture_id: TextureId,
}

/// 氛围滤镜枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmbientFilter {
    /// 变暗
    Darken {
        /// 强度
        intensity: f32,
    },
    /// 暖色调
    Warm {
        /// 强度
        intensity: f32,
    },
    /// 冷色调
    Cool {
        /// 强度
        intensity: f32,
    },
    /// 模糊
    Blur {
        /// 半径
        radius: f32,
    },
    /// 着色
    Tint {
        /// RGBA 颜色
        color: [f32; 4],
    },
}

/// 场景背景
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneBackground {
    /// 背景图资源路径
    pub asset_path: Option<String>,
    /// 转场效果
    pub transition: TransitionType,
    /// 氛围滤镜
    pub ambient_filter: Option<AmbientFilter>,
    /// 背景纹理标识
    #[serde(skip)]
    pub texture_id: TextureId,
}

/// 音效触发器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeTrigger {
    /// 音效资源路径
    pub asset_path: String,
    /// 音量
    pub volume: f32,
    /// 触发时间戳（None 表示立即）
    pub timestamp: Option<f32>,
}

/// 音频控制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioControl {
    /// BGM 资源路径
    pub bgm_path: Option<String>,
    /// BGM 音量（0.0-1.0）
    pub bgm_volume: f32,
    /// BGM 淡入时长（秒）
    pub bgm_fade_in_secs: f32,
    /// BGM 淡出时长（秒）
    pub bgm_fade_out_secs: f32,
    /// 待播放音效列表
    pub pending_se: Vec<SeTrigger>,
}

/// 选项状态
///
/// 作为全局资源存储当前选项列表、选中索引和激活状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceState {
    /// 可用选项列表
    pub choices: Vec<Choice>,
    /// 已选索引
    pub selected_index: Option<usize>,
    /// 选项是否激活
    pub is_active: bool,
}

/// 历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 说话者名称
    pub speaker_name: Option<String>,
    /// 对话文本
    pub text: String,
    /// 时间戳
    pub timestamp: f64,
}

/// 对话历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueHistory {
    /// 历史条目列表
    pub entries: Vec<HistoryEntry>,
    /// 当前对话节点 ID
    pub current_node_id: Option<String>,
}

/// 游戏变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVariables {
    /// 变量映射
    pub variables: HashMap<String, VariableValue>,
}

impl GameVariables {
    /// 获取变量值
    pub fn get_variable(&self, name: &str) -> Option<&VariableValue> {
        self.variables.get(name)
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: VariableValue) {
        self.variables.insert(name, value);
    }

    /// 评估条件表达式
    ///
    /// 支持简单条件格式：`variable_name >= value`、`variable_name <= value`、
    /// `variable_name > value`、`variable_name < value`、`variable_name == value`、
    /// `variable_name != value`
    pub fn evaluate_condition(&self, expression: &str) -> bool {
        let trimmed = expression.trim();

        let (var_name, operator, value_str) = if let Some(idx) = trimmed.find(">=") {
            (&trimmed[..idx], ">=", &trimmed[idx + 2..])
        }
        else if let Some(idx) = trimmed.find("<=") {
            (&trimmed[..idx], "<=", &trimmed[idx + 2..])
        }
        else if let Some(idx) = trimmed.find("!=") {
            (&trimmed[..idx], "!=", &trimmed[idx + 2..])
        }
        else if let Some(idx) = trimmed.find("==") {
            (&trimmed[..idx], "==", &trimmed[idx + 2..])
        }
        else if let Some(idx) = trimmed.find('>') {
            (&trimmed[..idx], ">", &trimmed[idx + 1..])
        }
        else if let Some(idx) = trimmed.find('<') {
            (&trimmed[..idx], "<", &trimmed[idx + 1..])
        }
        else {
            return false;
        };

        let var_name = var_name.trim();
        let value_str = value_str.trim();

        let var_value = match self.variables.get(var_name) {
            Some(v) => v,
            None => return false,
        };

        match operator {
            "==" => Self::compare_equal(var_value, value_str),
            "!=" => !Self::compare_equal(var_value, value_str),
            ">=" => Self::compare_order(var_value, value_str)
                .map_or(false, |o| o == std::cmp::Ordering::Greater || o == std::cmp::Ordering::Equal),
            "<=" => Self::compare_order(var_value, value_str)
                .map_or(false, |o| o == std::cmp::Ordering::Less || o == std::cmp::Ordering::Equal),
            ">" => Self::compare_order(var_value, value_str).map_or(false, |o| o == std::cmp::Ordering::Greater),
            "<" => Self::compare_order(var_value, value_str).map_or(false, |o| o == std::cmp::Ordering::Less),
            _ => false,
        }
    }

    fn compare_equal(var: &VariableValue, value_str: &str) -> bool {
        match var {
            VariableValue::Boolean(b) => value_str.parse::<bool>().map_or(false, |v| *b == v),
            VariableValue::Integer(i) => value_str.parse::<i64>().map_or(false, |v| *i == v),
            VariableValue::Float(f) => value_str.parse::<f64>().map_or(false, |v| (*f - v).abs() < f64::EPSILON),
            VariableValue::String(s) => *s == value_str,
        }
    }

    fn compare_order(var: &VariableValue, value_str: &str) -> Option<std::cmp::Ordering> {
        match var {
            VariableValue::Integer(i) => value_str.parse::<i64>().ok().map(|v| i.cmp(&v)),
            VariableValue::Float(f) => value_str.parse::<f64>().ok().map(|v| f.partial_cmp(&v)).flatten(),
            _ => None,
        }
    }

    /// 使用表达式求值器评估复合条件
    ///
    /// 支持逻辑运算（AND、OR、NOT）和比较运算的复合表达式。
    pub fn evaluate_expression(&self, expression: &str) -> bool {
        crate::expression::ExpressionEvaluator::evaluate(expression, self)
    }
}

/// 帧间隔时间资源
///
/// 存储当前帧与上一帧之间的时间间隔，供系统使用真实时间更新。
#[derive(Debug, Clone, Copy)]
pub struct DeltaTime {
    /// 帧间隔时间（秒）
    pub secs: f32,
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self { secs: 1.0 / 60.0 }
    }
}

/// 等待计时器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitTimer {
    /// 剩余等待时间（秒）
    pub remaining_secs: f32,
}

/// 对话脚本
///
/// 从 JSON 文件加载的对话数据，包含节点、角色和变量定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueScript {
    /// 对话节点列表
    pub nodes: Vec<DialogueNode>,
    /// 角色定义列表
    pub characters: Vec<CharacterDef>,
    /// 初始变量值
    pub variables: HashMap<String, VariableValue>,
}

impl DialogueScript {
    /// 从 JSON 字符串解析对话脚本
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}
