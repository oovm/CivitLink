use std::{collections::{HashMap, HashSet}, path::Path};

// 简化的类型定义，用于测试
enum PortraitPosition {
    Left,
    Center,
    Right,
    Custom { x: f32, y: f32 },
}

enum TransitionType {
    Fade { duration_secs: f32 },
    CrossDissolve { duration_secs: f32 },
}

enum VariableValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

struct Choice {
    text: String,
    next_node_id: String,
    condition: Option<String>,
}

enum DialogueCommand {
    PlayBgm { asset_path: String, volume: f32, fade_in_secs: f32 },
    ShowPortrait {
        character_id: String,
        expression: String,
        position: PortraitPosition,
        transition: TransitionType,
    },
    HidePortrait { character_id: String, transition: TransitionType },
    ChangeBackground { asset_path: String, transition: TransitionType },
    SetVariable { name: String, value: VariableValue },
    Wait { duration_secs: f32 },
}

struct DialogueNode {
    id: String,
    speaker_id: Option<String>,
    text: String,
    commands: Vec<DialogueCommand>,
    choices: Vec<Choice>,
    next_node_id: Option<String>,
}

// 简化的错误类型
type GResult<T> = Result<T, String>;

// 简化的 GscriptParser 实现
struct GscriptParser;

impl GscriptParser {
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
                    return Err(format!("Line {}: content appears before any @node directive", line_num));
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

    pub fn parse_file(path: &Path) -> GResult<Vec<DialogueNode>> {
        let source = std::fs::read_to_string(path).map_err(|e| format!("Failed to read gscript file '{}': {}", path.display(), e))?;
        Self::parse(&source)
    }

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

    fn split_choice_arrow(content: &str) -> Option<(&str, &str)> {
        let arrow_pos = content.find("->")?;
        let text = content[..arrow_pos].trim();
        let target = content[arrow_pos + 2..].trim();
        if target.is_empty() {
            return None;
        }
        Some((text, target))
    }

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

    fn parse_play_bgm(args: &str) -> Option<DialogueCommand> {
        let asset_path = args.trim().to_string();
        if asset_path.is_empty() {
            return None;
        }
        Some(DialogueCommand::PlayBgm { asset_path, volume: 1.0, fade_in_secs: 0.5 })
    }

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

    fn parse_hide_portrait(args: &str) -> Option<DialogueCommand> {
        let character_id = args.trim().to_string();
        if character_id.is_empty() {
            return None;
        }
        Some(DialogueCommand::HidePortrait { character_id, transition: TransitionType::Fade { duration_secs: 0.3 } })
    }

    fn parse_change_background(args: &str) -> Option<DialogueCommand> {
        let asset_path = args.trim().to_string();
        if asset_path.is_empty() {
            return None;
        }
        Some(DialogueCommand::ChangeBackground { asset_path, transition: TransitionType::CrossDissolve { duration_secs: 0.5 } })
    }

    fn parse_set_variable(args: &str) -> Option<DialogueCommand> {
        let eq_pos = args.find('=')?;
        let name = args[..eq_pos].trim().to_string();
        let value_str = args[eq_pos + 1..].trim();
        let value = Self::parse_variable_value(value_str)?;
        Some(DialogueCommand::SetVariable { name, value })
    }

    fn parse_wait(args: &str) -> Option<DialogueCommand> {
        let duration_secs = args.trim().parse::<f32>().ok()?;
        Some(DialogueCommand::Wait { duration_secs })
    }

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

// 简化的 ScriptCompiler 实现
struct StorySequence {
    pub nodes: Vec<DialogueNode>,
    pub node_index: HashMap<String, usize>,
}

struct DialogueDB {
    pub sequences: HashMap<String, StorySequence>,
    pub all_node_ids: HashSet<String>,
}

enum ValidationError {
    MissingNode { source_node: String, target_node: String },
    DuplicateNodeId { id: String },
    EmptyNode { id: String },
    ParseError { line: usize, message: String },
}

struct ScriptCompiler;

impl ScriptCompiler {
    pub fn compile_file(path: &Path) -> GResult<StorySequence> {
        let nodes = GscriptParser::parse_file(path)?;

        let mut node_index = HashMap::new();
        for (idx, node) in nodes.iter().enumerate() {
            node_index.insert(node.id.clone(), idx);
        }

        Ok(StorySequence { nodes, node_index })
    }

    pub fn compile_directory(dir: &Path) -> GResult<DialogueDB> {
        let mut db = DialogueDB { sequences: HashMap::new(), all_node_ids: HashSet::new() };

        let entries = std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;

            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("gscript") {
                let sequence = Self::compile_file(&path)?;

                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();

                for node in &sequence.nodes {
                    db.all_node_ids.insert(node.id.clone());
                }

                db.sequences.insert(file_name, sequence);
            }
        }

        Ok(db)
    }

    pub fn validate(sequence: &StorySequence) -> GResult<Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut seen_ids = HashSet::new();

