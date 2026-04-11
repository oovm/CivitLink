#![warn(missing_docs)]

//! 逃逸分析 Pass
//! 分析对象分配的逃逸状态，判断对象是否逃逸出创建函数的作用域

use std::{cell::RefCell, collections::HashMap};

use crate::{IrModule, OpCode, pass::IrPass};
use gg_core::GResult;

/// 对象的逃逸状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscapeState {
    /// 对象未逃逸出创建函数，可进行栈上分配优化
    NoEscape,
    /// 对象通过返回值、宿主调用或外部存储逃逸
    Escape,
    /// 对象通过函数参数传递逃逸
    ArgEscape,
}

/// 对象分配位置标识
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AllocationSite {
    /// 包含该分配的函数索引
    pub function_index: usize,
    /// 分配指令（NewObject/NewList/NewMap）的指令索引
    pub instruction_index: usize,
}

/// 逃逸分析结果
#[derive(Debug, Clone)]
pub struct EscapeInfo {
    /// 分配位置到逃逸状态的映射
    pub escape_states: HashMap<AllocationSite, EscapeState>,
}

impl EscapeInfo {
    /// 创建空的逃逸分析结果
    pub fn new() -> Self {
        Self { escape_states: HashMap::new() }
    }

    /// 查询指定分配位置的逃逸状态
    pub fn get_escape_state(&self, site: &AllocationSite) -> Option<&EscapeState> {
        self.escape_states.get(site)
    }
}

impl Default for EscapeInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// 逃逸分析 Pass
/// 分析每个函数中对象分配的逃逸状态，不修改 IR
pub struct EscapeAnalysisPass {
    /// 最近一次分析的结果，使用 RefCell 以支持 &self 下的内部可变性
    pub last_result: RefCell<Option<EscapeInfo>>,
}

impl EscapeAnalysisPass {
    /// 创建新的逃逸分析 Pass
    pub fn new() -> Self {
        Self { last_result: RefCell::new(None) }
    }
}

impl Default for EscapeAnalysisPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IrPass for EscapeAnalysisPass {
    fn name(&self) -> &str {
        "EscapeAnalysis"
    }

    fn run(&self, module: &mut IrModule) -> GResult<bool> {
        let mut info = EscapeInfo::new();

        for (func_idx, func) in module.functions.iter().enumerate() {
            analyze_function(func_idx, &func.instructions, &mut info);
        }

        *self.last_result.borrow_mut() = Some(info);
        Ok(false)
    }
}

/// 对单个函数执行逃逸分析
fn analyze_function(func_idx: usize, instructions: &[OpCode], info: &mut EscapeInfo) {
    let mut stack: Vec<Option<AllocationSite>> = Vec::new();
    let mut locals: HashMap<usize, Option<AllocationSite>> = HashMap::new();

    for (inst_idx, op) in instructions.iter().enumerate() {
        match op {
            OpCode::LoadConst(_) | OpCode::LoadNull | OpCode::LoadTrue | OpCode::LoadFalse => {
                stack.push(None);
            }
            OpCode::LoadLocal(idx) => {
                let val = locals.get(idx).cloned().flatten();
                stack.push(val);
            }
            OpCode::StoreLocal(idx) => {
                let val = stack.pop().flatten();
                locals.insert(*idx, val);
            }
            OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Mod
            | OpCode::Eq
            | OpCode::Ne
            | OpCode::Lt
            | OpCode::Le
            | OpCode::Gt
            | OpCode::Ge
            | OpCode::And
            | OpCode::Or => {
                stack.pop();
                stack.pop();
                stack.push(None);
            }
            OpCode::Neg | OpCode::Not => {
                stack.pop();
                stack.push(None);
            }
            OpCode::Jump(_) => {}
            OpCode::JumpIfFalse(_) | OpCode::JumpIfTrue(_) => {
                stack.pop();
            }
            OpCode::Call(arg_count) => {
                for _ in 0..*arg_count {
                    if let Some(Some(site)) = stack.pop() {
                        mark_escape(info, &site, EscapeState::ArgEscape);
                    }
                }
                stack.pop();
                stack.push(None);
            }
            OpCode::TailCall(arg_count) => {
                for _ in 0..*arg_count {
                    if let Some(Some(site)) = stack.pop() {
                        mark_escape(info, &site, EscapeState::ArgEscape);
                    }
                }
                stack.pop();
            }
            OpCode::Return => {
                if let Some(Some(site)) = stack.pop() {
                    mark_escape(info, &site, EscapeState::Escape);
                }
            }
            OpCode::SpawnEntity => {
                stack.push(None);
            }
            OpCode::DespawnEntity => {
                stack.pop();
            }
            OpCode::AddComponent(_) => {
                stack.pop();
                stack.pop();
            }
            OpCode::GetComponent(_) => {
                stack.pop();
                stack.push(None);
            }
            OpCode::SetComponent(_) => {
                stack.pop();
                stack.pop();
            }
            OpCode::HostCall(_, arg_count) => {
                for _ in 0..*arg_count {
                    if let Some(Some(site)) = stack.pop() {
                        mark_escape(info, &site, EscapeState::Escape);
                    }
                }
                stack.push(None);
            }
            OpCode::Pop => {
                stack.pop();
            }
            OpCode::Dup => {
                if let Some(top) = stack.last().cloned() {
                    stack.push(top);
                }
            }
            OpCode::GetField(_) => {
                stack.pop();
                stack.push(None);
            }
            OpCode::SetField(_) => {
                let val = stack.pop();
                let obj = stack.pop();
                if let Some(Some(site)) = obj {
                    mark_escape(info, &site, EscapeState::Escape);
                }
                if let Some(Some(site)) = val {
                    mark_escape(info, &site, EscapeState::Escape);
                }
            }
            OpCode::GetIndex => {
                stack.pop();
                stack.pop();
                stack.push(None);
            }
            OpCode::SetIndex => {
                let val = stack.pop();
                stack.pop();
                let container = stack.pop();
                if let Some(Some(site)) = container {
                    mark_escape(info, &site, EscapeState::Escape);
                }
                if let Some(Some(site)) = val {
                    mark_escape(info, &site, EscapeState::Escape);
                }
            }
            OpCode::NewObject(field_count) => {
                for _ in 0..*field_count {
                    stack.pop();
                }
                let site = AllocationSite { function_index: func_idx, instruction_index: inst_idx };
                info.escape_states.entry(site.clone()).or_insert(EscapeState::NoEscape);
                stack.push(Some(site));
            }
            OpCode::NewList(element_count) => {
                for _ in 0..*element_count {
                    stack.pop();
                }
                let site = AllocationSite { function_index: func_idx, instruction_index: inst_idx };
                info.escape_states.entry(site.clone()).or_insert(EscapeState::NoEscape);
                stack.push(Some(site));
            }
            OpCode::NewMap(pair_count) => {
                for _ in 0..*pair_count {
                    stack.pop();
                    stack.pop();
                }
                let site = AllocationSite { function_index: func_idx, instruction_index: inst_idx };
                info.escape_states.entry(site.clone()).or_insert(EscapeState::NoEscape);
                stack.push(Some(site));
            }
            OpCode::StringConcat(count) => {
                for _ in 0..*count {
                    stack.pop();
                }
                stack.push(None);
            }
        }
    }
}

