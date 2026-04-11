#![warn(missing_docs)]

//! gs AST → naga IR 转换器
//!
//! 将 gs 语言的类型化 AST 转换为 naga IR 中间表示，
//! 支持顶点着色器、片段着色器、计算着色器和 uniforms。

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use naga;

#[cfg(feature = "valkyrie-compiler")]
use indexmap::IndexMap;
#[cfg(feature = "valkyrie-compiler")]
use naga::{Binding, BinaryOperator, BuiltIn, Expression, FunctionArgument, FunctionResult, ScalarKind, Span as NagaSpan, TypeInner, VectorSize};
#[cfg(feature = "valkyrie-compiler")]
use oak_valkyrie::ast::{
    Attribute, Block, FieldDeclaration, Let, MicroDeclaration, Pattern,
    ShaderDeclaration, Statement, StatementNode, StructureDeclaration, TermExpression,
    TypeExpression,
};
#[cfg(feature = "valkyrie-compiler")]
use oak_valkyrie::lexer::token_type::ValkyrieTokenType;

/// gs 着色器属性信息
///
/// 表示 shader 块中的一个属性声明，如 `let albedo: texture = "white"`。
#[cfg(feature = "valkyrie-compiler")]
#[derive(Debug, Clone)]
pub struct GsProperty {
    /// 属性名称
    pub name: String,
    /// 属性类型名称（如 "texture"、"f32"、"color"、"vector"）
    pub type_name: String,
    /// 属性默认值
    pub default_value: Option<String>,
}

/// gs 着色器入口点信息
///
/// 表示一个带有 `@vertex`/`@fragment`/`@compute` 注解的 micro 函数。
#[cfg(feature = "valkyrie-compiler")]
#[derive(Debug, Clone)]
pub struct GsEntryPoint {
    /// 入口点名称
    pub name: String,
    /// 着色器阶段
    pub stage: naga::ShaderStage,
    /// 原始 micro 声明
    pub micro: MicroDeclaration,
}

/// gs uniform 字段信息
///
/// 表示 uniform 结构体中的一个字段。
#[cfg(feature = "valkyrie-compiler")]
#[derive(Debug, Clone)]
pub struct GsUniformField {
    /// 字段名称
    pub name: String,
    /// 字段类型名称（如 "mat44"、"vec3"、"vec4"、"f32"、"tex2"）
    pub type_name: String,
}

/// gs 着色器渲染状态信息
///
/// 表示 render_states 块中的键值对。
#[cfg(feature = "valkyrie-compiler")]
#[derive(Debug, Clone)]
pub struct GsRenderState {
    /// 状态名称
    pub name: String,
    /// 状态值
    pub value: String,
}

/// gs AST → naga IR 转换器
///
/// 将 oak-valkyrie 的 Shader AST 转换为 `naga::Module`，
/// 支持顶点着色器、片段着色器、计算着色器和 uniforms。
///
/// # 工作流程
///
/// 1. 第一遍遍历：收集 shader 块中的属性、入口点函数、uniforms 和渲染状态
/// 2. 第二遍遍历：创建 naga 类型、全局变量和入口点
pub struct GslLowerer {
    /// 局部变量名到 naga 句柄的映射
    local_vars: HashMap<String, naga::Handle<naga::LocalVariable>>,
    /// 全局变量名到 naga 句柄的映射
    global_vars: HashMap<String, naga::Handle<naga::GlobalVariable>>,
    /// 类型缓存，避免重复创建
    type_cache: HashMap<String, naga::Handle<naga::Type>>,
    /// 常量缓存，避免重复创建
    const_cache: HashMap<String, naga::Handle<naga::Constant>>,
    /// 下一个可用的 binding 编号
    next_binding: u32,
}

impl GslLowerer {
    /// 创建新的转换器
    pub fn new() -> Self {
        Self {
            local_vars: HashMap::new(),
            global_vars: HashMap::new(),
            type_cache: HashMap::new(),
            const_cache: HashMap::new(),
            next_binding: 1,
        }
    }

    /// 将 oak-valkyrie 的 Shader AST 转换为 naga Module
    ///
    /// 执行两遍遍历：第一遍收集信息，第二遍生成 naga IR。
    #[cfg(feature = "valkyrie-compiler")]
    pub fn lower(&mut self, shader: &ShaderDeclaration) -> GResult<naga::Module> {
        self.local_vars.clear();
        self.global_vars.clear();
        self.type_cache.clear();
        self.const_cache.clear();
        self.next_binding = 1;

        let mut module = naga::Module::default();

        let (properties, entry_points, uniform_fields, render_states) =
            self.collect_shader_items(shader)?;

        let uniform_buffer_ty = self.create_uniform_buffer(&uniform_fields, &mut module)?;

        for prop in &properties {
            self.create_property_globals(prop, &mut module)?;
        }

        for ep in &entry_points {
            let entry_point =
                self.lower_entry_point(ep, &mut module, uniform_buffer_ty)?;
            module.entry_points.push(entry_point);
        }

        let _ = render_states;

        Ok(module)
    }

    /// 第一遍遍历：收集 shader 块中的所有声明
    ///
    /// 从 `ShaderDeclaration.items` 中提取属性、入口点、uniform 字段和渲染状态。
    #[cfg(feature = "valkyrie-compiler")]
    fn collect_shader_items(
        &mut self,
        shader: &ShaderDeclaration,
    ) -> GResult<(
        Vec<GsProperty>,
        Vec<GsEntryPoint>,
        Vec<GsUniformField>,
        Vec<GsRenderState>,
    )> {
        let mut properties = Vec::new();
        let mut entry_points = Vec::new();
        let mut uniform_fields = Vec::new();
        let mut render_states = Vec::new();

        for item in &shader.items {
            match item {
                StatementNode::Let(let_stmt) => {
                    let prop = self.collect_property(let_stmt)?;
                    properties.push(prop);
                }
                StatementNode::Micro(micro) => {
                    if let Some(ep) = self.collect_entry_point(micro)? {
                        entry_points.push(ep);
                    }
                }
                StatementNode::Structure(structure) => {
                    let fields = self.collect_uniform_fields(structure)?;
                    uniform_fields.extend(fields);
                }
                StatementNode::Namespace(namespace) => {
                    let name = namespace.name.to_string();
                    let name_lower = name.to_lowercase();
                    if name_lower.contains("render_state") || name_lower.contains("renderstate") {
                        for inner_item in &namespace.items {
                            if let StatementNode::Let(let_stmt) = inner_item {
                                let name = match &let_stmt.pattern {
                                    Pattern::Variable(v) => v.name.name.clone(),
                                    _ => continue,
                                };
                                let value = self.expr_to_string(&let_stmt.expr);
                                render_states.push(GsRenderState { name, value });
                            }
                        }
                    } else {
                        for inner_item in &namespace.items {
                            if let StatementNode::Let(let_stmt) = inner_item {
                                let prop = self.collect_property(let_stmt)?;
                                properties.push(prop);
                            }
                        }
                    }
                }
                StatementNode::ExprStmt(expr_stmt) => {
                    let expr_str = self.expr_to_string(&expr_stmt.expr);
                    if expr_str.contains("uniforms") || expr_str.contains("render_states") {
                        // Skip expression statements that are section markers
                    }
                }
                _ => {}
            }
        }

        Ok((properties, entry_points, uniform_fields, render_states))
    }

