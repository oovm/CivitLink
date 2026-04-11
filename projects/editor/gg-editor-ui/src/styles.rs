//! 样式系统模块
//! 
//! 处理SCSS样式，支持变量、嵌套选择器等特性

/// 样式规则
#[derive(Debug, Clone)]
pub struct StyleRule {
    /// 选择器
    pub selector: String,
    /// 属性列表
    pub properties: Vec<(String, String)>,
}

/// 样式变量
#[derive(Debug, Clone)]
pub struct StyleVariable {
    /// 变量名
    pub name: String,
    /// 变量值
    pub value: String,
}

/// 样式上下文
pub struct StyleContext {
    /// 样式变量
    variables: Vec<StyleVariable>,
    /// 样式规则
    rules: Vec<StyleRule>,
}

impl StyleContext {
    /// 创建新的样式上下文
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            rules: Vec::new(),
        }
    }

    /// 添加样式变量
    pub fn add_variable(&mut self, name: &str, value: &str) {
        self.variables.push(StyleVariable {
            name: name.to_string(),
            value: value.to_string(),
        });
    }

    /// 添加样式规则
    pub fn add_rule(&mut self, selector: &str, properties: Vec<(String, String)>) {
        self.rules.push(StyleRule {
            selector: selector.to_string(),
            properties,
        });
    }

    /// 解析SCSS样式
    pub fn parse_scss(&mut self, scss: &str) {
        // 简单的SCSS解析
        let lines = scss.lines();
        let mut current_selector = String::new();
        let mut current_properties = Vec::new();
        let mut in_block = false;

        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') {
                continue;
            }

            if line.ends_with('{') {
                current_selector = line.trim_end_matches('{').trim().to_string();
                in_block = true;
                current_properties.clear();
            } else if line == '}' {
                if !current_selector.is_empty() && !current_properties.is_empty() {
                    self.add_rule(&current_selector, current_properties.clone());
                }
                in_block = false;
            } else if in_block {
                if let Some((name, value)) = line.split_once(':') {
                    let name = name.trim().to_string();
                    let value = value.trim_end_matches(';').trim().to_string();
                    current_properties.push((name, value));
                }
            } else if line.starts_with('$') {
                if let Some((name, value)) = line.split_once(':') {
                    let name = name.trim().to_string();
                    let value = value.trim_end_matches(';').trim().to_string();
                    self.add_variable(&name, &value);
                }
            }
        }
    }

    /// 生成CSS
    pub fn generate_css(&self) -> String {
        let mut css = String::new();

        // 生成变量
        for var in &self.variables {
            css.push_str(&format!("{}: {};\n", var.name, var.value));
        }

        // 生成规则
        for rule in &self.rules {
            css.push_str(&format!("{} {{\n", rule.selector));
            for (name, value) in &rule.properties {
                css.push_str(&format!("    {}: {};\n", name, value));
            }
            css.push_str("}\n\n");
        }

        css
    }
}
