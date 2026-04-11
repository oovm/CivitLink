//! 内置着色器 naga IR 构建模块
//!
//! 提供内置着色器（精灵、过渡、批渲染精灵）的 naga IR 构建函数，
//! 直接使用 naga IR API 构建，不依赖任何着色器语言前端。

use gg_core::{GError, GResult};
use naga::{
    AddressSpace, BinaryOperator, Binding, BuiltIn, Expression, FunctionArgument, FunctionResult, ImageClass, ImageDimension,
    Interpolation, MathFunction, Range as NagaRange, ResourceBinding, SampleLevel, Scalar, ScalarKind, Span as NagaSpan,
    Statement, Type, TypeInner, VectorSize,
};

const F32_SCALAR: Scalar = Scalar { kind: ScalarKind::Float, width: 4 };

struct BuiltinShaderBuilder {
    types: naga::UniqueArena<Type>,
    global_vars: naga::Arena<naga::GlobalVariable>,
    next_binding: u32,
}

impl BuiltinShaderBuilder {
    fn new() -> Self {
        Self { types: naga::UniqueArena::new(), global_vars: naga::Arena::new(), next_binding: 0 }
    }

    fn insert_type(&mut self, inner: TypeInner, name: Option<String>) -> naga::Handle<Type> {
        self.types.insert(Type { name, inner }, NagaSpan::UNDEFINED)
    }

    fn vec2_ty(&mut self) -> naga::Handle<Type> {
        self.insert_type(TypeInner::Vector { size: VectorSize::Bi, scalar: F32_SCALAR }, None)
    }

    fn vec3_ty(&mut self) -> naga::Handle<Type> {
        self.insert_type(TypeInner::Vector { size: VectorSize::Tri, scalar: F32_SCALAR }, None)
    }

    fn vec4_ty(&mut self) -> naga::Handle<Type> {
        self.insert_type(TypeInner::Vector { size: VectorSize::Quad, scalar: F32_SCALAR }, None)
    }

    fn mat4x4_ty(&mut self) -> naga::Handle<Type> {
        self.insert_type(TypeInner::Matrix { columns: VectorSize::Quad, rows: VectorSize::Quad, scalar: F32_SCALAR }, None)
    }

    fn sampler_ty(&mut self) -> naga::Handle<Type> {
        self.insert_type(TypeInner::Sampler { comparison: false }, None)
    }

    fn texture_2d_ty(&mut self) -> naga::Handle<Type> {
        self.insert_type(
            TypeInner::Image {
                dim: ImageDimension::D2,
                arrayed: false,
                class: ImageClass::Sampled { kind: ScalarKind::Float, multi: false },
            },
            None,
        )
    }

    fn create_struct(&mut self, name: &str, members: Vec<naga::StructMember>, span: u32) -> naga::Handle<Type> {
        self.insert_type(TypeInner::Struct { members, span }, Some(name.to_string()))
    }

    fn add_uniform_global(
        &mut self,
        name: &str,
        ty: naga::Handle<Type>,
        group: u32,
        binding: u32,
    ) -> naga::Handle<naga::GlobalVariable> {
        self.global_vars.append(
            naga::GlobalVariable {
                name: Some(name.to_string()),
                space: AddressSpace::Uniform,
                binding: Some(ResourceBinding { group, binding }),
                ty,
                init: None,
                memory_decorations: naga::MemoryDecorations::empty(),
            },
            NagaSpan::UNDEFINED,
        )
    }

    fn add_handle_global(
        &mut self,
        name: &str,
        ty: naga::Handle<Type>,
        group: u32,
        binding: u32,
    ) -> naga::Handle<naga::GlobalVariable> {
        self.global_vars.append(
            naga::GlobalVariable {
                name: Some(name.to_string()),
                space: AddressSpace::Handle,
                binding: Some(ResourceBinding { group, binding }),
                ty,
                init: None,
                memory_decorations: naga::MemoryDecorations::empty(),
            },
            NagaSpan::UNDEFINED,
        )
    }

    fn next_binding(&mut self) -> u32 {
        let b = self.next_binding;
        self.next_binding += 1;
        b
    }

    fn build_module(self, entry_points: Vec<naga::EntryPoint>) -> naga::Module {
        let mut module = naga::Module::default();
        module.types = self.types;
        module.global_variables = self.global_vars;
        module.entry_points = entry_points;
        module
    }
}

fn make_location_binding(location: u32) -> Binding {
    Binding::Location {
        location,
        interpolation: Some(Interpolation::Linear),
        sampling: None,
        blend_src: None,
        per_primitive: false,
    }
}

fn make_entry_point(function: naga::Function, name: &str, stage: naga::ShaderStage) -> naga::EntryPoint {
    naga::EntryPoint {
        name: name.to_string(),
        stage,
        early_depth_test: None,
        workgroup_size: [0; 3],
        workgroup_size_overrides: None,
        function,
        mesh_info: None,
        task_payload: None,
        incoming_ray_payload: None,
    }
}