    /// 从 let 语句中收集属性信息
    #[cfg(feature = "valkyrie-compiler")]
    fn collect_property(&self, let_stmt: &Let) -> GResult<GsProperty> {
        let name = match &let_stmt.pattern {
            Pattern::Variable(v) => v.name.name.clone(),
            Pattern::Wildcard(_) => "_".to_string(),
            _ => {
                return Err(GError {
                    kind: GErrorKind::Other,
                    message: format!("不支持的属性模式: {:?}", let_stmt.pattern),
                });
            }
        };

        let type_name = let_stmt
            .ty
            .as_ref()
            .map(|ty| self.type_expr_to_string(ty))
            .unwrap_or_else(|| "unknown".to_string());

        let default_value = let_stmt
            .expr
            .as_ref()
            .map(|expr| self.expr_to_string(&expr));

        Ok(GsProperty {
            name,
            type_name,
            default_value,
        })
    }

    /// 从 micro 声明中收集入口点信息
    ///
    /// 根据 `@vertex`/`@fragment`/`@compute` 注解确定着色器阶段。
    /// 如果没有阶段注解，则根据函数名推断。
    #[cfg(feature = "valkyrie-compiler")]
    fn collect_entry_point(&self, micro: &MicroDeclaration) -> GResult<Option<GsEntryPoint>> {
        let stage = self.detect_shader_stage(&micro.annotations, &micro.name.name)?;

        Ok(Some(GsEntryPoint {
            name: micro.name.name.clone(),
            stage,
            micro: micro.clone(),
        }))
    }

    /// 从注解或函数名检测着色器阶段
    #[cfg(feature = "valkyrie-compiler")]
    fn detect_shader_stage(
        &self,
        annotations: &[Attribute],
        name: &str,
    ) -> GResult<naga::ShaderStage> {
        for attr in annotations {
            let attr_name = attr.name.name.to_lowercase();
            match attr_name.as_str() {
                "vertex" => return Ok(naga::ShaderStage::Vertex),
                "fragment" => return Ok(naga::ShaderStage::Fragment),
                "compute" => return Ok(naga::ShaderStage::Compute),
                _ => {}
            }
        }

        let name_lower = name.to_lowercase();
        if name_lower.starts_with("vs") || name_lower.starts_with("vertex") {
            Ok(naga::ShaderStage::Vertex)
        } else if name_lower.starts_with("fs") || name_lower.starts_with("fragment") {
            Ok(naga::ShaderStage::Fragment)
        } else if name_lower.starts_with("cs") || name_lower.starts_with("compute") {
            Ok(naga::ShaderStage::Compute)
        } else {
            Err(GError {
                kind: GErrorKind::Other,
                message: format!(
                    "无法确定着色器阶段: 函数 '{}' 缺少 @vertex/@fragment/@compute 注解",
                    name
                ),
            })
        }
    }

    /// 从结构体声明中收集 uniform 字段
    #[cfg(feature = "valkyrie-compiler")]
    fn collect_uniform_fields(
        &self,
        structure: &StructureDeclaration,
    ) -> GResult<Vec<GsUniformField>> {
        let mut fields = Vec::new();
        for field in &structure.fields {
            let type_name = self.type_expr_to_string(&field.ty);
            fields.push(GsUniformField {
                name: field.name.name.clone(),
                type_name,
            });
        }
        Ok(fields)
    }

    /// 创建 uniform 缓冲区结构体和全局变量
    ///
    /// 将收集到的 uniform 字段创建为 naga 结构体类型，
    /// 并创建对应的 `GlobalVariable`（`AddressSpace::Uniform`，`@group(0) @binding(0)`）。
    #[cfg(feature = "valkyrie-compiler")]
    fn create_uniform_buffer(
        &mut self,
        fields: &[GsUniformField],
        module: &mut naga::Module,
    ) -> GResult<Option<naga::Handle<naga::Type>>> {
        if fields.is_empty() {
            return Ok(None);
        }

        let mut members = Vec::new();
        let mut offset = 0u32;

        for field in fields {
            let ty = self.get_or_create_naga_type(&field.type_name, module)?;
            let size = self.type_size_align(&field.type_name);
            offset = Self::align_offset(offset, size.align);
            members.push(naga::StructMember {
                name: Some(field.name.clone()),
                ty,
                binding: None,
                offset,
            });
            offset += size.size;
        }

        let struct_ty = module.types.insert(
            naga::Type {
                name: Some("Uniforms".to_string()),
                inner: naga::TypeInner::Struct {
                    members,
                    span: offset,
                },
            },
            NagaSpan::UNDEFINED,
        );

        let gv = module.global_variables.append(
            naga::GlobalVariable {
                name: Some("uniforms".to_string()),
                space: naga::AddressSpace::Uniform,
                binding: Some(naga::ResourceBinding {
                    group: 0,
                    binding: 0,
                }),
                ty: struct_ty,
                init: None,
                memory_decorations: naga::MemoryDecorations::empty(),
            },
            NagaSpan::UNDEFINED,
        );
        self.global_vars.insert("uniforms".to_string(), gv);

        self.type_cache.insert("Uniforms".to_string(), struct_ty);

        Ok(Some(struct_ty))
    }

