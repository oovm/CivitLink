//! gs AST → naga IR 转换器
//!
//! 将 gs 语言的类型化 AST 转换为 naga IR 中间表示，
//! 支持顶点着色器、片段着色器、计算着色器和 uniforms。

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use naga;

use crate::ast::*;

/// gs AST → naga IR 转换器
///
/// 将 `GslShaderBlock` AST 转换为 `naga::Module`，
/// 支持顶点着色器、片段着色器、计算着色器和 uniforms。
pub struct GslLowerer {
    /// 局部变量名到 naga 句柄的映射
    local_vars: HashMap<String, naga::Handle<naga::LocalVariable>>,
    /// 全局变量名到 naga 句柄的映射
    global_vars: HashMap<String, naga::Handle<naga::GlobalVariable>>,
    /// 类型缓存，避免重复创建
    type_cache: HashMap<String, naga::Handle<naga::Type>>,
}

impl GslLowerer {
    /// 创建新的转换器
    pub fn new() -> Self {
        Self {
            local_vars: HashMap::new(),
            global_vars: HashMap::new(),
            type_cache: HashMap::new(),
        }
    }

    /// 将 gs AST 着色器块转换为 naga Module
    pub fn lower(&mut self, shader: &GslShaderBlock) -> GResult<naga::Module> {
        self.local_vars.clear();
        self.global_vars.clear();
        self.type_cache.clear();

        let mut module = naga::Module::default();

        let uniform_buffer_members = self.create_uniform_buffer_members(shader, &mut module)?;
        let uniform_buffer_ty = if !uniform_buffer_members.is_empty() {
            let ty = module.types.insert(
                naga::Type::Struct {
                    members: uniform_buffer_members,
                    span: 0,
                },
                naga::Span::UNDEFINED,
            );
            let gv = module.global_variables.append(
                naga::GlobalVariable {
                    name: Some("uniforms".to_string()),
                    space: naga::AddressSpace::Uniform,
                    binding: Some(naga::ResourceBinding {
                        group: 0,
                        binding: 0,
                    }),
                    ty,
                },
                naga::Span::UNDEFINED,
            );
            self.global_vars.insert("uniforms".to_string(), gv);
            Some(ty)
        } else {
            None
        };

        let mut next_texture_binding = 0u32;
        for uniform in &shader.uniforms {
            match uniform.ty {
                GslType::Texture2D | GslType::Texture3D | GslType::TextureCube => {
                    let dim = match uniform.ty {
                        GslType::Texture2D => naga::TextureDimension::D2,
                        GslType::Texture3D => naga::TextureDimension::D3,
                        GslType::TextureCube => naga::TextureDimension::Cube,
                        _ => naga::TextureDimension::D2,
                    };
                    let tex_ty = module.types.insert(
                        naga::Type::Texture {
                            dim,
                            arrayed: false,
                            class: naga::TextureClass::Sampled {
                                multi: false,
                                kind: naga::ScalarKind::Float,
                            },
                        },
                        naga::Span::UNDEFINED,
                    );
                    let binding_group = get_decorator_group(&uniform.decorators).unwrap_or(1);
                    let binding_idx = get_decorator_binding(&uniform.decorators).unwrap_or(next_texture_binding);
                    next_texture_binding = next_texture_binding.max(binding_idx + 1);
                    let gv = module.global_variables.append(
                        naga::GlobalVariable {
                            name: Some(uniform.name.clone()),
                            space: naga::AddressSpace::Handle,
                            binding: Some(naga::ResourceBinding {
                                group: binding_group,
                                binding: binding_idx,
                            }),
                            ty: tex_ty,
                        },
                        naga::Span::UNDEFINED,
                    );
                    self.global_vars.insert(uniform.name.clone(), gv);
                }
                GslType::Sampler => {
                    let sampler_ty = module.types.insert(
                        naga::Type::Sampler {
                            comparison: false,
                        },
                        naga::Span::UNDEFINED,
                    );
                    let binding_group = get_decorator_group(&uniform.decorators).unwrap_or(1);
                    let binding_idx = get_decorator_binding(&uniform.decorators).unwrap_or(next_texture_binding);
                    next_texture_binding = next_texture_binding.max(binding_idx + 1);
                    let gv = module.global_variables.append(
                        naga::GlobalVariable {
                            name: Some(uniform.name.clone()),
                            space: naga::AddressSpace::Handle,
                            binding: Some(naga::ResourceBinding {
                                group: binding_group,
                                binding: binding_idx,
                            }),
                            ty: sampler_ty,
                        },
                        naga::Span::UNDEFINED,
                    );
                    self.global_vars.insert(uniform.name.clone(), gv);
                }
                _ => {}
            }
        }

        for func in &shader.functions {
            let entry_point = self.lower_entry_point(func, &mut module, uniform_buffer_ty)?;
            module.entry_points.push(entry_point);
        }

        Ok(module)
    }