/// 构建精灵着色器的 naga Module
pub fn builtin_sprite_shader() -> GResult<naga::Module> {
    let mut builder = BuiltinShaderBuilder::new();

    let mat4_ty = builder.mat4x4_ty();
    let vec4_ty = builder.vec4_ty();
    let vec2_ty = builder.vec2_ty();

    let uniforms_ty = builder.create_struct(
        "Uniforms",
        vec![
            naga::StructMember { name: Some("mvp".to_string()), ty: mat4_ty, binding: None, offset: 0 },
            naga::StructMember { name: Some("tint".to_string()), ty: vec4_ty, binding: None, offset: 64 },
            naga::StructMember { name: Some("uv_transform".to_string()), ty: vec4_ty, binding: None, offset: 80 },
        ],
        96,
    );

    let uniforms_gv = builder.add_uniform_global("uniforms", uniforms_ty, 0, 0);

    let sampler_ty = builder.sampler_ty();
    let texture_ty = builder.texture_2d_ty();

    let sampler_binding = builder.next_binding();
    let tex_sampler_gv = builder.add_handle_global("tex_sampler", sampler_ty, 1, sampler_binding);

    let tex_binding = builder.next_binding();
    let tex_gv = builder.add_handle_global("tex", texture_ty, 1, tex_binding);

    let vertex_input_ty = builder.create_struct(
        "VertexInput",
        vec![
            naga::StructMember {
                name: Some("position".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(0)),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(1)),
                offset: 8,
            },
        ],
        16,
    );

    let vertex_output_ty = builder.create_struct(
        "VertexOutput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(Binding::Location {
                    location: 0,
                    interpolation: Some(Interpolation::Linear),
                    sampling: None,
                    blend_src: None,
                    per_primitive: false,
                }),
                offset: 16,
            },
        ],
        24,
    );

    let mut vs_function = naga::Function::default();
    vs_function.name = Some("vs_main".to_string());

    let arg_idx = vs_function.arguments.len() as u32;
    vs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: vertex_input_ty, binding: None });
    vs_function.result = Some(FunctionResult { ty: vertex_output_ty, binding: None });

    let mut expressions = naga::Arena::new();
    let mut body = naga::Block::new();

    let input_expr = expressions.append(Expression::FunctionArgument(arg_idx), NagaSpan::UNDEFINED);
    let uniforms_expr = expressions.append(Expression::GlobalVariable(uniforms_gv), NagaSpan::UNDEFINED);

    let zero_f32 = expressions.append(Expression::Literal(naga::Literal::F32(0.0)), NagaSpan::UNDEFINED);
    let one_f32 = expressions.append(Expression::Literal(naga::Literal::F32(1.0)), NagaSpan::UNDEFINED);

    let position_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 0 }, NagaSpan::UNDEFINED);
    let uv_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 1 }, NagaSpan::UNDEFINED);

    let mvp_ptr = expressions.append(Expression::AccessIndex { base: uniforms_expr, index: 0 }, NagaSpan::UNDEFINED);
    let mvp_expr = expressions.append(Expression::Load { pointer: mvp_ptr }, NagaSpan::UNDEFINED);
    let uv_transform_ptr = expressions.append(Expression::AccessIndex { base: uniforms_expr, index: 2 }, NagaSpan::UNDEFINED);
    let uv_transform_expr = expressions.append(Expression::Load { pointer: uv_transform_ptr }, NagaSpan::UNDEFINED);

    let pos_x = expressions.append(Expression::AccessIndex { base: position_expr, index: 0 }, NagaSpan::UNDEFINED);
    let pos_y = expressions.append(Expression::AccessIndex { base: position_expr, index: 1 }, NagaSpan::UNDEFINED);

    let pos_vec4 = expressions
        .append(Expression::Compose { ty: vec4_ty, components: vec![pos_x, pos_y, zero_f32, one_f32] }, NagaSpan::UNDEFINED);

    let clip_pos = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: mvp_expr, right: pos_vec4 }, NagaSpan::UNDEFINED);

    let uv_transform_x = expressions.append(Expression::AccessIndex { base: uv_transform_expr, index: 0 }, NagaSpan::UNDEFINED);
    let uv_transform_y = expressions.append(Expression::AccessIndex { base: uv_transform_expr, index: 1 }, NagaSpan::UNDEFINED);
    let uv_transform_z = expressions.append(Expression::AccessIndex { base: uv_transform_expr, index: 2 }, NagaSpan::UNDEFINED);
    let uv_transform_w = expressions.append(Expression::AccessIndex { base: uv_transform_expr, index: 3 }, NagaSpan::UNDEFINED);

    let uv_0 = expressions.append(Expression::AccessIndex { base: uv_expr, index: 0 }, NagaSpan::UNDEFINED);
    let uv_1 = expressions.append(Expression::AccessIndex { base: uv_expr, index: 1 }, NagaSpan::UNDEFINED);

    let uv_mul_z = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: uv_0, right: uv_transform_z }, NagaSpan::UNDEFINED);
    let uv_mul_w = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: uv_1, right: uv_transform_w }, NagaSpan::UNDEFINED);

    let uv_x = expressions
        .append(Expression::Binary { op: BinaryOperator::Add, left: uv_transform_x, right: uv_mul_z }, NagaSpan::UNDEFINED);

    let uv_y = expressions
        .append(Expression::Binary { op: BinaryOperator::Add, left: uv_transform_y, right: uv_mul_w }, NagaSpan::UNDEFINED);

    let out_uv = expressions.append(Expression::Compose { ty: vec2_ty, components: vec![uv_x, uv_y] }, NagaSpan::UNDEFINED);

    let output = expressions
        .append(Expression::Compose { ty: vertex_output_ty, components: vec![clip_pos, out_uv] }, NagaSpan::UNDEFINED);

    body.push(Statement::Emit(NagaRange::new_from_bounds(position_expr, output)), NagaSpan::UNDEFINED);
    body.push(Statement::Return { value: Some(output) }, NagaSpan::UNDEFINED);

    vs_function.expressions = expressions;
    vs_function.body = body;

    let vs_entry = make_entry_point(vs_function, "vs_main", naga::ShaderStage::Vertex);

    let fragment_input_ty = builder.create_struct(
        "FragmentInput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(Binding::Location {
                    location: 0,
                    interpolation: Some(Interpolation::Linear),
                    sampling: None,
                    blend_src: None,
                    per_primitive: false,
                }),
                offset: 16,
            },
        ],
        24,
    );

    let mut fs_function = naga::Function::default();
    fs_function.name = Some("fs_main".to_string());

    let fs_arg_idx = fs_function.arguments.len() as u32;
    fs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: fragment_input_ty, binding: None });
    fs_function.result = Some(FunctionResult {
        ty: vec4_ty,
        binding: Some(Binding::Location {
            location: 0,
            interpolation: Some(Interpolation::Linear),
            sampling: None,
            blend_src: None,
            per_primitive: false,
        }),
    });

    let mut fs_expressions = naga::Arena::new();
    let mut fs_body = naga::Block::new();

    let fs_input_expr = fs_expressions.append(Expression::FunctionArgument(fs_arg_idx), NagaSpan::UNDEFINED);
    let sampler_expr = fs_expressions.append(Expression::GlobalVariable(tex_sampler_gv), NagaSpan::UNDEFINED);
    let tex_expr = fs_expressions.append(Expression::GlobalVariable(tex_gv), NagaSpan::UNDEFINED);

    let fs_uv_expr = fs_expressions.append(Expression::AccessIndex { base: fs_input_expr, index: 1 }, NagaSpan::UNDEFINED);

    let tex_color = fs_expressions.append(
        Expression::ImageSample {
            image: tex_expr,
            sampler: sampler_expr,
            gather: None,
            coordinate: fs_uv_expr,
            array_index: None,
            offset: None,
            level: SampleLevel::Auto,
            depth_ref: None,
            clamp_to_edge: false,
        },
        NagaSpan::UNDEFINED,
    );

    fs_body.push(Statement::Emit(NagaRange::new_from_bounds(fs_uv_expr, tex_color)), NagaSpan::UNDEFINED);
    fs_body.push(Statement::Return { value: Some(tex_color) }, NagaSpan::UNDEFINED);

    fs_function.expressions = fs_expressions;
    fs_function.body = fs_body;

    let fs_entry = make_entry_point(fs_function, "fs_main", naga::ShaderStage::Fragment);

    Ok(builder.build_module(vec![vs_entry, fs_entry]))
}

