//! Schema 编译器模块
//! 编排 Schema DSL 的完整编译流程：解析 → 验证 → IR 生成 → 绑定生成 → 迁移生成

use crate::{
    codegen::{MigrationFile, SchemaCodegen},
    error::{SchemaError, SchemaResult},
    ir::{
        AnnotationArgIr, AnnotationIr, EnumIr, EnumVariantIr, FieldIr, FieldTypeIr, MessageIr, ModelIr, RpcMethodType,
        SchemaConfigIr, SchemaIr, ServiceIr, ServiceMethodIr,
    },
    validator::SchemaValidator,
};

/// Schema 编译输出
#[derive(Debug)]
pub struct SchemaCompileOutput {
    /// 序列化后的 IR 数据
    pub ir_data: Vec<u8>,
    /// Valkyrie 绑定代码
    pub bindings: String,
    /// 数据库迁移文件列表
    pub migrations: Vec<MigrationFile>,
}

/// Schema 编译器，编排完整的编译流程
pub struct SchemaCompiler {
    /// 是否禁用优化
    optimize: bool,
}

impl SchemaCompiler {
    /// 创建新的 Schema 编译器
    pub fn new() -> Self {
        Self { optimize: true }
    }

    /// 禁用优化
    pub fn no_optimize(mut self) -> Self {
        self.optimize = false;
        self
    }

    /// 编译 Schema 源文件
    pub fn compile(&self, source: &str, module_name: &str) -> SchemaResult<SchemaCompileOutput> {
        let ir = self.parse(source, module_name)?;

        let validator = SchemaValidator::new();
        validator.validate(&ir)?;

        let codegen = SchemaCodegen::new();
        let ir_data = codegen.generate_ir(&ir)?;
        let bindings = codegen.generate_bindings(&ir)?;

        let dialect = ir.schema_configs.first().map(|c| c.dialect.as_str()).unwrap_or("sqlite");
        let migrations = codegen.generate_migrations(&ir, dialect)?;

        Ok(SchemaCompileOutput { ir_data, bindings, migrations })
    }

    /// 解析 Schema DSL 源码，生成 SchemaIr
    pub fn parse(&self, source: &str, module_name: &str) -> SchemaResult<SchemaIr> {
        let lines: Vec<&str> = source.lines().collect();
        let namespace = self.parse_namespace(&lines)?;

        let mut models = Vec::new();
        let mut enums = Vec::new();
        let mut messages = Vec::new();
        let mut services = Vec::new();
        let mut schema_configs = Vec::new();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            if line.starts_with("namespace") || line.starts_with("using") {
                i += 1;
                continue;
            }

            if let Some(rest) = line.strip_prefix("schema") {
                let name = rest.trim().trim_end_matches('{').trim().to_string();
                let (content, end) = self.collect_block(&lines, i)?;
                let configs = self.parse_schema_block(&content)?;
                schema_configs.push(SchemaConfigIr {
                    name: name.clone(),
                    dialect: configs
                        .iter()
                        .find(|c| c.0 == "dialect")
                        .map(|c| c.1.clone())
                        .unwrap_or_else(|| "sqlite".to_string()),
                    connection: configs.iter().find(|c| c.0 == "connection").map(|c| c.1.clone()).unwrap_or_default(),
                    pool_size: configs.iter().find(|c| c.0 == "pool_size").and_then(|c| c.1.parse().ok()),
                });

                let inner_lines: Vec<&str> = content.lines().collect();
                let mut j = 0;
                while j < inner_lines.len() {
                    let inner_line = inner_lines[j].trim();
                    if inner_line.is_empty() || inner_line.starts_with('#') {
                        j += 1;
                        continue;
                    }
                    if inner_line.starts_with("model") {
                        let model_name =
                            inner_line.strip_prefix("model").unwrap().trim().trim_end_matches('{').trim().to_string();
                        let (model_content, model_end) = self.collect_block(&inner_lines, j)?;
                        let model = self.parse_model(&model_name, &model_content)?;
                        models.push(model);
                        j = model_end + 1;
                        continue;
                    }
                    j += 1;
                }

                i = end + 1;
                continue;
            }

            if let Some(rest) = line.strip_prefix("model") {
                let name = rest.trim().trim_end_matches('{').trim().to_string();
                let (content, end) = self.collect_block(&lines, i)?;
                let model = self.parse_model(&name, &content)?;
                models.push(model);
                i = end + 1;
                continue;
            }

            if let Some(rest) = line.strip_prefix("enums") {
                let name = rest.trim().trim_end_matches('{').trim().to_string();
                let (content, end) = self.collect_block(&lines, i)?;
                let enum_ir = self.parse_enum(&name, &content)?;
                enums.push(enum_ir);
                i = end + 1;
                continue;
            }

            if let Some(rest) = line.strip_prefix("message") {
                let name = rest.trim().trim_end_matches('{').trim().to_string();
                let (content, end) = self.collect_block(&lines, i)?;
                let message = self.parse_message(&name, &content)?;
                messages.push(message);
                i = end + 1;
                continue;
            }

            if let Some(rest) = line.strip_prefix("service") {
                let name = rest.trim().trim_end_matches('{').trim().to_string();
                let (content, end) = self.collect_block(&lines, i)?;
                let service = self.parse_service(&name, &content)?;
                services.push(service);
                i = end + 1;
                continue;
            }

            i += 1;
        }

