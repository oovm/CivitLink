/**
 * 存储提供商类型
 */
export type StorageProviderType = "Cos" | "Oss" | "Local";

/**
 * 存储配置
 */
export interface StorageConfig {
    /** 提供商类型 */
    provider: StorageProviderType;
    /** 密钥 ID */
    secretId: string;
    /** 密钥 */
    secretKey: string;
    /** 存储桶名称 */
    bucket: string;
    /** 区域 */
    region: string;
    /** 端点 */
    endpoint?: string;
    /** CDN URL */
    cdnUrl?: string;
}

/**
 * 存储上传选项
 */
export interface UploadOptions {
    /** 文件键名 */
    key: string;
    /** 内容类型 */
    contentType?: string;
    /** 自定义元数据 */
    metadata?: Record<string, string>;
    /** 过期时间 (秒) */
    expiresIn?: number;
}

/**
 * 预签名上传结果
 */
export interface PresignedUploadResult {
    /** 预签名上传 URL */
    uploadUrl: string;
    /** 文件访问 URL */
    accessUrl: string;
    /** 过期时间戳 */
    expiresAt: number;
}

/**
 * 文件信息
 */
export interface FileInfo {
    /** 文件键名 */
    key: string;
    /** 文件大小 */
    size: number;
    /** 内容类型 */
    contentType: string;
    /** ETag */
    etag: string;
    /** 最后修改时间 */
    lastModified: number;
}
