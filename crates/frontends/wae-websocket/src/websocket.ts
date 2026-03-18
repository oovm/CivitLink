import type {
    Connection,
    ConnectionState,
    RoomId,
    WebSocketClientConfig,
    WebSocketEventHandler,
    WebSocketEventType,
    WebSocketMessage,
} from "@wae/core";
import { binaryMessage, isTextMessage, textMessage } from "@wae/core";

/**
 * WebSocket 客户端配置（带默认值）
 */
interface ResolvedWebSocketClientConfig {
    url: string;
    reconnectInterval: number;
    heartbeatInterval: number;
    connectionTimeout: number;
    maxReconnectAttempts: number;
}

/**
 * 创建默认配置
 */
function createDefaultConfig(url: string): ResolvedWebSocketClientConfig {
    return {
        url,
        reconnectInterval: 5000,
        heartbeatInterval: 30000,
        connectionTimeout: 10000,
        maxReconnectAttempts: 0,
    };
}

/**
 * WebSocket 客户端
 */
export class WebSocketClient {
    private config: ResolvedWebSocketClientConfig;
    private ws: WebSocket | null = null;
    private state: ConnectionState = "disconnected";
    private reconnectAttempts = 0;
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
    private connectionTimeoutTimer: ReturnType<typeof setTimeout> | null = null;
    private eventHandlers: Map<WebSocketEventType, Set<WebSocketEventHandler>> = new Map();
    private rooms: Set<RoomId> = new Set();
    private messageQueue: WebSocketMessage[] = [];
    private connectionId: string | null = null;

    /**
     * 创建 WebSocket 客户端实例
     * @param config 客户端配置
     */
    constructor(config: Partial<WebSocketClientConfig> & { url: string }) {
        this.config = { ...createDefaultConfig(config.url), ...config };
        this.initEventHandlers();
    }

    /**
     * 获取当前连接状态
     */
    getState(): ConnectionState {
        return this.state;
    }

    /**
     * 获取连接 ID
     */
    getConnectionId(): string | null {
        return this.connectionId;
    }

    /**
     * 连接 WebSocket
     */
    connect(): Promise<void> {
        return new Promise((resolve, reject) => {
            if (this.state === "connected" || this.state === "connecting") {
                resolve();
                return;
            }

            this.updateState("connecting");
            this.clearTimers();

            try {
                this.ws = new WebSocket(this.config.url);

                this.connectionTimeoutTimer = setTimeout(() => {
                    if (this.state === "connecting") {
                        this.ws?.close();
                        this.handleConnectionError(new Error("Connection timeout"));
                        reject(new Error("Connection timeout"));
                    }
                }, this.config.connectionTimeout);

                this.ws.onopen = () => {
                    this.clearConnectionTimeout();
                    this.updateState("connected");
                    this.reconnectAttempts = 0;
                    this.startHeartbeat();
                    this.flushMessageQueue();
                    this.emit("connect", { connection: this.createConnectionInfo() });
                    resolve();
                };

                this.ws.onmessage = (event) => {
                    this.handleMessage(event);
                };

                this.ws.onerror = (error) => {
                    this.clearConnectionTimeout();
                    this.handleConnectionError(error);
                    reject(error);
                };

                this.ws.onclose = (event) => {
                    this.clearConnectionTimeout();
                    this.handleDisconnect(event.reason);
                };
            } catch (error) {
                this.clearConnectionTimeout();
                this.handleConnectionError(error);
                reject(error);
            }
        });
    }

    /**
     * 断开连接
     */
    disconnect(): void {
        this.clearTimers();
        this.updateState("disconnecting");

        if (this.ws) {
            this.ws.close(1000, "Client disconnect");
            this.ws = null;
        }

        this.updateState("disconnected");
        this.emit("disconnect", {
            connection: this.createConnectionInfo(),
            reason: "Client disconnect",
        });
    }

    /**
     * 发送文本消息
     */
    sendText(text: string): void {
        this.send(textMessage(text));
    }

    /**
     * 发送二进制消息
     */
    sendBinary(data: Uint8Array): void {
        this.send(binaryMessage(data));
    }

    /**
     * 发送 JSON 消息
     */
    sendJson<T>(data: T): void {
        this.sendText(JSON.stringify(data));
    }

    /**
     * 发送消息
     */
    send(message: WebSocketMessage): void {
        if (this.state !== "connected" || !this.ws) {
            this.messageQueue.push(message);
            return;
        }

        try {
            switch (message.type) {
                case "Text":
                    this.ws.send(message.text ?? "");
                    break;
                case "Binary":
                    this.ws.send(message.binary ?? new Uint8Array());
                    break;
                case "Close":
                    this.ws.close();
                    break;
                default:
                    break;
            }
        } catch (error) {
            this.emit("error", { error: error as Error });
        }
    }

    /**
     * 加入房间
     */
    joinRoom(roomId: RoomId): void {
        this.rooms.add(roomId);
        this.sendJson({ type: "join_room", roomId });
    }

    /**
     * 离开房间
     */
    leaveRoom(roomId: RoomId): void {
        this.rooms.delete(roomId);
        this.sendJson({ type: "leave_room", roomId });
    }

    /**
     * 获取当前房间列表
     */
    getRooms(): RoomId[] {
        return Array.from(this.rooms);
    }

