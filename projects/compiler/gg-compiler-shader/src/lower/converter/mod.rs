//! gs AST → naga IR 转换器核心实现

use rustc_hash::FxHashMap;

use gg_core::{GError, GErrorKind, GResult};
use naga::{
    BinaryOperator, Binding, BuiltIn, Expression, FunctionArgument, FunctionResult, ImageClass, ImageDimension, SampleLevel,
    ScalarKind, Span as NagaSpan, SwizzleComponent, TypeInner, VectorSize,
};
use oak_valkyrie::{
    ast::{
        Attribute, Block, Let, LoopKind, MicroDeclaration, Pattern, ShaderDeclaration, Statement, StatementNode,
        StructureDeclaration, TermExpression, TypeExpression,
    },
    lexer::token_type::ValkyrieTokenType,
};

use crate::artifact::{BlendMode, CullMode, FallbackInfo, RenderStates};

use super::{
    EnabledKeywords, F32_SCALAR, GsEntryPoint, GsProperty, GsRenderState, GsUniformField, NamedExpressions, TypeSizeAlign,
    calc_std140_layout, name_path_to_string,
};

/// 装饰器解析结果
///
/// 从参数或返回值的注解中提取的绑定信息，
/// 支持 `@location`、`@builtin`、`@group`、`@binding` 等装饰器。
#[derive(Debug, Clone, Default)]
pub struct DecoratorInfo {
    /// `@location(N)` 中的位置编号
    pub location: Option<u32>,
    /// `@builtin(name)` 中的内置名称
    pub builtin: Option<String>,
    /// `@group(N)` 中的资源组编号
    pub group: Option<u32>,
    /// `@binding(N)` 中的绑定编号
    pub binding: Option<u32>,
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
    local_vars: FxHashMap<String, naga::Handle<naga::LocalVariable>>,
    /// 全局变量名到 naga 句柄的映射
    global_vars: FxHashMap<String, naga::Handle<naga::GlobalVariable>>,
    /// 类型缓存，避免重复创建
    type_cache: FxHashMap<String, naga::Handle<naga::Type>>,
    /// 下一个可用的 binding 编号
    next_binding: u32,
    /// 着色器声明中的顶层条目，用于查找结构体定义
    shader_items: Vec<StatementNode>,
    /// 变体条件编译启用的关键字集合
    enabled_keywords: EnabledKeywords,
    /// Uniform 结构体的类型句柄，用于字段索引查找
    uniform_type_handle: Option<naga::Handle<naga::Type>>,
    /// Uniform 结构体字段名到索引的映射
    uniform_field_index: FxHashMap<String, u32>,
    /// 自定义函数名到 naga 函数句柄的映射
    custom_functions: FxHashMap<String, naga::Handle<naga::Function>>,
    /// 根级结构体声明，用于查找 shader 块外的结构体定义
    root_structures: FxHashMap<String, oak_valkyrie::ast::StructureDeclaration>,
}

impl GslLowerer {
    /// 创建新的转换器
    pub fn new() -> Self {
        Self {
            local_vars: FxHashMap::default(),
            global_vars: FxHashMap::default(),
            type_cache: FxHashMap::default(),
            next_binding: 1,
            shader_items: Vec::new(),
            enabled_keywords: EnabledKeywords::default(),
            uniform_type_handle: None,
            uniform_field_index: FxHashMap::default(),
            custom_functions: FxHashMap::default(),
            root_structures: FxHashMap::default(),
        }
    }

    /// 创建带有指定启用关键字的转换器
    ///
    /// 用于变体条件编译，仅 lowered 带有 `@variant("keyword")` 注解
    /// 且 keyword 在启用集合中的语句。
    pub fn with_keywords(keywords: EnabledKeywords) -> Self {
        Self {
            local_vars: FxHashMap::default(),
            global_vars: FxHashMap::default(),
            type_cache: FxHashMap::default(),
            next_binding: 1,
            shader_items: Vec::new(),
            enabled_keywords: keywords,
            uniform_type_handle: None,
            uniform_field_index: FxHashMap::default(),
            custom_functions: FxHashMap::default(),
            root_structures: FxHashMap::default(),
        }
    }

    /// 将 oak-valkyrie 的 Shader AST 转换为 naga Module 及附加信息
    ///
    /// 执行两遍遍历：第一遍收集信息，第二遍生成 naga IR。
    /// 同时返回渲染状态和回退信息。
    pub fn lower(&mut self, shader: &ShaderDeclaration, root_structures: &FxHashMap<String, oak_valkyrie::ast::StructureDeclaration>) -> GResult<(naga::Module, RenderStates, Option<FallbackInfo>)> {
        self.local_vars.clear();
        self.global_vars.clear();
        self.type_cache.clear();
        self.next_binding = 1;
        self.shader_items = shader.items.clone();
        self.uniform_type_handle = None;
        self.uniform_field_index.clear();
        self.custom_functions.clear();
        self.root_structures = root_structures.clone();

        let mut module = naga::Module::default();

        let (properties, entry_points, uniform_fields, render_states, custom_micros, binding_decls) = self.collect_shader_items(shader)?;

        let uniform_buffer_ty = self.create_uniform_buffer(&uniform_fields, &mut module)?;

        for decl in &binding_decls {
            self.create_binding_global(decl, &mut module)?;
        }

        for prop in &properties {
            self.create_property_globals(prop, &mut module)?;
        }

        for micro_decl in &custom_micros {
            let func = self.lower_custom_function(micro_decl, &mut module, uniform_buffer_ty)?;
            let name = micro_decl.name.name.clone();
            let handle = module.functions.append(func, NagaSpan::UNDEFINED);
            self.custom_functions.insert(name, handle);
        }

        for ep in &entry_points {
            let entry_point = self.lower_entry_point(ep, &mut module, uniform_buffer_ty)?;
            module.entry_points.push(entry_point);
        }

        let render_states_result = Self::convert_render_states(&render_states);
        let fallback_result = Self::collect_fallback(shader);

        Ok((module, render_states_result, fallback_result))
    }

    /// 第一遍遍历：收集 shader 块中的所有声明
    fn collect_shader_items(
        &mut self,
        shader: &ShaderDeclaration,
    ) -> GResult<(Vec<GsProperty>, Vec<GsEntryPoint>, Vec<GsUniformField>, Vec<GsRenderState>, Vec<MicroDeclaration>, Vec<super::GsBindingDecl>)> {
        let mut properties = Vec::new();
        let mut entry_points = Vec::new();
        let mut uniform_fields = Vec::new();
        let mut render_states = Vec::new();
        let mut custom_micros: Vec<MicroDeclaration> = Vec::new();
        let mut binding_decls = Vec::new();

        for item in &shader.items {
            match item {
                StatementNode::Let(let_stmt) => {
                    let prop = self.collect_property(let_stmt)?;
                    properties.push(prop);
                }
                StatementNode::Micro(micro) => {
                    if self.is_entry_point(micro) {
                        if let Some(ep) = self.collect_entry_point(micro)? {
                            entry_points.push(ep);
                        }
                    }
                    else {
                        custom_micros.push((**micro).clone());
                    }
                }
                StatementNode::Structure(structure) => {
                    let fields = self.collect_uniform_fields(structure)?;
                    uniform_fields.extend(fields);
                }
                StatementNode::Namespace(namespace) => {
                    let name_str = name_path_to_string(&namespace.name);
                    let name_lower = name_str.to_lowercase();
                    if name_lower.contains("render_state") || name_lower.contains("renderstate") {
                        for inner_item in &namespace.items {
                            if let StatementNode::Let(let_stmt) = inner_item {
                                let rs_name = match &let_stmt.pattern {
                                    Pattern::Variable(v) => v.name.name.clone(),
                                    _ => continue,
                                };
                                let value = self.expr_to_string(&let_stmt.expr);
                                render_states.push(GsRenderState { name: rs_name, value });
                            }
                        }
                    }
                    else {
                        for inner_item in &namespace.items {
                            if let StatementNode::Let(let_stmt) = inner_item {
                                let prop = self.collect_property(let_stmt)?;
                                properties.push(prop);
                            }
                        }
                    }
                }
                StatementNode::UniformBinding(binding) => {
                    let type_name = self.type_expr_to_string(&binding.ty);
                    let decl = super::GsBindingDecl {
                        is_uniform: binding.is_uniform,
                        name: binding.name.name.clone(),
                        type_name,
                    };
                    binding_decls.push(decl);
                }
                _ => {}
            }
        }

        Ok((properties, entry_points, uniform_fields, render_states, custom_micros, binding_decls))
    }

    /// 从 let 语句中收集属性信息
    fn collect_property(&self, let_stmt: &Let) -> GResult<GsProperty> {
        let name = match &let_stmt.pattern {
            Pattern::Variable(v) => v.name.name.clone(),
            Pattern::Wildcard(_) => "_".to_string(),
            _ => {
                return Err(GError {
                    kind: GErrorKind::Other, message: format!("不支持的属性模式: {:?}", let_stmt.pattern)
                });
            }
        };

        let type_name = let_stmt.ty.as_ref().map(|ty| self.type_expr_to_string(ty)).unwrap_or_else(|| "unknown".to_string());

        let default_value = Some(self.expr_to_string(&let_stmt.expr));

        Ok(GsProperty { name, type_name, default_value })
    }

    /// 从 micro 声明中收集入口点信息
    fn collect_entry_point(&self, micro: &MicroDeclaration) -> GResult<Option<GsEntryPoint>> {
        let stage = self.detect_shader_stage(&micro.annotations, &micro.name.name)?;
        Ok(Some(GsEntryPoint { name: micro.name.name.clone(), stage, micro: micro.clone() }))
    }

    /// 判断 micro 声明是否为着色器入口点
    ///
    /// 如果 micro 带有 @vertex/@fragment/@compute 注解，
    /// 或函数名以 vs/vertex/fs/fragment/cs/compute 开头，则为入口点。
    fn is_entry_point(&self, micro: &MicroDeclaration) -> bool {
        for attr in &micro.annotations {
            let attr_name = attr.name.name.to_lowercase();
            match attr_name.as_str() {
                "vertex" | "fragment" | "compute" => return true,
                _ => {}
            }
        }
        let name_lower = micro.name.name.to_lowercase();
        name_lower.starts_with("vs")
            || name_lower.starts_with("vertex")
            || name_lower.starts_with("fs")
            || name_lower.starts_with("fragment")
            || name_lower.starts_with("cs")
            || name_lower.starts_with("compute")
    }

