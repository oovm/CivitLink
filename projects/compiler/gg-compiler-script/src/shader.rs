//! Shader 文件编译模块
//! 将 .shader 文件解析为结构化数据，并编译 Valkyrie 兼容代码为字节码

use gg_bytecode::BytecodeWriter;
use gg_compiler_core::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::{BuildContext, DiagnosticLevel},
    transformer::Transformer,
};
use gg_core::{GError, GErrorKind, GResult};
use gg_script::ScriptCompiler;

/// Shader 源码产物类型名称
const SHADER_SOURCE_TYPE: &str = "shader_source";
/// 字节码模块产物类型名称
const BYTECODE_MODULE_TYPE: &str = "bytecode_module";
/// GPU 着色器产物类型名称
const GPU_SHADER_TYPE: &str = "gpu_shader";

/// Shader 块中的函数类型
#[derive(Debug, Clone)]
pub enum ShaderFunctionKind {
    /// 顶点着色器
    Vertex,
    /// 片段着色器
    Fragment,
    /// 计算着色器
    Compute,
}

/// Shader 块中的函数定义
#[derive(Debug, Clone)]
pub struct ShaderFunction {
    /// 函数类型
    pub kind: ShaderFunctionKind,
    /// 函数参数文本
    pub params: String,
    /// 返回类型文本
    pub return_type: Option<String>,
    /// 函数体文本
    pub body: String,
}

/// Shader 块
#[derive(Debug, Clone)]
pub struct ShaderBlock {
    /// 着色器名称
    pub name: String,
    /// 着色器类型（如 PBR、Phong、Unlit、Compute）
    pub kind: String,
    /// 着色器属性文本
    pub properties: String,
    /// 渲染状态文本
    pub render_states: Option<String>,
    /// 着色器函数列表
    pub functions: Vec<ShaderFunction>,
    /// Uniforms 定义文本
    pub uniforms: Option<String>,
    /// 回退策略文本
    pub fallback: Option<String>,
}

/// .shader 文件解析结果
#[derive(Debug, Clone)]
pub struct ShaderFile {
    /// Shader 块列表
    pub shaders: Vec<ShaderBlock>,
    /// 独立的 micro 函数源码列表（Valkyrie 兼容代码）
    pub micro_functions: Vec<String>,
    /// 命名空间声明源码列表（Valkyrie 兼容代码）
    pub namespaces: Vec<String>,
    /// using 声明列表
    pub usings: Vec<String>,
}

/// .shader 文件解析器
/// 将 .shader 文件内容解析为 ShaderFile 结构
pub struct ShaderParser;

impl ShaderParser {
    /// 解析 .shader 文件内容为 ShaderFile 结构
    pub fn parse(content: &str) -> GResult<ShaderFile> {
        let mut shaders = Vec::new();
        let mut micro_functions = Vec::new();
        let mut namespaces = Vec::new();
        let mut usings = Vec::new();

        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        while pos < len {
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);

            if pos >= len {
                break;
            }

            if Self::match_keyword(&chars, pos, len, "shader") {
                let shader_block = Self::parse_shader_block(&chars, &mut pos, len)?;
                shaders.push(shader_block);
            }
            else if Self::match_keyword(&chars, pos, len, "micro") {
                let micro_src = Self::parse_brace_block_with_header(&chars, &mut pos, len, "micro")?;
                micro_functions.push(micro_src);
            }
            else if Self::match_keyword(&chars, pos, len, "namespace") {
                let ns_src = Self::parse_brace_block_with_header(&chars, &mut pos, len, "namespace")?;
                namespaces.push(ns_src);
            }
            else if Self::match_keyword(&chars, pos, len, "using") {
                let using_path = Self::parse_using(&chars, &mut pos, len)?;
                usings.push(using_path);
            }
            else {
                pos += 1;
            }
        }