/// 构建过渡着色器的 naga Module
pub fn builtin_transition_shader() -> GResult<naga::Module> {
    let mut builder = BuiltinShaderBuilder::new();

    let mat4_ty = builder.mat4x4_ty();
    let vec4_ty = builder.vec4_ty();
    let vec2_ty = builder.vec2_ty();

    let uniforms_ty = builder.create_struct(
        "Uniforms",
        vec![
            naga::StructMember { name: Some("mvp".to_string()), ty: mat4_ty, binding: None, offset: 0 },
            naga::StructMember { name: Some("params".to_string()), ty: vec4_ty, binding: None, offset: 64 },
            naga::StructMember { name: Some("tint".to_string()), ty: vec4_ty, binding: None, offset: 80 },
        ],
        96,
    );

    let uniforms_gv = builder.add_uniform_global("uniforms", uniforms_ty, 0, 0);

    let sampler_ty = builder.sampler_ty();
    let texture_ty = builder.texture_2d_ty();

    let sampler_binding = builder.next_binding();
    let sampler_gv = builder.add_handle_global("tex_sampler", sampler_ty, 1, sampler_binding);

    let old_tex_binding = builder.next_binding();
    let old_tex_gv = builder.add_handle_global("old_tex", texture_ty, 1, old_tex_binding);

    let new_tex_binding = builder.next_binding();
    let new_tex_gv = builder.add_handle_global("new_tex", texture_ty, 1, new_tex_binding);

    let vertex_input_ty = builder.create_struct(
        "VertexInput",
        vec![
            naga::StructMember {
                name: Some("position".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(0)),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(1)),
                offset: 8,
            },
        ],
        16,
    );

    let vertex_output_ty = builder.create_struct(
        "VertexOutput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(Binding::Location {
                    location: 0,
                    interpolation: Some(Interpolation::Linear),
                    sampling: None,
                    blend_src: None,
                    per_primitive: false,
                }),
                offset: 16,
            },
        ],
        24,
    );

    let mut vs_function = naga::Function::default();
    vs_function.name = Some("vs_main".to_string());

    let arg_idx = vs_function.arguments.len() as u32;
    vs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: vertex_input_ty, binding: None });
    vs_function.result = Some(FunctionResult { ty: vertex_output_ty, binding: None });

    let mut expressions = naga::Arena::new();
    let mut body = naga::Block::new();

    let input_expr = expressions.append(Expression::FunctionArgument(arg_idx), NagaSpan::UNDEFINED);
    let uniforms_expr = expressions.append(Expression::GlobalVariable(uniforms_gv), NagaSpan::UNDEFINED);

    let zero_f32 = expressions.append(Expression::Literal(naga::Literal::F32(0.0)), NagaSpan::UNDEFINED);
    let one_f32 = expressions.append(Expression::Literal(naga::Literal::F32(1.0)), NagaSpan::UNDEFINED);

    let position_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 0 }, NagaSpan::UNDEFINED);
    let uv_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 1 }, NagaSpan::UNDEFINED);

    let mvp_expr = expressions.append(Expression::AccessIndex { base: uniforms_expr, index: 0 }, NagaSpan::UNDEFINED);
    let mvp_loaded = expressions.append(Expression::Load { pointer: mvp_expr }, NagaSpan::UNDEFINED);

    let pos_x = expressions.append(Expression::AccessIndex { base: position_expr, index: 0 }, NagaSpan::UNDEFINED);
    let pos_y = expressions.append(Expression::AccessIndex { base: position_expr, index: 1 }, NagaSpan::UNDEFINED);

    let pos_vec4 = expressions
        .append(Expression::Compose { ty: vec4_ty, components: vec![pos_x, pos_y, zero_f32, one_f32] }, NagaSpan::UNDEFINED);

    let clip_pos = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: mvp_loaded, right: pos_vec4 }, NagaSpan::UNDEFINED);

    let output = expressions
        .append(Expression::Compose { ty: vertex_output_ty, components: vec![clip_pos, uv_expr] }, NagaSpan::UNDEFINED);

    body.push(Statement::Emit(NagaRange::new_from_bounds(position_expr, output)), NagaSpan::UNDEFINED);
    body.push(Statement::Return { value: Some(output) }, NagaSpan::UNDEFINED);

    vs_function.expressions = expressions;
    vs_function.body = body;

    let vs_entry = make_entry_point(vs_function, "vs_main", naga::ShaderStage::Vertex);

    let fragment_input_ty = builder.create_struct(
        "FragmentInput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(Binding::Location {
                    location: 0,
                    interpolation: Some(Interpolation::Linear),
                    sampling: None,
                    blend_src: None,
                    per_primitive: false,
                }),
                offset: 16,
            },
        ],
        24,
    );

    let mut fs_function = naga::Function::default();
    fs_function.name = Some("fs_main".to_string());

    let fs_arg_idx = fs_function.arguments.len() as u32;
    fs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: fragment_input_ty, binding: None });
    fs_function.result = Some(FunctionResult {
        ty: vec4_ty,
        binding: Some(Binding::Location {
            location: 0,
            interpolation: Some(Interpolation::Linear),
            sampling: None,
            blend_src: None,
            per_primitive: false,
        }),
    });

    let mut fs_expressions = naga::Arena::new();
    let mut fs_body = naga::Block::new();

    let fs_input_expr = fs_expressions.append(Expression::FunctionArgument(fs_arg_idx), NagaSpan::UNDEFINED);
    let fs_uv_expr = fs_expressions.append(Expression::AccessIndex { base: fs_input_expr, index: 1 }, NagaSpan::UNDEFINED);

    let fs_uniforms_expr = fs_expressions.append(Expression::GlobalVariable(uniforms_gv), NagaSpan::UNDEFINED);
    let fs_params_ptr =
        fs_expressions.append(Expression::AccessIndex { base: fs_uniforms_expr, index: 1 }, NagaSpan::UNDEFINED);
    let fs_tint_ptr = fs_expressions.append(Expression::AccessIndex { base: fs_uniforms_expr, index: 2 }, NagaSpan::UNDEFINED);
    let fs_params_x_ptr = fs_expressions.append(Expression::AccessIndex { base: fs_params_ptr, index: 0 }, NagaSpan::UNDEFINED);
    let fs_params_x = fs_expressions.append(Expression::Load { pointer: fs_params_x_ptr }, NagaSpan::UNDEFINED);
    let fs_tint = fs_expressions.append(Expression::Load { pointer: fs_tint_ptr }, NagaSpan::UNDEFINED);

    let sampler_expr = fs_expressions.append(Expression::GlobalVariable(sampler_gv), NagaSpan::UNDEFINED);
    let old_tex_expr = fs_expressions.append(Expression::GlobalVariable(old_tex_gv), NagaSpan::UNDEFINED);
    let new_tex_expr = fs_expressions.append(Expression::GlobalVariable(new_tex_gv), NagaSpan::UNDEFINED);

    let old_color = fs_expressions.append(
        Expression::ImageSample {
            image: old_tex_expr,
            sampler: sampler_expr,
            gather: None,
            coordinate: fs_uv_expr,
            array_index: None,
            offset: None,
            level: SampleLevel::Auto,
            depth_ref: None,
            clamp_to_edge: false,
        },
        NagaSpan::UNDEFINED,
    );
    let new_color = fs_expressions.append(
        Expression::ImageSample {
            image: new_tex_expr,
            sampler: sampler_expr,
            gather: None,
            coordinate: fs_uv_expr,
            array_index: None,
            offset: None,
            level: SampleLevel::Auto,
            depth_ref: None,
            clamp_to_edge: false,
        },
        NagaSpan::UNDEFINED,
    );

    let mixed = fs_expressions.append(
        Expression::Math { fun: MathFunction::Mix, arg: old_color, arg1: Some(new_color), arg2: Some(fs_params_x), arg3: None },
        NagaSpan::UNDEFINED,
    );

    let result = fs_expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: mixed, right: fs_tint }, NagaSpan::UNDEFINED);

    fs_body.push(Statement::Emit(NagaRange::new_from_bounds(fs_uv_expr, fs_uv_expr)), NagaSpan::UNDEFINED);
    fs_body.push(Statement::Emit(NagaRange::new_from_bounds(fs_params_ptr, fs_tint)), NagaSpan::UNDEFINED);
    fs_body.push(Statement::Emit(NagaRange::new_from_bounds(old_color, result)), NagaSpan::UNDEFINED);
    fs_body.push(Statement::Return { value: Some(result) }, NagaSpan::UNDEFINED);

    fs_function.expressions = fs_expressions;
    fs_function.body = fs_body;

    let fs_entry = make_entry_point(fs_function, "fs_main", naga::ShaderStage::Fragment);

    Ok(builder.build_module(vec![vs_entry, fs_entry]))
}

