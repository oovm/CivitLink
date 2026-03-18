/**
 * 云服务错误类型
 */
export type CloudErrorType =
    | "Authentication"
    | "InvalidParams"
    | "NotFound"
    | "PermissionDenied"
    | "RateLimit"
    | "Internal"
    | "Network"
    | "Unknown";

/**
 * 云服务错误
 */
export interface CloudError {
    /** 错误类型 */
    type: CloudErrorType;
    /** 错误消息 */
    message: string;
}

/**
 * 创建认证错误
 */
export function authenticationError(message: string): CloudError {
    return { type: "Authentication", message };
}

/**
 * 创建参数无效错误
 */
export function invalidParamsError(message: string): CloudError {
    return { type: "InvalidParams", message };
}

/**
 * 创建资源未找到错误
 */
export function notFoundError(message: string): CloudError {
    return { type: "NotFound", message };
}

/**
 * 创建权限不足错误
 */
export function permissionDeniedError(message: string): CloudError {
    return { type: "PermissionDenied", message };
}

/**
 * 创建速率限制错误
 */
export function rateLimitError(message: string): CloudError {
    return { type: "RateLimit", message };
}

/**
 * 创建内部错误
 */
export function internalError(message: string): CloudError {
    return { type: "Internal", message };
}

/**
 * 创建网络错误
 */
export function networkError(message: string): CloudError {
    return { type: "Network", message };
}

/**
 * 创建未知错误
 */
export function unknownError(message: string): CloudError {
    return { type: "Unknown", message };
}