    /// 为属性创建全局变量（sampler + texture）
    ///
    /// `texture` 类型属性会创建一个 sampler 和一个 texture_2d 全局变量。
    /// 非 texture 类型属性会作为 uniform 缓冲区的成员处理。
    #[cfg(feature = "valkyrie-compiler")]
    fn create_property_globals(
        &mut self,
        prop: &GsProperty,
        module: &mut naga::Module,
    ) -> GResult<()> {
        let type_lower = prop.type_name.to_lowercase();

        if type_lower == "texture" || type_lower == "tex2" || type_lower == "texture2d" {
            let sampler_name = format!("{}_sampler", prop.name);
            let sampler_ty = self.get_or_create_naga_type("sampler", module)?;
            let sampler_binding = self.next_binding;
            self.next_binding += 1;

            let sampler_gv = module.global_variables.append(
                naga::GlobalVariable {
                    name: Some(sampler_name.clone()),
                    space: naga::AddressSpace::Handle,
                    binding: Some(naga::ResourceBinding {
                        group: 1,
                        binding: sampler_binding,
                    }),
                    ty: sampler_ty,
                    init: None,
                    memory_decorations: naga::MemoryDecorations::empty(),
                },
                NagaSpan::UNDEFINED,
            );
            self.global_vars.insert(sampler_name, sampler_gv);

            let texture_binding = self.next_binding;
            self.next_binding += 1;
            let texture_ty = self.get_or_create_naga_type("texture_2d", module)?;

            let texture_gv = module.global_variables.append(
                naga::GlobalVariable {
                    name: Some(prop.name.clone()),
                    space: naga::AddressSpace::Handle,
                    binding: Some(naga::ResourceBinding {
                        group: 1,
                        binding: texture_binding,
                    }),
                    ty: texture_ty,
                    init: None,
                    memory_decorations: naga::MemoryDecorations::empty(),
                },
                NagaSpan::UNDEFINED,
            );
            self.global_vars.insert(prop.name.clone(), texture_gv);
        }

        Ok(())
    }

    /// 将函数转换为 naga EntryPoint
    ///
    /// 解析函数签名中的参数和返回类型，创建输入/输出结构体，
    /// 并将函数体表达式转换为 naga 表达式。
    #[cfg(feature = "valkyrie-compiler")]
    fn lower_entry_point(
        &mut self,
        ep: &GsEntryPoint,
        module: &mut naga::Module,
        uniform_buffer_ty: Option<naga::Handle<naga::Type>>,
    ) -> GResult<naga::EntryPoint> {
        self.local_vars.clear();

        let mut function = naga::Function::default();
        function.name = Some(ep.name.clone());

        let mut expressions = naga::Arena::new();
        let mut named_expressions = IndexMap::new();
        let mut body = naga::Block::new();

        let mut location_index: u32 = 0;

        for param in &ep.micro.params {
            let type_name = param
                .ty
                .as_ref()
                .map(|ty| self.type_expr_to_string(ty))
                .unwrap_or_else(|| "f32".to_string());

            let param_ty = self.get_or_create_naga_type(&type_name, module)?;

            let binding = self.infer_param_binding(
                &param.name.name,
                &type_name,
                ep.stage,
                &mut location_index,
            );

            function.arguments.push(FunctionArgument {
                name: Some(param.name.name.clone()),
                ty: param_ty,
                binding,
            });

            let arg_expr = expressions.append(
                Expression::FunctionArgument(function.arguments.len() - 1),
                NagaSpan::UNDEFINED,
            );
            named_expressions.insert(param.name.name.clone(), arg_expr);
        }

        if let Some(ret_type) = &ep.micro.return_type {
            let type_name = self.type_expr_to_string(ret_type);
            let result_ty = self.get_or_create_naga_type(&type_name, module)?;

            let result_binding = self.infer_result_binding(&type_name, ep.stage, &mut location_index);

            function.result = Some(FunctionResult {
                ty: result_ty,
                binding: result_binding,
            });
        }

        if let Some(ub_ty) = uniform_buffer_ty {
            let _ = ub_ty;
        }

        self.lower_block(
            &ep.micro.body,
            module,
            &mut expressions,
            &mut named_expressions,
            &mut body,
        )?;

        function.expressions = expressions;
        function.named_expressions = named_expressions;
        function.body = body;

        let workgroup_size = if ep.stage == naga::ShaderStage::Compute {
            [1u32; 3]
        } else {
            [0u32; 3]
        };

        Ok(naga::EntryPoint {
            name: ep.name.clone(),
            stage: ep.stage,
            early_depth_test: None,
            workgroup_size,
            workgroup_size_overrides: None,
            function,
            mesh_info: None,
            task_payload: None,
            incoming_ray_payload: None,
        })
    }

    /// 推断函数参数的绑定（location 或 builtin）
    #[cfg(feature = "valkyrie-compiler")]
    fn infer_param_binding(
        &self,
        param_name: &str,
        type_name: &str,
        stage: naga::ShaderStage,
        location_index: &mut u32,
    ) -> Option<Binding> {
        let name_lower = param_name.to_lowercase();
        let type_lower = type_name.to_lowercase();

        if stage == naga::ShaderStage::Vertex {
            match name_lower.as_str() {
                "vertex_id" => return Some(Binding::BuiltIn(BuiltIn::VertexIndex)),
                "instance_id" => return Some(Binding::BuiltIn(BuiltIn::InstanceIndex)),
                _ => {}
            }

            let loc = *location_index;
            *location_index += 1;
            Some(Binding::Location {
                location: loc,
                interpolation: None,
                sampling: None,
            })
        } else if stage == naga::ShaderStage::Fragment {
            match name_lower.as_str() {
                "clip_position" | "position" if type_lower.contains("vec4") => {
                    return Some(Binding::BuiltIn(BuiltIn::Position));
                }
                _ => {}
            }

            let loc = *location_index;
            *location_index += 1;
            Some(Binding::Location {
                location: loc,
                interpolation: None,
                sampling: None,
            })
        } else {
            let loc = *location_index;
            *location_index += 1;
            Some(Binding::Location {
                location: loc,
                interpolation: None,
                sampling: None,
            })
        }
    }

    /// 推断函数返回值的绑定
    #[cfg(feature = "valkyrie-compiler")]
    fn infer_result_binding(
        &self,
        type_name: &str,
        stage: naga::ShaderStage,
        location_index: &mut u32,
    ) -> Option<Binding> {
        if stage == naga::ShaderStage::Vertex {
            Some(Binding::BuiltIn(BuiltIn::Position))
        } else if stage == naga::ShaderStage::Fragment {
            let loc = *location_index;
            *location_index += 1;
            Some(Binding::Location {
                location: loc,
                interpolation: None,
                sampling: None,
            })
        } else {
            None
        }
    }

    /// 转换语句块
    #[cfg(feature = "valkyrie-compiler")]
    fn lower_block(
        &mut self,
        block: &Block,
        module: &mut naga::Module,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut IndexMap<String, naga::Handle<Expression>>,
        body: &mut naga::Block,
    ) -> GResult<()> {
        for stmt in &block.statements {
            self.lower_statement(stmt, module, expressions, named_expressions, body)?;
        }
        Ok(())
    }