/// 构建批渲染精灵着色器的 naga Module
pub fn builtin_batch_sprite_shader() -> GResult<naga::Module> {
    let mut builder = BuiltinShaderBuilder::new();

    let mat4_ty = builder.mat4x4_ty();
    let vec4_ty = builder.vec4_ty();
    let vec2_ty = builder.vec2_ty();
    let sampler_ty = builder.sampler_ty();
    let texture_ty = builder.texture_2d_ty();

    let sampler_binding = builder.next_binding();
    let tex_sampler_gv = builder.add_handle_global("tex_sampler", sampler_ty, 0, sampler_binding);

    let tex_binding = builder.next_binding();
    let tex_gv = builder.add_handle_global("tex", texture_ty, 0, tex_binding);

    let vertex_input_ty = builder.create_struct(
        "VertexInput",
        vec![
            naga::StructMember {
                name: Some("position".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(0)),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(1)),
                offset: 8,
            },
            naga::StructMember {
                name: Some("mvp_row0".to_string()),
                ty: vec4_ty,
                binding: Some(make_location_binding(2)),
                offset: 16,
            },
            naga::StructMember {
                name: Some("mvp_row1".to_string()),
                ty: vec4_ty,
                binding: Some(make_location_binding(3)),
                offset: 32,
            },
            naga::StructMember {
                name: Some("mvp_row2".to_string()),
                ty: vec4_ty,
                binding: Some(make_location_binding(4)),
                offset: 48,
            },
            naga::StructMember {
                name: Some("mvp_row3".to_string()),
                ty: vec4_ty,
                binding: Some(make_location_binding(5)),
                offset: 64,
            },
            naga::StructMember {
                name: Some("tint".to_string()),
                ty: vec4_ty,
                binding: Some(make_location_binding(6)),
                offset: 80,
            },
            naga::StructMember {
                name: Some("uv_transform".to_string()),
                ty: vec4_ty,
                binding: Some(make_location_binding(7)),
                offset: 96,
            },
        ],
        112,
    );

    let vertex_output_ty = builder.create_struct(
        "VertexOutput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(Binding::Location {
                    location: 0,
                    interpolation: Some(Interpolation::Linear),
                    sampling: None,
                    blend_src: None,
                    per_primitive: false,
                }),
                offset: 16,
            },
            naga::StructMember {
                name: Some("tint".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::Location {
                    location: 1,
                    interpolation: Some(Interpolation::Linear),
                    sampling: None,
                    blend_src: None,
                    per_primitive: false,
                }),
                offset: 24,
            },
        ],
        40,
    );

    let mut vs_function = naga::Function::default();
    vs_function.name = Some("vs_main".to_string());

    let arg_idx = vs_function.arguments.len() as u32;
    vs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: vertex_input_ty, binding: None });
    vs_function.result = Some(FunctionResult { ty: vertex_output_ty, binding: None });

    let mut expressions = naga::Arena::new();
    let mut body = naga::Block::new();

    let input_expr = expressions.append(Expression::FunctionArgument(arg_idx), NagaSpan::UNDEFINED);
    let zero_f32 = expressions.append(Expression::Literal(naga::Literal::F32(0.0)), NagaSpan::UNDEFINED);
    let one_f32 = expressions.append(Expression::Literal(naga::Literal::F32(1.0)), NagaSpan::UNDEFINED);

    let position_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 0 }, NagaSpan::UNDEFINED);
    let uv_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 1 }, NagaSpan::UNDEFINED);
    let mvp_row0 = expressions.append(Expression::AccessIndex { base: input_expr, index: 2 }, NagaSpan::UNDEFINED);
    let mvp_row1 = expressions.append(Expression::AccessIndex { base: input_expr, index: 3 }, NagaSpan::UNDEFINED);
    let mvp_row2 = expressions.append(Expression::AccessIndex { base: input_expr, index: 4 }, NagaSpan::UNDEFINED);
    let mvp_row3 = expressions.append(Expression::AccessIndex { base: input_expr, index: 5 }, NagaSpan::UNDEFINED);
    let tint_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 6 }, NagaSpan::UNDEFINED);
    let uv_transform_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 7 }, NagaSpan::UNDEFINED);

    let mvp = expressions.append(
        Expression::Compose { ty: mat4_ty, components: vec![mvp_row0, mvp_row1, mvp_row2, mvp_row3] },
        NagaSpan::UNDEFINED,
    );

    let pos_x = expressions.append(Expression::AccessIndex { base: position_expr, index: 0 }, NagaSpan::UNDEFINED);
    let pos_y = expressions.append(Expression::AccessIndex { base: position_expr, index: 1 }, NagaSpan::UNDEFINED);
    let pos_vec4 = expressions
        .append(Expression::Compose { ty: vec4_ty, components: vec![pos_x, pos_y, zero_f32, one_f32] }, NagaSpan::UNDEFINED);

    let clip_pos = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: mvp, right: pos_vec4 }, NagaSpan::UNDEFINED);

    let uv_transform_x = expressions.append(Expression::AccessIndex { base: uv_transform_expr, index: 0 }, NagaSpan::UNDEFINED);
    let uv_transform_y = expressions.append(Expression::AccessIndex { base: uv_transform_expr, index: 1 }, NagaSpan::UNDEFINED);
    let uv_transform_z = expressions.append(Expression::AccessIndex { base: uv_transform_expr, index: 2 }, NagaSpan::UNDEFINED);
    let uv_transform_w = expressions.append(Expression::AccessIndex { base: uv_transform_expr, index: 3 }, NagaSpan::UNDEFINED);

    let uv_offset = expressions
        .append(Expression::Compose { ty: vec2_ty, components: vec![uv_transform_x, uv_transform_y] }, NagaSpan::UNDEFINED);
    let uv_scale = expressions
        .append(Expression::Compose { ty: vec2_ty, components: vec![uv_transform_z, uv_transform_w] }, NagaSpan::UNDEFINED);
    let uv_mul = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: uv_expr, right: uv_scale }, NagaSpan::UNDEFINED);
    let out_uv =
        expressions.append(Expression::Binary { op: BinaryOperator::Add, left: uv_offset, right: uv_mul }, NagaSpan::UNDEFINED);

    let output = expressions.append(
        Expression::Compose { ty: vertex_output_ty, components: vec![clip_pos, out_uv, tint_expr] },
        NagaSpan::UNDEFINED,
    );

    body.push(Statement::Emit(NagaRange::new_from_bounds(position_expr, output)), NagaSpan::UNDEFINED);
    body.push(Statement::Return { value: Some(output) }, NagaSpan::UNDEFINED);

    vs_function.expressions = expressions;
    vs_function.body = body;

    let vs_entry = make_entry_point(vs_function, "vs_main", naga::ShaderStage::Vertex);

    let fragment_input_ty = builder.create_struct(
        "FragmentInput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(Binding::Location {
                    location: 0,
                    interpolation: Some(Interpolation::Linear),
                    sampling: None,
                    blend_src: None,
                    per_primitive: false,
                }),
                offset: 16,
            },
            naga::StructMember {
                name: Some("tint".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::Location {
                    location: 1,
                    interpolation: Some(Interpolation::Linear),
                    sampling: None,
                    blend_src: None,
                    per_primitive: false,
                }),
                offset: 24,
            },
        ],
        40,
    );

    let mut fs_function = naga::Function::default();
    fs_function.name = Some("fs_main".to_string());

    let fs_arg_idx = fs_function.arguments.len() as u32;
    fs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: fragment_input_ty, binding: None });
    fs_function.result = Some(FunctionResult {
        ty: vec4_ty,
        binding: Some(Binding::Location {
            location: 0,
            interpolation: Some(Interpolation::Linear),
            sampling: None,
            blend_src: None,
            per_primitive: false,
        }),
    });

    let mut fs_expressions = naga::Arena::new();
    let mut fs_body = naga::Block::new();

    let fs_input_expr = fs_expressions.append(Expression::FunctionArgument(fs_arg_idx), NagaSpan::UNDEFINED);
    let sampler_expr = fs_expressions.append(Expression::GlobalVariable(tex_sampler_gv), NagaSpan::UNDEFINED);
    let tex_expr = fs_expressions.append(Expression::GlobalVariable(tex_gv), NagaSpan::UNDEFINED);

    let fs_uv_expr = fs_expressions.append(Expression::AccessIndex { base: fs_input_expr, index: 1 }, NagaSpan::UNDEFINED);
    let fs_tint_expr = fs_expressions.append(Expression::AccessIndex { base: fs_input_expr, index: 2 }, NagaSpan::UNDEFINED);

    let tex_color = fs_expressions.append(
        Expression::ImageSample {
            image: tex_expr,
            sampler: sampler_expr,
            gather: None,
            coordinate: fs_uv_expr,
            array_index: None,
            offset: None,
            level: SampleLevel::Auto,
            depth_ref: None,
            clamp_to_edge: false,
        },
        NagaSpan::UNDEFINED,
    );

    let final_color = fs_expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: tex_color, right: fs_tint_expr }, NagaSpan::UNDEFINED);

    fs_body.push(Statement::Emit(NagaRange::new_from_bounds(fs_uv_expr, final_color)), NagaSpan::UNDEFINED);
    fs_body.push(Statement::Return { value: Some(final_color) }, NagaSpan::UNDEFINED);

    fs_function.expressions = fs_expressions;
    fs_function.body = fs_body;

    let fs_entry = make_entry_point(fs_function, "fs_main", naga::ShaderStage::Fragment);

    Ok(builder.build_module(vec![vs_entry, fs_entry]))
}

