#![feature(new_range_api)]
#![warn(missing_docs)]

use std::collections::HashMap;
use std::path::Path;

use gg_core::{GError, GErrorKind, GResult};
use gg_ir::{IrFunction, IrModule, IrValue, OpCode};
use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage, ValkyrieTokenType, ast::*};

/// 脚本编译器，将 Valkyrie 源码编译为 IR
pub struct ScriptCompiler {
    /// Valkyrie 语言配置
    language: ValkyrieLanguage,
}

impl ScriptCompiler {
    /// 创建新的脚本编译器
    pub fn new() -> Self {
        Self {
            language: ValkyrieLanguage::default(),
        }
    }

    /// 编译 Valkyrie 源码为 IR 模块
    ///
    /// 流程：源码 -> ValkyrieLexer -> ValkyrieParser -> ValkyrieBuilder -> ValkyrieRoot -> IrModule
    pub fn compile(&self, source: &str, module_name: &str) -> GResult<IrModule> {
        let builder = ValkyrieBuilder::new(&self.language);
        let source_text = SourceText::new(source);
        let mut cache = ParseSession::<ValkyrieLanguage>::default();

        let result = builder.build(&source_text, &[], &mut cache);

        match result.result {
            Ok(root) => {
                let mut converter = AstToIr::new(module_name);
                converter.convert(&root)
            }
            Err(e) => Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("Valkyrie compilation error: {}", e),
            }),
        }
    }
}

/// AST 到 IR 转换器，将 Valkyrie AST 转换为 IR
pub struct AstToIr {
    /// 当前 IR 模块
    module: IrModule,
    /// 局部变量名到索引的映射
    locals: HashMap<String, usize>,
    /// 下一个可用的局部变量索引
    next_local: usize,
    /// 下一个字符串常量标识符
    next_string_id: usize,
}

impl AstToIr {
    /// 创建新的转换器
    pub fn new(module_name: &str) -> Self {
        Self {
            module: IrModule::new(module_name),
            locals: HashMap::new(),
            next_local: 0,
            next_string_id: 0,
        }
    }

    /// 转换 ValkyrieRoot 为 IrModule
    pub fn convert(&mut self, root: &ValkyrieRoot) -> GResult<IrModule> {
        self.locals.clear();
        self.next_local = 0;

        let mut top_level_instructions = Vec::new();

        for item in &root.items {
            self.convert_item(item, &mut top_level_instructions)?;
        }

        if !top_level_instructions.is_empty() {
            top_level_instructions.push(OpCode::Return);
            let init_func = IrFunction {
                name: "__init__".to_string(),
                param_count: 0,
                local_count: self.next_local,
                instructions: top_level_instructions,
            };
            self.module.add_function(init_func);
        }

        Ok(self.module.clone())
    }