    /// 转换单条语句
    #[cfg(feature = "valkyrie-compiler")]
    fn lower_statement(
        &mut self,
        stmt: &Statement,
        module: &mut naga::Module,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut IndexMap<String, naga::Handle<Expression>>,
        body: &mut naga::Block,
    ) -> GResult<()> {
        match stmt {
            Statement::Let(let_stmt) => {
                self.lower_let_statement(let_stmt, module, expressions, named_expressions, body)?;
            }
            Statement::ExprStmt(expr_stmt) => {
                let expr_handle = self.lower_expression(
                    &expr_stmt.expr,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )?;
                if expr_stmt.semi {
                    body.push(
                        naga::Statement::Emit(expressions.range_from(expr_handle)),
                        NagaSpan::UNDEFINED,
                    );
                }
            }
        }
        Ok(())
    }

    /// 转换 let 绑定语句
    #[cfg(feature = "valkyrie-compiler")]
    fn lower_let_statement(
        &mut self,
        let_stmt: &Let,
        module: &mut naga::Module,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut IndexMap<String, naga::Handle<Expression>>,
        body: &mut naga::Block,
    ) -> GResult<()> {
        let var_name = match &let_stmt.pattern {
            Pattern::Variable(v) => v.name.name.clone(),
            Pattern::Wildcard(_) => "_".to_string(),
            _ => {
                return Err(GError {
                    kind: GErrorKind::Other,
                    message: format!("不支持的 let 绑定模式: {:?}", let_stmt.pattern),
                });
            }
        };

        let value_handle = self.lower_expression(
            &let_stmt.expr,
            module,
            expressions,
            named_expressions,
            body,
        )?;

        if let_stmt.is_mutable {
            let type_name = let_stmt
                .ty
                .as_ref()
                .map(|ty| self.type_expr_to_string(ty))
                .unwrap_or_else(|| "f32".to_string());

            let var_ty = self.get_or_create_naga_type(&type_name, module)?;

            let local_var = naga::LocalVariable {
                name: Some(var_name.clone()),
                ty: var_ty,
                init: None,
            };

            let local_handle = body.append_local(local_var);
            self.local_vars.insert(var_name.clone(), local_handle);

            let var_expr = expressions.append(
                Expression::LocalVariable(local_handle),
                NagaSpan::UNDEFINED,
            );

            body.push(
                naga::Statement::Store {
                    target: var_expr,
                    value: value_handle,
                },
                NagaSpan::UNDEFINED,
            );
        } else {
            named_expressions.insert(var_name, value_handle);
        }

        Ok(())
    }

