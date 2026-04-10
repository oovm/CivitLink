//! gs 语言解析器
//!
//! 将 GG Shader (gs) 语言源码解析为类型化 AST。
//! 支持完整的 gs 语言语法，包括着色器块、命名空间、
//! using 声明、micro 函数、渲染状态和 uniforms。

use gg_core::{GError, GErrorKind, GResult};

use crate::ast::*;

/// gs 语言解析器
///
/// 将 gs 源码文本解析为 `GslShaderFile` 类型的 AST。
pub struct GslParser;

impl GslParser {
    /// 解析 gs 源码为 AST
    pub fn parse(source: &str) -> GResult<GslShaderFile> {
        let mut shaders = Vec::new();
        let mut namespaces = Vec::new();
        let mut usings = Vec::new();
        let mut micro_functions = Vec::new();

        let chars: Vec<char> = source.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        while pos < len {
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);

            if pos >= len {
                break;
            }

            if Self::match_keyword(&chars, pos, len, "shader") {
                let shader_block = Self::parse_shader_block(&chars, &mut pos, len, source)?;
                shaders.push(shader_block);
            } else if Self::match_keyword(&chars, pos, len, "namespace") {
                let ns = Self::parse_namespace(&chars, &mut pos, len, source)?;
                namespaces.push(ns);
            } else if Self::match_keyword(&chars, pos, len, "using") {
                let using_decl = Self::parse_using(&chars, &mut pos, len, source)?;
                usings.push(using_decl);
            } else if Self::match_keyword(&chars, pos, len, "micro") {
                let func = Self::parse_micro_function(&chars, &mut pos, len, source)?;
                micro_functions.push(func);
            } else {
                pos += 1;
            }
        }

        Ok(GslShaderFile { shaders, namespaces, usings, micro_functions })
    }
}

/// 解析器内部辅助方法
impl GslParser {
    /// 跳过空白字符和注释
    ///
    /// gs 语言使用 `#` 作为注释起始符，`#?` 为文档注释。
    fn skip_whitespace_and_comments(chars: &[char], pos: &mut usize, len: usize) {
        while *pos < len {
            if chars[*pos].is_whitespace() {
                *pos += 1;
            } else if chars[*pos] == '#' {
                while *pos < len && chars[*pos] != '\n' {
                    *pos += 1;
                }
            } else {
                break;
            }
        }
    }

    /// 匹配关键字，确保有词边界
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

    /// 解析标识符
    fn parse_identifier(chars: &[char], pos: &mut usize, len: usize) -> String {
        let mut ident = String::new();
        while *pos < len && (chars[*pos].is_alphanumeric() || chars[*pos] == '_') {
            ident.push(chars[*pos]);
            *pos += 1;
        }
        ident
    }

    /// 解析命名空间路径（含 `::`）
    fn parse_path(chars: &[char], pos: &mut usize, len: usize) -> String {
        let mut path = String::new();
        while *pos < len {
            if chars[*pos].is_alphanumeric() || chars[*pos] == '_' {
                path.push(chars[*pos]);
                *pos += 1;
            } else if *pos + 1 < len && chars[*pos] == ':' && chars[*pos + 1] == ':' {
                path.push_str("::");
                *pos += 2;
            } else {
                break;
            }
        }
        path
    }