        Ok(ShaderFile { shaders, micro_functions, namespaces, usings })
    }

    fn skip_whitespace_and_comments(chars: &[char], pos: &mut usize, len: usize) {
        while *pos < len {
            if chars[*pos].is_whitespace() {
                *pos += 1;
            }
            else if *pos + 1 < len && chars[*pos] == '/' && chars[*pos + 1] == '/' {
                while *pos < len && chars[*pos] != '\n' {
                    *pos += 1;
                }
            }
            else if *pos + 1 < len && chars[*pos] == '/' && chars[*pos + 1] == '*' {
                *pos += 2;
                while *pos + 1 < len {
                    if chars[*pos] == '*' && chars[*pos + 1] == '/' {
                        *pos += 2;
                        break;
                    }
                    *pos += 1;
                }
            }
            else {
                break;
            }
        }
    }

    fn match_keyword(chars: &[char], pos: usize, len: usize, keyword: &str) -> bool {
        let kw_chars: Vec<char> = keyword.chars().collect();
        let kw_len = kw_chars.len();
        if pos + kw_len > len {
            return false;
        }
        for i in 0..kw_len {
            if chars[pos + i] != kw_chars[i] {
                return false;
            }
        }
        if pos + kw_len < len {
            let next = chars[pos + kw_len];
            if next.is_alphanumeric() || next == '_' {
                return false;
            }
        }
        true
    }

    fn parse_shader_block(chars: &[char], pos: &mut usize, len: usize) -> GResult<ShaderBlock> {
        *pos += "shader".len();
        Self::skip_whitespace_and_comments(chars, pos, len);

        let name = Self::parse_identifier(chars, pos, len);
        Self::skip_whitespace_and_comments(chars, pos, len);

        let mut kind = String::new();
        if Self::match_keyword(chars, *pos, len, "by") {
            *pos += "by".len();
            Self::skip_whitespace_and_comments(chars, pos, len);
            kind = Self::parse_identifier(chars, pos, len);
            Self::skip_whitespace_and_comments(chars, pos, len);
        }

        let properties = String::new();

        if *pos >= len || chars[*pos] != '{' {
            return Err(GError {
                kind: GErrorKind::Other,
                message: format!("Expected '{{' after shader declaration for '{}'", name),
            });
        }

        let block_content = Self::extract_brace_content(chars, pos, len)?;

        let (functions, render_states, uniforms, fallback) = Self::parse_shader_internals(block_content.trim());

        Ok(ShaderBlock { name, kind, properties, render_states, functions, uniforms, fallback })
    }

    fn parse_identifier(chars: &[char], pos: &mut usize, len: usize) -> String {
        let mut ident = String::new();
        while *pos < len && (chars[*pos].is_alphanumeric() || chars[*pos] == '_') {
            ident.push(chars[*pos]);
            *pos += 1;
        }
        ident
    }

    fn extract_brace_content(chars: &[char], pos: &mut usize, len: usize) -> GResult<String> {
        if *pos >= len || chars[*pos] != '{' {
            return Err(GError { kind: GErrorKind::Other, message: "Expected '{' to start block".to_string() });
        }

        let start = *pos;
        let mut depth = 1;
        *pos += 1;

        while *pos < len && depth > 0 {
            if chars[*pos] == '{' {
                depth += 1;
            }
            else if chars[*pos] == '}' {
                depth -= 1;
            }
            else if chars[*pos] == '"' || chars[*pos] == '\'' {
                let quote = chars[*pos];
                *pos += 1;
                while *pos < len && chars[*pos] != quote {
                    if chars[*pos] == '\\' {
                        *pos += 1;
                    }
                    *pos += 1;
                }
            }
            if depth > 0 {
                *pos += 1;
            }
        }

        if depth != 0 {
            return Err(GError { kind: GErrorKind::Other, message: "Unmatched braces in block".to_string() });
        }

        let content: String = chars[start + 1..*pos].iter().collect();
        *pos += 1;

        Ok(content)
    }

    fn parse_brace_block_with_header(chars: &[char], pos: &mut usize, len: usize, keyword: &str) -> GResult<String> {
        let header_start = *pos;
        *pos += keyword.len();

        while *pos < len && chars[*pos] != '{' {
            *pos += 1;
        }

        if *pos >= len {
            return Err(GError { kind: GErrorKind::Other, message: format!("Expected '{{' after {} declaration", keyword) });
        }

        let _content = Self::extract_brace_content(chars, pos, len)?;
        let block_end = *pos;

        let src: String = chars[header_start..block_end].iter().collect();
        Ok(src)
    }

    fn parse_using(chars: &[char], pos: &mut usize, len: usize) -> GResult<String> {
        let start = *pos;
        *pos += "using".len();

        let mut path = String::new();
        while *pos < len && chars[*pos] != ';' {
            if !chars[*pos].is_whitespace() {
                path.push(chars[*pos]);
            }
            *pos += 1;
        }

        if *pos < len {
            *pos += 1;
        }

        let full: String = chars[start..*pos].iter().collect();
        let _ = path;
        Ok(full.trim().to_string())
    }

    fn parse_shader_internals(content: &str) -> (Vec<ShaderFunction>, Option<String>, Option<String>, Option<String>) {
        let mut functions = Vec::new();
        let mut render_states = None;
        let mut uniforms = None;
        let mut fallback = None;

        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        while pos < len {
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);

            if pos >= len {
                break;
            }

            if Self::match_keyword(&chars, pos, len, "vertex") {
                if let Some(func) = Self::parse_shader_function(&chars, &mut pos, len, ShaderFunctionKind::Vertex) {
                    functions.push(func);
                }
            }
            else if Self::match_keyword(&chars, pos, len, "fragment") {
                if let Some(func) = Self::parse_shader_function(&chars, &mut pos, len, ShaderFunctionKind::Fragment) {
                    functions.push(func);
                }
            }
            else if Self::match_keyword(&chars, pos, len, "compute") {
                if let Some(func) = Self::parse_shader_function(&chars, &mut pos, len, ShaderFunctionKind::Compute) {
                    functions.push(func);
                }
            }
            else if Self::match_keyword(&chars, pos, len, "render_states") {
                Self::skip_whitespace_and_comments(&chars, &mut pos, len);
                if pos < len && chars[pos] == '{' {
                    match Self::extract_brace_content(&chars, &mut pos, len) {
                        Ok(rs_content) => render_states = Some(rs_content.trim().to_string()),
                        Err(_) => break,
                    }
                }
            }
            else if Self::match_keyword(&chars, pos, len, "uniforms") {
                Self::skip_whitespace_and_comments(&chars, &mut pos, len);
                if pos < len && chars[pos] == '{' {
                    match Self::extract_brace_content(&chars, &mut pos, len) {
                        Ok(u_content) => uniforms = Some(u_content.trim().to_string()),
                        Err(_) => break,
                    }
                }
            }
            else if Self::match_keyword(&chars, pos, len, "fallback") {
                Self::skip_whitespace_and_comments(&chars, &mut pos, len);
                if pos < len && chars[pos] == '{' {
                    match Self::extract_brace_content(&chars, &mut pos, len) {
                        Ok(fb_content) => fallback = Some(fb_content.trim().to_string()),
                        Err(_) => break,
                    }
                }
            }
            else {
                pos += 1;
            }
        }

        (functions, render_states, uniforms, fallback)
    }

    fn parse_shader_function(chars: &[char], pos: &mut usize, len: usize, kind: ShaderFunctionKind) -> Option<ShaderFunction> {
        let keyword_len = match &kind {
            ShaderFunctionKind::Vertex => "vertex".len(),
            ShaderFunctionKind::Fragment => "fragment".len(),
            ShaderFunctionKind::Compute => "compute".len(),
        };
        *pos += keyword_len;
        Self::skip_whitespace_and_comments(chars, pos, len);

        let mut params = String::new();
        if *pos < len && chars[*pos] == '(' {
            let mut depth = 0;
            let param_start = *pos;
            while *pos < len {
                if chars[*pos] == '(' {
                    depth += 1;
                }
                else if chars[*pos] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        *pos += 1;
                        break;
                    }
                }
                *pos += 1;
            }
            params = chars[param_start + 1..*pos - 1].iter().collect();
            params = params.trim().to_string();
        }

        Self::skip_whitespace_and_comments(chars, pos, len);

        let mut return_type = None;
        if *pos + 1 < len && chars[*pos] == '-' && chars[*pos + 1] == '>' {
            *pos += 2;
            Self::skip_whitespace_and_comments(chars, pos, len);
            let mut rt = String::new();
            while *pos < len
                && (chars[*pos].is_alphanumeric() || chars[*pos] == '_' || chars[*pos] == '<' || chars[*pos] == '>')
            {
                rt.push(chars[*pos]);
                *pos += 1;
            }
            if !rt.is_empty() {
                return_type = Some(rt);
            }
            Self::skip_whitespace_and_comments(chars, pos, len);
        }

        if *pos >= len || chars[*pos] != '{' {
            return None;
        }

        match Self::extract_brace_content(chars, pos, len) {
            Ok(body) => Some(ShaderFunction { kind, params, return_type, body: body.trim().to_string() }),
            Err(_) => None,
        }
    }
}

