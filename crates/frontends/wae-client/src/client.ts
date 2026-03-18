import type { ApiResponse, CloudError } from "@wae/core";

/**
 * HTTP 客户端配置
 */
export interface HttpClientConfig {
    /** 基础 URL */
    baseUrl: string;
    /** 请求超时时间 (毫秒) */
    timeout: number;
    /** 最大重试次数 */
    maxRetries: number;
    /** 重试间隔 (毫秒) */
    retryDelay: number;
    /** 默认请求头 */
    defaultHeaders: Record<string, string>;
}

/**
 * 请求拦截器函数类型
 */
export type RequestInterceptor = (
    url: string,
    options: RequestInit,
) => Promise<{ url: string; options: RequestInit }> | { url: string; options: RequestInit };

/**
 * 响应拦截器函数类型
 */
export type ResponseInterceptor = <T>(
    response: ApiResponse<T>,
    request: { url: string; options: RequestInit },
) => Promise<ApiResponse<T>> | ApiResponse<T>;

/**
 * 错误拦截器函数类型
 */
export type ErrorInterceptor = (
    error: CloudError,
    request: { url: string; options: RequestInit },
) => Promise<void> | void;

/**
 * HTTP 响应
 */
export interface HttpResponse<T> {
    /** 响应数据 */
    data: T;
    /** HTTP 状态码 */
    status: number;
    /** 响应头 */
    headers: Headers;
}

/**
 * 创建默认配置
 */
function createDefaultConfig(): HttpClientConfig {
    return {
        baseUrl: "",
        timeout: 30000,
        maxRetries: 3,
        retryDelay: 1000,
        defaultHeaders: {
            "Content-Type": "application/json",
        },
    };
}

/**
 * HTTP 客户端
 */
export class HttpClient {
    private config: HttpClientConfig;
    private requestInterceptors: RequestInterceptor[] = [];
    private responseInterceptors: ResponseInterceptor[] = [];
    private errorInterceptors: ErrorInterceptor[] = [];
    private abortController: AbortController | null = null;

    /**
     * 创建 HTTP 客户端实例
     * @param config 客户端配置
     */
    constructor(config: Partial<HttpClientConfig> = {}) {
        this.config = { ...createDefaultConfig(), ...config };
    }

    /**
     * 获取当前配置
     */
    getConfig(): Readonly<HttpClientConfig> {
        return this.config;
    }

    /**
     * 更新配置
     */
    updateConfig(config: Partial<HttpClientConfig>): void {
        this.config = { ...this.config, ...config };
    }

    /**
     * 添加请求拦截器
     */
    addRequestInterceptor(interceptor: RequestInterceptor): void {
        this.requestInterceptors.push(interceptor);
    }

    /**
     * 添加响应拦截器
     */
    addResponseInterceptor(interceptor: ResponseInterceptor): void {
        this.responseInterceptors.push(interceptor);
    }

    /**
     * 添加错误拦截器
     */
    addErrorInterceptor(interceptor: ErrorInterceptor): void {
        this.errorInterceptors.push(interceptor);
    }

    /**
     * 移除所有请求拦截器
     */
    clearRequestInterceptors(): void {
        this.requestInterceptors = [];
    }

    /**
     * 移除所有响应拦截器
     */
    clearResponseInterceptors(): void {
        this.responseInterceptors = [];
    }

    /**
     * 移除所有错误拦截器
     */
    clearErrorInterceptors(): void {
        this.errorInterceptors = [];
    }

    /**
     * 发送 GET 请求
     */
    async get<T>(url: string, params?: Record<string, string>): Promise<ApiResponse<T>> {
        const fullUrl = params ? this.buildUrlWithParams(url, params) : url;
        return this.request<T>("GET", fullUrl);
    }

    /**
     * 发送 POST 请求
     */
    async post<T>(url: string, body?: unknown): Promise<ApiResponse<T>> {
        return this.request<T>("POST", url, body);
    }

    /**
     * 发送 PUT 请求
     */
    async put<T>(url: string, body?: unknown): Promise<ApiResponse<T>> {
        return this.request<T>("PUT", url, body);
    }

    /**
     * 发送 PATCH 请求
     */
    async patch<T>(url: string, body?: unknown): Promise<ApiResponse<T>> {
        return this.request<T>("PATCH", url, body);
    }

    /**
     * 发送 DELETE 请求
     */
    async delete<T>(url: string): Promise<ApiResponse<T>> {
        return this.request<T>("DELETE", url);
    }