/// 构建圆角矩形着色器的 naga Module
pub fn builtin_rounded_rect_shader() -> GResult<naga::Module> {
    let mut builder = BuiltinShaderBuilder::new();

    let mat4_ty = builder.mat4x4_ty();
    let vec4_ty = builder.vec4_ty();
    let vec2_ty = builder.vec2_ty();

    let uniforms_ty = builder.create_struct(
        "Uniforms",
        vec![
            naga::StructMember { name: Some("mvp".to_string()), ty: mat4_ty, binding: None, offset: 0 },
            naga::StructMember { name: Some("rect_size".to_string()), ty: vec4_ty, binding: None, offset: 64 },
            naga::StructMember { name: Some("color".to_string()), ty: vec4_ty, binding: None, offset: 80 },
        ],
        96,
    );

    let uniforms_gv = builder.add_uniform_global("uniforms", uniforms_ty, 0, 0);

    let vertex_input_ty = builder.create_struct(
        "VertexInput",
        vec![
            naga::StructMember {
                name: Some("position".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(0)),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(1)),
                offset: 8,
            },
        ],
        16,
    );

    let vertex_output_ty = builder.create_struct(
        "VertexOutput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("local_pos".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(0)),
                offset: 16,
            },
        ],
        24,
    );

    let mut vs_function = naga::Function::default();
    vs_function.name = Some("vs_main".to_string());

    let arg_idx = vs_function.arguments.len() as u32;
    vs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: vertex_input_ty, binding: None });
    vs_function.result = Some(FunctionResult { ty: vertex_output_ty, binding: None });

    let mut expressions = naga::Arena::new();
    let mut body = naga::Block::new();

    let input_expr = expressions.append(Expression::FunctionArgument(arg_idx), NagaSpan::UNDEFINED);
    let uniforms_expr = expressions.append(Expression::GlobalVariable(uniforms_gv), NagaSpan::UNDEFINED);

    let zero_f32 = expressions.append(Expression::Literal(naga::Literal::F32(0.0)), NagaSpan::UNDEFINED);
    let one_f32 = expressions.append(Expression::Literal(naga::Literal::F32(1.0)), NagaSpan::UNDEFINED);

    let uv_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 1 }, NagaSpan::UNDEFINED);

    let mvp_ptr = expressions.append(Expression::AccessIndex { base: uniforms_expr, index: 0 }, NagaSpan::UNDEFINED);
    let mvp_expr = expressions.append(Expression::Load { pointer: mvp_ptr }, NagaSpan::UNDEFINED);
    let rect_size_ptr = expressions.append(Expression::AccessIndex { base: uniforms_expr, index: 1 }, NagaSpan::UNDEFINED);
    let rect_size_expr = expressions.append(Expression::Load { pointer: rect_size_ptr }, NagaSpan::UNDEFINED);

    let pos_vec4 = expressions.append(
        Expression::Compose { ty: vec4_ty, components: vec![zero_f32, zero_f32, zero_f32, one_f32] },
        NagaSpan::UNDEFINED,
    );

    let clip_pos = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: mvp_expr, right: pos_vec4 }, NagaSpan::UNDEFINED);

    let rect_size_xy = expressions.append(Expression::AccessIndex { base: rect_size_expr, index: 0 }, NagaSpan::UNDEFINED);
    let rect_size_y = expressions.append(Expression::AccessIndex { base: rect_size_expr, index: 1 }, NagaSpan::UNDEFINED);

    let uv_0 = expressions.append(Expression::AccessIndex { base: uv_expr, index: 0 }, NagaSpan::UNDEFINED);
    let uv_1 = expressions.append(Expression::AccessIndex { base: uv_expr, index: 1 }, NagaSpan::UNDEFINED);

    let local_pos_x = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: uv_0, right: rect_size_xy }, NagaSpan::UNDEFINED);
    let local_pos_y = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: uv_1, right: rect_size_y }, NagaSpan::UNDEFINED);

    let local_pos = expressions
        .append(Expression::Compose { ty: vec2_ty, components: vec![local_pos_x, local_pos_y] }, NagaSpan::UNDEFINED);

    let output = expressions
        .append(Expression::Compose { ty: vertex_output_ty, components: vec![clip_pos, local_pos] }, NagaSpan::UNDEFINED);

    body.push(Statement::Emit(NagaRange::new_from_bounds(uv_expr, output)), NagaSpan::UNDEFINED);
    body.push(Statement::Return { value: Some(output) }, NagaSpan::UNDEFINED);

    vs_function.expressions = expressions;
    vs_function.body = body;

    let vs_entry = make_entry_point(vs_function, "vs_main", naga::ShaderStage::Vertex);

    let fragment_input_ty = builder.create_struct(
        "FragmentInput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("local_pos".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(0)),
                offset: 16,
            },
        ],
        24,
    );

    let mut fs_function = naga::Function::default();
    fs_function.name = Some("fs_main".to_string());

    let _fs_arg_idx = fs_function.arguments.len() as u32;
    fs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: fragment_input_ty, binding: None });
    fs_function.result = Some(FunctionResult { ty: vec4_ty, binding: Some(make_location_binding(0)) });

    let mut fs_expressions = naga::Arena::new();
    let mut fs_body = naga::Block::new();

    let fs_uniforms_expr = fs_expressions.append(Expression::GlobalVariable(uniforms_gv), NagaSpan::UNDEFINED);
    let fs_color_ptr = fs_expressions.append(Expression::AccessIndex { base: fs_uniforms_expr, index: 2 }, NagaSpan::UNDEFINED);
    let fs_color_expr = fs_expressions.append(Expression::Load { pointer: fs_color_ptr }, NagaSpan::UNDEFINED);

    fs_body.push(Statement::Emit(NagaRange::new_from_bounds(fs_color_ptr, fs_color_expr)), NagaSpan::UNDEFINED);
    fs_body.push(Statement::Return { value: Some(fs_color_expr) }, NagaSpan::UNDEFINED);

    fs_function.expressions = fs_expressions;
    fs_function.body = fs_body;

    let fs_entry = make_entry_point(fs_function, "fs_main", naga::ShaderStage::Fragment);

    Ok(builder.build_module(vec![vs_entry, fs_entry]))
}

