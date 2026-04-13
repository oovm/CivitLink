use gg_ir::TargetPlatform;
use oak_valkyrie::ast::{Attribute, MicroDeclaration, StatementNode, StringSegment, TermExpression};

use super::{ValkyrieCompiler};

impl ValkyrieCompiler {
    /// 创建新的编译器
    pub fn new(module_name: &str) -> Self {
        Self {
            module: gg_ir::IrModule::new(module_name),
            locals: std::collections::HashMap::new(),
            local_names: Vec::new(),
            next_local: 0,
            loop_stack: Vec::new(),
            entry_points: Vec::new(),
            target_platform: None,
            module_init_instructions: Vec::new(),
        }
    }

    /// 创建带有目标平台的编译器
    pub fn with_target(module_name: &str, target: TargetPlatform) -> Self {
        Self {
            module: gg_ir::IrModule::new(module_name),
            locals: std::collections::HashMap::new(),
            local_names: Vec::new(),
            next_local: 0,
            loop_stack: Vec::new(),
            entry_points: Vec::new(),
            target_platform: Some(target),
            module_init_instructions: Vec::new(),
        }
    }

    /// 重置局部变量表
    pub(crate) fn reset_locals(&mut self) {
        self.locals.clear();
        self.local_names.clear();
        self.next_local = 0;
        self.loop_stack.clear();
    }

    /// 声明局部变量
    pub(crate) fn declare_local(&mut self, name: String) -> usize {
        let idx = self.next_local;
        self.locals.insert(name.clone(), idx);
        self.local_names.push(name);
        self.next_local += 1;
        idx
    }

    /// 处理 micro 声明上的注解，提取入口点信息
    pub(crate) fn process_annotations(&mut self, micro: &MicroDeclaration) {
        for attr in &micro.annotations {
            match attr.name.name.as_str() {
                "main" => {
                    let target = if let Some(TermExpression::StringLiteral(sl)) = attr.args.first() {
                        let content: String = sl
                            .segments
                            .iter()
                            .filter_map(|seg| match seg {
                                StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                                _ => None,
                            })
                            .collect();
                        TargetPlatform::from_str(&content).unwrap_or(TargetPlatform::All)
                    }
                    else {
                        TargetPlatform::All
                    };
                    let priority = self.entry_points.len() as u32;
                    self.entry_points.push(gg_ir::EntryPoint { name: micro.name.name.clone(), target, priority });
                }
                _ => {}
            }
        }
    }

    /// 解析 @target 注解，返回目标平台
    pub(crate) fn resolve_target_annotation(&self, micro: &MicroDeclaration) -> Option<TargetPlatform> {
        for attr in &micro.annotations {
            if attr.name.name == "target" {
                if let Some(TermExpression::StringLiteral(sl)) = attr.args.first() {
                    let content: String = sl
                        .segments
                        .iter()
                        .filter_map(|seg| match seg {
                            StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                            _ => None,
                        })
                        .collect();
                    return TargetPlatform::from_str(content.split(',').next().unwrap_or("").trim());
                }
            }
        }
        None
    }

    /// 从注解列表中解析 @target 注解，返回目标平台
    ///
    /// 支持 @target("platform") 格式的注解，取逗号分隔的第一个平台名称进行解析。
    pub(crate) fn resolve_target_from_annotations(&self, annotations: &[Attribute]) -> Option<TargetPlatform> {
        for attr in annotations {
            if attr.name.name == "target" {
                if let Some(TermExpression::StringLiteral(sl)) = attr.args.first() {
                    let content: String = sl
                        .segments
                        .iter()
                        .filter_map(|seg| match seg {
                            StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                            _ => None,
                        })
                        .collect();
                    return TargetPlatform::from_str(content.split(',').next().unwrap_or("").trim());
                }
            }
        }
        None
    }

