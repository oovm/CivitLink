import * as vscode from 'vscode';

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
        // TODO: 实现 LSP 客户端注册
        console.log(`LSP enabled with path: ${lspPath}`);
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