import { HttpClient, type HttpClientConfig } from "@wae/client";
import type { FileInfo, PresignedUploadResult } from "@wae/core";

/**
 * 存储客户端配置
 */
export interface StorageClientConfig extends Partial<HttpClientConfig> {
    /** 获取预签名上传 URL 接口路径 */
    presignUploadPath?: string;
    /** 获取带签名访问 URL 接口路径 */
    signUrlPath?: string;
    /** 获取文件信息接口路径 */
    fileInfoPath?: string;
    /** 删除文件接口路径 */
    deletePath?: string;
}

/**
 * 上传进度回调
 */
export type UploadProgressCallback = (progress: { loaded: number; total: number; percentage: number }) => void;

/**
 * 存储客户端
 */
export class StorageClient {
    private httpClient: HttpClient;
    private config: Required<StorageClientConfig>;

    /**
     * 创建存储客户端实例
     * @param config 客户端配置
     */
    constructor(config: StorageClientConfig = {}) {
        this.config = {
            baseUrl: "",
            timeout: 60000,
            maxRetries: 3,
            retryDelay: 1000,
            defaultHeaders: {},
            presignUploadPath: "/storage/presign-upload",
            signUrlPath: "/storage/sign-url",
            fileInfoPath: "/storage/file-info",
            deletePath: "/storage/delete",
            ...config,
        };

        this.httpClient = new HttpClient({
            baseUrl: this.config.baseUrl,
            timeout: this.config.timeout,
            maxRetries: this.config.maxRetries,
            retryDelay: this.config.retryDelay,
            defaultHeaders: this.config.defaultHeaders,
        });
    }

    /**
     * 获取预签名上传 URL
     * @param key 文件键名
     * @param contentType 内容类型
     * @param expiresIn 过期时间 (秒)
     */
    async getPresignedUploadUrl(key: string, contentType?: string, expiresIn?: number): Promise<PresignedUploadResult> {
        const params: Record<string, string> = { key };
        if (contentType) params.contentType = contentType;
        if (expiresIn) params.expiresIn = String(expiresIn);

        const response = await this.httpClient.get<PresignedUploadResult>(this.config.presignUploadPath, params);

        if (!response.success || !response.data) {
            throw new Error(response.error?.message ?? "Failed to get presigned upload URL");
        }

        return response.data;
    }

    /**
     * 获取带签名的访问 URL
     * @param key 文件键名
     * @param expiresIn 过期时间 (秒)
     */
    async getSignedUrl(key: string, expiresIn?: number): Promise<string> {
        const params: Record<string, string> = { key };
        if (expiresIn) params.expiresIn = String(expiresIn);

        const response = await this.httpClient.get<{ url: string }>(this.config.signUrlPath, params);

        if (!response.success || !response.data) {
            throw new Error(response.error?.message ?? "Failed to get signed URL");
        }

        return response.data.url;
    }

    /**
     * 获取文件信息
     * @param key 文件键名
     */
    async getFileInfo(key: string): Promise<FileInfo> {
        const response = await this.httpClient.get<FileInfo>(this.config.fileInfoPath, { key });

        if (!response.success || !response.data) {
            throw new Error(response.error?.message ?? "Failed to get file info");
        }

        return response.data;
    }

    /**
     * 删除文件
     * @param key 文件键名
     */
    async deleteFile(key: string): Promise<void> {
        const response = await this.httpClient.delete<{ success: boolean }>(
            `${this.config.deletePath}?key=${encodeURIComponent(key)}`,
        );

        if (!response.success) {
            throw new Error(response.error?.message ?? "Failed to delete file");
        }
    }

    /**
     * 上传文件
     * @param file 文件对象
     * @param key 文件键名
     * @param onProgress 上传进度回调
     */
    async uploadFile(file: File, key: string, onProgress?: UploadProgressCallback): Promise<PresignedUploadResult> {
        const presigned = await this.getPresignedUploadUrl(key, file.type);

        return new Promise((resolve, reject) => {
            const xhr = new XMLHttpRequest();

            xhr.upload.onprogress = (event) => {
                if (onProgress && event.lengthComputable) {
                    onProgress({
                        loaded: event.loaded,
                        total: event.total,
                        percentage: Math.round((event.loaded / event.total) * 100),
                    });
                }
            };

            xhr.onload = () => {
                if (xhr.status >= 200 && xhr.status < 300) {
                    resolve(presigned);
                } else {
                    reject(new Error(`Upload failed: ${xhr.status} ${xhr.statusText}`));
                }
            };

            xhr.onerror = () => {
                reject(new Error("Upload failed: Network error"));
            };

            xhr.open("PUT", presigned.uploadUrl, true);
            xhr.setRequestHeader("Content-Type", file.type);
            xhr.send(file);
        });
    }

    /**
     * 上传数据
     * @param data 数据
     * @param key 文件键名
     * @param contentType 内容类型
     */
    async uploadData(
        data: Blob | ArrayBuffer | string,
        key: string,
        contentType: string = "application/octet-stream",
    ): Promise<PresignedUploadResult> {
        const presigned = await this.getPresignedUploadUrl(key, contentType);

        let body: Blob;
        if (typeof data === "string") {
            body = new Blob([data], { type: contentType });
        } else if (data instanceof ArrayBuffer) {
            body = new Blob([data], { type: contentType });
        } else {
            body = data;
        }

        const response = await fetch(presigned.uploadUrl, {
            method: "PUT",
            headers: {
                "Content-Type": contentType,
            },
            body,
        });

        if (!response.ok) {
            throw new Error(`Upload failed: ${response.status} ${response.statusText}`);
        }

        return presigned;
    }

    /**
     * 下载文件
     * @param key 文件键名
     */
    async downloadFile(key: string): Promise<Blob> {
        const url = await this.getSignedUrl(key);
        const response = await fetch(url);

        if (!response.ok) {
            throw new Error(`Download failed: ${response.status} ${response.statusText}`);
        }

        return response.blob();
    }

    /**
     * 下载文件为文本
     * @param key 文件键名
     */
    async downloadAsText(key: string): Promise<string> {
        const url = await this.getSignedUrl(key);
        const response = await fetch(url);

        if (!response.ok) {
            throw new Error(`Download failed: ${response.status} ${response.statusText}`);
        }

        return response.text();
    }

    /**
     * 下载文件为 JSON
     * @param key 文件键名
     */
    async downloadAsJson<T>(key: string): Promise<T> {
        const text = await this.downloadAsText(key);
        return JSON.parse(text);
    }

    /**
     * 获取文件访问 URL
     * @param key 文件键名
     * @param expiresIn 过期时间 (秒)
     */
    async getAccessUrl(key: string, expiresIn?: number): Promise<string> {
        return this.getSignedUrl(key, expiresIn);
    }
}

/**
 * 创建存储客户端实例
 */
export function createStorageClient(config?: StorageClientConfig): StorageClient {
    return new StorageClient(config);
}