        for node in &sequence.nodes {
            if seen_ids.contains(&node.id) {
                errors.push(ValidationError::DuplicateNodeId { id: node.id.clone() });
            }
            else {
                seen_ids.insert(node.id.clone());
            }

            if node.text.is_empty() && node.commands.is_empty() && node.choices.is_empty() && node.next_node_id.is_none() {
                errors.push(ValidationError::EmptyNode { id: node.id.clone() });
            }

            if let Some(ref target) = node.next_node_id {
                if !seen_ids.contains(target) && !sequence.node_index.contains_key(target) {
                    errors.push(ValidationError::MissingNode { source_node: node.id.clone(), target_node: target.clone() });
                }
            }

            for choice in &node.choices {
                if !seen_ids.contains(&choice.next_node_id) && !sequence.node_index.contains_key(&choice.next_node_id) {
                    errors.push(ValidationError::MissingNode {
                        source_node: node.id.clone(),
                        target_node: choice.next_node_id.clone(),
                    });
                }
            }
        }

        Ok(errors)
    }

    pub fn validate_db(db: &DialogueDB) -> GResult<Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (_file_name, sequence) in &db.sequences {
            let seq_errors = Self::validate(sequence)?;
            errors.extend(seq_errors);
        }

        for (_file_name, sequence) in &db.sequences {
            for node in &sequence.nodes {
                if let Some(ref target) = node.next_node_id {
                    if !db.all_node_ids.contains(target) {
                        errors.push(ValidationError::MissingNode { source_node: node.id.clone(), target_node: target.clone() });
                    }
                }

                for choice in &node.choices {
                    if !db.all_node_ids.contains(&choice.next_node_id) {
                        errors.push(ValidationError::MissingNode {
                            source_node: node.id.clone(),
                            target_node: choice.next_node_id.clone(),
                        });
                    }
                }
            }
        }

        Ok(errors)
    }
}

fn main() {
    // 测试解析单个文件
    println!("Testing GscriptParser::parse_file...");
    match GscriptParser::parse_file(Path::new("test.gscript")) {
        Ok(nodes) => {
            println!("Successfully parsed {} nodes:", nodes.len());
            for (i, node) in nodes.iter().enumerate() {
                println!("Node {}: {}", i + 1, node.id);
                if let Some(speaker) = &node.speaker_id {
                    println!("  Speaker: {}", speaker);
                }
                if !node.text.is_empty() {
                    println!("  Text: {}", node.text);
                }
                if !node.commands.is_empty() {
                    println!("  Commands: {:?}", node.commands);
                }
                if !node.choices.is_empty() {
                    println!("  Choices: {:?}", node.choices);
                }
                if let Some(next) = &node.next_node_id {
                    println!("  Next: {}", next);
                }
            }
        }
        Err(e) => {
            println!("Error parsing file: {:?}", e);
        }
    }

    // 测试编译单个文件
    println!("\nTesting ScriptCompiler::compile_file...");
    match ScriptCompiler::compile_file(Path::new("test.gscript")) {
        Ok(sequence) => {
            println!("Successfully compiled sequence with {} nodes", sequence.nodes.len());
            println!("Node index: {:?}", sequence.node_index);
        }
        Err(e) => {
            println!("Error compiling file: {:?}", e);
        }
    }

    // 测试验证单个序列
    println!("\nTesting ScriptCompiler::validate...");
    match ScriptCompiler::compile_file(Path::new("test.gscript")) {
        Ok(sequence) => {
            match ScriptCompiler::validate(&sequence) {
                Ok(errors) => {
                    if errors.is_empty() {
                        println!("No validation errors found!");
                    } else {
                        println!("Found {} validation errors:", errors.len());
                        for error in errors {
                            println!("  {:?}", error);
                        }
                    }
                }
                Err(e) => {
                    println!("Error during validation: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error compiling file: {:?}", e);
        }
    }

    // 测试编译目录
    println!("\nTesting ScriptCompiler::compile_directory...");
    match ScriptCompiler::compile_directory(Path::new(".")) {
        Ok(db) => {
            println!("Successfully compiled directory with {} sequences", db.sequences.len());
            for (file_name, sequence) in &db.sequences {
                println!("  {}: {} nodes", file_name, sequence.nodes.len());
            }
        }
        Err(e) => {
            println!("Error compiling directory: {:?}", e);
        }
    }

    // 测试验证数据库
    println!("\nTesting ScriptCompiler::validate_db...");
    match ScriptCompiler::compile_directory(Path::new(".")) {
        Ok(db) => {
            match ScriptCompiler::validate_db(&db) {
                Ok(errors) => {
                    if errors.is_empty() {
                        println!("No validation errors found in database!");
                    } else {
                        println!("Found {} validation errors in database:", errors.len());
                        for error in errors {
                            println!("  {:?}", error);
                        }
                    }
                }
                Err(e) => {
                    println!("Error during database validation: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error compiling directory: {:?}", e);
        }
    }
}
