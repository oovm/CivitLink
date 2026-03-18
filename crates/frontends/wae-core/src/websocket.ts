/**
 * 连接 ID 类型
 */
export type ConnectionId = string;

/**
 * 房间 ID 类型
 */
export type RoomId = string;

/**
 * WebSocket 消息类型
 */
export type WebSocketMessageType = "Text" | "Binary" | "Ping" | "Pong" | "Close";

/**
 * WebSocket 消息
 */
export interface WebSocketMessage {
    /** 消息类型 */
    type: WebSocketMessageType;
    /** 文本内容 */
    text?: string;
    /** 二进制数据 */
    binary?: Uint8Array;
}

/**
 * 创建文本消息
 */
export function textMessage(content: string): WebSocketMessage {
    return { type: "Text", text: content };
}

/**
 * 创建二进制消息
 */
export function binaryMessage(data: Uint8Array): WebSocketMessage {
    return { type: "Binary", binary: data };
}

/**
 * 创建关闭消息
 */
export function closeMessage(): WebSocketMessage {
    return { type: "Close" };
}

/**
 * 检查是否为文本消息
 */
export function isTextMessage(message: WebSocketMessage): message is WebSocketMessage & { text: string } {
    return message.type === "Text" && message.text !== undefined;
}

/**
 * 检查是否为二进制消息
 */
export function isBinaryMessage(message: WebSocketMessage): message is WebSocketMessage & { binary: Uint8Array } {
    return message.type === "Binary" && message.binary !== undefined;
}

/**
 * 连接信息
 */
export interface Connection {
    /** 连接 ID */
    id: ConnectionId;
    /** 客户端地址 */
    addr: string;
    /** 连接时间 */
    connectedAt: number;
    /** 用户自定义数据 */
    metadata: Record<string, string>;
    /** 所属房间列表 */
    rooms: RoomId[];
}

/**
 * WebSocket 服务端配置
 */
export interface WebSocketServerConfig {
    /** 监听地址 */
    host: string;
    /** 监听端口 */
    port: number;
    /** 最大连接数 */
    maxConnections: number;
    /** 心跳间隔 (毫秒) */
    heartbeatInterval: number;
    /** 连接超时 (毫秒) */
    connectionTimeout: number;
}

/**
 * WebSocket 客户端配置
 */
export interface WebSocketClientConfig {
    /** 服务端 URL */
    url: string;
    /** 重连间隔 (毫秒) */
    reconnectInterval: number;
    /** 心跳间隔 (毫秒) */
    heartbeatInterval: number;
    /** 连接超时 (毫秒) */
    connectionTimeout: number;
    /** 最大重连次数 (0 表示无限重连) */
    maxReconnectAttempts: number;
}

/**
 * WebSocket 连接状态
 */
export type ConnectionState = "connecting" | "connected" | "disconnecting" | "disconnected";

/**
 * WebSocket 事件类型
 */
export type WebSocketEventType = "connect" | "disconnect" | "message" | "error" | "reconnect" | "reconnect_failed";

/**
 * WebSocket 事件处理器
 */
export type WebSocketEventHandler<T = unknown> = (data: T) => void;

/**
 * 连接事件数据
 */
export interface ConnectEventData {
    /** 连接信息 */
    connection: Connection;
}

/**
 * 断开连接事件数据
 */
export interface DisconnectEventData {
    /** 连接信息 */
    connection: Connection;
    /** 断开原因 */
    reason: string;
}

/**
 * 消息事件数据
 */
export interface MessageEventData {
    /** 连接信息 */
    connection: Connection;
    /** 消息内容 */
    message: WebSocketMessage;
}

/**
 * 错误事件数据
 */
export interface ErrorEventData {
    /** 错误信息 */
    error: Error;
}

/**
 * 重连事件数据
 */
export interface ReconnectEventData {
    /** 当前尝试次数 */
    attempt: number;
    /** 最大尝试次数 */
    maxAttempts: number;
}

/**
 * 重连失败事件数据
 */
export interface ReconnectFailedEventData {
    /** 总尝试次数 */
    totalAttempts: number;
}