    /**
     * 订阅事件
     */
    on<T = unknown>(event: WebSocketEventType, handler: WebSocketEventHandler<T>): () => void {
        const handlers = this.eventHandlers.get(event) ?? new Set();
        handlers.add(handler as WebSocketEventHandler);
        this.eventHandlers.set(event, handlers);

        return () => {
            handlers.delete(handler as WebSocketEventHandler);
        };
    }

    /**
     * 取消订阅事件
     */
    off<T = unknown>(event: WebSocketEventType, handler?: WebSocketEventHandler<T>): void {
        if (handler) {
            const handlers = this.eventHandlers.get(event);
            if (handlers) {
                handlers.delete(handler as WebSocketEventHandler);
            }
        } else {
            this.eventHandlers.delete(event);
        }
    }

    /**
     * 初始化事件处理器映射
     */
    private initEventHandlers(): void {
        const eventTypes: WebSocketEventType[] = [
            "connect",
            "disconnect",
            "message",
            "error",
            "reconnect",
            "reconnect_failed",
        ];
        eventTypes.forEach((type) => {
            this.eventHandlers.set(type, new Set());
        });
    }

    /**
     * 更新连接状态
     */
    private updateState(state: ConnectionState): void {
        this.state = state;
    }

    /**
     * 处理消息
     */
    private handleMessage(event: MessageEvent): void {
        let message: WebSocketMessage;

        if (typeof event.data === "string") {
            message = textMessage(event.data);
        } else if (event.data instanceof ArrayBuffer) {
            message = binaryMessage(new Uint8Array(event.data));
        } else if (event.data instanceof Blob) {
            event.data.arrayBuffer().then((buffer) => {
                this.handleMessage({ data: buffer } as MessageEvent);
            });
            return;
        } else {
            return;
        }

        if (isTextMessage(message)) {
            try {
                const parsed = JSON.parse(message.text);
                if (parsed.type === "pong") {
                    return;
                }
                if (parsed.type === "connection_id") {
                    this.connectionId = parsed.connectionId;
                    return;
                }
            } catch {
                // 不是 JSON 消息，继续处理
            }
        }

        this.emit("message", {
            connection: this.createConnectionInfo(),
            message,
        });
    }

    /**
     * 处理连接错误
     */
    private handleConnectionError(error: unknown): void {
        this.emit("error", { error: error as Error });
        this.updateState("disconnected");
        this.scheduleReconnect();
    }

    /**
     * 处理断开连接
     */
    private handleDisconnect(reason: string): void {
        this.stopHeartbeat();
        this.updateState("disconnected");
        this.ws = null;

        this.emit("disconnect", {
            connection: this.createConnectionInfo(),
            reason,
        });

        this.scheduleReconnect();
    }

    /**
     * 安排重连
     */
    private scheduleReconnect(): void {
        if (this.config.maxReconnectAttempts > 0 && this.reconnectAttempts >= this.config.maxReconnectAttempts) {
            this.emit("reconnect_failed", { totalAttempts: this.reconnectAttempts });
            return;
        }

        this.reconnectAttempts++;
        this.emit("reconnect", {
            attempt: this.reconnectAttempts,
            maxAttempts: this.config.maxReconnectAttempts,
        });

        this.reconnectTimer = setTimeout(() => {
            this.connect().catch(() => {
                // 重连失败，继续尝试
            });
        }, this.config.reconnectInterval);
    }

    /**
     * 开始心跳
     */
    private startHeartbeat(): void {
        this.stopHeartbeat();
        this.heartbeatTimer = setInterval(() => {
            if (this.state === "connected") {
                this.sendJson({ type: "ping" });
            }
        }, this.config.heartbeatInterval);
    }

    /**
     * 停止心跳
     */
    private stopHeartbeat(): void {
        if (this.heartbeatTimer) {
            clearInterval(this.heartbeatTimer);
            this.heartbeatTimer = null;
        }
    }

    /**
     * 刷新消息队列
     */
    private flushMessageQueue(): void {
        while (this.messageQueue.length > 0) {
            const message = this.messageQueue.shift();
            if (message) {
                this.send(message);
            }
        }
    }

    /**
     * 清除定时器
     */
    private clearTimers(): void {
        this.clearConnectionTimeout();
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        this.stopHeartbeat();
    }

    /**
     * 清除连接超时定时器
     */
    private clearConnectionTimeout(): void {
        if (this.connectionTimeoutTimer) {
            clearTimeout(this.connectionTimeoutTimer);
            this.connectionTimeoutTimer = null;
        }
    }

    /**
     * 创建连接信息
     */
    private createConnectionInfo(): Connection {
        return {
            id: this.connectionId ?? "",
            addr: "",
            connectedAt: Date.now(),
            metadata: {},
            rooms: Array.from(this.rooms),
        };
    }

    /**
     * 触发事件
     */
    private emit<T>(event: WebSocketEventType, data: T): void {
        const handlers = this.eventHandlers.get(event);
        if (handlers) {
            for (const handler of handlers) {
                handler(data);
            }
        }
    }
}

/**
 * 创建 WebSocket 客户端实例
 */
export function createWebSocketClient(config: Partial<WebSocketClientConfig> & { url: string }): WebSocketClient {
    return new WebSocketClient(config);
}
