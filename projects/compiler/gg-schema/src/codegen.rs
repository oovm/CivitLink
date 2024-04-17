//! Schema 代码生成器模块
//! 负责将 SchemaIr 生成 IR 序列化数据、Valkyrie 绑定代码和数据库迁移文件

use crate::{
    error::{SchemaError, SchemaResult},
    ir::{FieldTypeIr, SchemaIr},
};
use gg_ir::{IrFunction, IrModule, OpCode, TargetPlatform};
use serde::{Deserialize, Serialize};

/// 数据库迁移文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationFile {
    /// 迁移文件名称
    pub name: String,
    /// 迁移 SQL 内容
    pub content: String,
}

/// 迁移类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationType {
    /// 初始创建表
    CreateTable,
    /// 增量修改表
    AlterTable,
}

/// Schema IR 模块，包含从 SchemaIr 生成的结构化 IR 输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaIrModule {
    /// 模块名称
    pub module_name: String,
    /// 命名空间
    pub namespace: String,
    /// 模型构造函数列表
    pub constructors: Vec<SchemaIrFunction>,
    /// 字段访问器列表
    pub field_accessors: Vec<SchemaIrFunction>,
    /// 服务 RPC 方法列表
    pub rpc_stubs: Vec<SchemaIrFunction>,
}

/// Schema IR 函数描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaIrFunction {
    /// 函数全限定名称
    pub qualified_name: String,
    /// 参数列表
    pub params: Vec<SchemaIrParam>,
    /// 返回类型名称
    pub return_type: String,
}

/// Schema IR 参数描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaIrParam {
    /// 参数名称
    pub name: String,
    /// 参数类型名称
    pub type_name: String,
}

/// Schema 代码生成器
pub struct SchemaCodegen;

impl SchemaCodegen {
    /// 创建新的 Schema 代码生成器
    pub fn new() -> Self {
        Self
    }