    /// 根据 @target 注解过滤，判断当前声明是否应该编译
    pub(crate) fn filter_by_target(&self, annotations: &[Attribute]) -> bool {
        if let Some(current_target) = self.target_platform {
            for attr in annotations {
                if attr.name.name == "target" {
                    if let Some(TermExpression::StringLiteral(sl)) = attr.args.first() {
                        let content: String = sl
                            .segments
                            .iter()
                            .filter_map(|seg| match seg {
                                StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                                _ => None,
                            })
                            .collect();
                        let platforms: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
                        let matches = platforms.iter().any(|p| {
                            TargetPlatform::from_str(p).map_or(false, |tp| tp == current_target || tp == TargetPlatform::All)
                        });
                        if !matches {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    /// 编译顶层 Item
    pub(crate) fn compile_item(&mut self, item: &StatementNode) -> gg_core::GResult<()> {
        match item {
            StatementNode::Micro(micro) => {
                if !self.filter_by_target(&micro.annotations) {
                    return Ok(());
                }
                let func = self.compile_micro(micro)?;
                self.module.add_function(func);
            }
            StatementNode::Namespace(namespace) => {
                for inner_item in &namespace.items {
                    self.compile_item(inner_item)?;
                }
            }
            StatementNode::Let(let_stmt) => {
                if !self.filter_by_target(&let_stmt.annotations) {
                    return Ok(());
                }
                let mut instructions = Vec::new();
                self.compile_statement(&oak_valkyrie::ast::Statement::Let((**let_stmt).clone()), &mut instructions)?;
            }
            StatementNode::ExprStmt(expr_stmt) => {
                if !self.filter_by_target(&expr_stmt.annotations) {
                    return Ok(());
                }
                let mut instructions = Vec::new();
                self.compile_statement(&oak_valkyrie::ast::Statement::ExprStmt((**expr_stmt).clone()), &mut instructions)?;
            }
            StatementNode::Shader(shader) => {
                self.compile_shader(shader)?;
            }
            StatementNode::Class(class) => {
                self.compile_class(class)?;
            }
            StatementNode::Structure(structure) => {
                self.compile_structure(structure)?;
            }
            StatementNode::Trait(trait_decl) => {
                self.compile_trait_decl(trait_decl)?;
            }
            StatementNode::Singleton(singleton) => {
                self.compile_singleton(singleton)?;
            }
            StatementNode::Enums(enums) => {
                self.compile_enums(enums)?;
            }
            StatementNode::Flags(flags) => {
                self.compile_flags(flags)?;
            }
            StatementNode::Component(component) => {
                self.compile_component(component)?;
            }
            StatementNode::System(system) => {
                self.compile_system(system)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// 编译 ValkyrieRoot AST 为 IrModule
    ///
    /// 遍历 AST 中的顶层 Item，对每个 micro 函数定义编译为 IrFunction。
    /// 编译完成后，若存在类型注册指令，则生成 `__module_init__` 函数作为入口点。
    pub fn compile(mut self, root: &oak_valkyrie::ast::ValkyrieRoot, module_name: &str) -> gg_core::GResult<gg_ir::IrModule> {
        self.module = gg_ir::IrModule::new(module_name);
        self.module_init_instructions.clear();
        for item in &root.items {
            self.compile_item(item)?;
        }
        if !self.module_init_instructions.is_empty() {
            let mut init_instructions = std::mem::take(&mut self.module_init_instructions);
            init_instructions.push(gg_ir::OpCode::LoadNull);
            init_instructions.push(gg_ir::OpCode::Return);
            let init_func = gg_ir::IrFunction {
                name: "__module_init__".to_string(),
                param_count: 0,
                local_count: 0,
                local_names: vec![],
                instructions: init_instructions,
                is_entry: true,
                target: None,
            };
            self.module.add_function(init_func);
            self.entry_points
                .insert(0, gg_ir::EntryPoint { name: "__module_init__".to_string(), target: TargetPlatform::All, priority: 0 });
            for entry in &mut self.entry_points {
                entry.priority += 1;
            }
        }
        self.module.entry_points = self.entry_points.clone();
        self.module.target_platform = self.target_platform;
        Ok(self.module)
    }
}
