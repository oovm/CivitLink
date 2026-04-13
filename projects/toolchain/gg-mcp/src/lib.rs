#![warn(missing_docs)]

//! GG 引擎 MCP 模块
//! 提供 MCP (Model Context Protocol) 服务相关功能

use oak_vfs::MemoryVfs;

#[cfg(feature = "lsp")]
use gg_lsp::GgLanguageService;

#[cfg(feature = "voc")]
use gg_lsp::VocLanguageService;

/// Starts the GG Engine MCP service
pub async fn serve_gg_mcp(vfs: MemoryVfs) {
    #[cfg(feature = "lsp")]
    {
        let service = GgLanguageService::new(vfs);
        let server = oak_mcp::McpServer::new(service);
        let reader = tokio::io::BufReader::new(tokio::io::stdin());
        let writer = tokio::io::BufWriter::new(tokio::io::stdout());
        server.run(reader, writer).await.unwrap()
    }
    #[cfg(not(feature = "lsp"))]
    {
        panic!("MCP service requires 'lsp' feature to be enabled")
    }
}

/// 启动 VOC 语言 MCP 服务。
#[cfg(feature = "voc")]
pub async fn serve_voc_mcp(vfs: MemoryVfs) {
    let service = VocLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
