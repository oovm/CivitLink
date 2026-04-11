//! Galgame 字节码生成模块
//! 将 GalgameIr 编译为可执行字节码

use gg_bytecode::BytecodeWriter;
use gg_ir::{IrFunction, IrModule, IrValue, OpCode};

use crate::{
    error::{GalgameError, GalgameResult},
    ir::{CommandIr, GalgameIr},
};

/// Galgame 字节码生成器
pub struct GalgameCodegen;

impl GalgameCodegen {
    /// 创建新的字节码生成器
    pub fn new() -> Self {
        Self
    }

    /// 从 GalgameIr 生成字节码
    pub fn generate(ir: &GalgameIr) -> GalgameResult<Vec<u8>> {
        let module = Self::build_ir_module(ir, "galgame_main")?;
        BytecodeWriter::write(&module).map_err(|e| GalgameError::CodegenError(e.message))
    }

    /// 从 GalgameIr 构建 IrModule
    fn build_ir_module(ir: &GalgameIr, module_name: &str) -> GalgameResult<IrModule> {
        let mut module = IrModule::new(module_name);

        let mut instructions = Vec::new();
        let local_count: usize = 0;

        for dialogue in &ir.dialogues {
            let text_idx = module.add_or_get_constant(IrValue::String(dialogue.text.clone()));
            instructions.push(OpCode::LoadConst(text_idx));

            let speaker_arg_count = if dialogue.speaker_id.is_some() { 1 } else { 0 };

            if let Some(ref speaker_id) = dialogue.speaker_id {
                let speaker_idx = module.add_or_get_constant(IrValue::String(speaker_id.clone()));
                instructions.push(OpCode::LoadConst(speaker_idx));
            }

            let host_idx = module.add_or_get_string("display_dialogue".to_string());
            instructions.push(OpCode::HostCall(host_idx, 1 + speaker_arg_count));

            for command in &dialogue.commands {
                Self::emit_command(&mut module, &mut instructions, command);
            }

            if !dialogue.choices.is_empty() {
                Self::emit_choices(&mut module, &mut instructions, &dialogue.choices);
            }
        }

        instructions.push(OpCode::LoadNull);
        instructions.push(OpCode::Return);

        let function = IrFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count,
            local_names: vec![],
            instructions,
            is_entry: true,
            target: None,
        };
        module.add_function(function);

        Ok(module)
    }

    /// 发射命令指令
    fn emit_command(module: &mut IrModule, instructions: &mut Vec<OpCode>, command: &CommandIr) {
        match command {
            CommandIr::ChangeScene { name, bg } => {
                let name_idx = module.add_or_get_constant(IrValue::String(name.clone()));
                let bg_idx = module.add_or_get_constant(IrValue::String(bg.clone()));
                instructions.push(OpCode::LoadConst(name_idx));
                instructions.push(OpCode::LoadConst(bg_idx));
                let host_idx = module.add_or_get_string("change_scene".to_string());
                instructions.push(OpCode::HostCall(host_idx, 2));
            }
            CommandIr::ShowCharacter { name, sprite, position } => {
                let name_idx = module.add_or_get_constant(IrValue::String(name.clone()));
                let sprite_idx = module.add_or_get_constant(IrValue::String(sprite.clone()));
                let pos_idx = module.add_or_get_constant(IrValue::String(position.clone()));
                instructions.push(OpCode::LoadConst(name_idx));
                instructions.push(OpCode::LoadConst(sprite_idx));
                instructions.push(OpCode::LoadConst(pos_idx));
                let host_idx = module.add_or_get_string("show_character".to_string());
                instructions.push(OpCode::HostCall(host_idx, 3));
            }
            CommandIr::PlayBgm { path } => {
                let path_idx = module.add_or_get_constant(IrValue::String(path.clone()));
                instructions.push(OpCode::LoadConst(path_idx));
                let host_idx = module.add_or_get_string("play_bgm".to_string());
                instructions.push(OpCode::HostCall(host_idx, 1));
            }
            CommandIr::PlaySe { path } => {
                let path_idx = module.add_or_get_constant(IrValue::String(path.clone()));
                instructions.push(OpCode::LoadConst(path_idx));
                let host_idx = module.add_or_get_string("play_se".to_string());
                instructions.push(OpCode::HostCall(host_idx, 1));
            }
            CommandIr::FadeIn { duration } => {
                let dur_idx = module.add_or_get_constant(IrValue::Float(*duration as f64));
                instructions.push(OpCode::LoadConst(dur_idx));
                let host_idx = module.add_or_get_string("fade_in".to_string());
                instructions.push(OpCode::HostCall(host_idx, 1));
            }
            CommandIr::FadeOut { duration } => {
                let dur_idx = module.add_or_get_constant(IrValue::Float(*duration as f64));
                instructions.push(OpCode::LoadConst(dur_idx));
                let host_idx = module.add_or_get_string("fade_out".to_string());
                instructions.push(OpCode::HostCall(host_idx, 1));
            }
            CommandIr::SetVariable { name, value } => {
                let name_idx = module.add_or_get_constant(IrValue::String(name.clone()));
                let value_idx = module.add_or_get_constant(IrValue::String(value.clone()));
                instructions.push(OpCode::LoadConst(name_idx));
                instructions.push(OpCode::LoadConst(value_idx));
                let host_idx = module.add_or_get_string("set_variable".to_string());
                instructions.push(OpCode::HostCall(host_idx, 2));
            }
            CommandIr::CallFunction { name, args } => {
                for arg in args {
                    let arg_idx = module.add_or_get_constant(IrValue::String(arg.clone()));
                    instructions.push(OpCode::LoadConst(arg_idx));
                }
                let host_idx = module.add_or_get_string(name.clone());
                instructions.push(OpCode::HostCall(host_idx, args.len()));
            }
            CommandIr::Goto { target } => {
                let target_idx = module.add_or_get_constant(IrValue::String(target.clone()));
                instructions.push(OpCode::LoadConst(target_idx));
                let host_idx = module.add_or_get_string("goto_label".to_string());
                instructions.push(OpCode::HostCall(host_idx, 1));
            }
        }
    }

    /// 发射选项跳转指令
    fn emit_choices(module: &mut IrModule, instructions: &mut Vec<OpCode>, choices: &[ir::ChoiceIr]) {
        let count_idx = module.add_or_get_constant(IrValue::Int(choices.len() as i64));
        instructions.push(OpCode::LoadConst(count_idx));

        for choice in choices {
            let text_idx = module.add_or_get_constant(IrValue::String(choice.text.clone()));
            let next_idx = module.add_or_get_constant(IrValue::String(choice.next_node_id.clone()));
            instructions.push(OpCode::LoadConst(text_idx));
            instructions.push(OpCode::LoadConst(next_idx));
        }

        let host_idx = module.add_or_get_string("present_choices".to_string());
        instructions.push(OpCode::HostCall(host_idx, 1 + choices.len() * 2));
    }
}

impl Default for GalgameCodegen {
    fn default() -> Self {
        Self::new()
    }
}