    /// 计算源码偏移对应的行号和列号
    fn compute_line_col(source: &str, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, c) in source.char_indices() {
            if i >= offset {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    /// 创建包含位置信息的解析错误
    fn make_error(source: &str, offset: usize, message: &str) -> GError {
        let (line, col) = Self::compute_line_col(source, offset);
        GError {
            kind: GErrorKind::Other,
            message: format!("行 {} 列 {}: {}", line, col, message),
        }
    }

    /// 解析着色器块
    fn parse_shader_block(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<GslShaderBlock> {
        *pos += "shader".len();
        Self::skip_whitespace_and_comments(chars, pos, len);

        let name = Self::parse_identifier(chars, pos, len);
        if name.is_empty() {
            return Err(Self::make_error(source, *pos, "着色器名称不能为空"));
        }
        Self::skip_whitespace_and_comments(chars, pos, len);

        let mut kind = String::new();
        if Self::match_keyword(chars, *pos, len, "by") {
            *pos += "by".len();
            Self::skip_whitespace_and_comments(chars, pos, len);
            kind = Self::parse_identifier(chars, pos, len);
            Self::skip_whitespace_and_comments(chars, pos, len);
        }

        let mut properties = Vec::new();
        let mut render_queue = None;
        let mut render_states = None;
        let mut functions = Vec::new();
        let mut uniforms = Vec::new();
        let mut fallback = None;

        while *pos < len && chars[*pos] != '{' {
            if chars[*pos] == ':' {
                *pos += 1;
                Self::skip_whitespace_and_comments(chars, pos, len);
            }
            if Self::match_keyword(chars, *pos, len, "render_queue") {
                *pos += "render_queue".len();
                Self::skip_whitespace_and_comments(chars, pos, len);
                if *pos < len && chars[*pos] == '=' {
                    *pos += 1;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    render_queue = Some(Self::parse_string_literal(chars, pos, len, source)?);
                }
            } else {
                let prop = Self::parse_property(chars, pos, len, source)?;
                properties.push(prop);
            }
            Self::skip_whitespace_and_comments(chars, pos, len);
        }

        if *pos >= len || chars[*pos] != '{' {
            return Err(Self::make_error(source, *pos, &format!("期望 '{{' 在着色器 '{}' 声明之后", name)));
        }

        let block_content = Self::extract_brace_content(chars, pos, len, source)?;
        let block_chars: Vec<char> = block_content.chars().collect();
        let block_len = block_chars.len();
        let mut block_pos = 0;

        while block_pos < block_len {
            Self::skip_whitespace_and_comments(&block_chars, &mut block_pos, block_len);

            if block_pos >= block_len {
                break;
            }

            if Self::match_keyword(&block_chars, block_pos, block_len, "vertex") {
                if let Some(func) = Self::parse_shader_function(&block_chars, &mut block_pos, block_len, GslFunctionKind::Vertex, source)? {
                    functions.push(func);
                }
            } else if Self::match_keyword(&block_chars, block_pos, block_len, "fragment") {
                if let Some(func) = Self::parse_shader_function(&block_chars, &mut block_pos, block_len, GslFunctionKind::Fragment, source)? {
                    functions.push(func);
                }
            } else if Self::match_keyword(&block_chars, block_pos, block_len, "compute") {
                if let Some(func) = Self::parse_shader_function(&block_chars, &mut block_pos, block_len, GslFunctionKind::Compute, source)? {
                    functions.push(func);
                }
            } else if Self::match_keyword(&block_chars, block_pos, block_len, "render_states") {
                block_pos += "render_states".len();
                Self::skip_whitespace_and_comments(&block_chars, &mut block_pos, block_len);
                if block_pos < block_len && block_chars[block_pos] == '{' {
                    let rs_content = Self::extract_brace_content(&block_chars, &mut block_pos, block_len, source)?;
                    render_states = Some(Self::parse_render_states_content(&rs_content, source)?);
                }
            } else if Self::match_keyword(&block_chars, block_pos, block_len, "uniforms") {
                block_pos += "uniforms".len();
                Self::skip_whitespace_and_comments(&block_chars, &mut block_pos, block_len);
                if block_pos < block_len && block_chars[block_pos] == '{' {
                    let u_content = Self::extract_brace_content(&block_chars, &mut block_pos, block_len, source)?;
                    uniforms = Self::parse_uniforms_content(&u_content, source)?;
                }
            } else if Self::match_keyword(&block_chars, block_pos, block_len, "fallback") {
                block_pos += "fallback".len();
                Self::skip_whitespace_and_comments(&block_chars, &mut block_pos, block_len);
                if block_pos < block_len && block_chars[block_pos] == '{' {
                    let fb_content = Self::extract_brace_content(&block_chars, &mut block_pos, block_len, source)?;
                    fallback = Some(Self::parse_fallback_content(&fb_content, source)?);
                }
            } else {
                block_pos += 1;
            }
        }

        Ok(GslShaderBlock { name, kind, properties, render_queue, render_states, functions, uniforms, fallback })
    }

    /// 解析着色器属性
    fn parse_property(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<GslProperty> {
        let name = Self::parse_identifier(chars, pos, len);
        Self::skip_whitespace_and_comments(chars, pos, len);

        if *pos < len && chars[*pos] == ':' {
            *pos += 1;
            Self::skip_whitespace_and_comments(chars, pos, len);
        }

        let ty = Self::parse_identifier(chars, pos, len);
        Self::skip_whitespace_and_comments(chars, pos, len);

        let mut default_value = None;
        if *pos < len && chars[*pos] == '=' {
            *pos += 1;
            Self::skip_whitespace_and_comments(chars, pos, len);
            default_value = Some(Self::parse_value_literal(chars, pos, len));
        }
        Self::skip_whitespace_and_comments(chars, pos, len);

        let mut range = None;
        if *pos < len && chars[*pos] == '@' {
            let dec = Self::parse_decorator(chars, pos, len, source)?;
            if let GslDecorator::Range(min, max) = dec {
                range = Some((min, max));
            }
        }

        Ok(GslProperty { name, ty, default_value, range })
    }

    /// 解析值字面量（字符串、数字、布尔）
    fn parse_value_literal(chars: &[char], pos: &mut usize, len: usize) -> String {
        if *pos < len && chars[*pos] == '"' {
            let start = *pos;
            *pos += 1;
            while *pos < len && chars[*pos] != '"' {
                if chars[*pos] == '\\' {
                    *pos += 1;
                }
                *pos += 1;
            }
            if *pos < len {
                *pos += 1;
            }
            chars[start..*pos].iter().collect()
        } else {
            let mut val = String::new();
            while *pos < len && !chars[*pos].is_whitespace() && chars[*pos] != '#' && chars[*pos] != '@' {
                val.push(chars[*pos]);
                *pos += 1;
            }
            val
        }
    }

    /// 解析字符串字面量
    fn parse_string_literal(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<String> {
        if *pos >= len || chars[*pos] != '"' {
            return Err(Self::make_error(source, *pos, "期望字符串字面量"));
        }
        *pos += 1;
        let mut result = String::new();
        while *pos < len && chars[*pos] != '"' {
            if chars[*pos] == '\\' && *pos + 1 < len {
                *pos += 1;
                match chars[*pos] {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    c => result.push(c),
                }
            } else {
                result.push(chars[*pos]);
            }
            *pos += 1;
        }
        if *pos < len {
            *pos += 1;
        }
        Ok(result)
    }

    /// 提取花括号内容
    fn extract_brace_content(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<String> {
        if *pos >= len || chars[*pos] != '{' {
            return Err(Self::make_error(source, *pos, "期望 '{'"));
        }

        let start = *pos;
        let mut depth = 1;
        *pos += 1;

        while *pos < len && depth > 0 {
            if chars[*pos] == '{' {
                depth += 1;
            } else if chars[*pos] == '}' {
                depth -= 1;
            } else if chars[*pos] == '"' {
                *pos += 1;
                while *pos < len && chars[*pos] != '"' {
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
            return Err(Self::make_error(source, start, "花括号不匹配"));
        }

        let content: String = chars[start + 1..*pos].iter().collect();
        *pos += 1;
        Ok(content)
    }

    /// 解析着色器函数（vertex/fragment/compute）
    fn parse_shader_function(chars: &[char], pos: &mut usize, len: usize, kind: GslFunctionKind, source: &str) -> GResult<Option<GslFunction>> {
        let keyword = match kind {
            GslFunctionKind::Vertex => "vertex",
            GslFunctionKind::Fragment => "fragment",
            GslFunctionKind::Compute => "compute",
            GslFunctionKind::Micro => "micro",
        };
        *pos += keyword.len();
        Self::skip_whitespace_and_comments(chars, pos, len);

        let params = Self::parse_params(chars, pos, len, source)?;
        Self::skip_whitespace_and_comments(chars, pos, len);

        let mut return_type = None;
        let mut return_decorators = Vec::new();

        while *pos + 1 < len && chars[*pos] == '@' {
            let dec = Self::parse_decorator(chars, pos, len, source)?;
            return_decorators.push(dec);
            Self::skip_whitespace_and_comments(chars, pos, len);
        }

        if *pos + 1 < len && chars[*pos] == '-' && chars[*pos + 1] == '>' {
            *pos += 2;
            Self::skip_whitespace_and_comments(chars, pos, len);

            while *pos < len && chars[*pos] == '@' {
                let dec = Self::parse_decorator(chars, pos, len, source)?;
                return_decorators.push(dec);
                Self::skip_whitespace_and_comments(chars, pos, len);
            }

            return_type = Some(Self::parse_type(chars, pos, len, source)?);
            Self::skip_whitespace_and_comments(chars, pos, len);
        }

        if *pos >= len || chars[*pos] != '{' {
            return Ok(None);
        }

        let body_content = Self::extract_brace_content(chars, pos, len, source)?;
        let body = Self::parse_body(&body_content, source)?;

        let name = match kind {
            GslFunctionKind::Vertex => "vs_main".to_string(),
            GslFunctionKind::Fragment => "fs_main".to_string(),
            GslFunctionKind::Compute => "cs_main".to_string(),
            GslFunctionKind::Micro => "micro_fn".to_string(),
        };

        Ok(Some(GslFunction { kind, name, params, return_type, return_decorators, body }))
    }

    /// 解析函数参数列表
    fn parse_params(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<Vec<GslParam>> {
        let mut params = Vec::new();

        if *pos >= len || chars[*pos] != '(' {
            return Ok(params);
        }
        *pos += 1;

        loop {
            Self::skip_whitespace_and_comments(chars, pos, len);
            if *pos >= len || chars[*pos] == ')' {
                break;
            }

            let mut decorators = Vec::new();
            while *pos < len && chars[*pos] == '@' {
                let dec = Self::parse_decorator(chars, pos, len, source)?;
                decorators.push(dec);
                Self::skip_whitespace_and_comments(chars, pos, len);
            }

            let name = Self::parse_identifier(chars, pos, len);
            Self::skip_whitespace_and_comments(chars, pos, len);

            if *pos < len && chars[*pos] == ':' {
                *pos += 1;
                Self::skip_whitespace_and_comments(chars, pos, len);
            }

            let ty = Self::parse_type(chars, pos, len, source)?;
            Self::skip_whitespace_and_comments(chars, pos, len);

            params.push(GslParam { name, ty, decorators });

            if *pos < len && chars[*pos] == ',' {
                *pos += 1;
            }
        }

        if *pos < len && chars[*pos] == ')' {
            *pos += 1;
        }

        Ok(params)
    }

    /// 解析类型
    fn parse_type(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<GslType> {
        let type_name = Self::parse_identifier(chars, pos, len);
        Self::skip_whitespace_and_comments(chars, pos, len);

        if type_name == "struct" {
            if *pos < len && chars[*pos] == '{' {
                let content = Self::extract_brace_content(chars, pos, len, source)?;
                let fields = Self::parse_struct_fields(&content, source)?;
                return Ok(GslType::Struct { fields });
            }
        }

        let ty = match type_name.as_str() {
            "bool" => GslType::Scalar(GslScalarType::Bool),
            "u32" => GslType::Scalar(GslScalarType::U32),
            "i32" => GslType::Scalar(GslScalarType::I32),
            "f32" => GslType::Scalar(GslScalarType::F32),
            "vec2" => GslType::Vector { size: GslVectorSize::Bi, scalar: GslScalarType::F32 },
            "vec3" => GslType::Vector { size: GslVectorSize::Tri, scalar: GslScalarType::F32 },
            "vec4" | "vec4f" => GslType::Vector { size: GslVectorSize::Quad, scalar: GslScalarType::F32 },
            "mat22" => GslType::Matrix { columns: GslVectorSize::Bi, rows: GslVectorSize::Bi, scalar: GslScalarType::F32 },
            "mat33" => GslType::Matrix { columns: GslVectorSize::Tri, rows: GslVectorSize::Tri, scalar: GslScalarType::F32 },
            "mat44" => GslType::Matrix { columns: GslVectorSize::Quad, rows: GslVectorSize::Quad, scalar: GslScalarType::F32 },
            "tex2" | "texture" => GslType::Texture2D,
            "tex3" => GslType::Texture3D,
            "texcube" => GslType::TextureCube,
            "sampler" => GslType::Sampler,
            "buffer" => {
                Self::skip_whitespace_and_comments(chars, pos, len);
                if *pos < len && chars[*pos] == '<' {
                    *pos += 1;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    let inner = Self::parse_type(chars, pos, len, source)?;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    if *pos < len && chars[*pos] == '>' {
                        *pos += 1;
                    }
                    GslType::Buffer(Box::new(inner))
                } else {
                    GslType::Buffer(Box::new(GslType::Scalar(GslScalarType::U32)))
                }
            }
            _ => {
                return Err(Self::make_error(source, *pos, &format!("未知类型: '{}'", type_name)));
            }
        };

        Ok(ty)
    }

    /// 解析结构体字段
    fn parse_struct_fields(content: &str, source: &str) -> GResult<Vec<GslStructField>> {
        let mut fields = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        while pos < len {
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            if pos >= len {
                break;
            }

            let mut decorators = Vec::new();
            while pos < len && chars[pos] == '@' {
                let dec = Self::parse_decorator(&chars, &mut pos, len, source)?;
                decorators.push(dec);
                Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            }

            let name = Self::parse_identifier(&chars, &mut pos, len);
            if name.is_empty() {
                pos += 1;
                continue;
            }
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);

            if pos < len && chars[pos] == ':' {
                pos += 1;
                Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            }

            let ty = Self::parse_type(&chars, &mut pos, len, source)?;
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);

            fields.push(GslStructField { name, ty, decorators });
        }

        Ok(fields)
    }

    /// 解析装饰器
    fn parse_decorator(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<GslDecorator> {
        if *pos >= len || chars[*pos] != '@' {
            return Err(Self::make_error(source, *pos, "期望装饰器 '@'"));
        }
        *pos += 1;

        let name = Self::parse_identifier(chars, pos, len);
        Self::skip_whitespace_and_comments(chars, pos, len);

        match name.as_str() {
            "location" => {
                if *pos < len && chars[*pos] == '(' {
                    *pos += 1;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    let n = Self::parse_number(chars, pos, len);
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    if *pos < len && chars[*pos] == ')' {
                        *pos += 1;
                    }
                    Ok(GslDecorator::Location(n as u32))
                } else {
                    Ok(GslDecorator::Location(0))
                }
            }
            "builtin" => {
                if *pos < len && chars[*pos] == '(' {
                    *pos += 1;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    let builtin_name = Self::parse_identifier(chars, pos, len);
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    if *pos < len && chars[*pos] == ')' {
                        *pos += 1;
                    }
                    let kind = match builtin_name.as_str() {
                        "position" => GslBuiltinKind::Position,
                        "global_invocation_id" => GslBuiltinKind::GlobalInvocationId,
                        "vertex_index" => GslBuiltinKind::VertexIndex,
                        "instance_index" => GslBuiltinKind::InstanceIndex,
                        _ => GslBuiltinKind::Position,
                    };
                    Ok(GslDecorator::Builtin(kind))
                } else {
                    Ok(GslDecorator::Builtin(GslBuiltinKind::Position))
                }
            }
            "group" => {
                if *pos < len && chars[*pos] == '(' {
                    *pos += 1;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    let n = Self::parse_number(chars, pos, len);
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    if *pos < len && chars[*pos] == ')' {
                        *pos += 1;
                    }
                    Ok(GslDecorator::Group(n as u32))
                } else {
                    Ok(GslDecorator::Group(0))
                }
            }
            "binding" => {
                if *pos < len && chars[*pos] == '(' {
                    *pos += 1;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    let n = Self::parse_number(chars, pos, len);
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    if *pos < len && chars[*pos] == ')' {
                        *pos += 1;
                    }
                    Ok(GslDecorator::Binding(n as u32))
                } else {
                    Ok(GslDecorator::Binding(0))
                }
            }
            "range" => {
                if *pos < len && chars[*pos] == '(' {
                    *pos += 1;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    let min = Self::parse_number(chars, pos, len) as f32;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    if *pos < len && chars[*pos] == ',' {
                        *pos += 1;
                    }
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    let max = Self::parse_number(chars, pos, len) as f32;
                    Self::skip_whitespace_and_comments(chars, pos, len);
                    if *pos < len && chars[*pos] == ')' {
                        *pos += 1;
                    }
                    Ok(GslDecorator::Range(min, max))
                } else {
                    Ok(GslDecorator::Range(0.0, 1.0))
                }
            }
            _ => Err(Self::make_error(source, *pos, &format!("未知装饰器: '@{}'", name))),
        }
    }

    /// 解析数字
    fn parse_number(chars: &[char], pos: &mut usize, len: usize) -> f64 {
        let mut num_str = String::new();
        if *pos < len && chars[*pos] == '-' {
            num_str.push('-');
            *pos += 1;
        }
        while *pos < len && (chars[*pos].is_ascii_digit() || chars[*pos] == '.') {
            num_str.push(chars[*pos]);
            *pos += 1;
        }
        if *pos < len && (chars[*pos] == 'e' || chars[*pos] == 'E') {
            num_str.push(chars[*pos]);
            *pos += 1;
            if *pos < len && (chars[*pos] == '+' || chars[*pos] == '-') {
                num_str.push(chars[*pos]);
                *pos += 1;
            }
            while *pos < len && chars[*pos].is_ascii_digit() {
                num_str.push(chars[*pos]);
                *pos += 1;
            }
        }
        num_str.parse().unwrap_or(0.0)
    }

    /// 解析函数体
    fn parse_body(content: &str, source: &str) -> GResult<Vec<GslStmt>> {
        let mut stmts = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        while pos < len {
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            if pos >= len {
                break;
            }

            if let Some(stmt) = Self::parse_stmt(&chars, &mut pos, len, source)? {
                stmts.push(stmt);
            }
        }

        Ok(stmts)
    }

    /// 解析语句
    fn parse_stmt(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<Option<GslStmt>> {
        Self::skip_whitespace_and_comments(chars, pos, len);
        if *pos >= len {
            return Ok(None);
        }

        if Self::match_keyword(chars, *pos, len, "let") {
            *pos += "let".len();
            Self::skip_whitespace_and_comments(chars, pos, len);

            let is_mut = if Self::match_keyword(chars, *pos, len, "mut") {
                *pos += "mut".len();
                Self::skip_whitespace_and_comments(chars, pos, len);
                true
            } else {
                false
            };

            let name = Self::parse_identifier(chars, pos, len);
            Self::skip_whitespace_and_comments(chars, pos, len);

            let mut ty = None;
            if *pos < len && chars[*pos] == ':' {
                *pos += 1;
                Self::skip_whitespace_and_comments(chars, pos, len);
                ty = Some(Self::parse_type(chars, pos, len, source)?);
                Self::skip_whitespace_and_comments(chars, pos, len);
            }

            let mut value = GslExpr::LiteralInt(0);
            if *pos < len && chars[*pos] == '=' {
                *pos += 1;
                Self::skip_whitespace_and_comments(chars, pos, len);
                value = Self::parse_expr(chars, pos, len, source, 0)?;
            }

            if is_mut {
                Ok(Some(GslStmt::LetMut { name, ty, value }))
            } else {
                Ok(Some(GslStmt::Let { name, ty, value }))
            }
        } else if Self::match_keyword(chars, *pos, len, "return") {
            *pos += "return".len();
            Self::skip_whitespace_and_comments(chars, pos, len);

            let value = if *pos < len && chars[*pos] != '}' && !Self::is_statement_start(chars, *pos, len) {
                Some(Self::parse_expr(chars, pos, len, source, 0)?)
            } else {
                None
            };

            Ok(Some(GslStmt::Return { value }))
        } else if Self::match_keyword(chars, *pos, len, "if") {
            *pos += "if".len();
            Self::skip_whitespace_and_comments(chars, pos, len);

            let condition = Self::parse_expr(chars, pos, len, source, 0)?;
            Self::skip_whitespace_and_comments(chars, pos, len);

            let then_block = if *pos < len && chars[*pos] == '{' {
                let content = Self::extract_brace_content(chars, pos, len, source)?;
                Self::parse_body(&content, source)?
            } else {
                Vec::new()
            };
            Self::skip_whitespace_and_comments(chars, pos, len);

            let else_block = if Self::match_keyword(chars, *pos, len, "else") {
                *pos += "else".len();
                Self::skip_whitespace_and_comments(chars, pos, len);
                if *pos < len && chars[*pos] == '{' {
                    let content = Self::extract_brace_content(chars, pos, len, source)?;
                    Some(Self::parse_body(&content, source)?)
                } else {
                    None
                }
            } else {
                None
            };

            Ok(Some(GslStmt::If { condition, then_block, else_block }))
        } else {
            let expr = Self::parse_expr(chars, pos, len, source, 0)?;
            Self::skip_whitespace_and_comments(chars, pos, len);

            if *pos < len && chars[*pos] == '=' && (*pos + 1 >= len || chars[*pos + 1] != '=') {
                *pos += 1;
                Self::skip_whitespace_and_comments(chars, pos, len);
                let value = Self::parse_expr(chars, pos, len, source, 0)?;
                Ok(Some(GslStmt::Assign { target: expr, value }))
            } else {
                Ok(Some(GslStmt::Expr { expr }))
            }
        }
    }

    /// 检查当前位置是否为语句起始关键字
    fn is_statement_start(chars: &[char], pos: usize, len: usize) -> bool {
        Self::match_keyword(chars, pos, len, "let")
            || Self::match_keyword(chars, pos, len, "return")
            || Self::match_keyword(chars, pos, len, "if")
            || Self::match_keyword(chars, pos, len, "for")
            || Self::match_keyword(chars, pos, len, "while")
    }

    /// 解析表达式（优先级爬升）
    fn parse_expr(chars: &[char], pos: &mut usize, len: usize, source: &str, min_prec: u8) -> GResult<GslExpr> {
        let mut left = Self::parse_unary(chars, pos, len, source)?;

        loop {
            Self::skip_whitespace_and_comments(chars, pos, len);
            if *pos >= len {
                break;
            }

            let (op, prec) = match Self::peek_binary_op(chars, *pos, len) {
                Some(result) => result,
                None => break,
            };

            if prec < min_prec {
                break;
            }

            Self::consume_binary_op(chars, pos, len);
            Self::skip_whitespace_and_comments(chars, pos, len);

            let right = Self::parse_expr(chars, pos, len, source, prec + 1)?;
            left = GslExpr::Binary { left: Box::new(left), op, right: Box::new(right) };
        }

        Ok(left)
    }

    /// 查看二元运算符
    fn peek_binary_op(chars: &[char], pos: usize, len: usize) -> Option<(GslBinaryOp, u8)> {
        if pos + 1 < len {
            let two = [chars[pos], chars[pos + 1]];
            match two {
                ['&', '&'] => return Some((GslBinaryOp::And, 1)),
                ['|', '|'] => return Some((GslBinaryOp::Or, 0)),
                ['=', '='] => return Some((GslBinaryOp::Eq, 3)),
                ['!', '='] => return Some((GslBinaryOp::Ne, 3)),
                ['<', '='] => return Some((GslBinaryOp::Le, 3)),
                ['>', '='] => return Some((GslBinaryOp::Ge, 3)),
                _ => {}
            }
        }
        if pos < len {
            match chars[pos] {
                '<' => return Some((GslBinaryOp::Lt, 3)),
                '>' => return Some((GslBinaryOp::Gt, 3)),
                '+' => return Some((GslBinaryOp::Add, 4)),
                '-' => return Some((GslBinaryOp::Sub, 4)),
                '*' => return Some((GslBinaryOp::Mul, 5)),
                '/' => return Some((GslBinaryOp::Div, 5)),
                '%' => return Some((GslBinaryOp::Mod, 5)),
                _ => {}
            }
        }
        None
    }

    /// 消费二元运算符
    fn consume_binary_op(chars: &[char], pos: &mut usize, len: usize) {
        if *pos + 1 < len {
            let two = [chars[*pos], chars[*pos + 1]];
            match two {
                ['&', '&'] | ['|', '|'] | ['=', '='] | ['!', '='] | ['<', '='] | ['>', '='] => {
                    *pos += 2;
                    return;
                }
                _ => {}
            }
        }
        *pos += 1;
    }

    /// 解析一元表达式
    fn parse_unary(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<GslExpr> {
        Self::skip_whitespace_and_comments(chars, pos, len);

        if *pos < len && chars[*pos] == '-' {
            *pos += 1;
            Self::skip_whitespace_and_comments(chars, pos, len);
            let operand = Self::parse_unary(chars, pos, len, source)?;
            return Ok(GslExpr::Unary { op: GslUnaryOp::Neg, operand: Box::new(operand) });
        }

        if *pos < len && chars[*pos] == '!' {
            *pos += 1;
            Self::skip_whitespace_and_comments(chars, pos, len);
            let operand = Self::parse_unary(chars, pos, len, source)?;
            return Ok(GslExpr::Unary { op: GslUnaryOp::Not, operand: Box::new(operand) });
        }

        Self::parse_postfix(chars, pos, len, source)
    }

    /// 解析后缀表达式
    fn parse_postfix(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<GslExpr> {
        let mut expr = Self::parse_primary(chars, pos, len, source)?;

        loop {
            Self::skip_whitespace_and_comments(chars, pos, len);
            if *pos >= len {
                break;
            }

            if chars[*pos] == '.' {
                *pos += 1;
                Self::skip_whitespace_and_comments(chars, pos, len);
                let member = Self::parse_identifier(chars, pos, len);
                expr = GslExpr::Member { object: Box::new(expr), member };
            } else if chars[*pos] == '[' {
                *pos += 1;
                Self::skip_whitespace_and_comments(chars, pos, len);
                let index = Self::parse_expr(chars, pos, len, source, 0)?;
                Self::skip_whitespace_and_comments(chars, pos, len);
                if *pos < len && chars[*pos] == ']' {
                    *pos += 1;
                }
                expr = GslExpr::Index { object: Box::new(expr), index: Box::new(index) };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// 解析基本表达式
    fn parse_primary(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<GslExpr> {
        Self::skip_whitespace_and_comments(chars, pos, len);

        if *pos >= len {
            return Err(Self::make_error(source, *pos, "意外的表达式结尾"));
        }

        if chars[*pos] == '(' {
            *pos += 1;
            Self::skip_whitespace_and_comments(chars, pos, len);
            let expr = Self::parse_expr(chars, pos, len, source, 0)?;
            Self::skip_whitespace_and_comments(chars, pos, len);
            if *pos < len && chars[*pos] == ')' {
                *pos += 1;
            }
            return Ok(expr);
        }

        if chars[*pos] == '"' {
            let s = Self::parse_string_literal(chars, pos, len, source)?;
            return Ok(GslExpr::Variable(s));
        }

        if Self::match_keyword(chars, *pos, len, "true") {
            *pos += "true".len();
            return Ok(GslExpr::LiteralBool(true));
        }
        if Self::match_keyword(chars, *pos, len, "false") {
            *pos += "false".len();
            return Ok(GslExpr::LiteralBool(false));
        }

        if chars[*pos].is_ascii_digit() || (*pos + 1 < len && chars[*pos] == '-' && chars[*pos + 1].is_ascii_digit()) {
            let n = Self::parse_number(chars, pos, len);
            if n.fract() == 0.0 && n.abs() < i64::MAX as f64 {
                return Ok(GslExpr::LiteralInt(n as i64));
            }
            return Ok(GslExpr::LiteralFloat(n));
        }

        let ident = Self::parse_path(chars, pos, len);
        if ident.is_empty() {
            return Err(Self::make_error(source, *pos, "期望表达式"));
        }
        Self::skip_whitespace_and_comments(chars, pos, len);

        if *pos < len && chars[*pos] == '(' {
            *pos += 1;
            let mut args = Vec::new();
            loop {
                Self::skip_whitespace_and_comments(chars, pos, len);
                if *pos >= len || chars[*pos] == ')' {
                    break;
                }
                let arg = Self::parse_expr(chars, pos, len, source, 0)?;
                args.push(arg);
                Self::skip_whitespace_and_comments(chars, pos, len);
                if *pos < len && chars[*pos] == ',' {
                    *pos += 1;
                }
            }
            if *pos < len && chars[*pos] == ')' {
                *pos += 1;
            }

            let is_vector = matches!(ident.as_str(), "vec2" | "vec3" | "vec4" | "vec4f");
            let is_matrix = matches!(ident.as_str(), "mat22" | "mat33" | "mat44");

            if is_vector {
                let (size, scalar) = match ident.as_str() {
                    "vec2" => (GslVectorSize::Bi, GslScalarType::F32),
                    "vec3" => (GslVectorSize::Tri, GslScalarType::F32),
                    _ => (GslVectorSize::Quad, GslScalarType::F32),
                };
                return Ok(GslExpr::VectorConstruct { scalar, size, args });
            }
            if is_matrix {
                let (columns, rows) = match ident.as_str() {
                    "mat22" => (GslVectorSize::Bi, GslVectorSize::Bi),
                    "mat33" => (GslVectorSize::Tri, GslVectorSize::Tri),
                    _ => (GslVectorSize::Quad, GslVectorSize::Quad),
                };
                return Ok(GslExpr::MatrixConstruct { scalar: GslScalarType::F32, columns, rows, args });
            }

            if ident == "texture_sample" && args.len() >= 2 {
                let coords = if args.len() >= 3 { args.remove(2) } else { GslExpr::LiteralFloat(0.0) };
                let sampler = args.remove(1);
                let texture = args.remove(0);
                return Ok(GslExpr::TextureSample { texture: Box::new(texture), sampler: Box::new(sampler), coords: Box::new(coords) });
            }

            return Ok(GslExpr::Call { func: ident, args });
        }

        Ok(GslExpr::Variable(ident))
    }

    /// 解析 uniforms 内容
    fn parse_uniforms_content(content: &str, source: &str) -> GResult<Vec<GslUniform>> {
        let mut uniforms = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        while pos < len {
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            if pos >= len {
                break;
            }

            let mut decorators = Vec::new();
            while pos < len && chars[pos] == '@' {
                let dec = Self::parse_decorator(&chars, &mut pos, len, source)?;
                decorators.push(dec);
                Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            }

            let name = Self::parse_identifier(&chars, &mut pos, len);
            if name.is_empty() {
                pos += 1;
                continue;
            }
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);

            if pos < len && chars[pos] == ':' {
                pos += 1;
                Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            }

            let ty = Self::parse_type(&chars, &mut pos, len, source)?;
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);

            uniforms.push(GslUniform { name, ty, decorators });
        }

        Ok(uniforms)
    }

    /// 解析渲染状态内容
    fn parse_render_states_content(content: &str, source: &str) -> GResult<GslRenderStates> {
        let mut cull_mode = GslCullMode::None;
        let mut blend_mode = GslBlendMode::Opaque;
        let mut depth_test = false;
        let mut depth_write = false;
        let mut wireframe = false;

        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        while pos < len {
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            if pos >= len {
                break;
            }

            let key = Self::parse_identifier(&chars, &mut pos, len);
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);

            if pos < len && chars[pos] == '=' {
                pos += 1;
                Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            }

            match key.as_str() {
                "cull_mode" => {
                    let val = Self::parse_string_literal(&chars, &mut pos, len, source).unwrap_or_default();
                    cull_mode = match val.as_str() {
                        "back" => GslCullMode::Back,
                        "front" => GslCullMode::Front,
                        _ => GslCullMode::None,
                    };
                }
                "blend_mode" => {
                    let val = Self::parse_string_literal(&chars, &mut pos, len, source).unwrap_or_default();
                    blend_mode = match val.as_str() {
                        "alpha" => GslBlendMode::Alpha,
                        "additive" => GslBlendMode::Additive,
                        _ => GslBlendMode::Opaque,
                    };
                }
                "depth_test" => {
                    depth_test = Self::parse_bool_value(&chars, &mut pos, len);
                }
                "depth_write" => {
                    depth_write = Self::parse_bool_value(&chars, &mut pos, len);
                }
                "wireframe" => {
                    wireframe = Self::parse_bool_value(&chars, &mut pos, len);
                }
                _ => {
                    while pos < len && chars[pos] != '\n' && chars[pos] != '#' {
                        pos += 1;
                    }
                }
            }
        }

        Ok(GslRenderStates { cull_mode, blend_mode, depth_test, depth_write, wireframe })
    }

    /// 解析布尔值
    fn parse_bool_value(chars: &[char], pos: &mut usize, len: usize) -> bool {
        if Self::match_keyword(chars, *pos, len, "true") {
            *pos += "true".len();
            return true;
        }
        if Self::match_keyword(chars, *pos, len, "false") {
            *pos += "false".len();
            return false;
        }
        false
    }

    /// 解析回退策略内容
    fn parse_fallback_content(content: &str, source: &str) -> GResult<GslFallback> {
        let mut when = String::new();
        let mut shader = String::new();

        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        while pos < len {
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            if pos >= len {
                break;
            }

            let key = Self::parse_identifier(&chars, &mut pos, len);
            Self::skip_whitespace_and_comments(&chars, &mut pos, len);

            if pos < len && chars[pos] == ':' {
                pos += 1;
                Self::skip_whitespace_and_comments(&chars, &mut pos, len);
            }

            match key.as_str() {
                "when" => {
                    when = Self::parse_string_literal(&chars, &mut pos, len, source).unwrap_or_default();
                }
                "shader" => {
                    shader = Self::parse_path(&chars, &mut pos, len);
                }
                _ => {
                    while pos < len && chars[pos] != '\n' && chars[pos] != '#' {
                        pos += 1;
                    }
                }
            }
        }

        Ok(GslFallback { when, shader })
    }

    /// 解析命名空间
    fn parse_namespace(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<GslNamespace> {
        *pos += "namespace".len();
        Self::skip_whitespace_and_comments(chars, pos, len);

        let path = Self::parse_path(chars, pos, len);
        Self::skip_whitespace_and_comments(chars, pos, len);

        let mut functions = Vec::new();
        if *pos < len && chars[*pos] == '{' {
            let content = Self::extract_brace_content(chars, pos, len, source)?;
            let inner_chars: Vec<char> = content.chars().collect();
            let inner_len = inner_chars.len();
            let mut inner_pos = 0;

            while inner_pos < inner_len {
                Self::skip_whitespace_and_comments(&inner_chars, &mut inner_pos, inner_len);
                if inner_pos >= inner_len {
                    break;
                }

                if Self::match_keyword(&inner_chars, inner_pos, inner_len, "micro") {
                    if let Some(func) = Self::parse_shader_function(&inner_chars, &mut inner_pos, inner_len, GslFunctionKind::Micro, source)? {
                        functions.push(func);
                    }
                } else if Self::match_keyword(&inner_chars, inner_pos, inner_len, "fn") {
                    if let Some(func) = Self::parse_shader_function(&inner_chars, &mut inner_pos, inner_len, GslFunctionKind::Micro, source)? {
                        functions.push(func);
                    }
                } else {
                    inner_pos += 1;
                }
            }
        }

        Ok(GslNamespace { path, functions })
    }

    /// 解析 using 声明
    fn parse_using(chars: &[char], pos: &mut usize, len: usize, _source: &str) -> GResult<GslUsing> {
        *pos += "using".len();
        Self::skip_whitespace_and_comments(chars, pos, len);

        let path = Self::parse_path(chars, pos, len);
        Self::skip_whitespace_and_comments(chars, pos, len);

        if *pos < len && chars[*pos] == ';' {
            *pos += 1;
        }

        Ok(GslUsing { path })
    }

    /// 解析 micro 函数
    fn parse_micro_function(chars: &[char], pos: &mut usize, len: usize, source: &str) -> GResult<GslFunction> {
        Self::skip_whitespace_and_comments(chars, pos, len);

        match Self::parse_shader_function(chars, pos, len, GslFunctionKind::Micro, source)? {
            Some(func) => Ok(func),
            None => Err(Self::make_error(source, *pos, "无法解析 micro 函数")),
        }
    }
}
