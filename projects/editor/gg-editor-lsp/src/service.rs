//! LSP 服务插件
//!
//! 提供将语言服务器作为编辑器插件集成的功能，
//! 通过 EditorPlugin trait 与编辑器壳程序交互，
//! 并将诊断信息通过编辑器事件总线发布。

use std::sync::mpsc;

use gg_core::GResult;
use gg_editor_shell::{EditorContext, EditorEvent, EditorPlugin};

use crate::{
    client::LspClient,
    transport::StdioTransport,
    types::{DiagnosticSeverity, LspConfig},
};

/// LSP 诊断事件数据
#[derive(Debug, Clone)]
pub struct LspDiagnosticsData {
    /// 文档 URI
    pub uri: String,
    /// 错误数量
    pub error_count: usize,
    /// 警告数量
    pub warning_count: usize,
}

/// LSP 服务，作为编辑器插件集成语言服务器
pub struct LspService {
    /// LSP 连接配置
    pub config: LspConfig,
    /// LSP 客户端
    client: Option<LspClient>,
    /// 诊断事件接收器
    diagnostics_rx: Option<mpsc::Receiver<(String, usize, usize)>>,
}

impl LspService {
    /// 创建新的 LSP 服务
    ///
    /// # 参数
    /// - `config`: LSP 连接配置
    pub fn new(config: LspConfig) -> Self {
        Self { config, client: None, diagnostics_rx: None }
    }

    /// 处理待处理的诊断事件
    ///
    /// 从通道中读取所有待处理的诊断数据，并通过事件总线发布。
    /// 应在编辑器主循环中定期调用此方法。
    pub fn process_pending_diagnostics(&mut self, context: &mut EditorContext) {
        if let Some(rx) = &self.diagnostics_rx {
            while let Ok((uri, error_count, warning_count)) = rx.try_recv() {
                let diag_string = format!("{}:{}:{}", uri, error_count, warning_count);
                context
                    .events_mut()
                    .publish(EditorEvent::Custom { name: "LspDiagnostics".to_string(), data: Box::new(diag_string) });
            }
        }
    }
}

impl EditorPlugin for LspService {
    fn name(&self) -> &str {
        "LspService"
    }

    fn initialize(&mut self, context: &mut EditorContext) {
        let (tx, rx) = mpsc::channel::<(String, usize, usize)>();
        self.diagnostics_rx = Some(rx);

        let args: Vec<&str> = self.config.server_args.iter().map(|s| s.as_str()).collect();
        let transport = match StdioTransport::new(&self.config.server_command, &args) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("LSP 服务初始化失败: {}", e.message);
                return;
            }
        };

        let mut client = LspClient::new(Box::new(transport));

        client.set_on_diagnostics(Box::new(move |uri, diagnostics| {
            let error_count = diagnostics.iter().filter(|d| d.severity == Some(DiagnosticSeverity::Error)).count();
            let warning_count = diagnostics.iter().filter(|d| d.severity == Some(DiagnosticSeverity::Warning)).count();
            let _ = tx.send((uri.to_string(), error_count, warning_count));
        }));

        if let Err(e) = client.initialize(&self.config.root_uri) {
            eprintln!("LSP 客户端初始化失败: {}", e.message);
            return;
        }

        context.events_mut().publish(EditorEvent::Custom {
            name: "LspServiceInitialized".to_string(),
            data: Box::new(self.config.root_uri.clone()),
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
        self.diagnostics_rx = None;
    }
}
