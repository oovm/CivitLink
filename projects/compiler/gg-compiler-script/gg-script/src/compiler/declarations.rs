use gg_core::GResult;
use gg_ir::{IrFunction, OpCode, TargetPlatform};
use oak_valkyrie::ast::{
    ClassDeclaration, ComponentDeclaration, Enums, MethodDeclaration, MicroDeclaration, SingletonDeclaration,
    StructureDeclaration, SystemDeclaration, Trait, items_nodes::Flags, shader_nodes::ShaderDeclaration,
};

use super::ValkyrieCompiler;

impl ValkyrieCompiler {
    /// 编译 micro 函数定义
    ///
    /// 为每个 micro 函数生成 IrFunction，包含参数、局部变量和指令序列。
    pub(crate) fn compile_micro(&mut self, micro: &MicroDeclaration) -> GResult<IrFunction> {
        self.reset_locals();
        self.process_annotations(micro);

        let param_count = micro.params.len();

        for param in &micro.params {
            self.declare_local(param.name.name.clone());
        }

        let mut instructions = Vec::new();
        self.compile_block(&micro.body, &mut instructions)?;

        instructions.push(OpCode::LoadNull);
        instructions.push(OpCode::Return);

        let is_entry = micro.annotations.iter().any(|a| a.name.name == "main");
        let target = self.resolve_target_annotation(micro);
        Ok(IrFunction {
            name: micro.name.name.clone(),
            param_count,
            local_count: self.next_local,
            local_names: std::mem::take(&mut self.local_names),
            instructions,
            is_entry,
            target,
        })
    }

    /// 编译类对 trait 的实现块
    ///
    /// 当类通过 parents 继承 trait 时，类的方法即为 trait 方法的实现。
    /// 为每个有方法体的方法生成 `类名_trait名_方法名` 格式的 IR 函数。
    pub(crate) fn compile_impl_block(
        &mut self,
        class_name: &str,
        trait_name: &str,
        methods: &[MethodDeclaration],
    ) -> GResult<()> {
        for method in methods {
            if method.body.is_none() {
                continue;
            }
            self.reset_locals();
            let param_count = method.params.len();
            for param in &method.params {
                self.declare_local(param.name.name.clone());
            }
            let mut instructions = Vec::new();
            if let Some(body) = &method.body {
                self.compile_block(body, &mut instructions)?;
            }
            instructions.push(OpCode::LoadNull);
            instructions.push(OpCode::Return);
            let impl_name = format!("{}_{}_{}", class_name, trait_name, method.name.name);
            let func = IrFunction {
                name: impl_name,
                param_count,
                local_count: self.next_local,
                local_names: std::mem::take(&mut self.local_names),
                instructions,
                is_entry: false,
                target: None,
            };
            self.module.add_function(func);
        }
        Ok(())
    }

    /// 编译类声明
    ///
    /// 为类中每个有方法体的方法生成 `类名_方法名` 格式的 IR 函数，
    /// 并处理 @main 注解标记的入口点。对于类的父 trait，调用 compile_impl_block 生成实现函数。
    pub(crate) fn compile_class(&mut self, class: &ClassDeclaration) -> GResult<()> {
        if !self.filter_by_target(&class.annotations) {
            return Ok(());
        }
        for method in &class.methods {
            if method.body.is_none() {
                continue;
            }
            self.reset_locals();
            let param_count = method.params.len();
            for param in &method.params {
                self.declare_local(param.name.name.clone());
            }
            let mut instructions = Vec::new();
            if let Some(body) = &method.body {
                self.compile_block(body, &mut instructions)?;
            }
            instructions.push(OpCode::LoadNull);
            instructions.push(OpCode::Return);
            let func_name = format!("{}_{}", class.name.name, method.name.name);
            let is_entry = method.annotations.iter().any(|a| a.name.name == "main");
            let target = self.resolve_target_from_annotations(&method.annotations);
            let func = IrFunction {
                name: func_name.clone(),
                param_count,
                local_count: self.next_local,
                local_names: std::mem::take(&mut self.local_names),
                instructions,
                is_entry,
                target,
            };
            self.module.add_function(func);
            if is_entry {
                let entry_target = target.unwrap_or(TargetPlatform::All);
                self.entry_points.push(gg_ir::EntryPoint {
                    name: func_name.clone(),
                    target: entry_target,
                    priority: self.entry_points.len() as u32,
                });
            }
        }
        for parent in &class.parents {
            let trait_name = parent.name.parts.last().map(|p| p.name.clone()).unwrap_or_default();
            self.compile_impl_block(&class.name.name, &trait_name, &class.methods)?;
        }
        Ok(())
    }

