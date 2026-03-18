/**
 * WAE - 全栈异步框架前端库
 *
 * 提供与后端 Rust 框架配合的 TypeScript/JavaScript 客户端库。
 */

// 认证客户端
export {
    AuthClient,
    type AuthClientConfig,
    type AuthState,
    type AuthStateListener,
    createAuthClient,
    type StorageAdapter,
} from "@wae/auth";

// HTTP 客户端
export {
    createHttpClient,
    type ErrorInterceptor,
    HttpClient,
    type HttpClientConfig,
    type HttpResponse,
    type RequestInterceptor,
    type ResponseInterceptor,
} from "@wae/client";
// 类型定义
export * from "@wae/core";
// 存储客户端
export {
    createStorageClient,
    StorageClient,
    type StorageClientConfig,
    type UploadProgressCallback,
} from "@wae/storage";
// WebSocket 客户端
export {
    createWebSocketClient,
    WebSocketClient,
} from "@wae/websocket";