        Ok(SchemaIr {
            namespace: namespace.unwrap_or_else(|| module_name.to_string()),
            models,
            enums,
            messages,
            services,
            schema_configs,
        })
    }

    /// 解析命名空间声明
    pub fn parse_namespace(&self, lines: &[&str]) -> SchemaResult<Option<String>> {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("namespace") {
                let name = rest.trim().trim_end_matches(';').trim().to_string();
                if name.is_empty() {
                    return Err(SchemaError::parse_error_expected(
                        "namespace 名称不能为空",
                        i + 1,
                        trimmed.len() + 1,
                        "标识符",
                    ));
                }
                return Ok(Some(name));
            }
            break;
        }
        Ok(None)
    }

    /// 解析 schema 配置块，返回键值对列表
    pub fn parse_schema_block(&self, content: &str) -> SchemaResult<Vec<(String, String)>> {
        let mut configs = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with("model")
                || trimmed.starts_with("enums")
                || trimmed.starts_with('{')
                || trimmed.starts_with('}')
            {
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let value = trimmed[eq_pos + 1..].trim().trim_end_matches(';').trim().to_string();
                let value = value.trim_matches('"').to_string();
                if !key.is_empty() {
                    configs.push((key, value));
                }
            }
            else if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim().to_string();
                let value = trimmed[colon_pos + 1..].trim().trim_end_matches(';').trim().to_string();
                let value = value.trim_matches('"').to_string();
                if !key.is_empty() {
                    configs.push((key, value));
                }
            }
        }
        Ok(configs)
    }

    /// 解析模型定义
    pub fn parse_model(&self, name: &str, content: &str) -> SchemaResult<ModelIr> {
        let mut fields = Vec::new();
        let mut model_annotations = Vec::new();
        let mut pending_field_annotations: Vec<AnnotationIr> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "{" || trimmed == "}" {
                continue;
            }

            if trimmed.starts_with('@') {
                let anns = self.parse_line_annotations(trimmed)?;
                pending_field_annotations.extend(anns);
                continue;
            }

            if trimmed.contains(':') {
                let mut field = self.parse_field(trimmed)?;
                field.annotations.splice(0..0, pending_field_annotations.drain(..));
                fields.push(field);
                continue;
            }

            if !pending_field_annotations.is_empty() {
                model_annotations.extend(pending_field_annotations.drain(..));
            }
        }

        let table_name = SchemaCodegen::name_to_snake_case(name);
        Ok(ModelIr { name: name.to_string(), table_name, fields, annotations: model_annotations })
    }

    /// 解析枚举定义
    pub fn parse_enum(&self, name: &str, content: &str) -> SchemaResult<EnumIr> {
        let mut variants = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "{" || trimmed == "}" {
                continue;
            }

            if let Some(eq_pos) = trimmed.find('=') {
                let variant_name = trimmed[..eq_pos].trim().to_string();
                let value_str = trimmed[eq_pos + 1..].trim().trim_end_matches(',').trim_end_matches(';').trim();
                let value: i32 = value_str
                    .parse()
                    .map_err(|_| SchemaError::parse_error(format!("枚举变体值 '{value_str}' 不是有效的整数")))?;
                variants.push(EnumVariantIr { name: variant_name, value });
            }
        }

        Ok(EnumIr { name: name.to_string(), variants })
    }

    /// 解析消息定义
    pub fn parse_message(&self, name: &str, content: &str) -> SchemaResult<MessageIr> {
        let mut fields = Vec::new();
        let mut pending_annotations: Vec<AnnotationIr> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "{" || trimmed == "}" {
                continue;
            }

            if trimmed.starts_with('@') {
                let anns = self.parse_line_annotations(trimmed)?;
                pending_annotations.extend(anns);
                continue;
            }

            if trimmed.contains(':') {
                let mut field = self.parse_field(trimmed)?;
                field.annotations.splice(0..0, pending_annotations.drain(..));
                fields.push(field);
            }
        }

        Ok(MessageIr { name: name.to_string(), fields })
    }

    /// 解析服务定义
    pub fn parse_service(&self, name: &str, content: &str) -> SchemaResult<ServiceIr> {
        let mut methods = Vec::new();
        let mut pending_annotations: Vec<AnnotationIr> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "{" || trimmed == "}" {
                continue;
            }

            if trimmed.starts_with('@') {
                let anns = self.parse_line_annotations(trimmed)?;
                pending_annotations.extend(anns);
                continue;
            }

            if let Some(method) = self.parse_service_method(trimmed, &pending_annotations)? {
                methods.push(method);
                pending_annotations.clear();
            }
        }

        Ok(ServiceIr { name: name.to_string(), methods })
    }

    /// 解析单个字段定义
    pub fn parse_field(&self, field_str: &str) -> SchemaResult<FieldIr> {
        let field_str = field_str.trim_end_matches(';').trim();

        let mut annotations = Vec::new();
        let mut remaining = field_str.to_string();

        while let Some(at_pos) = remaining.rfind('@') {
            let after_at = &remaining[at_pos..];
            let ann_str = self.extract_annotation_str(after_at)?;
            let ann = self.parse_annotation(ann_str)?;
            annotations.insert(0, ann);
            remaining = remaining[..at_pos].trim_end().to_string();
        }

        let colon_pos =
            remaining.find(':').ok_or_else(|| SchemaError::parse_error(format!("字段定义缺少冒号: '{field_str}'")))?;

        let field_name = remaining[..colon_pos].trim().to_string();
        let type_part = remaining[colon_pos + 1..].trim();

        let (field_type_str, default_value) = if let Some(eq_pos) = type_part.find('=') {
            let ft = type_part[..eq_pos].trim().to_string();
            let dv = type_part[eq_pos + 1..].trim().to_string();
            (ft, Some(dv))
        }
        else {
            (type_part.to_string(), None)
        };

        let (field_type, is_optional) = self.parse_field_type(&field_type_str)?;

        Ok(FieldIr { name: field_name, field_type, default_value, annotations, is_optional })
    }

    /// 解析注解
    pub fn parse_annotation(&self, ann_str: &str) -> SchemaResult<AnnotationIr> {
        let ann_str = ann_str.trim();
        let ann_str = ann_str.strip_prefix('@').unwrap_or(ann_str);

        if let Some(paren_pos) = ann_str.find('(') {
            let name = ann_str[..paren_pos].trim().to_string();
            let args_str = ann_str[paren_pos + 1..].trim().trim_end_matches(')');
            let arguments = self.parse_annotation_args(args_str)?;
            Ok(AnnotationIr { name, arguments })
        }
        else {
            Ok(AnnotationIr { name: ann_str.trim().to_string(), arguments: Vec::new() })
        }
    }

    fn parse_annotation_args(&self, args_str: &str) -> SchemaResult<Vec<AnnotationArgIr>> {
        let mut args = Vec::new();
        if args_str.trim().is_empty() {
            return Ok(args);
        }

        let mut current = String::new();
        let mut depth = 0i32;
        for ch in args_str.chars() {
            match ch {
                '(' => {
                    depth += 1;
                    current.push(ch);
                }
                ')' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    let arg = self.parse_single_arg(current.trim())?;
                    args.push(arg);
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }
        if !current.trim().is_empty() {
            let arg = self.parse_single_arg(current.trim())?;
            args.push(arg);
        }
        Ok(args)
    }

    fn parse_single_arg(&self, arg_str: &str) -> SchemaResult<AnnotationArgIr> {
        let arg_str = arg_str.trim();
        if let Some(eq_pos) = arg_str.find('=') {
            let name = arg_str[..eq_pos].trim().to_string();
            let value = arg_str[eq_pos + 1..].trim().trim_matches('"').to_string();
            Ok(AnnotationArgIr { name: Some(name), value })
        }
        else {
            let value = arg_str.trim_matches('"').to_string();
            Ok(AnnotationArgIr { name: None, value })
        }
    }

    fn parse_field_type(&self, type_str: &str) -> SchemaResult<(FieldTypeIr, bool)> {
        let type_str = type_str.trim();

        if type_str.ends_with('?') {
            let inner = &type_str[..type_str.len() - 1];
            let (inner_type, _) = self.parse_field_type(inner)?;
            return Ok((FieldTypeIr::Optional(Box::new(inner_type)), true));
        }

        if type_str.starts_with("Option<") && type_str.ends_with('>') {
            let inner = &type_str[7..type_str.len() - 1];
            let (inner_type, _) = self.parse_field_type(inner)?;
            return Ok((FieldTypeIr::Optional(Box::new(inner_type)), true));
        }

        if type_str.starts_with('[') && type_str.ends_with(']') {
            let inner = &type_str[1..type_str.len() - 1];
            if inner.starts_with('&') {
                let ref_inner = &inner[1..];
                let (inner_type, _) = self.parse_field_type(ref_inner)?;
                return Ok((FieldTypeIr::RefArray(Box::new(inner_type)), false));
            }
            let (inner_type, _) = self.parse_field_type(inner)?;
            return Ok((FieldTypeIr::Array(Box::new(inner_type)), false));
        }

        if type_str.starts_with("map<") && type_str.ends_with('>') {
            let inner = &type_str[4..type_str.len() - 1];
            let mut depth = 0i32;
            let mut split_pos = None;
            for (i, ch) in inner.char_indices() {
                match ch {
                    '<' => depth += 1,
                    '>' => depth -= 1,
                    ',' if depth == 0 => {
                        split_pos = Some(i);
                        break;
                    }
                    _ => {}
                }
            }
            if let Some(pos) = split_pos {
                let key_str = inner[..pos].trim();
                let val_str = inner[pos + 1..].trim();
                let (key_type, _) = self.parse_field_type(key_str)?;
                let (val_type, _) = self.parse_field_type(val_str)?;
                return Ok((FieldTypeIr::Map(Box::new(key_type), Box::new(val_type)), false));
            }
            return Err(SchemaError::parse_error(format!("无效的 map 类型: '{type_str}'")));
        }

        if type_str.starts_with("Stream<") && type_str.ends_with('>') {
            return Ok((FieldTypeIr::Custom(type_str.to_string()), false));
        }

        let field_type = match type_str {
            "i8" => FieldTypeIr::I8,
            "i16" => FieldTypeIr::I16,
            "i32" => FieldTypeIr::I32,
            "i64" => FieldTypeIr::I64,
            "u8" => FieldTypeIr::U8,
            "u16" => FieldTypeIr::U16,
            "u32" => FieldTypeIr::U32,
            "u64" => FieldTypeIr::U64,
            "f32" => FieldTypeIr::F32,
            "f64" => FieldTypeIr::F64,
            "bool" => FieldTypeIr::Bool,
            "string" => FieldTypeIr::String,
            "bytes" => FieldTypeIr::Bytes,
            "datetime" => FieldTypeIr::Datetime,
            "uuid" => FieldTypeIr::Uuid,
            other => FieldTypeIr::Custom(other.to_string()),
        };

        Ok((field_type, false))
    }

    fn extract_annotation_str<'a>(&self, s: &'a str) -> SchemaResult<&'a str> {
        let s = s.trim();
        if let Some(paren_pos) = s.find('(') {
            let mut depth = 0i32;
            for (i, ch) in s[paren_pos..].char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(&s[..paren_pos + i + 1]);
                        }
                    }
                    _ => {}
                }
            }
            Err(SchemaError::parse_error(format!("注解括号不匹配: '{s}'")))
        }
        else {
            let end = s.find(|c: char| c.is_whitespace()).unwrap_or(s.len());
            Ok(&s[..end])
        }
    }

    fn parse_line_annotations(&self, line: &str) -> SchemaResult<Vec<AnnotationIr>> {
        let mut annotations = Vec::new();
        let mut remaining = line.trim();

        while remaining.starts_with('@') {
            let ann_str = self.extract_annotation_str(remaining)?;
            let ann = self.parse_annotation(ann_str)?;
            annotations.push(ann);
            remaining = remaining[ann_str.len()..].trim();
        }

        Ok(annotations)
    }

    fn parse_service_method(&self, line: &str, pending_annotations: &[AnnotationIr]) -> SchemaResult<Option<ServiceMethodIr>> {
        let line = line.trim().trim_end_matches(';').trim();

        if line.is_empty() {
            return Ok(None);
        }

        let (method_name, params_and_return) = if let Some(paren_pos) = line.find('(') {
            let name = line[..paren_pos].trim().to_string();
            let rest = &line[paren_pos + 1..];
            (name, rest)
        }
        else {
            return Ok(None);
        };

        let mut request_type = String::new();
        let mut method_type = RpcMethodType::Unary;

        let closing_paren = self.find_closing_paren(params_and_return)?;
        let params_str = &params_and_return[..closing_paren];
        let after_params = params_and_return[closing_paren + 1..].trim();

        let params_str = params_str.trim();
        if params_str.starts_with("Stream<") {
            method_type = RpcMethodType::ClientStream;
            if let Some(end) = params_str.find('>') {
                request_type = params_str[7..end].trim().to_string();
            }
        }
        else if !params_str.is_empty() {
            if let Some(colon_pos) = params_str.find(':') {
                request_type = params_str[colon_pos + 1..].trim().to_string();
            }
            else {
                request_type = params_str.to_string();
            }
        }

        let response_type = if after_params.starts_with("->") || after_params.starts_with(':') {
            let resp = after_params.trim_start_matches("->").trim_start_matches(':').trim();
            if resp.starts_with("Stream<") {
                method_type = match method_type {
                    RpcMethodType::ClientStream => RpcMethodType::Bidirectional,
                    _ => RpcMethodType::ServerStream,
                };
                if let Some(end) = resp.rfind('>') { resp[7..end].trim().to_string() } else { resp.to_string() }
            }
            else {
                resp.to_string()
            }
        }
        else {
            String::new()
        };

        Ok(Some(ServiceMethodIr {
            name: method_name,
            request_type,
            response_type,
            method_type,
            annotations: pending_annotations.to_vec(),
        }))
    }

    fn find_closing_paren(&self, s: &str) -> SchemaResult<usize> {
        let mut depth = 1i32;
        for (i, ch) in s.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(i);
                    }
                }
                _ => {}
            }
        }
        Err(SchemaError::parse_error(format!("未找到匹配的右括号: '{s}'")))
    }

    fn collect_block(&self, lines: &[&str], start: usize) -> SchemaResult<(String, usize)> {
        let mut depth = 0i32;
        let mut content_lines: Vec<String> = Vec::new();
        let mut found_open = false;

        for (i, line) in lines.iter().enumerate().skip(start) {
            let mut line_content = String::new();
            let mut chars = line.chars().peekable();

            while let Some(ch) = chars.next() {
                match ch {
                    '{' => {
                        depth += 1;
                        found_open = true;
                        if depth > 1 {
                            line_content.push(ch);
                        }
                    }
                    '}' => {
                        depth -= 1;
                        if depth == 0 && found_open {
                            if !line_content.trim().is_empty() {
                                content_lines.push(line_content);
                            }
                            let content = content_lines.join("\n");
                            return Ok((content, i));
                        }
                        if depth > 0 {
                            line_content.push(ch);
                        }
                    }
                    _ => {
                        if depth >= 1 {
                            line_content.push(ch);
                        }
                    }
                }
            }

            if depth > 0 {
                let trimmed = line_content.trim_end().to_string();
                if !trimmed.is_empty() {
                    content_lines.push(trimmed);
                }
            }
        }

        Err(SchemaError::parse_error("未找到块结束标记 '}'"))
    }
}

impl Default for SchemaCompiler {
    fn default() -> Self {
        Self::new()
    }
}
