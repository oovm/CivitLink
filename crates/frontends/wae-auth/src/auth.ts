import { HttpClient, type HttpClientConfig } from "@wae/client";
import type {
    AuthToken,
    ChangePasswordRequest,
    CreateUserRequest,
    Credentials,
    Role,
    TokenValidation,
    UpdateUserRequest,
    UserInfo,
} from "@wae/core";

/**
 * 认证客户端配置
 */
export interface AuthClientConfig extends Partial<HttpClientConfig> {
    /** Token 存储键名 */
    tokenKey?: string;
    /** 刷新 Token 存储键名 */
    refreshTokenKey?: string;
    /** 用户信息存储键名 */
    userKey?: string;
    /** 自动刷新 Token 提前时间 (毫秒) */
    refreshBeforeExpiry?: number;
    /** 登录接口路径 */
    loginPath?: string;
    /** 登出接口路径 */
    logoutPath?: string;
    /** 刷新 Token 接口路径 */
    refreshTokenPath?: string;
    /** 验证 Token 接口路径 */
    validateTokenPath?: string;
    /** 用户信息接口路径 */
    userPath?: string;
}

/**
 * 认证状态
 */
export interface AuthState {
    /** 是否已认证 */
    isAuthenticated: boolean;
    /** 当前用户信息 */
    user: UserInfo | null;
    /** 令牌信息 */
    token: AuthToken | null;
    /** 是否正在加载 */
    isLoading: boolean;
}

/**
 * 认证状态变化监听器
 */
export type AuthStateListener = (state: AuthState) => void;

/**
 * 存储接口
 */
export interface StorageAdapter {
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
    removeItem(key: string): void;
}

/**
 * 本地存储适配器
 */
class LocalStorageAdapter implements StorageAdapter {
    getItem(key: string): string | null {
        if (typeof window === "undefined") return null;
        return localStorage.getItem(key);
    }

    setItem(key: string, value: string): void {
        if (typeof window === "undefined") return;
        localStorage.setItem(key, value);
    }

    removeItem(key: string): void {
        if (typeof window === "undefined") return;
        localStorage.removeItem(key);
    }
}

/**
 * 认证客户端
 */
export class AuthClient {
    private httpClient: HttpClient;
    private config: Required<AuthClientConfig>;
    private storage: StorageAdapter;
    private state: AuthState;
    private listeners: Set<AuthStateListener> = new Set();
    private refreshTimer: ReturnType<typeof setTimeout> | null = null;

    /**
     * 创建认证客户端实例
     * @param config 客户端配置
     * @param storage 存储适配器
     */
    constructor(config: AuthClientConfig = {}, storage?: StorageAdapter) {
        this.config = {
            baseUrl: "",
            timeout: 30000,
            maxRetries: 3,
            retryDelay: 1000,
            defaultHeaders: {},
            tokenKey: "wae_access_token",
            refreshTokenKey: "wae_refresh_token",
            userKey: "wae_user",
            refreshBeforeExpiry: 60000,
            loginPath: "/auth/login",
            logoutPath: "/auth/logout",
            refreshTokenPath: "/auth/refresh",
            validateTokenPath: "/auth/validate",
            userPath: "/auth/user",
            ...config,
        };

        this.httpClient = new HttpClient({
            baseUrl: this.config.baseUrl,
            timeout: this.config.timeout,
            maxRetries: this.config.maxRetries,
            retryDelay: this.config.retryDelay,
            defaultHeaders: this.config.defaultHeaders,
        });

        this.storage = storage ?? new LocalStorageAdapter();

        this.state = {
            isAuthenticated: false,
            user: null,
            token: null,
            isLoading: false,
        };

        this.setupInterceptors();
        this.loadStoredAuth();
    }

    /**
     * 获取当前认证状态
     */
    getState(): Readonly<AuthState> {
        return { ...this.state };
    }