/// 将指定分配位置标记为逃逸，仅在状态升级时更新
fn mark_escape(info: &mut EscapeInfo, site: &AllocationSite, state: EscapeState) {
    if let Some(existing) = info.escape_states.get_mut(site) {
        match (&*existing, &state) {
            (EscapeState::NoEscape, _) => {
                *existing = state;
            }
            (EscapeState::ArgEscape, EscapeState::Escape) => {
                *existing = EscapeState::Escape;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IrFunction, IrModule};

    fn make_func(name: &str, instructions: Vec<OpCode>) -> IrFunction {
        IrFunction {
            name: name.to_string(),
            param_count: 0,
            local_count: 0,
            local_names: vec![],
            instructions,
            is_entry: false,
            target: None,
        }
    }

    fn make_module(funcs: Vec<IrFunction>) -> IrModule {
        let mut module = IrModule::new("test");
        module.functions = funcs;
        module
    }

    #[test]
    fn test_no_escape_local_use() {
        let module = make_module(vec![make_func(
            "local_use",
            vec![
                OpCode::NewObject(0),
                OpCode::StoreLocal(0),
                OpCode::LoadLocal(0),
                OpCode::GetField(0),
                OpCode::Pop,
                OpCode::LoadNull,
                OpCode::Return,
            ],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 0 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::NoEscape));
    }

    #[test]
    fn test_escape_via_return() {
        let module = make_module(vec![make_func("return_obj", vec![OpCode::NewObject(0), OpCode::Return])]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 0 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::Escape));
    }

    #[test]
    fn test_escape_via_host_call() {
        let module = make_module(vec![make_func(
            "host_call_obj",
            vec![OpCode::NewObject(0), OpCode::HostCall(0, 1), OpCode::Pop, OpCode::LoadNull, OpCode::Return],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 0 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::Escape));
    }

    #[test]
    fn test_escape_via_set_field() {
        let module = make_module(vec![make_func(
            "set_field_obj",
            vec![OpCode::NewObject(0), OpCode::LoadConst(0), OpCode::SetField(0), OpCode::LoadNull, OpCode::Return],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 0 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::Escape));
    }

    #[test]
    fn test_arg_escape_via_call() {
        let module = make_module(vec![make_func(
            "call_arg_obj",
            vec![OpCode::LoadConst(0), OpCode::NewObject(0), OpCode::Call(1), OpCode::Pop, OpCode::LoadNull, OpCode::Return],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 1 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::ArgEscape));
    }

    #[test]
    fn test_arg_escape_via_tail_call() {
        let module = make_module(vec![make_func(
            "tail_call_arg_obj",
            vec![OpCode::LoadConst(0), OpCode::NewObject(0), OpCode::TailCall(1)],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 1 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::ArgEscape));
    }

    #[test]
    fn test_escape_via_set_index() {
        let module = make_module(vec![make_func(
            "set_index_obj",
            vec![
                OpCode::NewObject(0),
                OpCode::LoadConst(0),
                OpCode::LoadConst(1),
                OpCode::SetIndex,
                OpCode::LoadNull,
                OpCode::Return,
            ],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 0 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::Escape));
    }

    #[test]
    fn test_new_list_no_escape() {
        let module = make_module(vec![make_func(
            "list_local",
            vec![
                OpCode::LoadConst(0),
                OpCode::NewList(1),
                OpCode::StoreLocal(0),
                OpCode::LoadLocal(0),
                OpCode::Pop,
                OpCode::LoadNull,
                OpCode::Return,
            ],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 1 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::NoEscape));
    }

    #[test]
    fn test_new_map_escape_via_return() {
        let module = make_module(vec![make_func(
            "map_return",
            vec![OpCode::LoadConst(0), OpCode::LoadConst(1), OpCode::NewMap(1), OpCode::Return],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 2 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::Escape));
    }

    #[test]
    fn test_multiple_allocations() {
        let module = make_module(vec![make_func(
            "multi_alloc",
            vec![OpCode::NewObject(0), OpCode::StoreLocal(0), OpCode::NewObject(0), OpCode::Return],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site0 = AllocationSite { function_index: 0, instruction_index: 0 };
        let site1 = AllocationSite { function_index: 0, instruction_index: 2 };
        assert_eq!(info.get_escape_state(&site0), Some(&EscapeState::NoEscape));
        assert_eq!(info.get_escape_state(&site1), Some(&EscapeState::Escape));
    }

    #[test]
    fn test_escape_upgrade_arg_to_escape() {
        let module = make_module(vec![make_func(
            "upgrade_escape",
            vec![OpCode::NewObject(0), OpCode::HostCall(0, 1), OpCode::Pop, OpCode::LoadNull, OpCode::Return],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 0 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::Escape));
    }

    #[test]
    fn test_empty_function() {
        let module = make_module(vec![make_func("empty", vec![])]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        assert!(info.escape_states.is_empty());
    }

    #[test]
    fn test_pass_name() {
        let pass = EscapeAnalysisPass::new();
        assert_eq!(pass.name(), "EscapeAnalysis");
    }

    #[test]
    fn test_pass_does_not_modify_ir() {
        let mut module = make_module(vec![make_func("no_modify", vec![OpCode::NewObject(0), OpCode::Return])]);
        let original_instructions = module.functions[0].instructions.clone();
        let pass = EscapeAnalysisPass::new();
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
        assert_eq!(module.functions[0].instructions, original_instructions);
    }

    #[test]
    fn test_multiple_functions() {
        let module = make_module(vec![
            make_func("local_fn", vec![OpCode::NewObject(0), OpCode::Pop, OpCode::LoadNull, OpCode::Return]),
            make_func("escape_fn", vec![OpCode::NewObject(0), OpCode::Return]),
        ]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site_local = AllocationSite { function_index: 0, instruction_index: 0 };
        let site_escape = AllocationSite { function_index: 1, instruction_index: 0 };
        assert_eq!(info.get_escape_state(&site_local), Some(&EscapeState::NoEscape));
        assert_eq!(info.get_escape_state(&site_escape), Some(&EscapeState::Escape));
    }

    #[test]
    fn test_allocation_site_equality() {
        let a = AllocationSite { function_index: 0, instruction_index: 1 };
        let b = AllocationSite { function_index: 0, instruction_index: 1 };
        let c = AllocationSite { function_index: 1, instruction_index: 1 };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_escape_state_ordering() {
        assert_eq!(EscapeState::NoEscape, EscapeState::NoEscape);
        assert_ne!(EscapeState::NoEscape, EscapeState::ArgEscape);
        assert_ne!(EscapeState::ArgEscape, EscapeState::Escape);
    }

    #[test]
    fn test_dup_propagates_allocation() {
        let module = make_module(vec![make_func(
            "dup_test",
            vec![OpCode::NewObject(0), OpCode::Dup, OpCode::HostCall(0, 2), OpCode::Pop, OpCode::LoadNull, OpCode::Return],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 0 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::Escape));
    }

    #[test]
    fn test_load_local_propagates_allocation() {
        let module = make_module(vec![make_func(
            "local_propagate",
            vec![OpCode::NewObject(0), OpCode::StoreLocal(0), OpCode::LoadLocal(0), OpCode::Return],
        )]);
        let pass = EscapeAnalysisPass::new();
        pass.run(&mut module.clone()).unwrap();

        let result = pass.last_result.borrow();
        let info = result.as_ref().unwrap();
        let site = AllocationSite { function_index: 0, instruction_index: 0 };
        assert_eq!(info.get_escape_state(&site), Some(&EscapeState::Escape));
    }
}