    /// 创建 uniform 缓冲区结构体成员
    fn create_uniform_buffer_members(
        &mut self,
        shader: &GslShaderBlock,
        module: &mut naga::Module,
    ) -> GResult<Vec<naga::StructMember>> {
        let mut members = Vec::new();
        let mut offset = 0u32;

        for uniform in &shader.uniforms {
            match uniform.ty {
                GslType::Texture2D | GslType::Texture3D | GslType::TextureCube | GslType::Sampler => {
                    continue;
                }
                _ => {}
            }

            let ty = self.lower_type(&uniform.ty, module);
            let size = self.type_size(&uniform.ty);
            if offset % 16 != 0 {
                offset = ((offset / 16) + 1) * 16;
            }
            members.push(naga::StructMember {
                name: Some(uniform.name.clone()),
                ty,
                binding: None,
                offset,
            });
            offset += size;
        }

        Ok(members)
    }

    /// 将 gs 类型转换为 naga 类型句柄
    fn lower_type(&mut self, ty: &GslType, module: &mut naga::Module) -> naga::Handle<naga::Type> {
        let key = format!("{:?}", ty);
        if let Some(&handle) = self.type_cache.get(&key) {
            return handle;
        }

        let naga_ty = match ty {
            GslType::Scalar(s) => naga::Type::Scalar {
                scalar: match s {
                    GslScalarType::Bool => naga::Scalar::BOOL,
                    GslScalarType::U32 => naga::Scalar::U32,
                    GslScalarType::I32 => naga::Scalar::I32,
                    GslScalarType::F32 => naga::Scalar::F32,
                },
            },
            GslType::Vector { size, scalar } => naga::Type::Vector {
                size: match size {
                    GslVectorSize::Bi => naga::VectorSize::Bi,
                    GslVectorSize::Tri => naga::VectorSize::Tri,
                    GslVectorSize::Quad => naga::VectorSize::Quad,
                },
                scalar: match scalar {
                    GslScalarType::Bool => naga::Scalar::BOOL,
                    GslScalarType::U32 => naga::Scalar::U32,
                    GslScalarType::I32 => naga::Scalar::I32,
                    GslScalarType::F32 => naga::Scalar::F32,
                },
            },
            GslType::Matrix { columns, rows, scalar } => naga::Type::Matrix {
                columns: match columns {
                    GslVectorSize::Bi => naga::VectorSize::Bi,
                    GslVectorSize::Tri => naga::VectorSize::Tri,
                    GslVectorSize::Quad => naga::VectorSize::Quad,
                },
                rows: match rows {
                    GslVectorSize::Bi => naga::VectorSize::Bi,
                    GslVectorSize::Tri => naga::VectorSize::Tri,
                    GslVectorSize::Quad => naga::VectorSize::Quad,
                },
                scalar: match scalar {
                    GslScalarType::F32 => naga::Scalar::F32,
                    _ => naga::Scalar::F32,
                },
            },
            GslType::Texture2D => naga::Type::Texture {
                dim: naga::TextureDimension::D2,
                arrayed: false,
                class: naga::TextureClass::Sampled {
                    multi: false,
                    kind: naga::ScalarKind::Float,
                },
            },
            GslType::Texture3D => naga::Type::Texture {
                dim: naga::TextureDimension::D3,
                arrayed: false,
                class: naga::TextureClass::Sampled {
                    multi: false,
                    kind: naga::ScalarKind::Float,
                },
            },
            GslType::TextureCube => naga::Type::Texture {
                dim: naga::TextureDimension::Cube,
                arrayed: false,
                class: naga::TextureClass::Sampled {
                    multi: false,
                    kind: naga::ScalarKind::Float,
                },
            },
            GslType::Sampler => naga::Type::Sampler { comparison: false },
            GslType::Buffer(inner) => {
                let inner_ty = self.lower_type(inner, module);
                naga::Type::BindingArray {
                    base: inner_ty,
                    size: naga::BindingArraySize::Dynamic,
                }
            }
            GslType::Struct { fields } => {
                let mut naga_members = Vec::new();
                let mut offset = 0u32;
                for field in fields {
                    let field_ty = self.lower_type(&field.ty, module);
                    let size = self.type_size(&field.ty);
                    if offset % 16 != 0 {
                        offset = ((offset / 16) + 1) * 16;
                    }
                    naga_members.push(naga::StructMember {
                        name: Some(field.name.clone()),
                        ty: field_ty,
                        binding: self.lower_binding_opt(&field.decorators),
                        offset,
                    });
                    offset += size;
                }
                naga::Type::Struct {
                    members: naga_members,
                    span: offset,
                }
            }
        };

        let handle = module.types.insert(naga_ty, naga::Span::UNDEFINED);
        self.type_cache.insert(key, handle);
        handle
    }