    /**
     * 订阅认证状态变化
     */
    subscribe(listener: AuthStateListener): () => void {
        this.listeners.add(listener);
        return () => {
            this.listeners.delete(listener);
        };
    }

    /**
     * 用户登录
     * @param credentials 用户凭证
     */
    async login(credentials: Credentials): Promise<AuthToken> {
        this.updateState({ isLoading: true });

        const response = await this.httpClient.post<AuthToken>(this.config.loginPath, credentials);

        if (!response.success || !response.data) {
            this.updateState({ isLoading: false });
            throw new Error(response.error?.message ?? "Login failed");
        }

        const token = response.data;
        await this.setToken(token);

        const userResponse = await this.httpClient.get<UserInfo>(this.config.userPath);
        if (userResponse.success && userResponse.data) {
            await this.setUser(userResponse.data);
        }

        this.updateState({
            isAuthenticated: true,
            token,
            user: userResponse.data ?? null,
            isLoading: false,
        });

        this.scheduleTokenRefresh(token);

        return token;
    }

    /**
     * 用户登出
     */
    async logout(): Promise<void> {
        this.updateState({ isLoading: true });

        try {
            await this.httpClient.post(this.config.logoutPath);
        } catch {
            // 忽略登出请求错误
        }

        await this.clearAuth();
        this.updateState({
            isAuthenticated: false,
            user: null,
            token: null,
            isLoading: false,
        });
    }

    /**
     * 刷新令牌
     */
    async refreshToken(): Promise<AuthToken> {
        const refreshToken = this.storage.getItem(this.config.refreshTokenKey);
        if (!refreshToken) {
            throw new Error("No refresh token available");
        }

        const response = await this.httpClient.post<AuthToken>(this.config.refreshTokenPath, { refreshToken });

        if (!response.success || !response.data) {
            await this.clearAuth();
            this.updateState({
                isAuthenticated: false,
                user: null,
                token: null,
            });
            throw new Error(response.error?.message ?? "Token refresh failed");
        }

        const token = response.data;
        await this.setToken(token);
        this.updateState({ token });
        this.scheduleTokenRefresh(token);

        return token;
    }

    /**
     * 验证令牌
     */
    async validateToken(): Promise<TokenValidation> {
        const response = await this.httpClient.get<TokenValidation>(this.config.validateTokenPath);

        if (!response.success || !response.data) {
            throw new Error(response.error?.message ?? "Token validation failed");
        }

        return response.data;
    }

    /**
     * 获取当前用户信息
     */
    async getUser(): Promise<UserInfo> {
        const response = await this.httpClient.get<UserInfo>(this.config.userPath);

        if (!response.success || !response.data) {
            throw new Error(response.error?.message ?? "Failed to get user");
        }

        await this.setUser(response.data);
        this.updateState({ user: response.data });

        return response.data;
    }

    /**
     * 更新用户信息
     */
    async updateUser(request: UpdateUserRequest): Promise<UserInfo> {
        const response = await this.httpClient.patch<UserInfo>(this.config.userPath, request);

        if (!response.success || !response.data) {
            throw new Error(response.error?.message ?? "Failed to update user");
        }

        await this.setUser(response.data);
        this.updateState({ user: response.data });

        return response.data;
    }

    /**
     * 修改密码
     */
    async changePassword(request: ChangePasswordRequest): Promise<void> {
        const response = await this.httpClient.post<void>(`${this.config.userPath}/password`, request);

        if (!response.success) {
            throw new Error(response.error?.message ?? "Failed to change password");
        }
    }

    /**
     * 创建用户
     */
    async createUser(request: CreateUserRequest): Promise<UserInfo> {
        const response = await this.httpClient.post<UserInfo>(this.config.userPath, request);

        if (!response.success || !response.data) {
            throw new Error(response.error?.message ?? "Failed to create user");
        }

        return response.data;
    }

    /**
     * 获取用户角色
     */
    async getUserRoles(): Promise<Role[]> {
        const response = await this.httpClient.get<Role[]>(`${this.config.userPath}/roles`);

        if (!response.success || !response.data) {
            throw new Error(response.error?.message ?? "Failed to get user roles");
        }

        return response.data;
    }

