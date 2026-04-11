"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
function activate(context) {
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
            const serverOptions = {
                run: {
                    command: lspPath,
                    transport: node_1.TransportKind.stdio
                },
                debug: {
                    command: lspPath,
                    transport: node_1.TransportKind.stdio,
                    options: {
                        env: { RUST_BACKTRACE: '1' }
                    }
                }
            };
            // 配置客户端选项
            const clientOptions = {
                documentSelector: [
                    { scheme: 'file', language: 'gg-animation' },
                    { scheme: 'file', language: 'gg-config' },
                    { scheme: 'file', language: 'gg-material' },
                    { scheme: 'file', language: 'gg-meta' },
                    { scheme: 'file', language: 'gg-prefab' },
                    { scheme: 'file', language: 'gg-scene' },
                    { scheme: 'file', language: 'gg-script' },
                    { scheme: 'file', language: 'gg-shader' },
                    { scheme: 'file', language: 'gg-widget' },
                    { scheme: 'file', language: 'gg-galgame' }
                ],
                synchronize: {
                    fileEvents: vscode.workspace.createFileSystemWatcher('**/*.*')
                }
            };
            // 创建并启动客户端
            const client = new node_1.LanguageClient('gg-lsp', 'GG Game Engine Language Server', serverOptions, clientOptions);
            // 启动客户端
            client.start();
            // 注册客户端到上下文，以便在插件停用时有正确清理
            context.subscriptions.push(client);
            console.log(`LSP enabled with path: ${lspPath}`);
        }
        catch (error) {
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
function deactivate() {
    console.log('GG Game Engine extension deactivated');
}
//# sourceMappingURL=extension.js.map