    /// 将 gs 装饰器列表转换为 naga Binding（可选）
    fn lower_binding_opt(&self, decorators: &[GslDecorator]) -> Option<naga::Binding> {
        for dec in decorators {
            match dec {
                GslDecorator::Location(loc) => {
                    return Some(naga::Binding::Location {
                        location: *loc,
                        interpolation: None,
                        sampling: None,
                    });
                }
                GslDecorator::Builtin(kind) => {
                    return Some(naga::Binding::BuiltIn(match kind {
                        GslBuiltinKind::Position => naga::BuiltIn::Position { invariant: false },
                        GslBuiltinKind::GlobalInvocationId => naga::BuiltIn::GlobalInvocationId,
                        GslBuiltinKind::VertexIndex => naga::BuiltIn::VertexIndex,
                        GslBuiltinKind::InstanceIndex => naga::BuiltIn::InstanceIndex,
                    }));
                }
                _ => {}
            }
        }
        None
    }

    /// 将 gs 函数转换为 naga EntryPoint
    fn lower_entry_point(
        &mut self,
        func: &GslFunction,
        module: &mut naga::Module,
        _uniform_buffer_ty: Option<naga::Handle<naga::Type>>,
    ) -> GResult<naga::EntryPoint> {
        self.local_vars.clear();

        let stage = self.lower_stage(&func.kind);
        let mut function = naga::Function::default();
        function.name = Some(func.name.clone());

        for param in &func.params {
            let ty = self.lower_type(&param.ty, module);
            let binding = self.lower_binding_opt(&param.decorators);
            function.arguments.push(naga::FunctionArgument {
                name: Some(param.name.clone()),
                ty,
                binding,
            });
        }

        if let Some(ref ret_ty) = func.return_type {
            let ty = self.lower_type(ret_ty, module);
            let binding = self.lower_binding_opt(&func.return_decorators);
            function.result = Some(naga::FunctionResult {
                ty,
                binding,
            });
        }

        let mut stmts = Vec::new();
        for stmt in &func.body {
            let lowered = self.lower_stmt(stmt, &mut function, module)?;
            stmts.extend(lowered);
        }

        for s in stmts {
            function.body.push(s, naga::Span::UNDEFINED);
        }

        Ok(naga::EntryPoint {
            name: func.name.clone(),
            stage,
            early_depth_test: None,
            workgroup_size: [0; 3],
            function,
        })
    }