/// .shader 文件转换器
/// 将 .shader 文件编译为字节码产物和 GPU 着色器描述
pub struct ShaderTransformer {
    /// 是否启用 IR 优化
    pub optimize: bool,
}

impl ShaderTransformer {
    /// 创建新的 Shader 转换器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true }
    }

    /// 创建不启用优化的 Shader 转换器
    pub fn no_optimize() -> Self {
        Self { optimize: false }
    }
}

impl Default for ShaderTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer for ShaderTransformer {
    fn name(&self) -> &str {
        "shader"
    }

    fn input_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(SHADER_SOURCE_TYPE, "*")]
    }

    fn output_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(BYTECODE_MODULE_TYPE, "*"), ArtifactKey::new(GPU_SHADER_TYPE, "*")]
    }

    fn transform(&self, inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet> {
        let mut output = ArtifactSet::new();
        let compiler = if self.optimize { ScriptCompiler::new() } else { ScriptCompiler::no_optimize() };

        for key in inputs.keys() {
            if key.type_name != SHADER_SOURCE_TYPE {
                continue;
            }

            let artifact = match inputs.get(&key) {
                Some(a) => a,
                None => {
                    context.add_diagnostic(
                        DiagnosticLevel::Warning,
                        self.name(),
                        &format!("Artifact not found for key: {}/{}", key.type_name, key.id),
                    );
                    continue;
                }
            };

            let source = match String::from_utf8(artifact.data.clone()) {
                Ok(s) => s,
                Err(e) => {
                    context.add_diagnostic(
                        DiagnosticLevel::Error,
                        self.name(),
                        &format!("Failed to decode source as UTF-8 for '{}': {}", key.id, e),
                    );
                    continue;
                }
            };

            let shader_file = match ShaderParser::parse(&source) {
                Ok(sf) => sf,
                Err(e) => {
                    context.add_diagnostic(
                        DiagnosticLevel::Error,
                        self.name(),
                        &format!("Failed to parse shader file '{}': {}", key.id, e),
                    );
                    continue;
                }
            };

            let mut valkyrie_parts = Vec::new();
            for using in &shader_file.usings {
                valkyrie_parts.push(using.clone());
            }
            for ns in &shader_file.namespaces {
                valkyrie_parts.push(ns.clone());
            }
            for micro in &shader_file.micro_functions {
                valkyrie_parts.push(micro.clone());
            }

            if !valkyrie_parts.is_empty() {
                let valkyrie_source = valkyrie_parts.join("\n");
                let module_name = &key.id;

                match compiler.compile_to_ir(&valkyrie_source, module_name) {
                    Ok(ir_module) => match BytecodeWriter::write(&ir_module) {
                        Ok(bytecode_data) => {
                            let output_key = ArtifactKey::new(BYTECODE_MODULE_TYPE, &key.id);
                            output.insert(Artifact::new(output_key, bytecode_data));
                        }
                        Err(e) => {
                            context.add_diagnostic(
                                DiagnosticLevel::Error,
                                self.name(),
                                &format!("Failed to serialize bytecode for '{}': {}", key.id, e),
                            );
                        }
                    },
                    Err(e) => {
                        context.add_diagnostic(
                            DiagnosticLevel::Error,
                            self.name(),
                            &format!("Failed to compile Valkyrie code in '{}': {}", key.id, e),
                        );
                    }
                }
            }

            let mut gpu_shader_text = String::new();
            for shader in &shader_file.shaders {
                gpu_shader_text.push_str(&format!("shader {} by {} {{\n", shader.name, shader.kind));

                if let Some(ref u) = shader.uniforms {
                    gpu_shader_text.push_str(&format!("  uniforms {{\n{}\n  }}\n", u));
                }

                if let Some(ref rs) = shader.render_states {
                    gpu_shader_text.push_str(&format!("  render_states {{\n{}\n  }}\n", rs));
                }

                for func in &shader.functions {
                    let keyword = match &func.kind {
                        ShaderFunctionKind::Vertex => "vertex",
                        ShaderFunctionKind::Fragment => "fragment",
                        ShaderFunctionKind::Compute => "compute",
                    };
                    let return_part = match &func.return_type {
                        Some(rt) => format!(" -> {}", rt),
                        None => String::new(),
                    };
                    gpu_shader_text
                        .push_str(&format!("  {}({}){} {{\n{}\n  }}\n", keyword, func.params, return_part, func.body));
                }

                if let Some(ref fb) = shader.fallback {
                    gpu_shader_text.push_str(&format!("  fallback {{\n{}\n  }}\n", fb));
                }

                gpu_shader_text.push_str("}\n");
            }

            if !gpu_shader_text.is_empty() {
                let gpu_key = ArtifactKey::new(GPU_SHADER_TYPE, &key.id);
                output.insert(Artifact::new(gpu_key, gpu_shader_text.into_bytes()));
            }
        }

        Ok(output)
    }
}
