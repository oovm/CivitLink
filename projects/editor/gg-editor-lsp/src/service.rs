//! LSP 服务插件
//!
//! 提供将语言服务器作为编辑器插件集成的功能，
//! 通过 EditorPlugin trait 与编辑器壳程序交互。

use gg_core::GResult;
use gg_editor_shell::{EditorContext, EditorEvent, EditorPlugin};

use crate::{client::LspClient, transport::StdioTransport};

/// LSP 服务，作为编辑器插件集成语言服务器
pub struct LspService {
    /// 语言服务器命令路径
    server_command: String,
    /// 语言服务器参数
    server_args: Vec<String>,
    /// LSP 客户端
    client: Option<LspClient>,
}

impl LspService {
    /// 创建新的 LSP 服务
    ///
    /// # 参数
    /// - `server_command`: 语言服务器可执行文件路径
    /// - `server_args`: 传递给语言服务器的命令行参数
    pub fn new(server_command: String, server_args: Vec<String>) -> Self {
        Self { server_command, server_args, client: None }
    }
}

impl EditorPlugin for LspService {
    fn name(&self) -> &str {
        "LspService"
    }

    fn initialize(&mut self, context: &mut EditorContext) {
        let args: Vec<&str> = self.server_args.iter().map(|s| s.as_str()).collect();
        let transport = match StdioTransport::new(&self.server_command, &args) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("LSP 服务初始化失败: {}", e.message);
                return;
            }
        };

        let mut client = LspClient::new(Box::new(transport));

        client.set_on_diagnostics(Box::new(|_uri, _diagnostics| {
            // 诊断回调将在外部通过事件总线发布事件
        }));

        if let Err(e) = client.initialize("file:///") {
            eprintln!("LSP 客户端初始化失败: {}", e.message);
            return;
        }

        context.events_mut().publish(EditorEvent::Custom {
            name: "LspDiagnostics".to_string(),
            data: Box::new("LspService initialized".to_string()),
        });

        self.client = Some(client);
    }

    fn shutdown(&mut self, _context: &mut EditorContext) {
        if let Some(client) = &mut self.client {
            if let Err(e) = client.shutdown() {
                eprintln!("LSP 客户端关闭失败: {}", e.message);
            }
        }
        self.client = None;
    }
}