    /// 将 gs 表达式转换为 naga Expression 句柄
    fn lower_expr(
        &self,
        expr: &GslExpr,
        function: &mut naga::Function,
        module: &naga::Module,
    ) -> GResult<naga::Handle<naga::Expression>> {
        let naga_expr = match expr {
            GslExpr::LiteralInt(n) => naga::Expression::Literal(naga::Literal::I32(*n as i32)),
            GslExpr::LiteralFloat(f) => naga::Expression::Literal(naga::Literal::F32(*f as f32)),
            GslExpr::LiteralBool(b) => naga::Expression::Literal(naga::Literal::Bool(*b)),
            GslExpr::Variable(name) => {
                if let Some(&local_handle) = self.local_vars.get(name) {
                    naga::Expression::Variable(naga::Variable::Local(local_handle))
                } else if let Some(&global_handle) = self.global_vars.get(name) {
                    naga::Expression::GlobalVariable(global_handle)
                } else {
                    return Err(GError {
                        kind: GErrorKind::Other,
                        message: format!("未定义的变量: '{}'", name),
                    });
                }
            }
            GslExpr::Binary { left, op, right } => {
                let left_handle = self.lower_expr(left, function, module)?;
                let right_handle = self.lower_expr(right, function, module)?;
                naga::Expression::Binary {
                    op: self.lower_binary_op(op),
                    left: left_handle,
                    right: right_handle,
                }
            }
            GslExpr::Unary { op, operand } => {
                let operand_handle = self.lower_expr(operand, function, module)?;
                naga::Expression::Unary {
                    op: self.lower_unary_op(op),
                    expr: operand_handle,
                }
            }
            GslExpr::Call { func: fn_name, args } => {
                let lowered_args: GResult<Vec<_>> = args
                    .iter()
                    .map(|a| self.lower_expr(a, function, module))
                    .collect();
                let arg_handles = lowered_args?;

                match fn_name.as_str() {
                    "normalize" | "dot" | "cross" | "max" | "min" | "clamp" | "mix" | "abs"
                    | "sign" | "floor" | "ceil" | "fract" | "sqrt" | "pow" | "exp" | "log"
                    | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "length" | "distance"
                    | "reflect" | "refract" | "step" | "smoothstep" => {
                        let math_fun = self.lower_math_fun(fn_name);
                        let arg = arg_handles.first().copied().unwrap_or_else(|| {
                            function.expressions.append(
                                naga::Expression::Literal(naga::Literal::F32(0.0)),
                                naga::Span::UNDEFINED,
                            )
                        });
                        let arg1 = arg_handles.get(1).copied();
                        let arg2 = arg_handles.get(2).copied();
                        let arg3 = arg_handles.get(3).copied();
                        naga::Expression::Math {
                            fun: math_fun,
                            arg,
                            arg1,
                            arg2,
                            arg3,
                        }
                    }
                    _ => {
                        if let Some(&global_handle) = self.global_vars.get(fn_name) {
                            naga::Expression::GlobalVariable(global_handle)
                        } else {
                            return Err(GError {
                                kind: GErrorKind::Other,
                                message: format!("未定义的函数: '{}'", fn_name),
                            });
                        }
                    }
                }
            }
            GslExpr::VectorConstruct { scalar, size, args } => {
                let lowered_args: GResult<Vec<_>> = args
                    .iter()
                    .map(|a| self.lower_expr(a, function, module))
                    .collect();
                let components = lowered_args?;
                let ty = {
                    let scalar_naga = match scalar {
                        GslScalarType::F32 => naga::Scalar::F32,
                        GslScalarType::I32 => naga::Scalar::I32,
                        GslScalarType::U32 => naga::Scalar::U32,
                        GslScalarType::Bool => naga::Scalar::BOOL,
                    };
                    let size_naga = match size {
                        GslVectorSize::Bi => naga::VectorSize::Bi,
                        GslVectorSize::Tri => naga::VectorSize::Tri,
                        GslVectorSize::Quad => naga::VectorSize::Quad,
                    };
                    module.types.fetch_or_append(
                        naga::Type::Vector {
                            size: size_naga,
                            scalar: scalar_naga,
                        },
                        naga::Span::UNDEFINED,
                    )
                };
                naga::Expression::Compose { ty, components }
            }
            GslExpr::MatrixConstruct { scalar, columns, rows, args } => {
                let lowered_args: GResult<Vec<_>> = args
                    .iter()
                    .map(|a| self.lower_expr(a, function, module))
                    .collect();
                let components = lowered_args?;
                let ty = {
                    let scalar_naga = match scalar {
                        GslScalarType::F32 => naga::Scalar::F32,
                        _ => naga::Scalar::F32,
                    };
                    let columns_naga = match columns {
                        GslVectorSize::Bi => naga::VectorSize::Bi,
                        GslVectorSize::Tri => naga::VectorSize::Tri,
                        GslVectorSize::Quad => naga::VectorSize::Quad,
                    };
                    let rows_naga = match rows {
                        GslVectorSize::Bi => naga::VectorSize::Bi,
                        GslVectorSize::Tri => naga::VectorSize::Tri,
                        GslVectorSize::Quad => naga::VectorSize::Quad,
                    };
                    module.types.fetch_or_append(
                        naga::Type::Matrix {
                            columns: columns_naga,
                            rows: rows_naga,
                            scalar: scalar_naga,
                        },
                        naga::Span::UNDEFINED,
                    )
                };
                naga::Expression::Compose { ty, components }
            }
            GslExpr::Member { object, member } => {
                let base = self.lower_expr(object, function, module)?;
                let field_index = self.lookup_member_index(object, member, module)?;
                naga::Expression::Member { base, field_index }
            }
            GslExpr::Index { object, index } => {
                let base = self.lower_expr(object, function, module);
                let index_handle = self.lower_expr(index, function, module);
                match (base, index_handle) {
                    (Ok(b), Ok(i)) => naga::Expression::Access { base, index: i },
                    (Err(e), _) | (_, Err(e)) => return Err(e),
                }
            }
            GslExpr::TextureSample { texture, sampler, coords } => {
                let texture_handle = self.lower_expr(texture, function, module)?;
                let sampler_handle = self.lower_expr(sampler, function, module)?;
                let coords_handle = self.lower_expr(coords, function, module)?;

                let texture_ty = self.get_expression_type(texture_handle, function, module);
                let dim = match texture_ty {
                    Some(naga::Type::Texture { dim, .. }) => dim,
                    _ => naga::TextureDimension::D2,
                };

                naga::Expression::ImageSample {
                    image: texture_handle,
                    sampler: sampler_handle,
                    gather: None,
                    coordinate: coords_handle,
                    array_index: None,
                    offset: None,
                    level: naga::SampleLevel::Auto,
                    depth_ref: None,
                }
            }
        };

        let handle = function.expressions.append(naga_expr, naga::Span::UNDEFINED);
        Ok(handle)
    }