/// 构建椭圆着色器的 naga Module
pub fn builtin_ellipse_shader() -> GResult<naga::Module> {
    let mut builder = BuiltinShaderBuilder::new();

    let mat4_ty = builder.mat4x4_ty();
    let vec4_ty = builder.vec4_ty();
    let vec2_ty = builder.vec2_ty();

    let uniforms_ty = builder.create_struct(
        "Uniforms",
        vec![
            naga::StructMember { name: Some("mvp".to_string()), ty: mat4_ty, binding: None, offset: 0 },
            naga::StructMember { name: Some("ellipse_params".to_string()), ty: vec4_ty, binding: None, offset: 64 },
            naga::StructMember { name: Some("fill_color".to_string()), ty: vec4_ty, binding: None, offset: 80 },
            naga::StructMember { name: Some("border_params".to_string()), ty: vec4_ty, binding: None, offset: 96 },
        ],
        112,
    );

    let uniforms_gv = builder.add_uniform_global("uniforms", uniforms_ty, 0, 0);

    let vertex_input_ty = builder.create_struct(
        "VertexInput",
        vec![
            naga::StructMember {
                name: Some("position".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(0)),
                offset: 0,
            },
            naga::StructMember {
                name: Some("uv".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(1)),
                offset: 8,
            },
        ],
        16,
    );

    let vertex_output_ty = builder.create_struct(
        "VertexOutput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("local_pos".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(0)),
                offset: 16,
            },
        ],
        24,
    );

    let mut vs_function = naga::Function::default();
    vs_function.name = Some("vs_main".to_string());

    let arg_idx = vs_function.arguments.len() as u32;
    vs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: vertex_input_ty, binding: None });
    vs_function.result = Some(FunctionResult { ty: vertex_output_ty, binding: None });

    let mut expressions = naga::Arena::new();
    let mut body = naga::Block::new();

    let input_expr = expressions.append(Expression::FunctionArgument(arg_idx), NagaSpan::UNDEFINED);
    let uniforms_expr = expressions.append(Expression::GlobalVariable(uniforms_gv), NagaSpan::UNDEFINED);

    let zero_f32 = expressions.append(Expression::Literal(naga::Literal::F32(0.0)), NagaSpan::UNDEFINED);
    let one_f32 = expressions.append(Expression::Literal(naga::Literal::F32(1.0)), NagaSpan::UNDEFINED);

    let uv_expr = expressions.append(Expression::AccessIndex { base: input_expr, index: 1 }, NagaSpan::UNDEFINED);

    let mvp_ptr = expressions.append(Expression::AccessIndex { base: uniforms_expr, index: 0 }, NagaSpan::UNDEFINED);
    let mvp_expr = expressions.append(Expression::Load { pointer: mvp_ptr }, NagaSpan::UNDEFINED);
    let ellipse_params_ptr = expressions.append(Expression::AccessIndex { base: uniforms_expr, index: 1 }, NagaSpan::UNDEFINED);
    let ellipse_params_expr = expressions.append(Expression::Load { pointer: ellipse_params_ptr }, NagaSpan::UNDEFINED);

    let pos_vec4 = expressions.append(
        Expression::Compose { ty: vec4_ty, components: vec![zero_f32, zero_f32, zero_f32, one_f32] },
        NagaSpan::UNDEFINED,
    );

    let clip_pos = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: mvp_expr, right: pos_vec4 }, NagaSpan::UNDEFINED);

    let ellipse_z = expressions.append(Expression::AccessIndex { base: ellipse_params_expr, index: 2 }, NagaSpan::UNDEFINED);
    let ellipse_w = expressions.append(Expression::AccessIndex { base: ellipse_params_expr, index: 3 }, NagaSpan::UNDEFINED);

    let uv_0 = expressions.append(Expression::AccessIndex { base: uv_expr, index: 0 }, NagaSpan::UNDEFINED);
    let uv_1 = expressions.append(Expression::AccessIndex { base: uv_expr, index: 1 }, NagaSpan::UNDEFINED);

    let local_pos_x = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: uv_0, right: ellipse_z }, NagaSpan::UNDEFINED);
    let local_pos_y = expressions
        .append(Expression::Binary { op: BinaryOperator::Multiply, left: uv_1, right: ellipse_w }, NagaSpan::UNDEFINED);

    let local_pos = expressions
        .append(Expression::Compose { ty: vec2_ty, components: vec![local_pos_x, local_pos_y] }, NagaSpan::UNDEFINED);

    let output = expressions
        .append(Expression::Compose { ty: vertex_output_ty, components: vec![clip_pos, local_pos] }, NagaSpan::UNDEFINED);

    body.push(Statement::Emit(NagaRange::new_from_bounds(uv_expr, output)), NagaSpan::UNDEFINED);
    body.push(Statement::Return { value: Some(output) }, NagaSpan::UNDEFINED);

    vs_function.expressions = expressions;
    vs_function.body = body;

    let vs_entry = make_entry_point(vs_function, "vs_main", naga::ShaderStage::Vertex);

    let fragment_input_ty = builder.create_struct(
        "FragmentInput",
        vec![
            naga::StructMember {
                name: Some("clip_position".to_string()),
                ty: vec4_ty,
                binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                offset: 0,
            },
            naga::StructMember {
                name: Some("local_pos".to_string()),
                ty: vec2_ty,
                binding: Some(make_location_binding(0)),
                offset: 16,
            },
        ],
        24,
    );

    let mut fs_function = naga::Function::default();
    fs_function.name = Some("fs_main".to_string());

    let _fs_arg_idx = fs_function.arguments.len() as u32;
    fs_function.arguments.push(FunctionArgument { name: Some("input".to_string()), ty: fragment_input_ty, binding: None });
    fs_function.result = Some(FunctionResult { ty: vec4_ty, binding: Some(make_location_binding(0)) });

    let mut fs_expressions = naga::Arena::new();
    let mut fs_body = naga::Block::new();

    let fs_uniforms_expr = fs_expressions.append(Expression::GlobalVariable(uniforms_gv), NagaSpan::UNDEFINED);
    let fs_fill_color_ptr =
        fs_expressions.append(Expression::AccessIndex { base: fs_uniforms_expr, index: 2 }, NagaSpan::UNDEFINED);
    let fs_fill_color = fs_expressions.append(Expression::Load { pointer: fs_fill_color_ptr }, NagaSpan::UNDEFINED);

    fs_body.push(Statement::Emit(NagaRange::new_from_bounds(fs_fill_color_ptr, fs_fill_color)), NagaSpan::UNDEFINED);
    fs_body.push(Statement::Return { value: Some(fs_fill_color) }, NagaSpan::UNDEFINED);

    fs_function.expressions = fs_expressions;
    fs_function.body = fs_body;

    let fs_entry = make_entry_point(fs_function, "fs_main", naga::ShaderStage::Fragment);

    Ok(builder.build_module(vec![vs_entry, fs_entry]))
}
