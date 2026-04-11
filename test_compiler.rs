use std::path::Path;
use gg_galgame::compiler::{GscriptParser, ScriptCompiler, StorySequence, DialogueDB};

fn main() {
    // 测试解析单个文件
    println!("Testing GscriptParser::parse_file...");
    match GscriptParser::parse_file(Path::new("test.gscript")) {
        Ok(nodes) => {
            println!("Successfully parsed {} nodes:", nodes.len());
            for (i, node) in nodes.iter().enumerate() {
                println!("Node {}: {}", i + 1, node.id);
                if let Some(speaker) = &node.speaker_id {
                    println!("  Speaker: {}", speaker);
                }
                if !node.text.is_empty() {
                    println!("  Text: {}", node.text);
                }
                if !node.commands.is_empty() {
                    println!("  Commands: {:?}", node.commands);
                }
                if !node.choices.is_empty() {
                    println!("  Choices: {:?}", node.choices);
                }
                if let Some(next) = &node.next_node_id {
                    println!("  Next: {}", next);
                }
            }
        }
        Err(e) => {
            println!("Error parsing file: {:?}", e);
        }
    }

    // 测试编译单个文件
    println!("\nTesting ScriptCompiler::compile_file...");
    match ScriptCompiler::compile_file(Path::new("test.gscript")) {
        Ok(sequence) => {
            println!("Successfully compiled sequence with {} nodes", sequence.len());
            println!("Node index: {:?}", sequence.node_index);
        }
        Err(e) => {
            println!("Error compiling file: {:?}", e);
        }
    }

    // 测试验证单个序列
    println!("\nTesting ScriptCompiler::validate...");
    match ScriptCompiler::compile_file(Path::new("test.gscript")) {
        Ok(sequence) => {
            match ScriptCompiler::validate(&sequence) {
                Ok(errors) => {
                    if errors.is_empty() {
                        println!("No validation errors found!");
                    } else {
                        println!("Found {} validation errors:", errors.len());
                        for error in errors {
                            println!("  {:?}", error);
                        }
                    }
                }
                Err(e) => {
                    println!("Error during validation: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error compiling file: {:?}", e);
        }
    }

    // 测试编译目录
    println!("\nTesting ScriptCompiler::compile_directory...");
    match ScriptCompiler::compile_directory(Path::new(".")) {
        Ok(db) => {
            println!("Successfully compiled directory with {} sequences", db.sequences.len());
            for (file_name, sequence) in &db.sequences {
                println!("  {}: {} nodes", file_name, sequence.len());
            }
        }
        Err(e) => {
            println!("Error compiling directory: {:?}", e);
        }
    }

    // 测试验证数据库
    println!("\nTesting ScriptCompiler::validate_db...");
    match ScriptCompiler::compile_directory(Path::new(".")) {
        Ok(db) => {
            match ScriptCompiler::validate_db(&db) {
                Ok(errors) => {
                    if errors.is_empty() {
                        println!("No validation errors found in database!");
                    } else {
                        println!("Found {} validation errors in database:", errors.len());
                        for error in errors {
                            println!("  {:?}", error);
                        }
                    }
                }
                Err(e) => {
                    println!("Error during database validation: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error compiling directory: {:?}", e);
        }
    }
}