    /// 将 gs 语句转换为 naga Statement 列表
    fn lower_stmt(
        &mut self,
        stmt: &GslStmt,
        function: &mut naga::Function,
        module: &naga::Module,
    ) -> GResult<Vec<naga::Statement>> {
        let mut result = Vec::new();

        match stmt {
            GslStmt::Let { name, ty: _, value } | GslStmt::LetMut { name, ty: _, value } => {
                let value_handle = self.lower_expr(value, function, module)?;
                let value_ty = self.infer_expr_type(value, module);

                let local_var = naga::LocalVariable {
                    name: Some(name.clone()),
                    ty: value_ty,
                    init: Some(value_handle),
                };
                let local_handle = function.local_variables.append(local_var, naga::Span::UNDEFINED);
                self.local_vars.insert(name.clone(), local_handle);

                result.push(naga::Statement::Emit(naga::Range::new_from(value_handle)));
            }
            GslStmt::Assign { target, value } => {
                let value_handle = self.lower_expr(value, function, module)?;
                let target_handle = self.lower_expr(target, function, module)?;
                result.push(naga::Statement::Emit(naga::Range::new_from(value_handle)));
                result.push(naga::Statement::Store {
                    pointer: target_handle,
                    value: value_handle,
                });
            }
            GslStmt::Return { value } => {
                if let Some(v) = value {
                    let handle = self.lower_expr(v, function, module)?;
                    result.push(naga::Statement::Emit(naga::Range::new_from(handle)));
                    result.push(naga::Statement::Return { value: Some(handle) });
                } else {
                    result.push(naga::Statement::Return { value: None });
                }
            }
            GslStmt::If { condition, then_block, else_block } => {
                let cond_handle = self.lower_expr(condition, function, module)?;
                result.push(naga::Statement::Emit(naga::Range::new_from(cond_handle)));

                let mut accept = naga::Block::new();
                for s in then_block {
                    let lowered = self.lower_stmt(s, function, module)?;
                    for st in lowered {
                        accept.push(st, naga::Span::UNDEFINED);
                    }
                }

                let reject = if let Some(else_stmts) = else_block {
                    let mut block = naga::Block::new();
                    for s in else_stmts {
                        let lowered = self.lower_stmt(s, function, module)?;
                        for st in lowered {
                            block.push(st, naga::Span::UNDEFINED);
                        }
                    }
                    block
                } else {
                    naga::Block::new()
                };

                result.push(naga::Statement::If {
                    condition: cond_handle,
                    accept,
                    reject,
                });
            }
            GslStmt::Expr { expr } => {
                let handle = self.lower_expr(expr, function, module)?;
                result.push(naga::Statement::Emit(naga::Range::new_from(handle)));
            }
        }

        Ok(result)
    }