    /// 从注解或函数名检测着色器阶段
    fn detect_shader_stage(&self, annotations: &[Attribute], name: &str) -> GResult<naga::ShaderStage> {
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
        }
        else if name_lower.starts_with("fs") || name_lower.starts_with("fragment") {
            Ok(naga::ShaderStage::Fragment)
        }
        else if name_lower.starts_with("cs") || name_lower.starts_with("compute") {
            Ok(naga::ShaderStage::Compute)
        }
        else {
            Err(GError {
                kind: GErrorKind::Other,
                message: format!("无法确定着色器阶段: 函数 '{}' 缺少 @vertex/@fragment/@compute 注解", name),
            })
        }
    }

    /// 从结构体声明中收集 uniform 字段
    fn collect_uniform_fields(&self, structure: &StructureDeclaration) -> GResult<Vec<GsUniformField>> {
        let mut fields = Vec::new();
        for field in &structure.fields {
            let type_name = self.type_expr_to_string(&field.ty);
            fields.push(GsUniformField { name: field.name.name.clone(), type_name });
        }
        Ok(fields)
    }

    /// 从属性列表中解析装饰器信息
    ///
    /// 支持以下装饰器：
    /// - `@location(N)` → location = N
    /// - `@builtin(name)` → builtin = name
    /// - `@group(N)` → group = N
    /// - `@binding(N)` → binding = N
    /// - `@position` → builtin = "position"
    /// - `@vertex_index` → builtin = "vertex_index"
    /// - `@instance_index` → builtin = "instance_index"
    /// - `@global_invocation_id` → builtin = "global_invocation_id"
    pub fn parse_decorators(&self, attrs: &[Attribute]) -> DecoratorInfo {
        let mut info = DecoratorInfo::default();

        for attr in attrs {
            let attr_name = attr.name.name.to_lowercase();

            match attr_name.as_str() {
                "location" => {
                    if let Some(val) = self.extract_u32_from_args(&attr.args) {
                        info.location = Some(val);
                    }
                }
                "builtin" => {
                    if let Some(name) = self.extract_string_from_args(&attr.args) {
                        info.builtin = Some(name);
                    }
                }
                "group" => {
                    if let Some(val) = self.extract_u32_from_args(&attr.args) {
                        info.group = Some(val);
                    }
                }
                "binding" => {
                    if let Some(val) = self.extract_u32_from_args(&attr.args) {
                        info.binding = Some(val);
                    }
                }
                "position" => {
                    info.builtin = Some("position".to_string());
                }
                "vertex_index" => {
                    info.builtin = Some("vertex_index".to_string());
                }
                "instance_index" => {
                    info.builtin = Some("instance_index".to_string());
                }
                "global_invocation_id" => {
                    info.builtin = Some("global_invocation_id".to_string());
                }
                _ => {}
            }
        }

        info
    }

    /// 从属性参数中提取 u32 值
    fn extract_u32_from_args(&self, args: &[TermExpression]) -> Option<u32> {
        if args.is_empty() {
            return None;
        }
        let s = self.expr_to_string(&args[0]);
        s.parse::<u32>().ok()
    }

    /// 从属性参数中提取字符串值
    fn extract_string_from_args(&self, args: &[TermExpression]) -> Option<String> {
        if args.is_empty() {
            return None;
        }
        let s = self.expr_to_string(&args[0]);
        if s.is_empty() { None } else { Some(s) }
    }

    /// 将 DecoratorInfo 中的 builtin 字符串映射为 naga BuiltIn
    fn map_builtin_name(name: &str) -> Option<BuiltIn> {
        match name.to_lowercase().as_str() {
            "position" => Some(BuiltIn::Position { invariant: false }),
            "vertex_index" | "vertexid" => Some(BuiltIn::VertexIndex),
            "instance_index" | "instanceid" => Some(BuiltIn::InstanceIndex),
            "global_invocation_id" => Some(BuiltIn::GlobalInvocationId),
            "local_invocation_id" => Some(BuiltIn::LocalInvocationId),
            "local_invocation_index" => Some(BuiltIn::LocalInvocationIndex),
            "workgroup_id" => Some(BuiltIn::WorkGroupId),
            "num_workgroups" => Some(BuiltIn::NumWorkGroups),
            "frag_depth" => Some(BuiltIn::FragDepth),
            "front_facing" => Some(BuiltIn::FrontFacing),
            "sample_index" => Some(BuiltIn::SampleIndex),
            "sample_mask" => Some(BuiltIn::SampleMask),
            _ => None,
        }
    }

    /// 创建 uniform 缓冲区结构体和全局变量
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

        for (idx, field) in fields.iter().enumerate() {
            let ty = self.get_or_create_naga_type(&field.type_name, module)?;
            let size = self.type_size_align(&field.type_name);
            offset = super::types::align_offset(offset, size.align);
            members.push(naga::StructMember { name: Some(field.name.clone()), ty, binding: None, offset });
            self.uniform_field_index.insert(field.name.clone(), idx as u32);
            offset += size.size;
        }

        let struct_ty = module.types.insert(
            naga::Type { name: Some("Uniforms".to_string()), inner: TypeInner::Struct { members, span: offset } },
            NagaSpan::UNDEFINED,
        );

        let gv = module.global_variables.append(
            naga::GlobalVariable {
                name: Some("uniforms".to_string()),
                space: naga::AddressSpace::Uniform,
                binding: Some(naga::ResourceBinding { group: 0, binding: 0 }),
                ty: struct_ty,
                init: None,
                memory_decorations: naga::MemoryDecorations::empty(),
            },
            NagaSpan::UNDEFINED,
        );
        self.global_vars.insert("uniforms".to_string(), gv);
        self.type_cache.insert("Uniforms".to_string(), struct_ty);
        self.uniform_type_handle = Some(struct_ty);

        Ok(Some(struct_ty))
    }

    /// 为属性创建全局变量（sampler + texture）
    fn create_property_globals(&mut self, prop: &GsProperty, module: &mut naga::Module) -> GResult<()> {
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
                    binding: Some(naga::ResourceBinding { group: 1, binding: sampler_binding }),
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
                    binding: Some(naga::ResourceBinding { group: 1, binding: texture_binding }),
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
        let mut named_expressions = NamedExpressions::default();
        let mut body = naga::Block::new();

        let mut location_index: u32 = 0;

        for param in &ep.micro.params {
            let type_name = param.ty.as_ref().map(|ty| self.type_expr_to_string(ty)).unwrap_or_else(|| "f32".to_string());

            let param_ty = self.get_or_create_naga_type(&type_name, module)?;

            let binding =
                self.resolve_param_binding(&param.name.name, &type_name, ep.stage, &mut location_index, &param.annotations);

            let arg_index = function.arguments.len() as u32;
            function.arguments.push(FunctionArgument { name: Some(param.name.name.clone()), ty: param_ty, binding });

            let arg_expr = expressions.append(Expression::FunctionArgument(arg_index), NagaSpan::UNDEFINED);
            named_expressions.insert(arg_expr, param.name.name.clone());
        }

        if let Some(ret_type) = &ep.micro.return_type {
            let type_name = self.type_expr_to_string(ret_type);

            if let Some(struct_decl) = self.find_struct_declaration(&type_name) {
                let struct_ty = self.lower_struct_type(&struct_decl, module)?;
                function.result = Some(FunctionResult { ty: struct_ty, binding: None });
            }
            else {
                let result_ty = self.get_or_create_naga_type(&type_name, module)?;
                let result_binding =
                    self.resolve_result_binding(&type_name, ep.stage, &mut location_index, &ep.micro.annotations);
                function.result = Some(FunctionResult { ty: result_ty, binding: result_binding });
            }
        }

        let _ = uniform_buffer_ty;

        self.lower_block(&ep.micro.body, module, &mut function, &mut expressions, &mut named_expressions, &mut body)?;

        function.expressions = expressions;
        function.named_expressions = named_expressions;
        function.body = body;

        let workgroup_size = self.parse_workgroup_size(&ep.micro.annotations, ep.stage);

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

    /// 将自定义 micro 函数降级为 naga Function
    ///
    /// 自定义函数不是着色器入口点，没有绑定信息。
    /// 它们可以被入口点或其他自定义函数调用。
    fn lower_custom_function(
        &mut self,
        micro_decl: &MicroDeclaration,
        module: &mut naga::Module,
        _uniform_buffer_ty: Option<naga::Handle<naga::Type>>,
    ) -> GResult<naga::Function> {
        self.local_vars.clear();

        let mut function = naga::Function::default();
        function.name = Some(micro_decl.name.name.clone());

        let mut expressions = naga::Arena::new();
        let mut named_expressions = NamedExpressions::default();
        let mut body = naga::Block::new();

        for param in &micro_decl.params {
            let type_name = param.ty.as_ref().map(|ty| self.type_expr_to_string(ty)).unwrap_or_else(|| "f32".to_string());
            let param_ty = self.get_or_create_naga_type(&type_name, module)?;

            let arg_index = function.arguments.len() as u32;
            function.arguments.push(FunctionArgument { name: Some(param.name.name.clone()), ty: param_ty, binding: None });

            let arg_expr = expressions.append(Expression::FunctionArgument(arg_index), NagaSpan::UNDEFINED);
            named_expressions.insert(arg_expr, param.name.name.clone());
        }

        if let Some(ret_type) = &micro_decl.return_type {
            let type_name = self.type_expr_to_string(ret_type);
            if let Some(struct_decl) = self.find_struct_declaration(&type_name) {
                let struct_ty = self.lower_struct_type(&struct_decl, module)?;
                function.result = Some(FunctionResult { ty: struct_ty, binding: None });
            }
            else {
                let result_ty = self.get_or_create_naga_type(&type_name, module)?;
                function.result = Some(FunctionResult { ty: result_ty, binding: None });
            }
        }

        self.lower_block(&micro_decl.body, module, &mut function, &mut expressions, &mut named_expressions, &mut body)?;

        function.expressions = expressions;
        function.named_expressions = named_expressions;
        function.body = body;

        Ok(function)
    }

    /// 从注解中解析 workgroup_size
    ///
    /// 查找 `@workgroup_size(x, y, z)` 注解，解析三个整数参数。
    /// 对于计算着色器，默认为 `[1, 1, 1]`；其他阶段为 `[0, 0, 0]`。
    fn parse_workgroup_size(&self, annotations: &[Attribute], stage: naga::ShaderStage) -> [u32; 3] {
        if stage != naga::ShaderStage::Compute {
            return [0u32; 3];
        }

        for attr in annotations {
            if attr.name.name.to_lowercase() == "workgroup_size" {
                let x = self.extract_u32_from_args(&attr.args).unwrap_or(1);
                let y = if attr.args.len() > 1 { self.extract_u32_from_args(&attr.args[1..2]).unwrap_or(1) } else { 1 };
                let z = if attr.args.len() > 2 { self.extract_u32_from_args(&attr.args[2..3]).unwrap_or(1) } else { 1 };
                return [x, y, z];
            }
        }

        [1u32; 3]
    }

    /// 创建 Location 绑定
    fn make_location_binding(location: u32) -> Binding {
        Binding::Location { location, interpolation: None, sampling: None, blend_src: None, per_primitive: false }
    }

    /// 解析函数参数的绑定（location 或 builtin）
    ///
    /// 首先尝试从注解中解析装饰器信息，如果装饰器提供了绑定信息则使用之；
    /// 否则回退到基于名称的推断逻辑。
    fn resolve_param_binding(
        &self,
        param_name: &str,
        type_name: &str,
        stage: naga::ShaderStage,
        location_index: &mut u32,
        annotations: &[Attribute],
    ) -> Option<Binding> {
        let decorators = self.parse_decorators(annotations);

        if let Some(ref builtin_name) = decorators.builtin {
            if let Some(builtin) = Self::map_builtin_name(builtin_name) {
                return Some(Binding::BuiltIn(builtin));
            }
        }

        if let Some(loc) = decorators.location {
            return Some(Self::make_location_binding(loc));
        }

        self.infer_param_binding_fallback(param_name, type_name, stage, location_index)
    }

    /// 基于名称推断函数参数的绑定（回退逻辑）
    fn infer_param_binding_fallback(
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
            Some(Self::make_location_binding(loc))
        }
        else if stage == naga::ShaderStage::Fragment {
            match name_lower.as_str() {
                "clip_position" | "position" if type_lower.contains("vec4") => {
                    return Some(Binding::BuiltIn(BuiltIn::Position { invariant: false }));
                }
                _ => {}
            }
            let loc = *location_index;
            *location_index += 1;
            Some(Self::make_location_binding(loc))
        }
        else {
            let loc = *location_index;
            *location_index += 1;
            Some(Self::make_location_binding(loc))
        }
    }

    /// 解析函数返回值的绑定
    ///
    /// 首先尝试从注解中解析装饰器信息，如果装饰器提供了绑定信息则使用之；
    /// 否则回退到基于阶段的推断逻辑。
    fn resolve_result_binding(
        &self,
        type_name: &str,
        stage: naga::ShaderStage,
        location_index: &mut u32,
        annotations: &[Attribute],
    ) -> Option<Binding> {
        let decorators = self.parse_decorators(annotations);

        if let Some(ref builtin_name) = decorators.builtin {
            if let Some(builtin) = Self::map_builtin_name(builtin_name) {
                return Some(Binding::BuiltIn(builtin));
            }
        }

        if let Some(loc) = decorators.location {
            return Some(Self::make_location_binding(loc));
        }

        self.infer_result_binding_fallback(type_name, stage, location_index)
    }

    /// 基于阶段推断函数返回值的绑定（回退逻辑）
    fn infer_result_binding_fallback(
        &self,
        _type_name: &str,
        stage: naga::ShaderStage,
        location_index: &mut u32,
    ) -> Option<Binding> {
        if stage == naga::ShaderStage::Vertex {
            Some(Binding::BuiltIn(BuiltIn::Position { invariant: false }))
        }
        else if stage == naga::ShaderStage::Fragment {
            let loc = *location_index;
            *location_index += 1;
            Some(Self::make_location_binding(loc))
        }
        else {
            None
        }
    }

    fn create_binding_global(&mut self, decl: &super::GsBindingDecl, module: &mut naga::Module) -> GResult<()> {
        if decl.is_uniform {
            let struct_name = &decl.type_name;
            let struct_decl = self.find_struct_declaration(struct_name);
            let fields = if let Some(ref s) = struct_decl {
                self.collect_uniform_fields(s)?
            } else {
                Vec::new()
            };
            if !fields.is_empty() {
                self.create_uniform_buffer(&fields, module)?;
            }
        } else {
            let type_lower = decl.type_name.to_lowercase();
            if type_lower == "sampler" {
                let sampler_ty = self.get_or_create_naga_type("sampler", module)?;
                let binding = self.next_binding;
                self.next_binding += 1;
                let gv = module.global_variables.append(
                    naga::GlobalVariable {
                        name: Some(decl.name.clone()),
                        space: naga::AddressSpace::Handle,
                        binding: Some(naga::ResourceBinding { group: 1, binding }),
                        ty: sampler_ty,
                        init: None,
                        memory_decorations: naga::MemoryDecorations::empty(),
                    },
                    NagaSpan::UNDEFINED,
                );
                self.global_vars.insert(decl.name.clone(), gv);
            } else if type_lower.starts_with("texture") {
                let texture_ty = self.get_or_create_naga_type(&decl.type_name, module)?;
                let binding = self.next_binding;
                self.next_binding += 1;
                let gv = module.global_variables.append(
                    naga::GlobalVariable {
                        name: Some(decl.name.clone()),
                        space: naga::AddressSpace::Handle,
                        binding: Some(naga::ResourceBinding { group: 1, binding }),
                        ty: texture_ty,
                        init: None,
                        memory_decorations: naga::MemoryDecorations::empty(),
                    },
                    NagaSpan::UNDEFINED,
                );
                self.global_vars.insert(decl.name.clone(), gv);
            }
        }
        Ok(())
    }

    /// 在着色器声明中查找结构体定义
    ///
    /// 根据名称在 `shader_items` 中搜索匹配的结构体声明，
    /// 用于入口点返回类型为结构体时的类型创建。
    fn find_struct_declaration(&self, name: &str) -> Option<StructureDeclaration> {
        self.shader_items.iter().find_map(|item| {
            if let StatementNode::Structure(s) = item {
                if s.name.name == name {
                    return Some((**s).clone());
                }
            }
            None
        }).or_else(|| {
            self.root_structures.get(name).cloned()
        })
    }

    /// 从结构体声明创建 naga 类型
    ///
    /// 遍历结构体的每个字段，解析其注解中的 `@location(N)` 或 `@builtin(name)` 装饰器，
    /// 为每个字段生成正确的 naga 绑定信息，并使用 std140 布局规则计算字段偏移量和结构体大小。
    fn lower_struct_type(
        &mut self,
        struct_decl: &StructureDeclaration,
        module: &mut naga::Module,
    ) -> GResult<naga::Handle<naga::Type>> {
        let mut field_pairs = Vec::with_capacity(struct_decl.fields.len());
        for field in &struct_decl.fields {
            let field_type_name = self.type_expr_to_string(&field.ty);
            field_pairs.push((field.name.name.clone(), field_type_name));
        }

        let layout = calc_std140_layout(&field_pairs, |type_name| self.type_size_align(type_name));

        let mut members = Vec::with_capacity(struct_decl.fields.len());

        for (i, field) in struct_decl.fields.iter().enumerate() {
            let field_type_name = &field_pairs[i].1;
            let ty = self.get_or_create_naga_type(field_type_name, module)?;
            let binding = self.resolve_member_binding(&field.annotations, i);
            members.push(naga::StructMember {
                name: Some(field.name.name.clone()),
                ty,
                binding,
                offset: layout.field_offsets[i],
            });
        }

        let struct_type =
            naga::Type { name: Some(struct_decl.name.name.clone()), inner: TypeInner::Struct { members, span: layout.size } };

        let handle = module.types.insert(struct_type, NagaSpan::UNDEFINED);
        self.type_cache.insert(struct_decl.name.name.to_lowercase(), handle);
        Ok(handle)
    }

    /// 解析结构体成员的绑定信息
    ///
    /// 从字段的注解中提取 `@location(N)` 或 `@builtin(name)` 装饰器，
    /// 生成对应的 naga 绑定。如果未指定装饰器，则使用字段索引作为 location。
    fn resolve_member_binding(&self, annotations: &[Attribute], index: usize) -> Option<Binding> {
        let decorators = self.parse_decorators(annotations);

        if let Some(ref builtin_name) = decorators.builtin {
            if let Some(builtin) = Self::map_builtin_name(builtin_name) {
                return Some(Binding::BuiltIn(builtin));
            }
        }

        if let Some(loc) = decorators.location {
            return Some(Self::make_location_binding(loc));
        }

        let location = index as u32;
        Some(Self::make_location_binding(location))
    }

    /// 转换语句块
    fn lower_block(
        &mut self,
        block: &Block,
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
        body: &mut naga::Block,
    ) -> GResult<()> {
        for stmt in &block.statements {
            self.lower_statement(stmt, module, function, expressions, named_expressions, body)?;
        }
        Ok(())
    }

    /// 转换单条语句
    fn lower_statement(
        &mut self,
        stmt: &Statement,
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
        body: &mut naga::Block,
    ) -> GResult<()> {
        let annotations = match stmt {
            Statement::Let(let_stmt) => &let_stmt.annotations,
            Statement::ExprStmt(expr_stmt) => &expr_stmt.annotations,
        };

        if let Some(variant_keyword) = self.extract_variant_keyword(annotations) {
            if !self.enabled_keywords.contains(&variant_keyword) {
                return Ok(());
            }
        }

        match stmt {
            Statement::Let(let_stmt) => {
                self.lower_let_statement(let_stmt, module, function, expressions, named_expressions, body)?;
            }
            Statement::ExprStmt(expr_stmt) => {
                let _ = self.lower_expression(&expr_stmt.expr, module, function, expressions, named_expressions, body)?;
            }
        }
        Ok(())
    }

    /// 从注解列表中提取 @variant 关键字
    ///
    /// 查找 `@variant("keyword")` 注解，返回其中的关键字字符串。
    /// 如果没有 @variant 注解则返回 None。
    fn extract_variant_keyword(&self, annotations: &[Attribute]) -> Option<String> {
        for attr in annotations {
            if attr.name.name.to_lowercase() == "variant" {
                if let Some(keyword) = self.extract_string_from_args(&attr.args) {
                    return Some(keyword);
                }
            }
        }
        None
    }

    /// 转换 let 绑定语句
    fn lower_let_statement(
        &mut self,
        let_stmt: &Let,
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
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

        let value_handle = self.lower_expression(&let_stmt.expr, module, function, expressions, named_expressions, body)?;

        if let_stmt.is_mutable {
            let type_name = let_stmt.ty.as_ref().map(|ty| self.type_expr_to_string(ty)).unwrap_or_else(|| "f32".to_string());

            let var_ty = self.get_or_create_naga_type(&type_name, module)?;

            let local_handle = function
                .local_variables
                .append(naga::LocalVariable { name: Some(var_name.clone()), ty: var_ty, init: None }, NagaSpan::UNDEFINED);
            self.local_vars.insert(var_name.clone(), local_handle);

            let var_expr = expressions.append(Expression::LocalVariable(local_handle), NagaSpan::UNDEFINED);

            body.push(naga::Statement::Store { pointer: var_expr, value: value_handle }, NagaSpan::UNDEFINED);
        }
        else {
            named_expressions.insert(value_handle, var_name);
        }

        Ok(())
    }

    /// 创建一个 f32 零值表达式
    fn make_zero_value(
        module: &mut naga::Module,
        expressions: &mut naga::Arena<Expression>,
    ) -> GResult<naga::Handle<Expression>> {
        let ty = module.types.insert(naga::Type { name: None, inner: TypeInner::Scalar(F32_SCALAR) }, NagaSpan::UNDEFINED);
        Ok(expressions.append(Expression::ZeroValue(ty), NagaSpan::UNDEFINED))
    }

    /// 转换表达式
    fn lower_expression(
        &mut self,
        expr: &TermExpression,
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        match expr {
            TermExpression::NamePath(name_path) => {
                if name_path.parts.len() == 1 {
                    let name = &name_path.parts[0].name;
                    if name == "discard" {
                        body.push(naga::Statement::Kill, NagaSpan::UNDEFINED);
                        return Self::make_zero_value(module, expressions);
                    }
                    self.resolve_name(name, expressions, named_expressions)
                }
                else {
                    let first = &name_path.parts[0].name;
                    let first_expr = self.resolve_name(first, expressions, named_expressions)?;
                    let mut current = first_expr;
                    let mut current_base = first.clone();
                    for part in &name_path.parts[1..] {
                        let idx = self.resolve_field_index(&current_base, &part.name, module);
                        current =
                            expressions.append(Expression::AccessIndex { base: current, index: idx }, NagaSpan::UNDEFINED);
                        current_base = part.name.clone();
                    }
                    Ok(current)
                }
            }

            TermExpression::Binary(node) => {
                let left = self.lower_expression(&node.lhs, module, function, expressions, named_expressions, body)?;
                let right = self.lower_expression(&node.rhs, module, function, expressions, named_expressions, body)?;
                let op = self.map_binary_operator(&node.operator)?;
                Ok(expressions.append(Expression::Binary { op, left, right }, NagaSpan::UNDEFINED))
            }

            TermExpression::Unary(node) => {
                let operand = self.lower_expression(&node.base, module, function, expressions, named_expressions, body)?;
                let op = self.map_unary_operator(&node.operator)?;
                Ok(expressions.append(Expression::Unary { op, expr: operand }, NagaSpan::UNDEFINED))
            }

            TermExpression::ApplyCall { callee, args, .. } => {
                self.lower_call(callee, args, module, function, expressions, named_expressions, body)
            }

            TermExpression::DotCall { receiver, field, .. } => {
                let base = self.lower_expression(receiver, module, function, expressions, named_expressions, body)?;
                let field_name = &field.name;
                if field_name.len() > 1 && Self::is_swizzle_pattern(field_name) {
                    let (size, pattern) = Self::parse_swizzle(field_name);
                    Ok(expressions.append(Expression::Swizzle { size, vector: base, pattern }, NagaSpan::UNDEFINED))
                }
                else if field_name.len() == 1
                    && matches!(
                        field_name.chars().next().unwrap().to_ascii_lowercase(),
                        'x' | 'y' | 'z' | 'w' | 'r' | 'g' | 'b' | 'a'
                    )
                {
                    let idx = self.swizzle_component_index(field_name);
                    Ok(expressions.append(Expression::AccessIndex { base, index: idx }, NagaSpan::UNDEFINED))
                }
                else {
                    let base_name = self.extract_receiver_name(receiver);
                    let idx = self.resolve_field_index(&base_name, field_name, module);
                    Ok(expressions.append(Expression::AccessIndex { base, index: idx }, NagaSpan::UNDEFINED))
                }
            }

            TermExpression::Index { receiver, index, .. } => {
                let base = self.lower_expression(receiver, module, function, expressions, named_expressions, body)?;
                if let Some(const_idx) = self.extract_const_index(index) {
                    Ok(expressions.append(Expression::AccessIndex { base, index: const_idx }, NagaSpan::UNDEFINED))
                }
                else {
                    let index_expr = self.lower_expression(index, module, function, expressions, named_expressions, body)?;
                    Ok(expressions.append(Expression::Access { base, index: index_expr }, NagaSpan::UNDEFINED))
                }
            }

            TermExpression::Paren { expr: inner, .. } => {
                self.lower_expression(inner, module, function, expressions, named_expressions, body)
            }

            TermExpression::Return(ret) => {
                if let Some(return_expr) = &ret.base {
                    let value = self.lower_expression(return_expr, module, function, expressions, named_expressions, body)?;
                    body.push(naga::Statement::Return { value: Some(value) }, NagaSpan::UNDEFINED);
                }
                else {
                    body.push(naga::Statement::Return { value: None }, NagaSpan::UNDEFINED);
                }
                Self::make_zero_value(module, expressions)
            }

            TermExpression::Bool { value, .. } => {
                Ok(expressions.append(Expression::Literal(naga::Literal::Bool(*value)), NagaSpan::UNDEFINED))
            }

            TermExpression::StringLiteral(string_literal) => {
                let content: String = string_literal
                    .segments
                    .iter()
                    .filter_map(|seg| match seg {
                        oak_valkyrie::ast::StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                        _ => None,
                    })
                    .collect();

                if let Ok(float_val) = content.parse::<f32>() {
                    Ok(expressions.append(Expression::Literal(naga::Literal::F32(float_val)), NagaSpan::UNDEFINED))
                }
                else if let Ok(int_val) = content.parse::<i32>() {
                    Ok(expressions.append(Expression::Literal(naga::Literal::I32(int_val)), NagaSpan::UNDEFINED))
                }
                else {
                    Err(GError {
                        kind: GErrorKind::Other, message: format!("着色器中不支持字符串字面量: '{}'", content)
                    })
                }
            }

            TermExpression::If { condition, then_branch, else_branch, .. } => {
                let cond = self.lower_expression(condition, module, function, expressions, named_expressions, body)?;
                let mut then_body = naga::Block::new();
                self.lower_block(then_branch, module, function, expressions, named_expressions, &mut then_body)?;

                if let Some(else_block) = else_branch {
                    let mut else_body = naga::Block::new();
                    self.lower_block(else_block, module, function, expressions, named_expressions, &mut else_body)?;
                    body.push(
                        naga::Statement::If { condition: cond, accept: then_body, reject: else_body },
                        NagaSpan::UNDEFINED,
                    );
                }
                else {
                    body.push(
                        naga::Statement::If { condition: cond, accept: then_body, reject: naga::Block::new() },
                        NagaSpan::UNDEFINED,
                    );
                }
                Self::make_zero_value(module, expressions)
            }

            TermExpression::Loop { kind, condition, body: loop_body, .. } => {
                match kind {
                    LoopKind::For => {
                        let mut loop_body_block = naga::Block::new();
                        let continuing_block = naga::Block::new();

                        if let Some(cond) = condition {
                            let cond_expr = self.lower_expression(
                                cond,
                                module,
                                function,
                                expressions,
                                named_expressions,
                                &mut loop_body_block,
                            )?;
                            let not_cond = expressions.append(
                                Expression::Unary { op: naga::UnaryOperator::LogicalNot, expr: cond_expr },
                                NagaSpan::UNDEFINED,
                            );
                            let mut break_block = naga::Block::new();
                            break_block.push(naga::Statement::Break, NagaSpan::UNDEFINED);
                            loop_body_block.push(
                                naga::Statement::If { condition: not_cond, accept: break_block, reject: naga::Block::new() },
                                NagaSpan::UNDEFINED,
                            );
                        }

                        self.lower_block(loop_body, module, function, expressions, named_expressions, &mut loop_body_block)?;

                        body.push(
                            naga::Statement::Loop { body: loop_body_block, continuing: continuing_block, break_if: None },
                            NagaSpan::UNDEFINED,
                        );
                    }
                    LoopKind::Loop => {
                        let mut loop_body_block = naga::Block::new();
                        let continuing_block = naga::Block::new();

                        if let Some(cond) = condition {
                            let cond_expr = self.lower_expression(
                                cond,
                                module,
                                function,
                                expressions,
                                named_expressions,
                                &mut loop_body_block,
                            )?;
                            let not_cond = expressions.append(
                                Expression::Unary { op: naga::UnaryOperator::LogicalNot, expr: cond_expr },
                                NagaSpan::UNDEFINED,
                            );
                            let mut break_block = naga::Block::new();
                            break_block.push(naga::Statement::Break, NagaSpan::UNDEFINED);
                            loop_body_block.push(
                                naga::Statement::If { condition: not_cond, accept: break_block, reject: naga::Block::new() },
                                NagaSpan::UNDEFINED,
                            );
                        }

                        self.lower_block(loop_body, module, function, expressions, named_expressions, &mut loop_body_block)?;

                        body.push(
                            naga::Statement::Loop { body: loop_body_block, continuing: continuing_block, break_if: None },
                            NagaSpan::UNDEFINED,
                        );
                    }
                    LoopKind::While => {
                        let mut loop_body_block = naga::Block::new();
                        let continuing_block = naga::Block::new();

                        if let Some(cond) = condition {
                            let cond_expr = self.lower_expression(
                                cond,
                                module,
                                function,
                                expressions,
                                named_expressions,
                                &mut loop_body_block,
                            )?;
                            let not_cond = expressions.append(
                                Expression::Unary { op: naga::UnaryOperator::LogicalNot, expr: cond_expr },
                                NagaSpan::UNDEFINED,
                            );
                            let mut break_block = naga::Block::new();
                            break_block.push(naga::Statement::Break, NagaSpan::UNDEFINED);
                            loop_body_block.push(
                                naga::Statement::If { condition: not_cond, accept: break_block, reject: naga::Block::new() },
                                NagaSpan::UNDEFINED,
                            );
                        }

                        self.lower_block(loop_body, module, function, expressions, named_expressions, &mut loop_body_block)?;

                        body.push(
                            naga::Statement::Loop { body: loop_body_block, continuing: continuing_block, break_if: None },
                            NagaSpan::UNDEFINED,
                        );
                    }
                }
                Self::make_zero_value(module, expressions)
            }

            TermExpression::Block(block) => {
                self.lower_block(block, module, function, expressions, named_expressions, body)?;
                Self::make_zero_value(module, expressions)
            }

            TermExpression::Break(_) => {
                body.push(naga::Statement::Break, NagaSpan::UNDEFINED);
                Self::make_zero_value(module, expressions)
            }

            TermExpression::Continue(_) => {
                body.push(naga::Statement::Continue, NagaSpan::UNDEFINED);
                Self::make_zero_value(module, expressions)
            }

            _ => Err(GError {
                kind: GErrorKind::Other,
                message: format!("暂不支持的表达式类型: {:?}", std::mem::discriminant(expr)),
            }),
        }
    }

    /// 转换函数调用表达式
    fn lower_call(
        &mut self,
        callee: &TermExpression,
        args: &[TermExpression],
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        let func_name = self.extract_callee_name(callee);

        match func_name.as_deref() {
            Some("texture_sample") | Some("textureSample") => {
                self.lower_texture_sample(args, module, function, expressions, named_expressions, body)
            }
            Some("texture_sample_lod") | Some("textureSampleLod") => {
                self.lower_texture_sample_lod(args, module, function, expressions, named_expressions, body)
            }
            Some("texture_sample_compare") | Some("textureSampleCompare") => {
                self.lower_texture_sample_compare(args, module, function, expressions, named_expressions, body)
            }
            Some("vec2") | Some("vec2f") => {
                self.lower_vec_constructor(2, args, module, function, expressions, named_expressions, body)
            }
            Some("vec3") | Some("vec3f") => {
                self.lower_vec_constructor(3, args, module, function, expressions, named_expressions, body)
            }
            Some("vec4") | Some("vec4f") => {
                self.lower_vec_constructor(4, args, module, function, expressions, named_expressions, body)
            }
            Some("normalize") => self.lower_math_func(
                naga::MathFunction::Normalize,
                args,
                module,
                function,
                expressions,
                named_expressions,
                body,
            ),
            Some("length") => {
                self.lower_math_func(naga::MathFunction::Length, args, module, function, expressions, named_expressions, body)
            }
            Some("dot") => {
                self.lower_math_func(naga::MathFunction::Dot, args, module, function, expressions, named_expressions, body)
            }
            Some("mul") => {
                if args.len() == 2 {
                    let left = self.lower_expression(&args[0], module, function, expressions, named_expressions, body)?;
                    let right = self.lower_expression(&args[1], module, function, expressions, named_expressions, body)?;
                    Ok(expressions
                        .append(Expression::Binary { op: BinaryOperator::Multiply, left, right }, NagaSpan::UNDEFINED))
                }
                else {
                    Err(GError {
                        kind: GErrorKind::Other, message: format!("mul() 需要 2 个参数，得到 {}", args.len())
                    })
                }
            }
            Some("max") => {
                self.lower_math_func(naga::MathFunction::Max, args, module, function, expressions, named_expressions, body)
            }
            Some("min") => {
                self.lower_math_func(naga::MathFunction::Min, args, module, function, expressions, named_expressions, body)
            }
            Some("clamp") => {
                self.lower_math_func(naga::MathFunction::Clamp, args, module, function, expressions, named_expressions, body)
            }
            Some("mix") => {
                self.lower_math_func(naga::MathFunction::Mix, args, module, function, expressions, named_expressions, body)
            }
            Some("pow") => {
                self.lower_math_func(naga::MathFunction::Pow, args, module, function, expressions, named_expressions, body)
            }
            Some("abs") => {
                self.lower_math_func(naga::MathFunction::Abs, args, module, function, expressions, named_expressions, body)
            }
            Some("sqrt") => {
                self.lower_math_func(naga::MathFunction::Sqrt, args, module, function, expressions, named_expressions, body)
            }
            Some("floor") => {
                self.lower_math_func(naga::MathFunction::Floor, args, module, function, expressions, named_expressions, body)
            }
            Some("ceil") => {
                self.lower_math_func(naga::MathFunction::Ceil, args, module, function, expressions, named_expressions, body)
            }
            Some("fract") => {
                self.lower_math_func(naga::MathFunction::Fract, args, module, function, expressions, named_expressions, body)
            }
            Some("step") => {
                self.lower_math_func(naga::MathFunction::Step, args, module, function, expressions, named_expressions, body)
            }
            Some("smoothstep") => self.lower_math_func(
                naga::MathFunction::SmoothStep,
                args,
                module,
                function,
                expressions,
                named_expressions,
                body,
            ),
            Some("reflect") => {
                self.lower_math_func(naga::MathFunction::Reflect, args, module, function, expressions, named_expressions, body)
            }
            Some("cross") => {
                self.lower_math_func(naga::MathFunction::Cross, args, module, function, expressions, named_expressions, body)
            }
            Some("transpose") => self.lower_math_func(
                naga::MathFunction::Transpose,
                args,
                module,
                function,
                expressions,
                named_expressions,
                body,
            ),
            Some("determinant") => self.lower_math_func(
                naga::MathFunction::Determinant,
                args,
                module,
                function,
                expressions,
                named_expressions,
                body,
            ),
            Some("inverse") => {
                self.lower_math_func(naga::MathFunction::Inverse, args, module, function, expressions, named_expressions, body)
            }
            Some("sign") => {
                self.lower_math_func(naga::MathFunction::Sign, args, module, function, expressions, named_expressions, body)
            }
            Some("sin") => {
                self.lower_math_func(naga::MathFunction::Sin, args, module, function, expressions, named_expressions, body)
            }
            Some("cos") => {
                self.lower_math_func(naga::MathFunction::Cos, args, module, function, expressions, named_expressions, body)
            }
            Some("tan") => {
                self.lower_math_func(naga::MathFunction::Tan, args, module, function, expressions, named_expressions, body)
            }
            Some("saturate") => {
                self.lower_math_func(naga::MathFunction::Saturate, args, module, function, expressions, named_expressions, body)
            }
            Some("asin") => {
                self.lower_math_func(naga::MathFunction::Asin, args, module, function, expressions, named_expressions, body)
            }
            Some("acos") => {
                self.lower_math_func(naga::MathFunction::Acos, args, module, function, expressions, named_expressions, body)
            }
            Some("atan") => {
                self.lower_math_func(naga::MathFunction::Atan, args, module, function, expressions, named_expressions, body)
            }
            Some("atan2") => {
                self.lower_math_func(naga::MathFunction::Atan2, args, module, function, expressions, named_expressions, body)
            }
            Some("exp") => {
                self.lower_math_func(naga::MathFunction::Exp, args, module, function, expressions, named_expressions, body)
            }
            Some("log") => {
                self.lower_math_func(naga::MathFunction::Log, args, module, function, expressions, named_expressions, body)
            }
            Some("log2") => {
                self.lower_math_func(naga::MathFunction::Log2, args, module, function, expressions, named_expressions, body)
            }
            Some("exp2") => {
                self.lower_math_func(naga::MathFunction::Exp2, args, module, function, expressions, named_expressions, body)
            }
            Some("distance") => {
                self.lower_math_func(naga::MathFunction::Distance, args, module, function, expressions, named_expressions, body)
            }
            Some("refract") => {
                self.lower_math_func(naga::MathFunction::Refract, args, module, function, expressions, named_expressions, body)
            }
            Some("faceForward") | Some("faceforward") => self.lower_math_func(
                naga::MathFunction::FaceForward,
                args,
                module,
                function,
                expressions,
                named_expressions,
                body,
            ),
            Some("select") => {
                if args.len() == 3 {
                    let false_val = self.lower_expression(&args[0], module, function, expressions, named_expressions, body)?;
                    let true_val = self.lower_expression(&args[1], module, function, expressions, named_expressions, body)?;
                    let condition = self.lower_expression(&args[2], module, function, expressions, named_expressions, body)?;
                    Ok(expressions
                        .append(Expression::Select { condition, accept: true_val, reject: false_val }, NagaSpan::UNDEFINED))
                }
                else {
                    Err(GError {
                        kind: GErrorKind::Other,
                        message: format!("select() 需要 3 个参数（false_val, true_val, condition），得到 {}", args.len()),
                    })
                }
            }
            Some("countOneBits")
            | Some("countTrailingZeros")
            | Some("countLeadingZeros")
            | Some("reverseBits")
            | Some("firstLeadingBit")
            | Some("firstTrailingBit") => {
                let func = match func_name.as_deref() {
                    Some("countOneBits") => naga::MathFunction::CountOneBits,
                    Some("countTrailingZeros") => naga::MathFunction::CountTrailingZeros,
                    Some("countLeadingZeros") => naga::MathFunction::CountLeadingZeros,
                    Some("reverseBits") => naga::MathFunction::ReverseBits,
                    Some("firstLeadingBit") => naga::MathFunction::FirstLeadingBit,
                    Some("firstTrailingBit") => naga::MathFunction::FirstTrailingBit,
                    _ => unreachable!(),
                };
                self.lower_math_func(func, args, module, function, expressions, named_expressions, body)
            }
            Some("arrayLength") => {
                if args.len() == 1 {
                    let pointer = self.lower_expression(&args[0], module, function, expressions, named_expressions, body)?;
                    Ok(expressions.append(Expression::ArrayLength(pointer), NagaSpan::UNDEFINED))
                }
                else {
                    Err(GError {
                        kind: GErrorKind::Other,
                        message: format!("arrayLength() 需要 1 个参数，得到 {}", args.len()),
                    })
                }
            }
            Some("storageBarrier") => {
                body.push(naga::Statement::MemoryBarrier(naga::Barrier::STORAGE), NagaSpan::UNDEFINED);
                Self::make_zero_value(module, expressions)
            }
            Some("workgroupBarrier") => {
                body.push(naga::Statement::ControlBarrier(naga::Barrier::WORK_GROUP), NagaSpan::UNDEFINED);
                Self::make_zero_value(module, expressions)
            }
            _ => {
                if let Some(name) = func_name.as_deref() {
                    if let Some(&func_handle) = self.custom_functions.get(name) {
                        let mut lowered_args = Vec::new();
                        for arg in args {
                            let arg_expr =
                                self.lower_expression(arg, module, function, expressions, named_expressions, body)?;
                            lowered_args.push(arg_expr);
                        }
                        let has_result = module.functions[func_handle].result.is_some();
                        let result = if has_result {
                            let result_expr = expressions.append(Expression::CallResult(func_handle), NagaSpan::UNDEFINED);
                            Some(result_expr)
                        }
                        else {
                            None
                        };
                        let call = naga::Statement::Call { function: func_handle, arguments: lowered_args, result };
                        body.push(call, NagaSpan::UNDEFINED);
                        if let Some(result_expr) = result {
                            return Ok(result_expr);
                        }
                        return Self::make_zero_value(module, expressions);
                    }
                    if let Some(&gv_handle) = self.global_vars.get(name) {
                        return Ok(expressions.append(Expression::GlobalVariable(gv_handle), NagaSpan::UNDEFINED));
                    }
                }
                Err(GError {
                    kind: GErrorKind::Other,
                    message: format!("未知的函数调用: '{}'", func_name.unwrap_or_else(|| "(complex callee)".to_string())),
                })
            }
        }
    }

    /// 转换 texture_sample 调用
    fn lower_texture_sample(
        &mut self,
        args: &[TermExpression],
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        if args.len() < 3 {
            return Err(GError {
                kind: GErrorKind::Other,
                message: format!("texture_sample() 至少需要 3 个参数（纹理、采样器、坐标），得到 {}", args.len()),
            });
        }

        let texture_name = self.extract_callee_name(&args[0]);

        let image_expr = if let Some(name) = texture_name.as_deref() {
            if let Some(&tex_handle) = self.global_vars.get(name) {
                expressions.append(Expression::GlobalVariable(tex_handle), NagaSpan::UNDEFINED)
            }
            else {
                self.lower_expression(&args[0], module, function, expressions, named_expressions, body)?
            }
        }
        else {
            self.lower_expression(&args[0], module, function, expressions, named_expressions, body)?
        };

        let sampler_expr = if let Some(name) = texture_name.as_deref() {
            let sampler_name = format!("{}_sampler", name);
            if let Some(&sampler_handle) = self.global_vars.get(&sampler_name) {
                expressions.append(Expression::GlobalVariable(sampler_handle), NagaSpan::UNDEFINED)
            }
            else {
                self.lower_expression(&args[1], module, function, expressions, named_expressions, body)?
            }
        }
        else {
            self.lower_expression(&args[1], module, function, expressions, named_expressions, body)?
        };

        let coordinate = self.lower_expression(&args[2], module, function, expressions, named_expressions, body)?;

        Ok(expressions.append(
            Expression::ImageSample {
                image: image_expr,
                sampler: sampler_expr,
                gather: None,
                coordinate,
                array_index: None,
                offset: None,
                level: SampleLevel::Auto,
                depth_ref: None,
                clamp_to_edge: false,
            },
            NagaSpan::UNDEFINED,
        ))
    }

    /// 转换 texture_sample_lod 调用
    ///
    /// `texture_sample_lod(tex, sampler, uv, lod)` 映射为
    /// `Expression::ImageSample { level: SampleLevel::Exact(lod_expr), ... }`。
    fn lower_texture_sample_lod(
        &mut self,
        args: &[TermExpression],
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        if args.len() < 4 {
            return Err(GError {
                kind: GErrorKind::Other,
                message: format!("texture_sample_lod() 至少需要 4 个参数（纹理、采样器、坐标、lod），得到 {}", args.len()),
            });
        }

        let texture_name = self.extract_callee_name(&args[0]);

        let image_expr = if let Some(name) = texture_name.as_deref() {
            if let Some(&tex_handle) = self.global_vars.get(name) {
                expressions.append(Expression::GlobalVariable(tex_handle), NagaSpan::UNDEFINED)
            }
            else {
                self.lower_expression(&args[0], module, function, expressions, named_expressions, body)?
            }
        }
        else {
            self.lower_expression(&args[0], module, function, expressions, named_expressions, body)?
        };

        let sampler_expr = if let Some(name) = texture_name.as_deref() {
            let sampler_name = format!("{}_sampler", name);
            if let Some(&sampler_handle) = self.global_vars.get(&sampler_name) {
                expressions.append(Expression::GlobalVariable(sampler_handle), NagaSpan::UNDEFINED)
            }
            else {
                self.lower_expression(&args[1], module, function, expressions, named_expressions, body)?
            }
        }
        else {
            self.lower_expression(&args[1], module, function, expressions, named_expressions, body)?
        };

        let coordinate = self.lower_expression(&args[2], module, function, expressions, named_expressions, body)?;
        let lod_expr = self.lower_expression(&args[3], module, function, expressions, named_expressions, body)?;

        Ok(expressions.append(
            Expression::ImageSample {
                image: image_expr,
                sampler: sampler_expr,
                gather: None,
                coordinate,
                array_index: None,
                offset: None,
                level: SampleLevel::Exact(lod_expr),
                depth_ref: None,
                clamp_to_edge: false,
            },
            NagaSpan::UNDEFINED,
        ))
    }

    /// 转换 texture_sample_compare 调用
    ///
    /// `texture_sample_compare(tex, sampler, uv, compare)` 映射为
    /// `Expression::ImageSample { depth_ref: Some(compare_expr), level: SampleLevel::Zero, ... }`。
    fn lower_texture_sample_compare(
        &mut self,
        args: &[TermExpression],
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        if args.len() < 4 {
            return Err(GError {
                kind: GErrorKind::Other,
                message: format!(
                    "texture_sample_compare() 至少需要 4 个参数（纹理、采样器、坐标、比较值），得到 {}",
                    args.len()
                ),
            });
        }

        let texture_name = self.extract_callee_name(&args[0]);

        let image_expr = if let Some(name) = texture_name.as_deref() {
            if let Some(&tex_handle) = self.global_vars.get(name) {
                expressions.append(Expression::GlobalVariable(tex_handle), NagaSpan::UNDEFINED)
            }
            else {
                self.lower_expression(&args[0], module, function, expressions, named_expressions, body)?
            }
        }
        else {
            self.lower_expression(&args[0], module, function, expressions, named_expressions, body)?
        };

        let sampler_expr = if let Some(name) = texture_name.as_deref() {
            let sampler_name = format!("{}_sampler", name);
            if let Some(&sampler_handle) = self.global_vars.get(&sampler_name) {
                expressions.append(Expression::GlobalVariable(sampler_handle), NagaSpan::UNDEFINED)
            }
            else {
                self.lower_expression(&args[1], module, function, expressions, named_expressions, body)?
            }
        }
        else {
            self.lower_expression(&args[1], module, function, expressions, named_expressions, body)?
        };

        let coordinate = self.lower_expression(&args[2], module, function, expressions, named_expressions, body)?;
        let compare_expr = self.lower_expression(&args[3], module, function, expressions, named_expressions, body)?;

        Ok(expressions.append(
            Expression::ImageSample {
                image: image_expr,
                sampler: sampler_expr,
                gather: None,
                coordinate,
                array_index: None,
                offset: None,
                level: SampleLevel::Zero,
                depth_ref: Some(compare_expr),
                clamp_to_edge: false,
            },
            NagaSpan::UNDEFINED,
        ))
    }

    /// 转换向量构造函数（vec2/vec3/vec4）
    fn lower_vec_constructor(
        &mut self,
        size: usize,
        args: &[TermExpression],
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        let vector_size = match size {
            2 => VectorSize::Bi,
            3 => VectorSize::Tri,
            4 => VectorSize::Quad,
            _ => return Err(GError { kind: GErrorKind::Other, message: format!("不支持的向量大小: {}", size) }),
        };

        let ty = module.types.insert(
            naga::Type { name: None, inner: TypeInner::Vector { size: vector_size, scalar: F32_SCALAR } },
            NagaSpan::UNDEFINED,
        );

        let mut components = Vec::new();
        for arg in args {
            let comp = self.lower_expression(arg, module, function, expressions, named_expressions, body)?;
            components.push(comp);
        }

        Ok(expressions.append(Expression::Compose { ty, components }, NagaSpan::UNDEFINED))
    }

    /// 转换数学函数调用
    fn lower_math_func(
        &mut self,
        func: naga::MathFunction,
        args: &[TermExpression],
        module: &mut naga::Module,
        function: &mut naga::Function,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &mut NamedExpressions,
        body: &mut naga::Block,
    ) -> GResult<naga::Handle<Expression>> {
        let mut arg_handles = Vec::new();
        for arg in args {
            let handle = self.lower_expression(arg, module, function, expressions, named_expressions, body)?;
            arg_handles.push(handle);
        }

        if arg_handles.is_empty() {
            return Err(GError {
                kind: GErrorKind::Other, message: format!("数学函数 {:?} 至少需要 1 个参数", func)
            });
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
    fn resolve_name(
        &self,
        name: &str,
        expressions: &mut naga::Arena<Expression>,
        named_expressions: &NamedExpressions,
    ) -> GResult<naga::Handle<Expression>> {
        for (handle, expr_name) in named_expressions.iter() {
            if expr_name == name {
                return Ok(*handle);
            }
        }

        if let Some(&local_handle) = self.local_vars.get(name) {
            return Ok(expressions.append(Expression::LocalVariable(local_handle), NagaSpan::UNDEFINED));
        }

        if let Some(&gv_handle) = self.global_vars.get(name) {
            return Ok(expressions.append(Expression::GlobalVariable(gv_handle), NagaSpan::UNDEFINED));
        }

        Err(GError { kind: GErrorKind::Other, message: format!("未定义的变量: '{}'", name) })
    }

    /// 提取被调用者的函数名
    fn extract_callee_name(&self, callee: &TermExpression) -> Option<String> {
        match callee {
            TermExpression::NamePath(name_path) if name_path.parts.len() == 1 => Some(name_path.parts[0].name.clone()),
            _ => None,
        }
    }

    /// 将 Valkyrie 二元运算符映射到 naga BinaryOperator
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
            _ => Err(GError { kind: GErrorKind::Other, message: format!("不支持的二元运算符: {:?}", op) }),
        }
    }

    /// 将 Valkyrie 一元运算符映射到 naga UnaryOperator
    fn map_unary_operator(&self, op: &ValkyrieTokenType) -> GResult<naga::UnaryOperator> {
        match op {
            ValkyrieTokenType::Minus => Ok(naga::UnaryOperator::Negate),
            ValkyrieTokenType::Bang => Ok(naga::UnaryOperator::LogicalNot),
            _ => Err(GError { kind: GErrorKind::Other, message: format!("不支持的一元运算符: {:?}", op) }),
        }
    }

    /// 获取或创建 naga 类型
    fn get_or_create_naga_type(&mut self, type_name: &str, module: &mut naga::Module) -> GResult<naga::Handle<naga::Type>> {
        let key = type_name.to_lowercase();

        if let Some(&handle) = self.type_cache.get(&key) {
            return Ok(handle);
        }

        if key.starts_with("buffer") {
            return self.create_buffer_type(&key, module);
        }

        if key.starts_with("array") {
            return self.create_array_type(type_name, module);
        }

        let inner = match key.as_str() {
            "f32" | "float" => TypeInner::Scalar(F32_SCALAR),
            "i32" | "int" => TypeInner::Scalar(super::types::I32_SCALAR),
            "u32" | "uint" => TypeInner::Scalar(super::types::U32_SCALAR),
            "bool" => TypeInner::Scalar(super::types::BOOL_SCALAR),
            "vec2" | "vec2f" | "vector2" => TypeInner::Vector { size: VectorSize::Bi, scalar: F32_SCALAR },
            "vec3" | "vec3f" | "vector" | "vector3" => TypeInner::Vector { size: VectorSize::Tri, scalar: F32_SCALAR },
            "vec4" | "vec4f" | "color" => TypeInner::Vector { size: VectorSize::Quad, scalar: F32_SCALAR },
            "mat22" | "mat2x2" | "mat2x2f" => {
                TypeInner::Matrix { columns: VectorSize::Bi, rows: VectorSize::Bi, scalar: F32_SCALAR }
            }
            "mat33" | "mat3x3" | "mat3x3f" => {
                TypeInner::Matrix { columns: VectorSize::Tri, rows: VectorSize::Tri, scalar: F32_SCALAR }
            }
            "mat44" | "mat4x4" | "mat4x4f" => {
                TypeInner::Matrix { columns: VectorSize::Quad, rows: VectorSize::Quad, scalar: F32_SCALAR }
            }
            "sampler" => TypeInner::Sampler { comparison: false },
            "sampler_comparison" => TypeInner::Sampler { comparison: true },
            "texture_2d" | "tex2" | "texture" | "texture2d" => TypeInner::Image {
                dim: ImageDimension::D2,
                arrayed: false,
                class: ImageClass::Sampled { kind: ScalarKind::Float, multi: false },
            },
            "texture_2d_array" => TypeInner::Image {
                dim: ImageDimension::D2,
                arrayed: true,
                class: ImageClass::Sampled { kind: ScalarKind::Float, multi: false },
            },
            "texture_cube" => TypeInner::Image {
                dim: ImageDimension::Cube,
                arrayed: false,
                class: ImageClass::Sampled { kind: ScalarKind::Float, multi: false },
            },
            "texture_3d" | "tex3" | "texture3d" => TypeInner::Image {
                dim: ImageDimension::D3,
                arrayed: false,
                class: ImageClass::Sampled { kind: ScalarKind::Float, multi: false },
            },
            "texture_cube_array" | "texcube_array" => TypeInner::Image {
                dim: ImageDimension::Cube,
                arrayed: true,
                class: ImageClass::Sampled { kind: ScalarKind::Float, multi: false },
            },
            _ => return Err(GError { kind: GErrorKind::Other, message: format!("未知的 gs 类型: '{}'", type_name) }),
        };

        let handle = module.types.insert(naga::Type { name: None, inner }, NagaSpan::UNDEFINED);
        self.type_cache.insert(key, handle);
        Ok(handle)
    }

    /// 创建存储缓冲区类型
    ///
    /// 对于 `buffer<T>` 类型名称，提取内部类型 T 并创建一个
    /// 包含该类型字段的结构体，注册为存储缓冲区全局变量。
    fn create_buffer_type(&mut self, key: &str, module: &mut naga::Module) -> GResult<naga::Handle<naga::Type>> {
        let inner_type_name = key.trim_start_matches("buffer").trim_start_matches('<').trim_end_matches('>').trim();

        let inner_ty = if inner_type_name.is_empty() {
            self.get_or_create_naga_type("vec4", module)?
        }
        else {
            self.get_or_create_naga_type(inner_type_name, module)?
        };

        let struct_name = format!("Buffer_{}", inner_type_name);
        let struct_ty = module.types.insert(
            naga::Type {
                name: Some(struct_name.clone()),
                inner: TypeInner::Struct {
                    members: vec![naga::StructMember {
                        name: Some("value".to_string()),
                        ty: inner_ty,
                        binding: None,
                        offset: 0,
                    }],
                    span: self.type_size_align(inner_type_name).size,
                },
            },
            NagaSpan::UNDEFINED,
        );

        let binding = self.next_binding;
        self.next_binding += 1;

        let gv = module.global_variables.append(
            naga::GlobalVariable {
                name: Some(struct_name.clone()),
                space: naga::AddressSpace::Storage { access: naga::StorageAccess::LOAD | naga::StorageAccess::STORE },
                binding: Some(naga::ResourceBinding { group: 0, binding }),
                ty: struct_ty,
                init: None,
                memory_decorations: naga::MemoryDecorations::empty(),
            },
            NagaSpan::UNDEFINED,
        );
        self.global_vars.insert(struct_name.clone(), gv);
        self.type_cache.insert(key.to_string(), struct_ty);

        Ok(struct_ty)
    }

    /// 创建数组类型
    ///
    /// 解析 `Array<T, N>` 格式的类型名称，创建 naga 数组类型。
    /// 例如 `Array<f32, 4>` 创建一个包含 4 个 f32 的固定长度数组。
    fn create_array_type(&mut self, type_name: &str, module: &mut naga::Module) -> GResult<naga::Handle<naga::Type>> {
        let key = type_name.to_lowercase();
        let inner = type_name.trim_start_matches("Array").trim_start_matches("array");
        let inner = inner.trim_start_matches('<').trim_end_matches('>').trim();

        let (element_type, size_str) = if inner.contains(',') {
            let mut parts = inner.splitn(2, ',');
            let elem = parts.next().unwrap().trim();
            let size = parts.next().unwrap().trim();
            (elem, size.to_string())
        }
        else {
            (inner, "0".to_string())
        };

        let element_ty = self.get_or_create_naga_type(element_type, module)?;

        let size = if let Ok(n) = size_str.parse::<u32>() {
            if n == 0 { naga::ArraySize::Dynamic } else { naga::ArraySize::Constant(std::num::NonZeroU32::new(n).unwrap()) }
        }
        else {
            naga::ArraySize::Dynamic
        };

        let stride = {
            let elem_size = self.type_size_align(element_type);
            super::types::align_offset(elem_size.size, elem_size.align)
        };

        let array_ty = module.types.insert(
            naga::Type {
                name: Some(format!("Array_{}_{}", element_type, size_str)),
                inner: TypeInner::Array { base: element_ty, size, stride },
            },
            NagaSpan::UNDEFINED,
        );
        self.type_cache.insert(key, array_ty);
        Ok(array_ty)
    }
    /// 优先从 uniform 字段索引映射中查找，
    /// 然后从 naga Module 的类型定义中查找结构体成员，
    /// 最后回退到 swizzle 分量索引（xyzw/rgba）。
    fn resolve_field_index(&self, base_name: &str, field_name: &str, module: &naga::Module) -> u32 {
        if let Some(&idx) = self.uniform_field_index.get(field_name) {
            return idx;
        }

        if let Some(gv_handle) = self.global_vars.get(base_name) {
            let gv = &module.global_variables[*gv_handle];
            let ty = &module.types[gv.ty];
            if let TypeInner::Struct { ref members, .. } = ty.inner {
                for (idx, member) in members.iter().enumerate() {
                    if let Some(ref name) = member.name {
                        if name == field_name {
                            return idx as u32;
                        }
                    }
                }
            }
        }

        if let Some(ty_handle) = self.type_cache.get(base_name) {
            let ty = &module.types[*ty_handle];
            if let TypeInner::Struct { ref members, .. } = ty.inner {
                for (idx, member) in members.iter().enumerate() {
                    if let Some(ref name) = member.name {
                        if name == field_name {
                            return idx as u32;
                        }
                    }
                }
            }
        }

        self.swizzle_component_index(field_name)
    }

    /// 获取单个 swizzle 分量的索引
    fn swizzle_component_index(&self, name: &str) -> u32 {
        match name.to_lowercase().as_str() {
            "x" | "r" => 0,
            "y" | "g" => 1,
            "z" | "b" => 2,
            "w" | "a" => 3,
            _ => 0,
        }
    }

    /// 从 DotCall 的 receiver 表达式中提取基础名称
    ///
    /// 用于在 resolve_field_index 中查找结构体字段索引。
    /// 如果 receiver 是 NamePath，返回第一段名称；
    /// 如果 receiver 是另一个 DotCall，递归提取最左侧名称；
    /// 否则返回空字符串。
    fn extract_receiver_name(&self, receiver: &TermExpression) -> String {
        match receiver {
            TermExpression::NamePath(np) => np.parts.first().map(|p| p.name.clone()).unwrap_or_default(),
            TermExpression::DotCall { receiver: inner, .. } => self.extract_receiver_name(inner),
            _ => String::new(),
        }
    }

    /// 从 AST 索引表达式中提取常量索引值
    ///
    /// 如果索引是整数字面量，返回 Some(index)；否则返回 None。
    fn extract_const_index(&self, index: &TermExpression) -> Option<u32> {
        match index {
            TermExpression::StringLiteral(sl) => {
                let s: String = sl
                    .segments
                    .iter()
                    .filter_map(|seg| match seg {
                        oak_valkyrie::ast::StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                        _ => None,
                    })
                    .collect();
                s.parse::<u32>().ok()
            }
            _ => None,
        }
    }

    /// 判断字段名是否为 swizzle 模式
    ///
    /// swizzle 模式由 1-4 个 xyzw 或 rgba 字符组成。
    fn is_swizzle_pattern(name: &str) -> bool {
        let len = name.len();
        if len == 0 || len > 4 {
            return false;
        }
        name.chars().all(|c| matches!(c.to_ascii_lowercase(), 'x' | 'y' | 'z' | 'w' | 'r' | 'g' | 'b' | 'a'))
    }

    /// 解析 swizzle 字段名为 naga SwizzleComponent 数组
    ///
    /// 将 xyzw/rgba 映射为 SwizzleComponent 枚举值，
    /// 并根据分量数量确定结果向量大小。
    fn parse_swizzle(name: &str) -> (VectorSize, [SwizzleComponent; 4]) {
        let components: Vec<SwizzleComponent> = name
            .chars()
            .map(|c| match c.to_ascii_lowercase() {
                'x' | 'r' => SwizzleComponent::X,
                'y' | 'g' => SwizzleComponent::Y,
                'z' | 'b' => SwizzleComponent::Z,
                'w' | 'a' => SwizzleComponent::W,
                _ => SwizzleComponent::X,
            })
            .collect();

        let size = match components.len() {
            2 => VectorSize::Bi,
            3 => VectorSize::Tri,
            _ => VectorSize::Quad,
        };

        let mut pattern = [SwizzleComponent::X; 4];
        for (i, comp) in components.iter().enumerate() {
            pattern[i] = *comp;
        }

        (size, pattern)
    }

    /// 将 TypeExpression 转换为类型名称字符串
    fn type_expr_to_string(&self, ty: &TypeExpression) -> String {
        match ty {
            TypeExpression::Namepath(np) => name_path_to_string(np),
            TypeExpression::Generic(r#gen) => r#gen.name.name.clone(),
            TypeExpression::Tuple(tuple) => {
                let elements: Vec<String> = tuple.elements.iter().map(|e| self.type_expr_to_string(e)).collect();
                format!("({})", elements.join(", "))
            }
            TypeExpression::Optional(opt) => format!("{}?", self.type_expr_to_string(&opt.inner)),
            TypeExpression::Binary(bin) => {
                format!("{} {:?} {}", self.type_expr_to_string(&bin.lhs), bin.operator, self.type_expr_to_string(&bin.rhs))
            }
            TypeExpression::Unary(un) => format!("{:?}{}", un.operator, self.type_expr_to_string(&un.base)),
            TypeExpression::Function(func) => {
                let params: Vec<String> = func.params.iter().map(|p| self.type_expr_to_string(p)).collect();
                format!("({}) -> {}", params.join(", "), self.type_expr_to_string(&func.return_type))
            }
            TypeExpression::AssociatedType(assoc) => format!("{}::{}", assoc.base.name, assoc.name.name),
            TypeExpression::QualifiedAssociatedType(qual) => {
                format!(
                    "<{} as {}>::{}",
                    self.type_expr_to_string(&qual.ty),
                    name_path_to_string(&qual.trait_path),
                    qual.name.name
                )
            }
        }
    }

    /// 将 TermExpression 转换为可读字符串（用于调试和默认值提取）
    fn expr_to_string(&self, expr: &TermExpression) -> String {
        match expr {
            TermExpression::NamePath(np) => name_path_to_string(np),
            TermExpression::StringLiteral(sl) => sl
                .segments
                .iter()
                .filter_map(|seg| match seg {
                    oak_valkyrie::ast::StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                    _ => None,
                })
                .collect(),
            TermExpression::Bool { value, .. } => value.to_string(),
            TermExpression::Binary(node) => {
                format!("{} {:?} {}", self.expr_to_string(&node.lhs), node.operator, self.expr_to_string(&node.rhs))
            }
            TermExpression::Unary(node) => format!("{:?}{}", node.operator, self.expr_to_string(&node.base)),
            TermExpression::ApplyCall { callee, args, .. } => {
                let arg_strs: Vec<String> = args.iter().map(|a| self.expr_to_string(a)).collect();
                format!("{}({})", self.expr_to_string(callee), arg_strs.join(", "))
            }
            TermExpression::DotCall { receiver, field, .. } => format!("{}.{}", self.expr_to_string(receiver), field.name),
            TermExpression::Paren { expr: inner, .. } => format!("({})", self.expr_to_string(inner)),
            TermExpression::Object { callee, fields, .. } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(name, value)| match value {
                        Some(v) => format!("{}: {}", name.name, self.expr_to_string(v)),
                        None => name.name.clone(),
                    })
                    .collect();
                format!("{} {{ {} }}", self.expr_to_string(callee), field_strs.join(", "))
            }
            _ => format!("{:?}", std::mem::discriminant(expr)),
        }
    }

    /// 获取 gs 类型的大小和对齐信息
    fn type_size_align(&self, type_name: &str) -> TypeSizeAlign {
        let key = type_name.to_lowercase();
        match key.as_str() {
            "f32" | "float" | "i32" | "int" | "u32" | "uint" => TypeSizeAlign { size: 4, align: 4 },
            "vec2" | "vec2f" => TypeSizeAlign { size: 8, align: 8 },
            "vec3" | "vec3f" | "vector" => TypeSizeAlign { size: 12, align: 16 },
            "vec4" | "vec4f" | "color" => TypeSizeAlign { size: 16, align: 16 },
            "mat22" | "mat2x2" | "mat2x2f" => TypeSizeAlign { size: 16, align: 8 },
            "mat33" | "mat3x3" | "mat3x3f" => TypeSizeAlign { size: 48, align: 16 },
            "mat44" | "mat4x4" | "mat4x4f" => TypeSizeAlign { size: 64, align: 16 },
            _ => TypeSizeAlign { size: 16, align: 16 },
        }
    }

    /// 将 GsRenderState 键值对列表转换为 RenderStates 结构体
    ///
    /// 从渲染状态的原始键值对中提取并映射各个渲染状态字段：
    /// - `cull_mode`: "none" → None, "front" → Front, "back" → Back
    /// - `blend_mode`: "opaque" → Opaque, "alpha" → Alpha, "additive" → Additive, "multiply" → Multiply
    /// - 布尔字段: "true" → true, "false" → false
    pub fn convert_render_states(render_states: &[GsRenderState]) -> RenderStates {
        let mut result = RenderStates::default();

        for rs in render_states {
            let key = rs.name.to_lowercase();
            let value = rs.value.to_lowercase();

            match key.as_str() {
                "cull_mode" | "cullmode" => {
                    result.cull_mode = match value.as_str() {
                        "none" => CullMode::None,
                        "front" => CullMode::Front,
                        "back" => CullMode::Back,
                        _ => CullMode::Back,
                    };
                }
                "blend_mode" | "blendmode" => {
                    result.blend_mode = match value.as_str() {
                        "opaque" => BlendMode::Opaque,
                        "alpha" => BlendMode::Alpha,
                        "additive" => BlendMode::Additive,
                        "multiply" => BlendMode::Multiply,
                        _ => BlendMode::Opaque,
                    };
                }
                "depth_test" | "depthtest" => {
                    result.depth_test = value == "true";
                }
                "depth_write" | "depthwrite" => {
                    result.depth_write = value == "true";
                }
                "wireframe" => {
                    result.wireframe = value == "true";
                }
                "stencil_test" | "stenciltest" => {
                    result.stencil_test = value == "true";
                }
                "multisample" => {
                    result.multisample = value == "true";
                }
                _ => {}
            }
        }

        result
    }

    /// 从 shader 声明中收集回退信息
    ///
    /// 查找名为 "fallback" 的命名空间块，从中提取回退条件和目标着色器名称。
    /// 期望的格式为：
    /// ```text
    /// fallback {
    ///     let condition: string = "..."
    ///     let shader: string = "..."
    /// }
    /// ```
    pub fn collect_fallback(shader: &ShaderDeclaration) -> Option<FallbackInfo> {
        for item in &shader.items {
            if let StatementNode::Namespace(namespace) = item {
                let name_str = name_path_to_string(&namespace.name);
                if name_str.to_lowercase() == "fallback" {
                    let mut condition = None;
                    let mut fallback_shader = None;

                    for inner_item in &namespace.items {
                        if let StatementNode::Let(let_stmt) = inner_item {
                            let field_name = match &let_stmt.pattern {
                                Pattern::Variable(v) => v.name.name.to_lowercase(),
                                _ => continue,
                            };
                            let value = Self::extract_let_string_value(let_stmt);

                            match field_name.as_str() {
                                "condition" => condition = value,
                                "shader" => fallback_shader = value,
                                _ => {}
                            }
                        }
                    }

                    if let (Some(cond), Some(shader)) = (condition, fallback_shader) {
                        return Some(FallbackInfo { condition: cond, fallback_shader: shader });
                    }
                }
            }
        }
        None
    }

    /// 从 let 语句中提取字符串值
    fn extract_let_string_value(let_stmt: &Let) -> Option<String> {
        match &let_stmt.expr {
            TermExpression::StringLiteral(sl) => Some(
                sl.segments
                    .iter()
                    .filter_map(|seg| match seg {
                        oak_valkyrie::ast::StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                        _ => None,
                    })
                    .collect(),
            ),
            TermExpression::NamePath(np) if np.parts.len() == 1 => Some(np.parts[0].name.clone()),
            _ => None,
        }
    }
}

impl Default for GslLowerer {
    fn default() -> Self {
        Self::new()
    }
}