    /**
     * 检查权限
     */
    async checkPermission(permission: string): Promise<boolean> {
        const response = await this.httpClient.get<{ hasPermission: boolean }>(
            `${this.config.userPath}/permissions/${permission}`,
        );

        if (!response.success || !response.data) {
            return false;
        }

        return response.data.hasPermission;
    }

    /**
     * 获取访问令牌
     */
    getAccessToken(): string | null {
        return this.storage.getItem(this.config.tokenKey);
    }

    /**
     * 获取刷新令牌
     */
    getRefreshToken(): string | null {
        return this.storage.getItem(this.config.refreshTokenKey);
    }

    /**
     * 设置令牌
     */
    private async setToken(token: AuthToken): Promise<void> {
        this.storage.setItem(this.config.tokenKey, token.accessToken);
        if (token.refreshToken) {
            this.storage.setItem(this.config.refreshTokenKey, token.refreshToken);
        }
    }

    /**
     * 设置用户信息
     */
    private async setUser(user: UserInfo): Promise<void> {
        this.storage.setItem(this.config.userKey, JSON.stringify(user));
    }

    /**
     * 清除认证信息
     */
    private async clearAuth(): Promise<void> {
        this.storage.removeItem(this.config.tokenKey);
        this.storage.removeItem(this.config.refreshTokenKey);
        this.storage.removeItem(this.config.userKey);
        this.clearRefreshTimer();
    }

    /**
     * 加载存储的认证信息
     */
    private loadStoredAuth(): void {
        const tokenStr = this.storage.getItem(this.config.tokenKey);
        const userStr = this.storage.getItem(this.config.userKey);

        if (tokenStr && userStr) {
            try {
                const user = JSON.parse(userStr) as UserInfo;
                const token: AuthToken = {
                    accessToken: tokenStr,
                    refreshToken: this.storage.getItem(this.config.refreshTokenKey),
                    tokenType: "Bearer",
                    expiresIn: 0,
                    expiresAt: 0,
                };

                this.state = {
                    isAuthenticated: true,
                    user,
                    token,
                    isLoading: false,
                };
            } catch {
                this.clearAuth();
            }
        }
    }

    /**
     * 设置请求拦截器
     */
    private setupInterceptors(): void {
        this.httpClient.addRequestInterceptor((url, options) => {
            const token = this.storage.getItem(this.config.tokenKey);
            if (token) {
                options.headers = {
                    ...options.headers,
                    Authorization: `Bearer ${token}`,
                };
            }
            return { url, options };
        });
    }

    /**
     * 安排令牌刷新
     */
    private scheduleTokenRefresh(token: AuthToken): void {
        this.clearRefreshTimer();

        if (!token.expiresAt) return;

        const now = Date.now();
        const expiresAt = token.expiresAt * 1000;
        const refreshTime = expiresAt - now - this.config.refreshBeforeExpiry;

        if (refreshTime > 0) {
            this.refreshTimer = setTimeout(() => {
                this.refreshToken().catch(() => {
                    this.clearAuth();
                    this.updateState({
                        isAuthenticated: false,
                        user: null,
                        token: null,
                    });
                });
            }, refreshTime);
        }
    }

    /**
     * 清除刷新定时器
     */
    private clearRefreshTimer(): void {
        if (this.refreshTimer) {
            clearTimeout(this.refreshTimer);
            this.refreshTimer = null;
        }
    }

    /**
     * 更新状态并通知监听器
     */
    private updateState(partial: Partial<AuthState>): void {
        this.state = { ...this.state, ...partial };
        for (const listener of this.listeners) {
            listener(this.state);
        }
    }
}

/**
 * 创建认证客户端实例
 */
export function createAuthClient(config?: AuthClientConfig, storage?: StorageAdapter): AuthClient {
    return new AuthClient(config, storage);
}