    /// 将 gs 二元运算符转换为 naga BinaryOperator
    fn lower_binary_op(&self, op: &GslBinaryOp) -> naga::BinaryOperator {
        match op {
            GslBinaryOp::Add => naga::BinaryOperator::Add,
            GslBinaryOp::Sub => naga::BinaryOperator::Subtract,
            GslBinaryOp::Mul => naga::BinaryOperator::Multiply,
            GslBinaryOp::Div => naga::BinaryOperator::Divide,
            GslBinaryOp::Mod => naga::BinaryOperator::Modulo,
            GslBinaryOp::Eq => naga::BinaryOperator::Equal,
            GslBinaryOp::Ne => naga::BinaryOperator::NotEqual,
            GslBinaryOp::Lt => naga::BinaryOperator::Less,
            GslBinaryOp::Le => naga::BinaryOperator::LessEqual,
            GslBinaryOp::Gt => naga::BinaryOperator::Greater,
            GslBinaryOp::Ge => naga::BinaryOperator::GreaterEqual,
            GslBinaryOp::And => naga::BinaryOperator::LogicalAnd,
            GslBinaryOp::Or => naga::BinaryOperator::LogicalOr,
        }
    }

    /// 将 gs 一元运算符转换为 naga UnaryOperator
    fn lower_unary_op(&self, op: &GslUnaryOp) -> naga::UnaryOperator {
        match op {
            GslUnaryOp::Neg => naga::UnaryOperator::Negate,
            GslUnaryOp::Not => naga::UnaryOperator::Not,
        }
    }

    /// 将 gs 函数类型转换为 naga ShaderStage
    fn lower_stage(&self, kind: &GslFunctionKind) -> naga::ShaderStage {
        match kind {
            GslFunctionKind::Vertex => naga::ShaderStage::Vertex,
            GslFunctionKind::Fragment => naga::ShaderStage::Fragment,
            GslFunctionKind::Compute => naga::ShaderStage::Compute,
            GslFunctionKind::Micro => naga::ShaderStage::Compute,
        }
    }

    /// 将 gs 内置函数名转换为 naga MathFunction
    fn lower_math_fun(&self, name: &str) -> naga::MathFunction {
        match name {
            "abs" => naga::MathFunction::Abs,
            "sign" => naga::MathFunction::Sign,
            "floor" => naga::MathFunction::Floor,
            "ceil" => naga::MathFunction::Ceil,
            "fract" => naga::MathFunction::Fract,
            "sqrt" => naga::MathFunction::Sqrt,
            "pow" => naga::MathFunction::Pow,
            "exp" => naga::MathFunction::Exp,
            "log" => naga::MathFunction::Log,
            "sin" => naga::MathFunction::Sin,
            "cos" => naga::MathFunction::Cos,
            "tan" => naga::MathFunction::Tan,
            "asin" => naga::MathFunction::Asin,
            "acos" => naga::MathFunction::Acos,
            "atan" => naga::MathFunction::Atan,
            "normalize" => naga::MathFunction::Normalize,
            "dot" => naga::MathFunction::Dot,
            "cross" => naga::MathFunction::Cross,
            "max" => naga::MathFunction::Max,
            "min" => naga::MathFunction::Min,
            "clamp" => naga::MathFunction::Clamp,
            "mix" => naga::MathFunction::Mix,
            "length" => naga::MathFunction::Length,
            "distance" => naga::MathFunction::Distance,
            "reflect" => naga::MathFunction::Reflect,
            "refract" => naga::MathFunction::Refract,
            "step" => naga::MathFunction::Step,
            "smoothstep" => naga::MathFunction::SmoothStep,
            _ => naga::MathFunction::Abs,
        }
    }

