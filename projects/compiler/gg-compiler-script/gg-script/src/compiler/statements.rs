use gg_core::GResult;
use gg_ir::OpCode;
use oak_valkyrie::ast::{Block, Statement};

use super::{ValkyrieCompiler};

impl ValkyrieCompiler {
    /// 编译语句块
    pub(crate) fn compile_block(&mut self, block: &Block, instructions: &mut Vec<OpCode>) -> GResult<()> {
        for stmt in &block.statements {
            self.compile_statement(stmt, instructions)?;
        }
        Ok(())
    }

    /// 编译语句
    ///
    /// TODO: Pattern 枚举目前没有 Tuple 变体，待 oak-valkyrie 添加后需实现元组解构绑定：
    /// 编译表达式后，为每个元素生成 GetIndex + StoreLocal 指令。
    pub(crate) fn compile_statement(&mut self, stmt: &Statement, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match stmt {
            Statement::Let(let_stmt) => {
                self.compile_expr(&let_stmt.expr, instructions)?;
                match &let_stmt.pattern {
                    oak_valkyrie::ast::Pattern::Variable(var) => {
                        let var_name = var.name.name.clone();
                        let idx = self.declare_local(var_name);
                        instructions.push(OpCode::StoreLocal(idx));
                    }
                    oak_valkyrie::ast::Pattern::Wildcard(_) => {
                        instructions.push(OpCode::Pop);
                    }
                    oak_valkyrie::ast::Pattern::Class(class_pat) => {
                        let temp_local = self.declare_local("__destructure_temp".to_string());
                        instructions.push(OpCode::StoreLocal(temp_local));
                        for (field_name, field_pattern) in &class_pat.fields {
                            instructions.push(OpCode::LoadLocal(temp_local));
                            let field_idx = self.module.add_or_get_string(field_name.name.clone());
                            instructions.push(OpCode::GetField(field_idx));
                            match field_pattern {
                                Some(oak_valkyrie::ast::Pattern::Variable(var)) => {
                                    let var_name = var.name.name.clone();
                                    let idx = self.declare_local(var_name);
                                    instructions.push(OpCode::StoreLocal(idx));
                                }
                                Some(oak_valkyrie::ast::Pattern::Wildcard(_)) => {
                                    instructions.push(OpCode::Pop);
                                }
                                Some(oak_valkyrie::ast::Pattern::Class(_)) => {
                                    let idx = self.declare_local(format!("__destructure_{}", self.next_local));
                                    instructions.push(OpCode::StoreLocal(idx));
                                }
                                None => {
                                    let var_name = field_name.name.clone();
                                    let idx = self.declare_local(var_name);
                                    instructions.push(OpCode::StoreLocal(idx));
                                }
                                _ => {
                                    instructions.push(OpCode::Pop);
                                }
                            }
                        }
                    }
                    oak_valkyrie::ast::Pattern::Literal(_) | oak_valkyrie::ast::Pattern::Type(_) | oak_valkyrie::ast::Pattern::Else(_) => {
                        instructions.push(OpCode::Pop);
                    }
                }
            }
            Statement::ExprStmt(expr_stmt) => {
                self.compile_expr(&expr_stmt.expr, instructions)?;
                if expr_stmt.semi {
                    instructions.push(OpCode::Pop);
                }
            }
        }
        Ok(())
    }
}