    /// 编译结构体声明
    ///
    /// 根据 @target 注解进行过滤，将结构体名称和每个字段名称注册到字符串池，
    /// 并生成 `register_structure` 宿主调用指令。
    /// 指令参数为：名称索引、字段数量、各字段索引。
    pub(crate) fn compile_structure(&mut self, structure: &StructureDeclaration) -> GResult<()> {
        if !self.filter_by_target(&structure.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(structure.name.name.clone());
        let field_count = structure.fields.len();
        let mut field_indices = Vec::with_capacity(field_count);
        for field in &structure.fields {
            let idx = self.module.add_or_get_string(field.name.name.clone());
            field_indices.push(idx);
        }
        let register_fn_idx = self.module.add_or_get_string("register_structure".to_string());
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(name_idx as i64))));
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(field_count as i64))));
        for idx in field_indices {
            self.module_init_instructions
                .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(idx as i64))));
        }
        self.module_init_instructions.push(OpCode::HostCall(register_fn_idx, 2 + field_count));
        Ok(())
    }

    /// 编译 trait 声明
    ///
    /// 根据 @target 注解进行过滤，将 trait 名称和每个方法名称注册到字符串池，
    /// 并生成 `register_trait` 宿主调用指令。
    /// 指令参数为：名称索引、方法数量、各方法索引。
    pub(crate) fn compile_trait_decl(&mut self, trait_decl: &Trait) -> GResult<()> {
        if !self.filter_by_target(&trait_decl.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(trait_decl.name.name.clone());
        let method_count = trait_decl.methods.len();
        let mut method_indices = Vec::with_capacity(method_count);
        for method in &trait_decl.methods {
            let idx = self.module.add_or_get_string(method.name.name.clone());
            method_indices.push(idx);
        }
        let register_fn_idx = self.module.add_or_get_string("register_trait".to_string());
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(name_idx as i64))));
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(method_count as i64))));
        for idx in method_indices {
            self.module_init_instructions
                .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(idx as i64))));
        }
        self.module_init_instructions.push(OpCode::HostCall(register_fn_idx, 2 + method_count));
        Ok(())
    }

    /// 编译单例声明
    ///
    /// 为单例中每个有方法体的方法生成 `单例名_方法名` 格式的 IR 函数。
    pub(crate) fn compile_singleton(&mut self, singleton: &SingletonDeclaration) -> GResult<()> {
        if !self.filter_by_target(&singleton.annotations) {
            return Ok(());
        }
        for method in &singleton.methods {
            if method.body.is_none() {
                continue;
            }
            self.reset_locals();
            let param_count = method.params.len();
            for param in &method.params {
                self.declare_local(param.name.name.clone());
            }
            let mut instructions = Vec::new();
            if let Some(body) = &method.body {
                self.compile_block(body, &mut instructions)?;
            }
            instructions.push(OpCode::LoadNull);
            instructions.push(OpCode::Return);
            let func_name = format!("{}_{}", singleton.name.name, method.name.name);
            let func = IrFunction {
                name: func_name,
                param_count,
                local_count: self.next_local,
                local_names: std::mem::take(&mut self.local_names),
                instructions,
                is_entry: false,
                target: None,
            };
            self.module.add_function(func);
        }
        Ok(())
    }

    /// 编译枚举声明
    ///
    /// 为枚举的每个变体生成 `枚举名_变体名` 格式的字符串常量并加入常量池。
    pub(crate) fn compile_enums(&mut self, enums: &Enums) -> GResult<()> {
        if !self.filter_by_target(&enums.annotations) {
            return Ok(());
        }
        for variant in &enums.variants {
            let idx = self.module.add_constant(gg_ir::IrValue::String(format!("{}_{}", enums.name.name, variant.name.name)));
            let _ = idx;
        }
        Ok(())
    }

    /// 编译 ECS 组件声明
    ///
    /// 将组件名称加入字符串池，目前仅记录组件名称。
    pub(crate) fn compile_component(&mut self, component: &ComponentDeclaration) -> GResult<()> {
        if !self.filter_by_target(&component.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(component.name.name.clone());
        let _ = name_idx;
        Ok(())
    }

    /// 编译着色器声明
    ///
    /// 将着色器名称和类型注册到字符串池，并生成 `register_shader` 宿主调用指令。
    /// 着色器类型包括 PBR、Unlit、Phong、Compute、UiUnlit、UiSdf、UiCustom。
    pub(crate) fn compile_shader(&mut self, shader: &ShaderDeclaration) -> GResult<()> {
        if !self.filter_by_target(&shader.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(shader.name.name.clone());
        let kind_idx = self.module.add_or_get_string(shader.kind.name.clone());
        let register_fn_idx = self.module.add_or_get_string("register_shader".to_string());
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(name_idx as i64))));
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(kind_idx as i64))));
        self.module_init_instructions.push(OpCode::HostCall(register_fn_idx, 2));
        Ok(())
    }

    /// 编译位标志声明
    ///
    /// 将位标志名称和每个变体名称注册到字符串池，并生成 `register_flags` 宿主调用指令。
    /// 指令参数为：名称索引、变体数量、各变体索引。
    pub(crate) fn compile_flags(&mut self, flags: &Flags) -> GResult<()> {
        if !self.filter_by_target(&flags.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(flags.name.name.clone());
        let variant_count = flags.variants.len();
        let mut variant_indices = Vec::with_capacity(variant_count);
        for variant in &flags.variants {
            let idx = self.module.add_or_get_string(variant.name.name.clone());
            variant_indices.push(idx);
        }
        let register_fn_idx = self.module.add_or_get_string("register_flags".to_string());
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(name_idx as i64))));
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(variant_count as i64))));
        for idx in variant_indices {
            self.module_init_instructions
                .push(OpCode::LoadConst(self.module.add_or_get_constant(gg_ir::IrValue::Int(idx as i64))));
        }
        self.module_init_instructions.push(OpCode::HostCall(register_fn_idx, 2 + variant_count));
        Ok(())
    }

    /// 编译 ECS 系统声明
    ///
    /// 为系统中每个有方法体的方法生成 `系统名_方法名` 格式的 IR 函数。
    pub(crate) fn compile_system(&mut self, system: &SystemDeclaration) -> GResult<()> {
        if !self.filter_by_target(&system.annotations) {
            return Ok(());
        }
        for method in &system.methods {
            if method.body.is_none() {
                continue;
            }
            self.reset_locals();
            let param_count = method.params.len();
            for param in &method.params {
                self.declare_local(param.name.name.clone());
            }
            let mut instructions = Vec::new();
            if let Some(body) = &method.body {
                self.compile_block(body, &mut instructions)?;
            }
            instructions.push(OpCode::LoadNull);
            instructions.push(OpCode::Return);
            let func_name = format!("{}_{}", system.name.name, method.name.name);
            let func = IrFunction {
                name: func_name,
                param_count,
                local_count: self.next_local,
                local_names: std::mem::take(&mut self.local_names),
                instructions,
                is_entry: false,
                target: None,
            };
            self.module.add_function(func);
        }
        Ok(())
    }
}