    /// 查找结构体成员的索引
    fn lookup_member_index(
        &self,
        object: &GslExpr,
        member: &str,
        module: &naga::Module,
    ) -> GResult<u32> {
        if let GslExpr::Variable(name) = object {
            if name == "uniforms" {
                let mut idx = 0u32;
                for (_, ty_handle) in module.types.iter() {
                    if let naga::Type::Struct { members, .. } = ty_handle {
                        for m in members {
                            if m.name.as_deref() == Some(member) {
                                return Ok(idx);
                            }
                            idx += 1;
                        }
                    }
                }
            }
        }
        Ok(0)
    }

    /// 获取表达式对应的 naga 类型
    fn get_expression_type(
        &self,
        _handle: naga::Handle<naga::Expression>,
        _function: &naga::Function,
        module: &naga::Module,
    ) -> Option<naga::Type> {
        None
    }

    /// 推断表达式的类型
    fn infer_expr_type(&self, expr: &GslExpr, module: &naga::Module) -> naga::Handle<naga::Type> {
        match expr {
            GslExpr::LiteralInt(_) => module.types.fetch_or_append(
                naga::Type::Scalar { scalar: naga::Scalar::I32 },
                naga::Span::UNDEFINED,
            ),
            GslExpr::LiteralFloat(_) => module.types.fetch_or_append(
                naga::Type::Scalar { scalar: naga::Scalar::F32 },
                naga::Span::UNDEFINED,
            ),
            GslExpr::LiteralBool(_) => module.types.fetch_or_append(
                naga::Type::Scalar { scalar: naga::Scalar::BOOL },
                naga::Span::UNDEFINED,
            ),
            GslExpr::VectorConstruct { scalar, size, .. } => module.types.fetch_or_append(
                naga::Type::Vector {
                    size: match size {
                        GslVectorSize::Bi => naga::VectorSize::Bi,
                        GslVectorSize::Tri => naga::VectorSize::Tri,
                        GslVectorSize::Quad => naga::VectorSize::Quad,
                    },
                    scalar: match scalar {
                        GslScalarType::F32 => naga::Scalar::F32,
                        GslScalarType::I32 => naga::Scalar::I32,
                        GslScalarType::U32 => naga::Scalar::U32,
                        GslScalarType::Bool => naga::Scalar::BOOL,
                    },
                },
                naga::Span::UNDEFINED,
            ),
            _ => module.types.fetch_or_append(
                naga::Type::Scalar { scalar: naga::Scalar::F32 },
                naga::Span::UNDEFINED,
            ),
        }
    }

    /// 估算类型大小（字节）
    fn type_size(&self, ty: &GslType) -> u32 {
        match ty {
            GslType::Scalar(_) => 4,
            GslType::Vector { size, .. } => match size {
                GslVectorSize::Bi => 8,
                GslVectorSize::Tri => 12,
                GslVectorSize::Quad => 16,
            },
            GslType::Matrix { columns, rows, .. } => {
                let col_size: u32 = match rows {
                    GslVectorSize::Bi => 8,
                    GslVectorSize::Tri => 12,
                    GslVectorSize::Quad => 16,
                };
                let num_cols: u32 = match columns {
                    GslVectorSize::Bi => 2,
                    GslVectorSize::Tri => 3,
                    GslVectorSize::Quad => 4,
                };
                col_size * num_cols
            }
            GslType::Texture2D | GslType::Texture3D | GslType::TextureCube | GslType::Sampler => 0,
            GslType::Buffer(_) => 8,
            GslType::Struct { fields } => {
                let mut total = 0u32;
                for field in fields {
                    let size = self.type_size(&field.ty);
                    if total % 16 != 0 {
                        total = ((total / 16) + 1) * 16;
                    }
                    total += size;
                }
                total
            }
        }
    }
}

impl Default for GslLowerer {
    fn default() -> Self {
        Self::new()
    }
}

/// 从装饰器列表获取 group 值
fn get_decorator_group(decorators: &[GslDecorator]) -> Option<u32> {
    for dec in decorators {
        if let GslDecorator::Group(g) = dec {
            return Some(*g);
        }
    }
    None
}

/// 从装饰器列表获取 binding 值
fn get_decorator_binding(decorators: &[GslDecorator]) -> Option<u32> {
    for dec in decorators {
        if let GslDecorator::Binding(b) = dec {
            return Some(*b);
        }
    }
    None
}
