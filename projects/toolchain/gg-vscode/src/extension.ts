import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
    console.log('GG Game Engine extension activated');
    
    // 读取配置
    const config = vscode.workspace.getConfiguration('gg-vscode');
    const enableSyntaxHighlighting = config.get('enableSyntaxHighlighting', true);
    const enableLSP = config.get('enableLSP', false);
    const enableMCP = config.get('enableMCP', false);
    const lspPath = config.get('lspPath', '');
    const mcpPath = config.get('mcpPath', '');
    
    console.log(`GG Game Engine config: syntax=${enableSyntaxHighlighting}, lsp=${enableLSP}, mcp=${enableMCP}`);
    
    // 注册 LSP 客户端
    if (enableLSP && lspPath) {
        try {
            // 配置服务器选项
            const serverOptions: ServerOptions = {
                run: {
                    command: lspPath,
                    transport: TransportKind.stdio
                },
                debug: {
                    command: lspPath,
                    transport: TransportKind.stdio,
                    options: {
                        env: { RUST_BACKTRACE: '1' }
                    }
                }
            };
            
            // 配置客户端选项
            const clientOptions: LanguageClientOptions = {
                documentSelector: [
                    { scheme: 'file', language: 'gg-animation' },
                    { scheme: 'file', language: 'gg-config' },
                    { scheme: 'file', language: 'gg-material' },
                    { scheme: 'file', language: 'gg-meta' },
                    { scheme: 'file', language: 'gg-prefab' },
                    { scheme: 'file', language: 'gg-scene' },
                    { scheme: 'file', language: 'gg-script' },
                    { scheme: 'file', language: 'gg-compiler-shader' },
                    { scheme: 'file', language: 'gg-compiler-widget' },
                    { scheme: 'file', language: 'gg-galgame' }
                ],
                synchronize: {
                    fileEvents: vscode.workspace.createFileSystemWatcher('**/*.*')
                }
            };
            
            // 创建并启动客户端
            const client = new LanguageClient(
                'gg-lsp',
                'GG Game Engine Language Server',
                serverOptions,
                clientOptions
            );
            
            // 启动客户端
            client.start();
            
            // 注册客户端到上下文，以便在插件停用时有正确清理
            context.subscriptions.push(client);
            
            console.log(`LSP enabled with path: ${lspPath}`);
        } catch (error: any) {
            console.error('Failed to start LSP client:', error);
            vscode.window.showErrorMessage(`Failed to start LSP client: ${error.message || String(error)}`);
        }
    }
    
    // 注册 MCP 客户端
    if (enableMCP && mcpPath) {
        // TODO: 实现 MCP 客户端注册
        console.log(`MCP enabled with path: ${mcpPath}`);
    }
    
    // 注册一个命令，用于测试插件是否正常工作
    const disposable = vscode.commands.registerCommand('gg-vscode.helloWorld', () => {
        vscode.window.showInformationMessage('Hello from GG Game Engine!');
    });
    
    context.subscriptions.push(disposable);
}

export function deactivate() {
    console.log('GG Game Engine extension deactivated');
}