    /// 将 PascalCase 名称转换为 snake_case
    pub fn name_to_snake_case(name: &str) -> String {
        let mut result = String::new();
        for (i, ch) in name.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap_or(ch));
            }
            else {
                result.push(ch);
            }
        }
        result
    }

    /// 将 SchemaIr 生成 IrModule 和 SchemaIrModule 的序列化数据
    pub fn generate_ir(&self, ir: &SchemaIr) -> SchemaResult<Vec<u8>> {
        let ir_module = self.build_ir_module(ir);
        let schema_ir_module = self.build_schema_ir_module(ir);

        let mut output = serde_json::to_vec(&schema_ir_module)
            .map_err(|e| SchemaError::SerializeError(format!("序列化 SchemaIrModule 失败: {e}")))?;
        output.push(b'\n');
        let ir_data =
            serde_json::to_vec(&ir_module).map_err(|e| SchemaError::SerializeError(format!("序列化 IrModule 失败: {e}")))?;
        output.extend_from_slice(&ir_data);
        Ok(output)
    }

    fn build_ir_module(&self, ir: &SchemaIr) -> IrModule {
        let mut module = IrModule::new(&ir.namespace);

        for model in &ir.models {
            let ctor_name = format!("{}_new", Self::name_to_snake_case(&model.name));
            let param_count = model.fields.len();

            let mut instructions = Vec::new();
            let field_count = model.fields.len();
            instructions.push(OpCode::NewObject(field_count));

            for (idx, field) in model.fields.iter().enumerate() {
                let field_str_idx = module.add_or_get_string(field.name.clone());
                instructions.push(OpCode::Dup);
                instructions.push(OpCode::LoadLocal(idx));
                instructions.push(OpCode::SetField(field_str_idx));
            }

            instructions.push(OpCode::Return);

            module.add_function(IrFunction {
                name: ctor_name,
                param_count,
                local_count: param_count,
                local_names: vec![],
                instructions,
                is_entry: false,
                target: None,
            });

            for field in &model.fields {
                let accessor_name = format!("{}_get_{}", Self::name_to_snake_case(&model.name), field.name);
                let field_str_idx = module.add_or_get_string(field.name.clone());

                let accessor_instructions = vec![OpCode::LoadLocal(0), OpCode::GetField(field_str_idx), OpCode::Return];

                module.add_function(IrFunction {
                    name: accessor_name,
                    param_count: 1,
                    local_count: 1,
                    local_names: vec![],
                    instructions: accessor_instructions,
                    is_entry: false,
                    target: None,
                });
            }
        }

        for service in &ir.services {
            for method in &service.methods {
                let stub_name = format!("{}_{}", Self::name_to_snake_case(&service.name), method.name);
                let param_count = if method.request_type.is_empty() { 0 } else { 1 };

                let stub_instructions = vec![OpCode::LoadNull, OpCode::Return];

                module.add_function(IrFunction {
                    name: stub_name,
                    param_count,
                    local_count: param_count,
                    local_names: vec![],
                    instructions: stub_instructions,
                    is_entry: false,
                    target: Some(TargetPlatform::Server),
                });
            }
        }

        module
    }

    fn build_schema_ir_module(&self, ir: &SchemaIr) -> SchemaIrModule {
        let mut constructors = Vec::new();
        let mut field_accessors = Vec::new();
        let mut rpc_stubs = Vec::new();

        for model in &ir.models {
            let params: Vec<SchemaIrParam> = model
                .fields
                .iter()
                .map(|f| SchemaIrParam { name: f.name.clone(), type_name: self.field_type_to_ir_name(&f.field_type) })
                .collect();

            constructors.push(SchemaIrFunction {
                qualified_name: format!("{}::{}::new", ir.namespace, model.name),
                params,
                return_type: model.name.clone(),
            });

            for field in &model.fields {
                field_accessors.push(SchemaIrFunction {
                    qualified_name: format!("{}::{}::get_{}", ir.namespace, model.name, field.name),
                    params: vec![SchemaIrParam { name: "self".to_string(), type_name: model.name.clone() }],
                    return_type: self.field_type_to_ir_name(&field.field_type),
                });
            }
        }

        for service in &ir.services {
            for method in &service.methods {
                let mut params = Vec::new();
                if !method.request_type.is_empty() {
                    params.push(SchemaIrParam { name: "request".to_string(), type_name: method.request_type.clone() });
                }
                rpc_stubs.push(SchemaIrFunction {
                    qualified_name: format!("{}::{}::{}", ir.namespace, service.name, method.name),
                    params,
                    return_type: method.response_type.clone(),
                });
            }
        }

        SchemaIrModule {
            module_name: ir.namespace.clone(),
            namespace: ir.namespace.clone(),
            constructors,
            field_accessors,
            rpc_stubs,
        }
    }

    /// 生成 Valkyrie 脚本绑定代码
    pub fn generate_bindings(&self, ir: &SchemaIr) -> SchemaResult<String> {
        let mut output = String::new();

        output.push_str(&format!("using {};\n\n", ir.namespace));

        for enum_def in &ir.enums {
            output.push_str(&format!("enum {} {{\n", enum_def.name));
            for variant in &enum_def.variants {
                output.push_str(&format!("    {} = {},\n", variant.name, variant.value));
            }
            output.push_str("}\n\n");
        }

        for model in &ir.models {
            self.generate_model_binding(&mut output, ir, model);
        }

        for message in &ir.messages {
            self.generate_message_binding(&mut output, ir, message);
        }

        for service in &ir.services {
            self.generate_service_binding(&mut output, ir, service);
        }

        Ok(output)
    }

    fn generate_model_binding(&self, output: &mut String, _ir: &SchemaIr, model: &crate::ir::ModelIr) {
        output.push_str(&format!("class {} {{\n", model.name));

        for field in &model.fields {
            let type_name = self.field_type_to_valkyrie(&field.field_type);
            if field.is_optional {
                output.push_str(&format!("    var {}: {}?;\n", field.name, type_name));
            }
            else {
                output.push_str(&format!("    var {}: {};\n", field.name, type_name));
            }
        }

        output.push_str(&format!("    fn new("));
        let params: Vec<String> = model
            .fields
            .iter()
            .map(|f| {
                let t = self.field_type_to_valkyrie(&f.field_type);
                if f.is_optional { format!("{}: {}?", f.name, t) } else { format!("{}: {}", f.name, t) }
            })
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(") {\n");
        for field in &model.fields {
            output.push_str(&format!("        this.{} = {};\n", field.name, field.name));
        }
        output.push_str("    }\n");

        for field in &model.fields {
            let type_name = self.field_type_to_valkyrie(&field.field_type);
            let ret_type = if field.is_optional { format!("{}?", type_name) } else { type_name };
            output.push_str(&format!("    fn get_{}(self) -> {} {{\n", field.name, ret_type));
            output.push_str(&format!("        return this.{};\n", field.name));
            output.push_str("    }\n");
        }

        output.push_str("    fn to_von(self) -> string {\n");
        output.push_str("        let parts: Array<string> = [];\n");
        for field in &model.fields {
            output.push_str(&format!("        parts.push(\"{}=\" + this.{}.to_string());\n", field.name, field.name));
        }
        output.push_str("        return \"{\" + parts.join(\", \") + \"}\";\n");
        output.push_str("    }\n");

        output.push_str(&format!("    fn from_von(data: string) -> {} {{\n", model.name));
        output.push_str(&format!("        let obj = {}();\n", model.name));
        output.push_str("        return obj;\n");
        output.push_str("    }\n");

        output.push_str("}\n\n");
    }

    fn generate_message_binding(&self, output: &mut String, _ir: &SchemaIr, message: &crate::ir::MessageIr) {
        output.push_str(&format!("class {} {{\n", message.name));

        for field in &message.fields {
            let type_name = self.field_type_to_valkyrie(&field.field_type);
            if field.is_optional {
                output.push_str(&format!("    var {}: {}?;\n", field.name, type_name));
            }
            else {
                output.push_str(&format!("    var {}: {};\n", field.name, type_name));
            }
        }

        output.push_str("    fn new(");
        let params: Vec<String> = message
            .fields
            .iter()
            .map(|f| {
                let t = self.field_type_to_valkyrie(&f.field_type);
                if f.is_optional { format!("{}: {}?", f.name, t) } else { format!("{}: {}", f.name, t) }
            })
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(") {\n");
        for field in &message.fields {
            output.push_str(&format!("        this.{} = {};\n", field.name, field.name));
        }
        output.push_str("    }\n");

        output.push_str("}\n\n");
    }

    fn generate_service_binding(&self, output: &mut String, _ir: &SchemaIr, service: &crate::ir::ServiceIr) {
        output.push_str(&format!("service {} {{\n", service.name));

        for method in &service.methods {
            let req_param =
                if method.request_type.is_empty() { String::new() } else { format!("request: {}", method.request_type) };

            match method.method_type {
                crate::ir::RpcMethodType::Unary => {
                    output.push_str(&format!("    fn {}({}) -> {};\n", method.name, req_param, method.response_type));
                }
                crate::ir::RpcMethodType::ServerStream => {
                    output.push_str(&format!(
                        "    fn {}_stream({}) -> Stream<{}>;\n",
                        method.name, req_param, method.response_type
                    ));
                }
                crate::ir::RpcMethodType::ClientStream => {
                    output.push_str(&format!(
                        "    fn {}(requests: Stream<{}>) -> {};\n",
                        method.name, method.request_type, method.response_type
                    ));
                }
                crate::ir::RpcMethodType::Bidirectional => {
                    output.push_str(&format!(
                        "    fn {}_stream(requests: Stream<{}>) -> Stream<{}>;\n",
                        method.name, method.request_type, method.response_type
                    ));
                }
            }
        }

        output.push_str("}\n\n");
    }

    /// 生成数据库迁移文件
    pub fn generate_migrations(&self, ir: &SchemaIr, dialect: &str) -> SchemaResult<Vec<MigrationFile>> {
        self.generate_migrations_with_previous(ir, dialect, None)
    }

    /// 生成数据库迁移文件，支持增量迁移
    pub fn generate_migrations_with_previous(
        &self,
        ir: &SchemaIr,
        dialect: &str,
        previous_ir: Option<&SchemaIr>,
    ) -> SchemaResult<Vec<MigrationFile>> {
        let mut migrations = Vec::new();

        match previous_ir {
            None => {
                let mut sql = String::new();
                for model in &ir.models {
                    sql.push_str(&self.generate_create_table(model, dialect));
                    sql.push('\n');
                }

                let index_sql = self.generate_indexes(ir, dialect);
                if !index_sql.is_empty() {
                    sql.push('\n');
                    sql.push_str(&index_sql);
                }

                if !sql.is_empty() {
                    let name =
                        ir.schema_configs.first().map(|c| format!("{}_init", c.name)).unwrap_or_else(|| "init".to_string());
                    migrations.push(MigrationFile { name: format!("{name}.sql"), content: sql });
                }
            }
            Some(prev) => {
                let sql = self.generate_alter_statements(ir, prev, dialect);
                if !sql.is_empty() {
                    migrations.push(MigrationFile { name: "alter.sql".to_string(), content: sql });
                }
            }
        }

        Ok(migrations)
    }

    fn generate_create_table(&self, model: &crate::ir::ModelIr, dialect: &str) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", model.table_name);
        let mut column_defs = Vec::new();
        let mut primary_key_columns = Vec::new();
        let mut foreign_keys = Vec::new();

        for field in &model.fields {
            let col_type = self.field_type_to_sql(&field.field_type, dialect);
            let mut col_def = format!("    {} {}", field.name, col_type);

            if field.annotations.iter().any(|a| a.name == "primary_key") {
                primary_key_columns.push(field.name.clone());
                if dialect == "sqlite" || dialect == "postgresql" {
                    col_def.push_str(" NOT NULL");
                }
                if field.annotations.iter().any(|a| a.name == "auto_generate") {
                    match dialect {
                        "sqlite" => col_def.push_str(" PRIMARY KEY AUTOINCREMENT"),
                        "postgresql" => col_def.push_str(" GENERATED ALWAYS AS IDENTITY"),
                        "mysql" => col_def.push_str(" AUTO_INCREMENT"),
                        _ => {}
                    }
                }
            }

            if !field.is_optional
                && !field.annotations.iter().any(|a| a.name == "primary_key")
                && !field.annotations.iter().any(|a| a.name == "auto_generate")
            {
                col_def.push_str(" NOT NULL");
            }

            if let Some(ref default) = field.default_value {
                col_def.push_str(&format!(" DEFAULT {}", self.format_default_value(default, dialect)));
            }

            if field.annotations.iter().any(|a| a.name == "unique")
                && !field.annotations.iter().any(|a| a.name == "primary_key")
            {
                col_def.push_str(" UNIQUE");
            }

            column_defs.push(col_def);

            for annotation in &field.annotations {
                if annotation.name == "references" {
                    if let Some(arg) = annotation.arguments.first() {
                        let ref_target = &arg.value;
                        let parts: Vec<&str> = ref_target.split('.').collect();
                        if parts.len() == 2 {
                            let ref_table = Self::name_to_snake_case(parts[0]);
                            let ref_column = parts[1];
                            let on_delete = field
                                .annotations
                                .iter()
                                .find(|a| a.name == "on_delete")
                                .and_then(|a| a.arguments.first())
                                .map(|a| a.value.to_uppercase())
                                .unwrap_or_else(|| "RESTRICT".to_string());
                            foreign_keys.push(format!(
                                "    FOREIGN KEY ({}) REFERENCES {}({}) ON DELETE {}",
                                field.name, ref_table, ref_column, on_delete
                            ));
                        }
                    }
                }
            }
        }

        if !primary_key_columns.is_empty()
            && !model.fields.iter().any(|f| f.annotations.iter().any(|a| a.name == "auto_generate"))
        {
            column_defs.push(format!("    PRIMARY KEY ({})", primary_key_columns.join(", ")));
        }

        for fk in &foreign_keys {
            column_defs.push(fk.clone());
        }

        sql.push_str(&column_defs.join(",\n"));
        sql.push_str("\n);");
        sql
    }

    fn generate_indexes(&self, ir: &SchemaIr, _dialect: &str) -> String {
        let mut sql = String::new();

        for model in &ir.models {
            for field in &model.fields {
                for annotation in &field.annotations {
                    match annotation.name.as_str() {
                        "index" => {
                            let idx_name = format!("idx_{}_{}", model.table_name, field.name);
                            sql.push_str(&format!("CREATE INDEX {} ON {} ({});\n", idx_name, model.table_name, field.name));
                        }
                        "unique" => {
                            if !field.annotations.iter().any(|a| a.name == "primary_key") {
                                let idx_name = format!("idx_{}_{}", model.table_name, field.name);
                                sql.push_str(&format!(
                                    "CREATE UNIQUE INDEX {} ON {} ({});\n",
                                    idx_name, model.table_name, field.name
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        sql
    }

    fn generate_alter_statements(&self, ir: &SchemaIr, previous_ir: &SchemaIr, dialect: &str) -> String {
        let mut sql = String::new();

        for model in &ir.models {
            let prev_model = previous_ir.models.iter().find(|m| m.name == model.name);

            match prev_model {
                None => {
                    sql.push_str(&self.generate_create_table(model, dialect));
                    sql.push('\n');
                }
                Some(prev) => {
                    let alter_sql = self.generate_alter_table(model, prev, dialect);
                    if !alter_sql.is_empty() {
                        sql.push_str(&alter_sql);
                        sql.push('\n');
                    }
                }
            }
        }

        sql
    }

    fn generate_alter_table(&self, model: &crate::ir::ModelIr, prev_model: &crate::ir::ModelIr, dialect: &str) -> String {
        let mut sql = String::new();

        for field in &model.fields {
            let prev_field = prev_model.fields.iter().find(|f| f.name == field.name);
            if prev_field.is_none() {
                let col_type = self.field_type_to_sql(&field.field_type, dialect);
                let mut col_def = format!("ADD COLUMN {} {}", field.name, col_type);
                if !field.is_optional {
                    col_def.push_str(" NOT NULL");
                }
                if let Some(ref default) = field.default_value {
                    col_def.push_str(&format!(" DEFAULT {}", self.format_default_value(default, dialect)));
                }
                sql.push_str(&format!("ALTER TABLE {} {};\n", model.table_name, col_def));
            }
        }

        if dialect != "sqlite" {
            for prev_field in &prev_model.fields {
                let exists = model.fields.iter().any(|f| f.name == prev_field.name);
                if !exists {
                    sql.push_str(&format!("ALTER TABLE {} DROP COLUMN {};\n", model.table_name, prev_field.name));
                }
            }
        }

        sql
    }

    fn format_default_value(&self, value: &str, dialect: &str) -> String {
        if value == "now()" {
            match dialect {
                "postgresql" => "NOW()".to_string(),
                "mysql" => "CURRENT_TIMESTAMP".to_string(),
                _ => "CURRENT_TIMESTAMP".to_string(),
            }
        }
        else if value.starts_with('"') && value.ends_with('"') {
            value.to_string()
        }
        else if value == "true" || value == "false" {
            match dialect {
                "sqlite" => {
                    if value == "true" {
                        "1".to_string()
                    }
                    else {
                        "0".to_string()
                    }
                }
                _ => value.to_uppercase(),
            }
        }
        else {
            value.to_string()
        }
    }

    fn field_type_to_ir_name(&self, ft: &FieldTypeIr) -> String {
        match ft {
            FieldTypeIr::I8 => "i8".to_string(),
            FieldTypeIr::I16 => "i16".to_string(),
            FieldTypeIr::I32 => "i32".to_string(),
            FieldTypeIr::I64 => "i64".to_string(),
            FieldTypeIr::U8 => "u8".to_string(),
            FieldTypeIr::U16 => "u16".to_string(),
            FieldTypeIr::U32 => "u32".to_string(),
            FieldTypeIr::U64 => "u64".to_string(),
            FieldTypeIr::F32 => "f32".to_string(),
            FieldTypeIr::F64 => "f64".to_string(),
            FieldTypeIr::Bool => "bool".to_string(),
            FieldTypeIr::String => "string".to_string(),
            FieldTypeIr::Bytes => "bytes".to_string(),
            FieldTypeIr::Datetime => "datetime".to_string(),
            FieldTypeIr::Uuid => "uuid".to_string(),
            FieldTypeIr::Array(inner) => format!("[{}]", self.field_type_to_ir_name(inner)),
            FieldTypeIr::RefArray(inner) => format!("[&{}]", self.field_type_to_ir_name(inner)),
            FieldTypeIr::Map(k, v) => format!("map<{}, {}>", self.field_type_to_ir_name(k), self.field_type_to_ir_name(v)),
            FieldTypeIr::Custom(name) => name.clone(),
            FieldTypeIr::Optional(inner) => format!("{}?", self.field_type_to_ir_name(inner)),
        }
    }

    fn field_type_to_valkyrie(&self, ft: &FieldTypeIr) -> String {
        match ft {
            FieldTypeIr::I8 => "Int8".to_string(),
            FieldTypeIr::I16 => "Int16".to_string(),
            FieldTypeIr::I32 => "Int32".to_string(),
            FieldTypeIr::I64 => "Int64".to_string(),
            FieldTypeIr::U8 => "UInt8".to_string(),
            FieldTypeIr::U16 => "UInt16".to_string(),
            FieldTypeIr::U32 => "UInt32".to_string(),
            FieldTypeIr::U64 => "UInt64".to_string(),
            FieldTypeIr::F32 => "Float32".to_string(),
            FieldTypeIr::F64 => "Float64".to_string(),
            FieldTypeIr::Bool => "Bool".to_string(),
            FieldTypeIr::String => "String".to_string(),
            FieldTypeIr::Bytes => "Bytes".to_string(),
            FieldTypeIr::Datetime => "Datetime".to_string(),
            FieldTypeIr::Uuid => "Uuid".to_string(),
            FieldTypeIr::Array(inner) => format!("Array<{}>", self.field_type_to_valkyrie(inner)),
            FieldTypeIr::RefArray(inner) => format!("RefArray<{}>", self.field_type_to_valkyrie(inner)),
            FieldTypeIr::Map(k, v) => format!("Map<{}, {}>", self.field_type_to_valkyrie(k), self.field_type_to_valkyrie(v)),
            FieldTypeIr::Custom(name) => name.clone(),
            FieldTypeIr::Optional(inner) => self.field_type_to_valkyrie(inner),
        }
    }

    fn field_type_to_sql(&self, ft: &FieldTypeIr, dialect: &str) -> String {
        match ft {
            FieldTypeIr::I8 => "TINYINT".to_string(),
            FieldTypeIr::I16 => "SMALLINT".to_string(),
            FieldTypeIr::I32 => "INTEGER".to_string(),
            FieldTypeIr::I64 => "BIGINT".to_string(),
            FieldTypeIr::U8 => "TINYINT UNSIGNED".to_string(),
            FieldTypeIr::U16 => "SMALLINT UNSIGNED".to_string(),
            FieldTypeIr::U32 => "INTEGER UNSIGNED".to_string(),
            FieldTypeIr::U64 => "BIGINT UNSIGNED".to_string(),
            FieldTypeIr::F32 => "REAL".to_string(),
            FieldTypeIr::F64 => "DOUBLE".to_string(),
            FieldTypeIr::Bool => match dialect {
                "postgresql" => "BOOLEAN".to_string(),
                "sqlite" => "INTEGER".to_string(),
                _ => "TINYINT".to_string(),
            },
            FieldTypeIr::String => match dialect {
                "postgresql" => "TEXT".to_string(),
                "mysql" => "VARCHAR(255)".to_string(),
                _ => "TEXT".to_string(),
            },
            FieldTypeIr::Bytes => "BLOB".to_string(),
            FieldTypeIr::Datetime => match dialect {
                "postgresql" => "TIMESTAMP".to_string(),
                "mysql" => "DATETIME".to_string(),
                _ => "TEXT".to_string(),
            },
            FieldTypeIr::Uuid => match dialect {
                "postgresql" => "UUID".to_string(),
                _ => "TEXT".to_string(),
            },
            FieldTypeIr::Array(_) | FieldTypeIr::RefArray(_) => "TEXT".to_string(),
            FieldTypeIr::Map(_, _) => "TEXT".to_string(),
            FieldTypeIr::Custom(name) => format!("-- custom type: {name}"),
            FieldTypeIr::Optional(inner) => self.field_type_to_sql(inner, dialect),
        }
    }
}

impl Default for SchemaCodegen {
    fn default() -> Self {
        Self::new()
    }
}