    /// 转换顶层项
    fn convert_item(&mut self, item: &Item, top_level_instructions: &mut Vec<OpCode>) -> GResult<()> {
        match item {
            Item::Micro(micro) => {
                self.convert_micro(micro)?;
            }
            Item::Statement(stmt) => {
                self.convert_statement(stmt, top_level_instructions)?;
            }
            Item::Namespace(ns) => {
                for inner_item in &ns.items {
                    self.convert_item(inner_item, top_level_instructions)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 转换 Micro 函数定义
    fn convert_micro(&mut self, micro: &MicroDefinition) -> GResult<()> {
        let saved_locals = self.locals.clone();
        let saved_next_local = self.next_local;

        self.locals.clear();
        self.next_local = 0;

        for param in &micro.params {
            self.locals.insert(param.name.name.clone(), self.next_local);
            self.next_local += 1;
        }

        let param_count = micro.params.len();
        let mut instructions = Vec::new();

        for stmt in &micro.body.statements {
            self.convert_statement(stmt, &mut instructions)?;
        }

        instructions.push(OpCode::LoadNull);
        instructions.push(OpCode::Return);

        let func = IrFunction {
            name: micro.name.name.clone(),
            param_count,
            local_count: self.next_local,
            instructions,
        };

        self.module.add_function(func);

        self.locals = saved_locals;
        self.next_local = saved_next_local;

        Ok(())
    }

    /// 转换表达式
    fn convert_expr(&mut self, expr: &Expr, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match expr {
            Expr::Ident(ident) => {
                match self.locals.get(&ident.name) {
                    Some(&idx) => {
                        instructions.push(OpCode::LoadLocal(idx));
                    }
                    None => {
                        eprintln!("Warning: Variable not found: {}", ident.name);
                        instructions.push(OpCode::LoadNull);
                    }
                }
            }
            Expr::Path(path) => {
                let name = path
                    .parts
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::");
                match self.locals.get(&name) {
                    Some(&idx) => {
                        instructions.push(OpCode::LoadLocal(idx));
                    }
                    None => {
                        eprintln!("Warning: Path not found as local: {}", name);
                        instructions.push(OpCode::LoadNull);
                    }
                }
            }
            Expr::StringLiteral(lit) => {
                self.convert_string_literal(lit, instructions)?;
            }
            Expr::Bool { value, .. } => {
                if *value {
                    instructions.push(OpCode::LoadTrue);
                } else {
                    instructions.push(OpCode::LoadFalse);
                }
            }
            Expr::Binary { left, op, right, .. } => {
                self.convert_expr(left, instructions)?;
                self.convert_expr(right, instructions)?;
                self.convert_binary_op(*op, instructions);
            }
            Expr::Unary { op, expr: operand, .. } => {
                self.convert_expr(operand, instructions)?;
                self.convert_unary_op(*op, instructions);
            }
            Expr::Call { callee, args, .. } => {
                self.convert_expr(callee, instructions)?;
                for arg in args {
                    self.convert_expr(arg, instructions)?;
                }
                instructions.push(OpCode::Call(args.len()));
            }
            Expr::Field { receiver, field, .. } => {
                self.convert_expr(receiver, instructions)?;
                let string_id = self.next_string_id;
                self.next_string_id += 1;
                let field_idx = self.module.add_constant(IrValue::String(string_id));
                instructions.push(OpCode::LoadConst(field_idx));
                instructions.push(OpCode::HostCall("get_field".to_string(), 2));
            }
            Expr::Paren { expr: inner, .. } => {
                self.convert_expr(inner, instructions)?;
            }
            Expr::Return { expr: ret_expr, .. } => {
                match ret_expr {
                    Some(e) => self.convert_expr(e, instructions)?,
                    None => instructions.push(OpCode::LoadNull),
                }
                instructions.push(OpCode::Return);
            }
            Expr::Block(block) => {
                for stmt in &block.statements {
                    self.convert_statement(stmt, instructions)?;
                }
            }
            _ => {
                eprintln!("Warning: Unsupported expression type, generating LoadNull");
                instructions.push(OpCode::LoadNull);
            }
        }
        Ok(())
    }

    /// 转换语句
    fn convert_statement(&mut self, stmt: &Statement, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match stmt {
            Statement::Let { pattern, expr, .. } => {
                self.convert_expr(expr, instructions)?;
                match pattern {
                    Pattern::Variable { name, .. } => {
                        let idx = self.next_local;
                        self.locals.insert(name.name.clone(), idx);
                        self.next_local += 1;
                        instructions.push(OpCode::StoreLocal(idx));
                    }
                    Pattern::Wildcard { .. } => {
                        instructions.push(OpCode::Pop);
                    }
                    _ => {
                        eprintln!("Warning: Unsupported pattern type in let binding, discarding value");
                        instructions.push(OpCode::Pop);
                    }
                }
            }
            Statement::ExprStmt { expr, .. } => {
                self.convert_expr(expr, instructions)?;
                instructions.push(OpCode::Pop);
            }
        }
        Ok(())
    }

    /// 转换字符串字面量
    ///
    /// Valkyrie 中整数和浮点数字面量也以 StringLiteral 形式存储
    /// （prefix 为 None，quote_count 为 0），需要特殊处理
    fn convert_string_literal(&mut self, lit: &StringLiteral, instructions: &mut Vec<OpCode>) -> GResult<()> {
        if lit.prefix.is_none() && lit.quote_count == 0 {
            if let Some(StringSegment::Text { content, .. }) = lit.segments.first() {
                if let Ok(int_val) = content.parse::<i64>() {
                    let idx = self.module.add_constant(IrValue::Int(int_val));
                    instructions.push(OpCode::LoadConst(idx));
                    return Ok(());
                }
                if let Ok(float_val) = content.parse::<f64>() {
                    let idx = self.module.add_constant(IrValue::Float(float_val));
                    instructions.push(OpCode::LoadConst(idx));
                    return Ok(());
                }
            }
        }

        let mut has_interpolation = false;
        let mut full_text = String::new();
        for segment in &lit.segments {
            match segment {
                StringSegment::Text { content, .. } => {
                    full_text.push_str(content);
                }
                StringSegment::Interpolation { .. } => {
                    has_interpolation = true;
                }
            }
        }

        if has_interpolation {
            eprintln!("Warning: String interpolation is not yet supported, generating LoadNull");
            instructions.push(OpCode::LoadNull);
            return Ok(());
        }

        let string_id = self.next_string_id;
        self.next_string_id += 1;
        let idx = self.module.add_constant(IrValue::String(string_id));
        instructions.push(OpCode::LoadConst(idx));
        Ok(())
    }

    /// 转换二元运算符
    fn convert_binary_op(&self, op: ValkyrieTokenType, instructions: &mut Vec<OpCode>) {
        let opcode = match op {
            ValkyrieTokenType::Plus => OpCode::Add,
            ValkyrieTokenType::Minus => OpCode::Sub,
            ValkyrieTokenType::Star => OpCode::Mul,
            ValkyrieTokenType::Slash => OpCode::Div,
            ValkyrieTokenType::EqEq => OpCode::Eq,
            ValkyrieTokenType::NotEq => OpCode::Ne,
            ValkyrieTokenType::LessThan => OpCode::Lt,
            ValkyrieTokenType::LessEq => OpCode::Le,
            ValkyrieTokenType::GreaterThan => OpCode::Gt,
            ValkyrieTokenType::GreaterEq => OpCode::Ge,
            ValkyrieTokenType::AndAnd => OpCode::And,
            ValkyrieTokenType::OrOr => OpCode::Or,
            _ => {
                eprintln!("Warning: Unsupported binary operator: {:?}", op);
                OpCode::LoadNull
            }
        };
        instructions.push(opcode);
    }

    /// 转换一元运算符
    fn convert_unary_op(&self, op: ValkyrieTokenType, instructions: &mut Vec<OpCode>) {
        let opcode = match op {
            ValkyrieTokenType::Minus => OpCode::Neg,
            ValkyrieTokenType::Bang => OpCode::Not,
            _ => {
                eprintln!("Warning: Unsupported unary operator: {:?}", op);
                OpCode::LoadNull
            }
        };
        instructions.push(opcode);
    }
}

/// 脚本加载器，从文件系统加载 .valkyrie 脚本
pub struct ScriptLoader {
    /// 脚本编译器
    compiler: ScriptCompiler,
}

impl ScriptLoader {
    /// 创建新的脚本加载器
    pub fn new() -> Self {
        Self {
            compiler: ScriptCompiler::new(),
        }
    }

    /// 从文件加载脚本并编译为 IR
    pub fn load_file(&self, path: &Path) -> GResult<IrModule> {
        let source = std::fs::read_to_string(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read script file: {}", e),
        })?;

        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        self.compiler.compile(&source, module_name)
    }

    /// 从字符串加载脚本并编译为 IR
    pub fn load_string(&self, source: &str, module_name: &str) -> GResult<IrModule> {
        self.compiler.compile(source, module_name)
    }
}