    /**
     * 发送请求
     */
    async request<T>(
        method: string,
        url: string,
        body?: unknown,
        headers?: Record<string, string>,
    ): Promise<ApiResponse<T>> {
        let fullUrl = this.buildUrl(url);
        let options: RequestInit = {
            method,
            headers: {
                ...this.config.defaultHeaders,
                ...headers,
            },
        };

        if (body !== undefined) {
            options.body = JSON.stringify(body);
        }

        for (const interceptor of this.requestInterceptors) {
            const result = await interceptor(fullUrl, options);
            fullUrl = result.url;
            options = result.options;
        }

        let lastError: CloudError | null = null;

        for (let attempt = 0; attempt <= this.config.maxRetries; attempt++) {
            if (attempt > 0) {
                await this.delay(this.config.retryDelay * attempt);
            }

            try {
                this.abortController = new AbortController();
                options.signal = this.abortController.signal;

                const controller = new AbortController();
                const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);
                options.signal = controller.signal;

                const response = await fetch(fullUrl, options);
                clearTimeout(timeoutId);

                const apiResponse = await this.parseResponse<T>(response);

                for (const interceptor of this.responseInterceptors) {
                    await interceptor(apiResponse, { url: fullUrl, options });
                }

                if (!apiResponse.success) {
                    const error: CloudError = {
                        type: this.mapStatusToErrorType(response.status),
                        message: apiResponse.error?.message ?? "Unknown error",
                    };
                    lastError = error;

                    if (this.isRetryableStatus(response.status) && attempt < this.config.maxRetries) {
                        continue;
                    }

                    for (const interceptor of this.errorInterceptors) {
                        await interceptor(error, { url: fullUrl, options });
                    }

                    return apiResponse;
                }

                return apiResponse;
            } catch (err) {
                const error: CloudError = this.mapErrorToCloudError(err);
                lastError = error;

                if (this.isRetryableError(error) && attempt < this.config.maxRetries) {
                    continue;
                }

                for (const interceptor of this.errorInterceptors) {
                    await interceptor(error, { url: fullUrl, options });
                }

                return {
                    success: false,
                    data: null,
                    error: { code: error.type, message: error.message },
                    traceId: null,
                };
            }
        }

        return {
            success: false,
            data: null,
            error: { code: lastError?.type ?? "Unknown", message: lastError?.message ?? "Unknown error" },
            traceId: null,
        };
    }

    /**
     * 取消当前请求
     */
    abort(): void {
        this.abortController?.abort();
    }

    /**
     * 构建完整 URL
     */
    private buildUrl(path: string): string {
        if (path.startsWith("http://") || path.startsWith("https://")) {
            return path;
        }
        const baseUrl = this.config.baseUrl.endsWith("/") ? this.config.baseUrl.slice(0, -1) : this.config.baseUrl;
        const url = path.startsWith("/") ? path : `/${path}`;
        return `${baseUrl}${url}`;
    }

    /**
     * 构建带查询参数的 URL
     */
    private buildUrlWithParams(url: string, params: Record<string, string>): string {
        const searchParams = new URLSearchParams(params);
        const separator = url.includes("?") ? "&" : "?";
        return `${url}${separator}${searchParams.toString()}`;
    }

    /**
     * 解析响应
     */
    private async parseResponse<T>(response: Response): Promise<ApiResponse<T>> {
        const contentType = response.headers.get("content-type");
        if (contentType?.includes("application/json")) {
            return response.json();
        }
        const text = await response.text();
        return {
            success: response.ok,
            data: response.ok ? (text as T) : null,
            error: response.ok ? null : { code: String(response.status), message: text },
            traceId: response.headers.get("x-trace-id"),
        };
    }

    /**
     * 映射 HTTP 状态码到错误类型
     */
    private mapStatusToErrorType(status: number): CloudError["type"] {
        switch (status) {
            case 401:
                return "Authentication";
            case 403:
                return "PermissionDenied";
            case 404:
                return "NotFound";
            case 429:
                return "RateLimit";
            case 400:
                return "InvalidParams";
            default:
                if (status >= 500) {
                    return "Internal";
                }
                return "Unknown";
        }
    }

    /**
     * 映射错误到 CloudError
     */
    private mapErrorToCloudError(error: unknown): CloudError {
        if (error instanceof Error) {
            if (error.name === "AbortError") {
                return { type: "Network", message: "Request aborted" };
            }
            return { type: "Network", message: error.message };
        }
        return { type: "Unknown", message: String(error) };
    }

    /**
     * 判断状态码是否可重试
     */
    private isRetryableStatus(status: number): boolean {
        return [408, 429, 500, 502, 503, 504].includes(status);
    }

    /**
     * 判断错误是否可重试
     */
    private isRetryableError(error: CloudError): boolean {
        return error.type === "Network" || error.type === "Unknown";
    }

    /**
     * 延迟函数
     */
    private delay(ms: number): Promise<void> {
        return new Promise((resolve) => setTimeout(resolve, ms));
    }
}

/**
 * 创建 HTTP 客户端实例
 */
export function createHttpClient(config?: Partial<HttpClientConfig>): HttpClient {
    return new HttpClient(config);
}
