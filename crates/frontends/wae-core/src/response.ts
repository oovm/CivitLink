/**
 * API 错误体
 */
export interface ApiErrorBody {
    /** 错误码 */
    code: string;
    /** 错误消息 */
    message: string;
}

/**
 * 统一 API 响应结构
 */
export interface ApiResponse<T> {
    /** 是否成功 */
    success: boolean;
    /** 响应数据 */
    data: T | null;
    /** 错误信息 */
    error: ApiErrorBody | null;
    /** 请求追踪 ID */
    traceId: string | null;
}

/**
 * 创建成功响应
 */
export function successResponse<T>(data: T): ApiResponse<T> {
    return {
        success: true,
        data,
        error: null,
        traceId: null,
    };
}

/**
 * 创建成功响应（带追踪 ID）
 */
export function successResponseWithTrace<T>(data: T, traceId: string): ApiResponse<T> {
    return {
        success: true,
        data,
        error: null,
        traceId,
    };
}

/**
 * 创建错误响应
 */
export function errorResponse<T>(code: string, message: string): ApiResponse<T> {
    return {
        success: false,
        data: null,
        error: { code, message },
        traceId: null,
    };
}

/**
 * 创建错误响应（带追踪 ID）
 */
export function errorResponseWithTrace<T>(code: string, message: string, traceId: string): ApiResponse<T> {
    return {
        success: false,
        data: null,
        error: { code, message },
        traceId,
    };
}

/**
 * 检查响应是否成功
 */
export function isSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { data: T } {
    return response.success && response.data !== null;
}

/**
 * 从响应获取数据，如果失败则抛出错误
 */
export function unwrapResponse<T>(response: ApiResponse<T>): T {
    if (response.success && response.data !== null) {
        return response.data;
    }
    throw new Error(response.error?.message ?? "Unknown error");
}