    /// 转换表达式
    #[cfg(feature = "valkyrie-compiler")]
    fn lower_expression(
        &mut self,
        expr: &TermExpression,
        module: &mut naga::Module,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut IndexMap<String, naga::Handle<Expression>>,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        match expr {
            TermExpression::NamePath(name_path) => {
                if name_path.parts.len() == 1 {
                    let name = &name_path.parts[0].name;
                    self.resolve_name(name, expressions, named_expressions)
                } else {
                    let first = &name_path.parts[0].name;
                    let first_expr = self.resolve_name(first, expressions, named_expressions)?;
                    let mut current = first_expr;
                    for part in &name_path.parts[1..] {
                        current = expressions.append(
                            Expression::AccessIndex {
                                base: current,
                                index: self.field_index(&part.name) as u32,
                            },
                            NagaSpan::UNDEFINED,
                        );
                    }
                    Ok(current)
                }
            }

            TermExpression::Binary(node) => {
                let left = self.lower_expression(
                    &node.lhs,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )?;
                let right = self.lower_expression(
                    &node.rhs,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )?;
                let op = self.map_binary_operator(&node.operator)?;
                Ok(expressions.append(
                    Expression::Binary { op, left, right },
                    NagaSpan::UNDEFINED,
                ))
            }

            TermExpression::Unary(node) => {
                let operand = self.lower_expression(
                    &node.base,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )?;
                let op = self.map_unary_operator(&node.operator)?;
                Ok(expressions.append(
                    Expression::Unary { op, expr: operand },
                    NagaSpan::UNDEFINED,
                ))
            }

            TermExpression::ApplyCall { callee, args, .. } => {
                self.lower_call(callee, args, module, expressions, named_expressions, body)
            }

            TermExpression::DotCall {
                receiver,
                field,
                ..
            } => {
                let base = self.lower_expression(
                    receiver,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )?;
                let idx = self.field_index(&field.name);
                Ok(expressions.append(
                    Expression::AccessIndex {
                        base,
                        index: idx as u32,
                    },
                    NagaSpan::UNDEFINED,
                ))
            }

            TermExpression::Paren { expr: inner, .. } => self.lower_expression(
                inner,
                module,
                expressions,
                named_expressions,
                body,
            ),

            TermExpression::Return(ret) => {
                if let Some(return_expr) = &ret.base {
                    let value = self.lower_expression(
                        return_expr,
                        module,
                        expressions,
                        named_expressions,
                        body,
                    )?;
                    body.push(
                        naga::Statement::Return { value: Some(value) },
                        NagaSpan::UNDEFINED,
                    );
                } else {
                    body.push(
                        naga::Statement::Return { value: None },
                        NagaSpan::UNDEFINED,
                    );
                }
                Ok(expressions.append(Expression::Value(naga::ZeroValue::Scalar), NagaSpan::UNDEFINED))
            }

            TermExpression::Bool { value, .. } => {
                let literal = naga::Literal::Bool(*value);
                Ok(expressions.append(Expression::Literal(literal), NagaSpan::UNDEFINED))
            }

            TermExpression::StringLiteral(string_literal) => {
                let content: String = string_literal
                    .segments
                    .iter()
                    .filter_map(|seg| match seg {
                        oak_valkyrie::ast::StringSegment::Text(text_seg) => {
                            Some(text_seg.content.as_str())
                        }
                        _ => None,
                    })
                    .collect();

                if let Ok(float_val) = content.parse::<f32>() {
                    let literal = naga::Literal::F32(float_val);
                    Ok(expressions.append(Expression::Literal(literal), NagaSpan::UNDEFINED))
                } else if let Ok(int_val) = content.parse::<i32>() {
                    let literal = naga::Literal::I32(int_val);
                    Ok(expressions.append(Expression::Literal(literal), NagaSpan::UNDEFINED))
                } else {
                    Err(GError {
                        kind: GErrorKind::Other,
                        message: format!("着色器中不支持字符串字面量: '{}'", content),
                    })
                }
            }

            TermExpression::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond = self.lower_expression(
                    condition,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )?;

                let mut then_body = naga::Block::new();
                self.lower_block(
                    then_branch,
                    module,
                    expressions,
                    named_expressions,
                    &mut then_body,
                )?;

                if let Some(else_block) = else_branch {
                    let mut else_body = naga::Block::new();
                    self.lower_block(
                        else_block,
                        module,
                        expressions,
                        named_expressions,
                        &mut else_body,
                    )?;
                    body.push(
                        naga::Statement::If {
                            condition: cond,
                            accept: then_body,
                            reject: else_body,
                        },
                        NagaSpan::UNDEFINED,
                    );
                } else {
                    body.push(
                        naga::Statement::If {
                            condition: cond,
                            accept: then_body,
                            reject: naga::Block::new(),
                        },
                        NagaSpan::UNDEFINED,
                    );
                }

                Ok(expressions.append(Expression::Value(naga::ZeroValue::Scalar), NagaSpan::UNDEFINED))
            }

            TermExpression::Block(block) => {
                self.lower_block(
                    block,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )?;
                Ok(expressions.append(Expression::Value(naga::ZeroValue::Scalar), NagaSpan::UNDEFINED))
            }

            _ => Err(GError {
                kind: GErrorKind::Other,
                message: format!("暂不支持的表达式类型: {:?}", std::mem::discriminant(expr)),
            }),
        }
    }

    /// 转换函数调用表达式
    ///
    /// 映射 gs 内置函数到 naga 操作：
    /// - `texture_sample` → naga::Expression::Sampling
    /// - `normalize` → naga::Expression::Math { fun: Normalize }
    /// - `dot` → naga::Expression::Binary { op: Dot }
    /// - `mul` → naga::Expression::Binary { op: Multiply }
    /// - `vec2/vec3/vec4` → naga::Expression::Compose
    /// - `max/min/clamp/mix/pow` → naga::Expression::Math
    #[cfg(feature = "valkyrie-compiler")]
    fn lower_call(
        &mut self,
        callee: &TermExpression,
        args: &[TermExpression],
        module: &mut naga::Module,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut IndexMap<String, naga::Handle<Expression>>,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        let func_name = self.extract_callee_name(callee);

        match func_name.as_deref() {
            Some("texture_sample") | Some("textureSample") => {
                self.lower_texture_sample(args, module, expressions, named_expressions, body)
            }

            Some("vec2") | Some("vec2f") => {
                self.lower_vec_constructor(2, args, module, expressions, named_expressions, body)
            }
            Some("vec3") | Some("vec3f") => {
                self.lower_vec_constructor(3, args, module, expressions, named_expressions, body)
            }
            Some("vec4") | Some("vec4f") => {
                self.lower_vec_constructor(4, args, module, expressions, named_expressions, body)
            }

            Some("normalize") => {
                self.lower_math_func(
                    naga::MathFunction::Normalize,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("length") => {
                self.lower_math_func(
                    naga::MathFunction::Length,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("dot") => {
                if args.len() == 2 {
                    let left = self.lower_expression(
                        &args[0],
                        module,
                        expressions,
                        named_expressions,
                        body,
                    )?;
                    let right = self.lower_expression(
                        &args[1],
                        module,
                        expressions,
                        named_expressions,
                        body,
                    )?;
                    Ok(expressions.append(
                        Expression::Binary {
                            op: BinaryOperator::Dot,
                            left,
                            right,
                        },
                        NagaSpan::UNDEFINED,
                    ))
                } else {
                    Err(GError {
                        kind: GErrorKind::Other,
                        message: format!("dot() 需要 2 个参数，得到 {}", args.len()),
                    })
                }
            }
            Some("mul") => {
                if args.len() == 2 {
                    let left = self.lower_expression(
                        &args[0],
                        module,
                        expressions,
                        named_expressions,
                        body,
                    )?;
                    let right = self.lower_expression(
                        &args[1],
                        module,
                        expressions,
                        named_expressions,
                        body,
                    )?;
                    Ok(expressions.append(
                        Expression::Binary {
                            op: BinaryOperator::Multiply,
                            left,
                            right,
                        },
                        NagaSpan::UNDEFINED,
                    ))
                } else {
                    Err(GError {
                        kind: GErrorKind::Other,
                        message: format!("mul() 需要 2 个参数，得到 {}", args.len()),
                    })
                }
            }
            Some("max") => {
                self.lower_math_func(
                    naga::MathFunction::Max,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("min") => {
                self.lower_math_func(
                    naga::MathFunction::Min,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("clamp") => {
                self.lower_math_func(
                    naga::MathFunction::Clamp,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("mix") => {
                self.lower_math_func(
                    naga::MathFunction::Mix,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("pow") => {
                self.lower_math_func(
                    naga::MathFunction::Pow,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("abs") => {
                self.lower_math_func(
                    naga::MathFunction::Abs,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("sqrt") => {
                self.lower_math_func(
                    naga::MathFunction::Sqrt,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("floor") => {
                self.lower_math_func(
                    naga::MathFunction::Floor,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("ceil") => {
                self.lower_math_func(
                    naga::MathFunction::Ceil,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("fract") => {
                self.lower_math_func(
                    naga::MathFunction::Fract,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("step") => {
                self.lower_math_func(
                    naga::MathFunction::Step,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("smoothstep") => {
                self.lower_math_func(
                    naga::MathFunction::SmoothStep,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("reflect") => {
                self.lower_math_func(
                    naga::MathFunction::Reflect,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("cross") => {
                self.lower_math_func(
                    naga::MathFunction::Cross,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("transpose") => {
                self.lower_math_func(
                    naga::MathFunction::Transpose,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("determinant") => {
                self.lower_math_func(
                    naga::MathFunction::Determinant,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("inverse") => {
                self.lower_math_func(
                    naga::MathFunction::Inverse,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("sign") => {
                self.lower_math_func(
                    naga::MathFunction::Sign,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("sin") => {
                self.lower_math_func(
                    naga::MathFunction::Sin,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("cos") => {
                self.lower_math_func(
                    naga::MathFunction::Cos,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }
            Some("tan") => {
                self.lower_math_func(
                    naga::MathFunction::Tan,
                    args,
                    module,
                    expressions,
                    named_expressions,
                    body,
                )
            }

            _ => {
                if let Some(name) = func_name.as_deref() {
                    if let Some(&gv_handle) = self.global_vars.get(name) {
                        return Ok(expressions.append(
                            Expression::GlobalVariable(gv_handle),
                            NagaSpan::UNDEFINED,
                        ));
                    }
                }
                Err(GError {
                    kind: GErrorKind::Other,
                    message: format!(
                        "未知的函数调用: '{}'",
                        func_name.unwrap_or("(complex callee)")
                    ),
                })
            }
        }
    }

    /// 转换 texture_sample 调用
    #[cfg(feature = "valkyrie-compiler")]
    fn lower_texture_sample(
        &mut self,
        args: &[TermExpression],
        module: &mut naga::Module,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut IndexMap<String, naga::Handle<Expression>>,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        if args.len() < 3 {
            return Err(GError {
                kind: GErrorKind::Other,
                message: format!(
                    "texture_sample() 至少需要 3 个参数（纹理、采样器、坐标），得到 {}",
                    args.len()
                ),
            });
        }

        let texture_name = self.extract_callee_name(&args[0]);
        let texture_expr = if let Some(name) = texture_name.as_deref() {
            let sampler_name = format!("{}_sampler", name);
            if let Some(&sampler_handle) = self.global_vars.get(&sampler_name) {
                expressions.append(
                    Expression::GlobalVariable(sampler_handle),
                    NagaSpan::UNDEFINED,
                )
            } else {
                self.lower_expression(&args[0], module, expressions, named_expressions, body)?
            }
        } else {
            self.lower_expression(&args[0], module, expressions, named_expressions, body)?
        };

        let texture_name2 = self.extract_callee_name(&args[0]);
        let sampler_expr = if let Some(name) = texture_name2.as_deref() {
            let sampler_name = format!("{}_sampler", name);
            if let Some(&sampler_handle) = self.global_vars.get(&sampler_name) {
                expressions.append(
                    Expression::GlobalVariable(sampler_handle),
                    NagaSpan::UNDEFINED,
                )
            } else {
                self.lower_expression(&args[1], module, expressions, named_expressions, body)?
            }
        } else {
            self.lower_expression(&args[1], module, expressions, named_expressions, body)?
        };

        let actual_texture = if let Some(name) = self.extract_callee_name(&args[0]).as_deref() {
            if let Some(&tex_handle) = self.global_vars.get(name) {
                expressions.append(
                    Expression::GlobalVariable(tex_handle),
                    NagaSpan::UNDEFINED,
                )
            } else {
                self.lower_expression(&args[0], module, expressions, named_expressions, body)?
            }
        } else {
            self.lower_expression(&args[0], module, expressions, named_expressions, body)?
        };

        let coordinate = self.lower_expression(
            &args[2],
            module,
            expressions,
            named_expressions,
            body,
        )?;

        let _ = (texture_expr, sampler_expr);

        Ok(expressions.append(
            Expression::Sampling {
                image: actual_texture,
                sampler: if let Some(name) = self.extract_callee_name(&args[0]).as_deref() {
                    let sampler_name = format!("{}_sampler", name);
                    if let Some(&sampler_handle) = self.global_vars.get(&sampler_name) {
                        expressions.append(
                            Expression::GlobalVariable(sampler_handle),
                            NagaSpan::UNDEFINED,
                        )
                    } else {
                        return Err(GError {
                            kind: GErrorKind::Other,
                            message: format!("找不到采样器: {}", sampler_name),
                        });
                    }
                } else {
                    return Err(GError {
                        kind: GErrorKind::Other,
                        message: "texture_sample 的纹理参数必须是命名纹理".to_string(),
                    });
                },
                coordinate,
                array_index: None,
                offset: None,
            },
            NagaSpan::UNDEFINED,
        ))
    }

    /// 转换向量构造函数（vec2/vec3/vec4）
    #[cfg(feature = "valkyrie-compiler")]
    fn lower_vec_constructor(
        &mut self,
        size: usize,
        args: &[TermExpression],
        module: &mut naga::Module,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut IndexMap<String, naga::Handle<Expression>>,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        let vector_size = match size {
            2 => VectorSize::Bi,
            3 => VectorSize::Tri,
            4 => VectorSize::Quad,
            _ => {
                return Err(GError {
                    kind: GErrorKind::Other,
                    message: format!("不支持的向量大小: {}", size),
                });
            }
        };

        let ty = module.types.insert(
            naga::Type {
                name: None,
                inner: TypeInner::Vector {
                    size: vector_size,
                    kind: ScalarKind::Float,
                    width: 4,
                },
            },
            NagaSpan::UNDEFINED,
        );

        let mut components = Vec::new();
        for arg in args {
            let comp = self.lower_expression(arg, module, expressions, named_expressions, body)?;
            components.push(comp);
        }

        Ok(expressions.append(
            Expression::Compose { ty, components },
            NagaSpan::UNDEFINED,
        ))
    }

    /// 转换数学函数调用
    #[cfg(feature = "valkyrie-compiler")]
    fn lower_math_func(
        &mut self,
        func: naga::MathFunction,
        args: &[TermExpression],
        module: &mut naga::Module,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut IndexMap<String, naga::Handle<Expression>>,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        let mut arg_handles = Vec::new();
        for arg in args {
            let handle =
                self.lower_expression(arg, module, expressions, named_expressions, body)?;
            arg_handles.push(handle);
        }

        Ok(expressions.append(
            Expression::Math {
                fun: func,
                arg: arg_handles[0],
                arg1: arg_handles.get(1).copied(),
                arg2: arg_handles.get(2).copied(),
                arg3: arg_handles.get(3).copied(),
            },
            NagaSpan::UNDEFINED,
        ))
    }

    /// 解析名称引用
    ///
    /// 依次查找：命名表达式 → 局部变量 → 全局变量。
    #[cfg(feature = "valkyrie-compiler")]
    fn resolve_name(
        &self,
        name: &str,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &IndexMap<String, naga::Handle<Expression>>,
    ) -> GResult<naga::Handle<Expression>> {
        if let Some(&handle) = named_expressions.get(name) {
            return Ok(handle);
        }

        if let Some(&local_handle) = self.local_vars.get(name) {
            return Ok(expressions.append(
                Expression::LocalVariable(local_handle),
                NagaSpan::UNDEFINED,
            ));
        }

        if let Some(&gv_handle) = self.global_vars.get(name) {
            return Ok(expressions.append(
                Expression::GlobalVariable(gv_handle),
                NagaSpan::UNDEFINED,
            ));
        }

        match name {
            "position" | "clip_position" => {
                if let Some(&gv_handle) = self.global_vars.get("uniforms") {
                    return Ok(expressions.append(
                        Expression::GlobalVariable(gv_handle),
                        NagaSpan::UNDEFINED,
                    ));
                }
            }
            _ => {}
        }

        Err(GError {
            kind: GErrorKind::Other,
            message: format!("未定义的变量: '{}'", name),
        })
    }

    /// 提取被调用者的函数名
    #[cfg(feature = "valkyrie-compiler")]
    fn extract_callee_name(&self, callee: &TermExpression) -> Option<String> {
        match callee {
            TermExpression::NamePath(name_path) if name_path.parts.len() == 1 => {
                Some(name_path.parts[0].name.clone())
            }
            _ => None,
        }
    }

    /// 将 Valkyrie 二元运算符映射到 naga BinaryOperator
    #[cfg(feature = "valkyrie-compiler")]
    fn map_binary_operator(&self, op: &ValkyrieTokenType) -> GResult<BinaryOperator> {
        match op {
            ValkyrieTokenType::Plus => Ok(BinaryOperator::Add),
            ValkyrieTokenType::Minus => Ok(BinaryOperator::Subtract),
            ValkyrieTokenType::Star => Ok(BinaryOperator::Multiply),
            ValkyrieTokenType::Slash => Ok(BinaryOperator::Divide),
            ValkyrieTokenType::Percent => Ok(BinaryOperator::Modulo),
            ValkyrieTokenType::EqEq => Ok(BinaryOperator::Equal),
            ValkyrieTokenType::NotEq => Ok(BinaryOperator::NotEqual),
            ValkyrieTokenType::LessThan => Ok(BinaryOperator::Less),
            ValkyrieTokenType::LessEq => Ok(BinaryOperator::LessEqual),
            ValkyrieTokenType::GreaterThan => Ok(BinaryOperator::Greater),
            ValkyrieTokenType::GreaterEq => Ok(BinaryOperator::GreaterEqual),
            ValkyrieTokenType::AndAnd => Ok(BinaryOperator::LogicalAnd),
            ValkyrieTokenType::OrOr => Ok(BinaryOperator::LogicalOr),
            ValkyrieTokenType::Ampersand => Ok(BinaryOperator::And),
            ValkyrieTokenType::Pipe => Ok(BinaryOperator::InclusiveOr),
            ValkyrieTokenType::Caret => Ok(BinaryOperator::ExclusiveOr),
            ValkyrieTokenType::LeftShift => Ok(BinaryOperator::ShiftLeft),
            ValkyrieTokenType::RightShift => Ok(BinaryOperator::ShiftRight),
            _ => Err(GError {
                kind: GErrorKind::Other,
                message: format!("不支持的二元运算符: {:?}", op),
            }),
        }
    }

    /// 将 Valkyrie 一元运算符映射到 naga UnaryOperator
    #[cfg(feature = "valkyrie-compiler")]
    fn map_unary_operator(&self, op: &ValkyrieTokenType) -> GResult<naga::UnaryOperator> {
        match op {
            ValkyrieTokenType::Minus => Ok(naga::UnaryOperator::Negate),
            ValkyrieTokenType::Bang => Ok(naga::UnaryOperator::Not),
            _ => Err(GError {
                kind: GErrorKind::Other,
                message: format!("不支持的一元运算符: {:?}", op),
            }),
        }
    }

    /// 获取或创建 naga 类型
    ///
    /// 将 gs 类型名称映射到 naga 类型：
    /// - `f32` → Scalar { kind: Float, width: 4 }
    /// - `i32` → Scalar { kind: Sint, width: 4 }
    /// - `u32` → Scalar { kind: Uint, width: 4 }
    /// - `bool` → Scalar { kind: Bool, width: 1 }
    /// - `vec2`/`vec2f` → Vector { size: Bi, kind: Float, width: 4 }
    /// - `vec3`/`vec3f` → Vector { size: Tri, kind: Float, width: 4 }
    /// - `vec4`/`vec4f`/`color` → Vector { size: Quad, kind: Float, width: 4 }
    /// - `mat44`/`mat4x4` → Matrix { columns: Quad, rows: Quad, kind: Float, width: 4 }
    /// - `sampler` → Sampler { comparison: false }
    /// - `texture_2d` → Texture { dim: Dim::D2, ... }
    #[cfg(feature = "valkyrie-compiler")]
    fn get_or_create_naga_type(
        &mut self,
        type_name: &str,
        module: &mut naga::Module,
    ) -> GResult<naga::Handle<naga::Type>> {
        let key = type_name.to_lowercase();

        if let Some(&handle) = self.type_cache.get(&key) {
            return Ok(handle);
        }

        let inner = match key.as_str() {
            "f32" | "float" => TypeInner::Scalar {
                kind: ScalarKind::Float,
                width: 4,
            },
            "i32" | "int" => TypeInner::Scalar {
                kind: ScalarKind::Sint,
                width: 4,
            },
            "u32" | "uint" => TypeInner::Scalar {
                kind: ScalarKind::Uint,
                width: 4,
            },
            "bool" => TypeInner::Scalar {
                kind: ScalarKind::Bool,
                width: 1,
            },
            "vec2" | "vec2f" | "vector2" => TypeInner::Vector {
                size: VectorSize::Bi,
                kind: ScalarKind::Float,
                width: 4,
            },
            "vec3" | "vec3f" | "vector" | "vector3" => TypeInner::Vector {
                size: VectorSize::Tri,
                kind: ScalarKind::Float,
                width: 4,
            },
            "vec4" | "vec4f" | "color" => TypeInner::Vector {
                size: VectorSize::Quad,
                kind: ScalarKind::Float,
                width: 4,
            },
            "mat44" | "mat4x4" | "mat4x4f" => TypeInner::Matrix {
                columns: VectorSize::Quad,
                rows: VectorSize::Quad,
                kind: ScalarKind::Float,
                width: 4,
            },
            "mat33" | "mat3x3" | "mat3x3f" => TypeInner::Matrix {
                columns: VectorSize::Tri,
                rows: VectorSize::Tri,
                kind: ScalarKind::Float,
                width: 4,
            },
            "sampler" => TypeInner::Sampler {
                comparison: false,
            },
            "sampler_comparison" => TypeInner::Sampler {
                comparison: true,
            },
            "texture_2d" | "tex2" | "texture" | "texture2d" => TypeInner::Texture {
                dim: naga::ImageDimension::D2,
                arrayed: false,
                class: naga::ImageClass::Sampled {
                    kind: ScalarKind::Float,
                    multi: false,
                },
            },
            "texture_2d_array" => TypeInner::Texture {
                dim: naga::ImageDimension::D2,
                arrayed: true,
                class: naga::ImageClass::Sampled {
                    kind: ScalarKind::Float,
                    multi: false,
                },
            },
            "texture_cube" => TypeInner::Texture {
                dim: naga::ImageDimension::Cube,
                arrayed: false,
                class: naga::ImageClass::Sampled {
                    kind: ScalarKind::Float,
                    multi: false,
                },
            },
            _ => {
                return Err(GError {
                    kind: GErrorKind::Other,
                    message: format!("未知的 gs 类型: '{}'", type_name),
                });
            }
        };

        let handle = module.types.insert(
            naga::Type {
                name: None,
                inner,
            },
            NagaSpan::UNDEFINED,
        );

        self.type_cache.insert(key, handle);
        Ok(handle)
    }

    /// 推断字段在结构体中的索引
    ///
    /// 使用常见着色器字段名的约定来推断索引。
    fn field_index(&self, name: &str) -> usize {
        match name.to_lowercase().as_str() {
            "x" | "r" => 0,
            "y" | "g" => 1,
            "z" | "b" => 2,
            "w" | "a" => 3,
            "xy" | "rg" => 0,
            "yz" | "gb" => 1,
            "zw" | "ab" => 2,
            "xyz" | "rgb" => 0,
            "yzw" | "gba" => 1,
            _ => 0,
        }
    }

    /// 将 TypeExpression 转换为类型名称字符串
    #[cfg(feature = "valkyrie-compiler")]
    fn type_expr_to_string(&self, ty: &TypeExpression) -> String {
        match ty {
            TypeExpression::Namepath(np) => {
                np.parts
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join("::")
            }
            TypeExpression::Generic(gen) => gen.name.name.clone(),
            TypeExpression::Tuple(tuple) => {
                let elements: Vec<String> =
                    tuple.elements.iter().map(|e| self.type_expr_to_string(e)).collect();
                format!("({})", elements.join(", "))
            }
            TypeExpression::Optional(opt) => {
                format!("{}?", self.type_expr_to_string(&opt.inner))
            }
            TypeExpression::Binary(bin) => {
                format!(
                    "{} {:?} {}",
                    self.type_expr_to_string(&bin.lhs),
                    bin.operator,
                    self.type_expr_to_string(&bin.rhs)
                )
            }
            TypeExpression::Unary(un) => {
                format!("{:?}{}", un.operator, self.type_expr_to_string(&un.base))
            }
            TypeExpression::Function(func) => {
                let params: Vec<String> =
                    func.params.iter().map(|p| self.type_expr_to_string(p)).collect();
                format!(
                    "({}) -> {}",
                    params.join(", "),
                    self.type_expr_to_string(&func.return_type)
                )
            }
            TypeExpression::AssociatedType(assoc) => {
                format!("{}::{}", assoc.base.name, assoc.name.name)
            }
            TypeExpression::QualifiedAssociatedType(qual) => {
                format!(
                    "<{} as {}>::{}",
                    self.type_expr_to_string(&qual.ty),
                    qual.trait_path
                        .parts
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>()
                        .join("::"),
                    qual.name.name
                )
            }
        }
    }

    /// 将 TermExpression 转换为可读字符串（用于调试和默认值提取）
    #[cfg(feature = "valkyrie-compiler")]
    fn expr_to_string(&self, expr: &TermExpression) -> String {
        match expr {
            TermExpression::NamePath(np) => np
                .parts
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join("::"),
            TermExpression::StringLiteral(sl) => sl
                .segments
                .iter()
                .filter_map(|seg| match seg {
                    oak_valkyrie::ast::StringSegment::Text(text_seg) => {
                        Some(text_seg.content.as_str())
                    }
                    _ => None,
                })
                .collect(),
            TermExpression::Bool { value, .. } => value.to_string(),
            TermExpression::Binary(node) => format!(
                "{} {:?} {}",
                self.expr_to_string(&node.lhs),
                node.operator,
                self.expr_to_string(&node.rhs)
            ),
            TermExpression::Unary(node) => {
                format!("{:?}{}", node.operator, self.expr_to_string(&node.base))
            }
            TermExpression::ApplyCall { callee, args, .. } => {
                let arg_strs: Vec<String> =
                    args.iter().map(|a| self.expr_to_string(a)).collect();
                format!("{}({})", self.expr_to_string(callee), arg_strs.join(", "))
            }
            TermExpression::DotCall {
                receiver, field, ..
            } => format!("{}.{}", self.expr_to_string(receiver), field.name),
            TermExpression::Paren { expr: inner, .. } => {
                format!("({})", self.expr_to_string(inner))
            }
            TermExpression::Object { callee, fields, .. } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(name, value)| {
                        match value {
                            Some(v) => format!("{}: {}", name.name, self.expr_to_string(v)),
                            None => name.name.clone(),
                        }
                    })
                    .collect();
                format!("{} {{ {} }}", self.expr_to_string(callee), field_strs.join(", "))
            }
            _ => format!("{:?}", std::mem::discriminant(expr)),
        }
    }

    /// 对齐偏移量
    fn align_offset(offset: u32, align: u32) -> u32 {
        if align == 0 {
            return offset;
        }
        (offset + align - 1) / align * align
    }
}

/// 类型大小和对齐信息
struct TypeSizeAlign {
    /// 类型大小（字节）
    size: u32,
    /// 类型对齐（字节）
    align: u32,
}

impl GslLowerer {
    /// 获取 gs 类型的大小和对齐信息
    ///
    /// 用于计算 uniform 缓冲区结构体的成员偏移量。
    fn type_size_align(&self, type_name: &str) -> TypeSizeAlign {
        let key = type_name.to_lowercase();
        match key.as_str() {
            "f32" | "float" | "i32" | "int" | "u32" | "uint" => TypeSizeAlign {
                size: 4,
                align: 4,
            },
            "vec2" | "vec2f" => TypeSizeAlign {
                size: 8,
                align: 8,
            },
            "vec3" | "vec3f" | "vector" => TypeSizeAlign {
                size: 12,
                align: 16,
            },
            "vec4" | "vec4f" | "color" => TypeSizeAlign {
                size: 16,
                align: 16,
            },
            "mat44" | "mat4x4" | "mat4x4f" => TypeSizeAlign {
                size: 64,
                align: 16,
            },
            "mat33" | "mat3x3" | "mat3x3f" => TypeSizeAlign {
                size: 48,
                align: 16,
            },
            _ => TypeSizeAlign {
                size: 16,
                align: 16,
            },
        }
    }
}

impl Default for GslLowerer {
    fn default() -> Self {
        Self::new()
    }
}
