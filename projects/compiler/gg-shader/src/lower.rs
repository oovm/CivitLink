//! gs AST → naga IR 转换器
//! 
//! 将 gs 语言的类型化 AST 转换为 naga IR 中间表示，
//! 支持顶点着色器、片段着色器、计算着色器和 uniforms。

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use naga;
use oak_valkyrie::ast::{Shader, Item, MicroDefinition, Namespace, Statement};

/// gs AST → naga IR 转换器
///
/// 将 oak-valkyrie 的 Shader AST 转换为 `naga::Module`，
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

    /// 将 oak-valkyrie 的 Shader AST 转换为 naga Module
    pub fn lower(&mut self, shader: &Shader) -> GResult<naga::Module> {
        self.local_vars.clear();
        self.global_vars.clear();
        self.type_cache.clear();

        let mut module = naga::Module::default();

        // 处理 shader 内的 items，提取 uniforms 和 functions
        let mut uniforms = Vec::new();
        let mut functions = Vec::new();

        for item in &shader.items {
            match item {
                Item::Micro(micro) => {
                    // 处理 micro 函数
                }
                Item::Namespace(namespace) => {
                    // 处理命名空间
                }
                Item::Statement(stmt) => {
                    // 处理语句
                }
                _ => {}
            }
        }

        // 创建 uniform 缓冲区
        let uniform_buffer_members = self.create_uniform_buffer_members(&uniforms, &mut module)?;
        let uniform_buffer_ty = if !uniform_buffer_members.is_empty() {
            let ty = module.types.insert(
                naga::Type {
                    name: Some("Uniforms".to_string()),
                    inner: naga::TypeInner::Struct {
                        members: uniform_buffer_members,
                        span: 0,
                    },
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
                    init: None,
                    memory_decorations: naga::MemoryDecorations::empty(),
                },
                naga::Span::UNDEFINED,
            );
            self.global_vars.insert("uniforms".to_string(), gv);
            Some(ty)
        } else {
            None
        };

        // 处理 functions
        for func in &functions {
            let entry_point = self.lower_entry_point(func, &mut module, uniform_buffer_ty)?;
            module.entry_points.push(entry_point);
        }

        Ok(module)
    }

    /// 创建 uniform 缓冲区结构体成员
    fn create_uniform_buffer_members(
        &mut self,
        uniforms: &Vec<()>,
        module: &mut naga::Module,
    ) -> GResult<Vec<naga::StructMember>> {
        let mut members = Vec::new();
        let mut offset = 0u32;

        // TODO: 从 oak-valkyrie 的 AST 中提取 uniform 成员
        // 目前返回空的成员列表

        Ok(members)
    }

    /// 将函数转换为 naga EntryPoint
    fn lower_entry_point(
        &mut self,
        func: &(),
        module: &mut naga::Module,
        _uniform_buffer_ty: Option<naga::Handle<naga::Type>>,
    ) -> GResult<naga::EntryPoint> {
        self.local_vars.clear();

        // TODO: 从 oak-valkyrie 的 AST 中提取函数信息
        let stage = naga::ShaderStage::Vertex;
        let mut function = naga::Function::default();
        function.name = Some("vertex".to_string());

        // TODO: 处理函数参数

        // TODO: 处理返回类型

        // TODO: 处理函数体

        Ok(naga::EntryPoint {
            name: "vertex".to_string(),
            stage,
            early_depth_test: None,
            workgroup_size: [0; 3],
            workgroup_size_overrides: None,
            function,
            mesh_info: None,
            task_payload: None,
            incoming_ray_payload: None,
        })
    }
}

impl Default for GslLowerer {
    fn default() -> Self {
        Self::new()
    }
}
