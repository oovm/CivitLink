//! .script 剧本脚本解析器模块
//! 将 .script 源码解析为 DialogueNode 列表

use gg_core::{GError, GErrorKind, GResult};
use gg_galgame_schema::components::{Choice, DialogueCommand, DialogueNode, PortraitPosition, TransitionType, VariableValue};

/// .script 剧本脚本解析器
///
/// 将 .script 格式的剧本源码解析为结构化的 DialogueNode 列表。
/// 支持 @node 定义节点、[speaker:id] 标记说话者、+ 选项行、
/// [command:args] 内联命令和 -> 跳转指令。
pub struct GscriptParser;

impl GscriptParser {
    /// 解析 .script 源码为 DialogueNode 列表
    ///
    /// 每行以 @node 开头定义新节点，[speaker:id] 标记说话者，
    /// + 开头的行为选项，[command:args] 为内联命令，-> 开头为跳转目标。
    pub fn parse(source: &str) -> GResult<Vec<DialogueNode>> {
        let mut nodes: Vec<DialogueNode> = Vec::new();
        let mut current_node: Option<DialogueNode> = None;

        for (line_num, raw_line) in source.lines().enumerate() {
            let line_num = line_num + 1;
            let line = raw_line.trim();

            if line.is_empty() {
                continue;
            }

            if let Some(node_id) = Self::parse_node_directive(line) {
                if let Some(node) = current_node.take() {
                    nodes.push(node);
                }
                current_node = Some(DialogueNode {
                    id: node_id,
                    speaker_id: None,
                    text: String::new(),
                    commands: Vec::new(),
                    choices: Vec::new(),
                    next_node_id: None,
                });
                continue;
            }

            let node = match current_node.as_mut() {
                Some(n) => n,
                None => {
                    return Err(GError {
                        kind: GErrorKind::Other,
                        message: format!("Line {}: content appears before any @node directive", line_num),
                    });
                }
            };

            if let Some(speaker_id) = Self::parse_speaker(line) {
                node.speaker_id = Some(speaker_id);
                continue;
            }

            if let Some(choice) = Self::parse_choice(line) {
                node.choices.push(choice);
                continue;
            }

            if let Some(command) = Self::parse_command(line) {
                node.commands.push(command);
                continue;
            }

            if let Some(target) = Self::parse_jump(line) {
                node.next_node_id = Some(target);
                continue;
            }

            if node.text.is_empty() {
                node.text = line.to_string();
            }
            else {
                node.text.push('\n');
                node.text.push_str(line);
            }
        }

        if let Some(node) = current_node.take() {
            nodes.push(node);
        }

        Ok(nodes)
    }

