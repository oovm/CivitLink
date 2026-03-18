/**
 * 用户 ID 类型
 */
export type UserId = string;

/**
 * 角色 ID 类型
 */
export type RoleId = string;

/**
 * 权限代码类型
 */
export type PermissionCode = string;

/**
 * 用户信息
 */
export interface UserInfo {
    /** 用户 ID */
    id: UserId;
    /** 用户名 */
    username: string;
    /** 邮箱 */
    email: string | null;
    /** 手机号 */
    phone: string | null;
    /** 显示名称 */
    displayName: string | null;
    /** 头像 URL */
    avatarUrl: string | null;
    /** 是否已验证 */
    verified: boolean;
    /** 是否已禁用 */
    disabled: boolean;
    /** 自定义属性 */
    attributes: Record<string, unknown>;
    /** 创建时间 */
    createdAt: number;
    /** 更新时间 */
    updatedAt: number;
}

/**
 * 用户凭证
 */
export interface Credentials {
    /** 用户名或邮箱 */
    identifier: string;
    /** 密码 */
    password: string;
    /** 额外参数 */
    extra?: Record<string, string>;
}

/**
 * 认证令牌
 */
export interface AuthToken {
    /** 访问令牌 */
    accessToken: string;
    /** 刷新令牌 */
    refreshToken: string | null;
    /** 令牌类型 */
    tokenType: string;
    /** 过期时间 (秒) */
    expiresIn: number;
    /** 过期时间戳 */
    expiresAt: number;
}

/**
 * 角色信息
 */
export interface Role {
    /** 角色 ID */
    id: RoleId;
    /** 角色名称 */
    name: string;
    /** 角色描述 */
    description: string | null;
    /** 权限列表 */
    permissions: PermissionCode[];
}

/**
 * Token 验证结果
 */
export interface TokenValidation {
    /** 用户 ID */
    userId: UserId;
    /** 用户信息 */
    user: UserInfo | null;
    /** 角色 */
    roles: Role[];
    /** 权限 */
    permissions: PermissionCode[];
    /** Token 元数据 */
    metadata: Record<string, string>;
}

/**
 * 用户创建请求
 */
export interface CreateUserRequest {
    /** 用户名 */
    username: string;
    /** 密码 */
    password: string;
    /** 邮箱 */
    email?: string;
    /** 手机号 */
    phone?: string;
    /** 显示名称 */
    displayName?: string;
    /** 自定义属性 */
    attributes?: Record<string, unknown>;
}

/**
 * 用户更新请求
 */
export interface UpdateUserRequest {
    /** 显示名称 */
    displayName?: string;
    /** 头像 URL */
    avatarUrl?: string;
    /** 自定义属性 */
    attributes?: Record<string, unknown>;
}

/**
 * 密码修改请求
 */
export interface ChangePasswordRequest {
    /** 旧密码 */
    oldPassword: string;
    /** 新密码 */
    newPassword: string;
}

/**
 * 认证配置
 */
export interface AuthConfig {
    /** Token 过期时间 (秒) */
    tokenExpiresIn: number;
    /** 刷新 Token 过期时间 (秒) */
    refreshTokenExpiresIn: number;
    /** Token 签发者 */
    issuer: string;
    /** Token 受众 */
    audience: string;
    /** 密码最小长度 */
    passwordMinLength: number;
    /** 是否要求密码包含数字 */
    passwordRequireDigit: boolean;
    /** 是否要求密码包含特殊字符 */
    passwordRequireSpecial: boolean;
    /** 登录失败锁定阈值 */
    maxLoginAttempts: number;
    /** 锁定时间 (秒) */
    lockoutDuration: number;
}

/**
 * API Key 信息
 */
export interface ApiKeyInfo {
    /** Key ID */
    id: string;
    /** Key 名称 */
    name: string;
    /** Key 前缀 (用于识别) */
    prefix: string;
    /** 创建时间 */
    createdAt: number;
    /** 过期时间 */
    expiresAt: number | null;
    /** 最后使用时间 */
    lastUsedAt: number | null;
}
