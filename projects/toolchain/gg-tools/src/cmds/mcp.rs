#![warn(missing_docs)]

//! `gg mcp` 命令实现
//!
//! 启动 GG 引擎的 MCP 服务

use crate::{GError, GErrorKind, GResult};

/// 执行 `mcp` 子命令
///
/// 启动 GG 引擎的 MCP (Model Context Protocol) 服务。
/// 需要启用 `lsp` feature 才能使用，因为 gg-mcp 依赖 gg-lsp。
pub fn cmd_mcp(workspace: &str) -> GResult<()> {
    println!("Starting GG Engine MCP service in workspace: {}", workspace);

    #[cfg(feature = "lsp")]
    {
        use oak_vfs::MemoryVfs;

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to create tokio runtime: {}", e) })?
            .block_on(async {
                let vfs = MemoryVfs::new();
                gg_mcp::serve_gg_mcp(vfs).await;
                Ok(())
            })
    }

    #[cfg(not(feature = "lsp"))]
    {
        eprintln!(
            "MCP service requires the 'lsp' feature to be enabled.\n\
             Rebuild with: cargo build -p gg-tools --features lsp"
        );
        Err(GError { kind: GErrorKind::Runtime, message: "LSP feature is not enabled (required for MCP)".to_string() })
    }
}