    /// 从文件解析 .script 剧本
    pub fn parse_file(path: &std::path::Path) -> GResult<Vec<DialogueNode>> {
        let source = std::fs::read_to_string(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read script file '{}': {}", path.display(), e),
        })?;
        Self::parse(&source)
    }

    /// 解析 @node 指令行
    fn parse_node_directive(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("@node ") {
            let node_id = trimmed["@node ".len()..].trim();
            if !node_id.is_empty() {
                return Some(node_id.to_string());
            }
        }
        None
    }

    /// 解析 [speaker:id] 行
    fn parse_speaker(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if !trimmed.starts_with('[') {
            return None;
        }
        let end = trimmed.find(']')?;
        let inner = &trimmed[1..end];
        if let Some(speaker_id) = inner.strip_prefix("speaker:") {
            let id = speaker_id.trim();
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
        None
    }

    /// 解析 + 选项行
    ///
    /// 格式: `+ 选项文本 -> target_node_id {condition_expression}`
    /// 或: `+ 选项文本 -> target_node_id`
    fn parse_choice(line: &str) -> Option<Choice> {
        let trimmed = line.trim();
        if !trimmed.starts_with('+') {
            return None;
        }
        let content = trimmed[1..].trim();

        let (text, next_node_id, condition) = if let Some(brace_start) = content.rfind('{') {
            if let Some(brace_end) = content.rfind('}') {
                let cond = content[brace_start + 1..brace_end].trim().to_string();
                let before_brace = &content[..brace_start];
                let (t, target) = Self::split_choice_arrow(before_brace)?;
                (t, target, Some(cond))
            }
            else {
                let (t, target) = Self::split_choice_arrow(content)?;
                (t, target, None)
            }
        }
        else {
            let (t, target) = Self::split_choice_arrow(content)?;
            (t, target, None)
        };

        Some(Choice { text: text.to_string(), next_node_id: next_node_id.to_string(), condition })
    }

    /// 在选项内容中按 "->" 分割文本和目标节点
    fn split_choice_arrow(content: &str) -> Option<(&str, &str)> {
        let arrow_pos = content.find("->")?;
        let text = content[..arrow_pos].trim();
        let target = content[arrow_pos + 2..].trim();
        if target.is_empty() {
            return None;
        }
        Some((text, target))
    }

    /// 解析 [command:args] 内联命令
    fn parse_command(line: &str) -> Option<DialogueCommand> {
        let trimmed = line.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return None;
        }
        let inner = &trimmed[1..trimmed.len() - 1];
        let parts: Vec<&str> = inner.splitn(2, ':').collect();
        if parts.len() < 2 {
            return None;
        }
        let cmd_name = parts[0].trim();
        let cmd_args = parts[1].trim();

        match cmd_name {
            "play_bgm" => Self::parse_play_bgm(cmd_args),
            "show_portrait" => Self::parse_show_portrait(cmd_args),
            "hide_portrait" => Self::parse_hide_portrait(cmd_args),
            "change_background" => Self::parse_change_background(cmd_args),
            "set" => Self::parse_set_variable(cmd_args),
            "wait" => Self::parse_wait(cmd_args),
            _ => None,
        }
    }

    /// 解析 play_bgm 命令参数
    fn parse_play_bgm(args: &str) -> Option<DialogueCommand> {
        let asset_path = args.trim().to_string();
        if asset_path.is_empty() {
            return None;
        }
        Some(DialogueCommand::PlayBgm { asset_path, volume: 1.0, fade_in_secs: 0.5 })
    }

    /// 解析 show_portrait 命令参数
    ///
    /// 格式: `character_id:expression:position`
    fn parse_show_portrait(args: &str) -> Option<DialogueCommand> {
        let parts: Vec<&str> = args.split(':').collect();
        if parts.len() < 3 {
            return None;
        }
        let character_id = parts[0].trim().to_string();
        let expression = parts[1].trim().to_string();
        let position = Self::parse_portrait_position(parts[2].trim())?;

        Some(DialogueCommand::ShowPortrait {
            character_id,
            expression,
            position,
            transition: TransitionType::Fade { duration_secs: 0.3 },
        })
    }

    /// 解析 hide_portrait 命令参数
    fn parse_hide_portrait(args: &str) -> Option<DialogueCommand> {
        let character_id = args.trim().to_string();
        if character_id.is_empty() {
            return None;
        }
        Some(DialogueCommand::HidePortrait { character_id, transition: TransitionType::Fade { duration_secs: 0.3 } })
    }

    /// 解析 change_background 命令参数
    fn parse_change_background(args: &str) -> Option<DialogueCommand> {
        let asset_path = args.trim().to_string();
        if asset_path.is_empty() {
            return None;
        }
        Some(DialogueCommand::ChangeBackground { asset_path, transition: TransitionType::CrossDissolve { duration_secs: 0.5 } })
    }

    /// 解析 set 命令参数
    ///
    /// 格式: `variable_name=value`
    fn parse_set_variable(args: &str) -> Option<DialogueCommand> {
        let eq_pos = args.find('=')?;
        let name = args[..eq_pos].trim().to_string();
        let value_str = args[eq_pos + 1..].trim();
        let value = Self::parse_variable_value(value_str)?;
        Some(DialogueCommand::SetVariable { name, value })
    }

    /// 解析 wait 命令参数
    fn parse_wait(args: &str) -> Option<DialogueCommand> {
        let duration_secs = args.trim().parse::<f32>().ok()?;
        Some(DialogueCommand::Wait { duration_secs })
    }

    /// 解析立绘位置字符串
    fn parse_portrait_position(s: &str) -> Option<PortraitPosition> {
        match s.to_lowercase().as_str() {
            "left" => Some(PortraitPosition::Left),
            "center" => Some(PortraitPosition::Center),
            "right" => Some(PortraitPosition::Right),
            _ => {
                let coords: Vec<&str> = s.split(',').collect();
                if coords.len() == 2 {
                    let x = coords[0].trim().parse::<f32>().ok()?;
                    let y = coords[1].trim().parse::<f32>().ok()?;
                    Some(PortraitPosition::Custom { x, y })
                }
                else {
                    None
                }
            }
        }
    }

    /// 解析变量值字符串
    fn parse_variable_value(s: &str) -> Option<VariableValue> {
        if s == "true" {
            return Some(VariableValue::Boolean(true));
        }
        if s == "false" {
            return Some(VariableValue::Boolean(false));
        }
        if let Ok(i) = s.parse::<i64>() {
            return Some(VariableValue::Integer(i));
        }
        if let Ok(f) = s.parse::<f64>() {
            return Some(VariableValue::Float(f));
        }
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            return Some(VariableValue::String(s[1..s.len() - 1].to_string()));
        }
        Some(VariableValue::String(s.to_string()))
    }

    /// 解析 -> 跳转指令
    fn parse_jump(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("-> ") {
            let target = trimmed[3..].trim();
            if !target.is_empty() {
                return Some(target.to_string());
            }
        }
        None
    }
}
