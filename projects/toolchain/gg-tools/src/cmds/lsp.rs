#![warn(missing_docs)]

//! `gg lsp` 命令实现
//!
//! 启动 GG 引擎的 LSP 服务

use crate::{GError, GErrorKind, GResult};

/// 执行 `lsp` 子命令
///
/// 启动 GG 引擎的 LSP 服务，通过 stdio 传输协议与编辑器通信。
/// 需要启用 `lsp` feature 才能使用真正的 LSP 服务实现。
pub fn cmd_lsp(workspace: &str) -> GResult<()> {
    println!("Starting GG Engine LSP service in workspace: {}", workspace);

    #[cfg(feature = "lsp")]
    {
        use gg_lsp::GgLanguageService;
        use oak_lsp::LspServer;
        use oak_vfs::MemoryVfs;
        use std::sync::Arc;

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| GError {
                kind: GErrorKind::Runtime,
                message: format!("Failed to create tokio runtime: {}", e),
            })?
            .block_on(async {
                let vfs = MemoryVfs::new();
                let service = GgLanguageService::new(vfs);
                let server = LspServer::new(Arc::new(service));

                let stdin = tokio::io::BufReader::new(tokio::io::stdin());
                let stdout = tokio::io::BufWriter::new(tokio::io::stdout());

                server
                    .run(stdin, stdout)
                    .await
                    .map_err(|e| GError {
                        kind: GErrorKind::Runtime,
                        message: format!("LSP server error: {}", e),
                    })?;

                Ok(())
            })
    }

    #[cfg(not(feature = "lsp"))]
    {
        eprintln!(
            "LSP service requires the 'lsp' feature to be enabled.\n\
             Rebuild with: cargo build -p gg-tools --features lsp"
        );
        Err(GError {
            kind: GErrorKind::Runtime,
            message: "LSP feature is not enabled".to_string(),
        })
    }
}
