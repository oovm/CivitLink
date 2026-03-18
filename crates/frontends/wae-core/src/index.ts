/**
 * @wae/core - WAE TypeScript 核心类型定义
 *
 * 提供与后端 Rust 类型对应的 TypeScript 类型定义。
 */

export {
    ApiKeyInfo,
    AuthConfig,
    AuthToken,
    ChangePasswordRequest,
    CreateUserRequest,
    Credentials,
    PermissionCode,
    Role,
    RoleId,
    TokenValidation,
    UpdateUserRequest,
    UserId,
    UserInfo,
} from "./auth.js";

export {
    BillingCostConfig,
    BillingDimensions,
    calculateTotalCost,
    ImageCost,
    TextCost,
} from "./billing.js";
export {
    authenticationError,
    CloudError,
    CloudErrorType,
    internalError,
    invalidParamsError,
    networkError,
    notFoundError,
    permissionDeniedError,
    rateLimitError,
    unknownError,
} from "./error.js";
export {
    ApiErrorBody,
    ApiResponse,
    errorResponse,
    errorResponseWithTrace,
    isSuccess,
    successResponse,
    successResponseWithTrace,
    unwrapResponse,
} from "./response.js";

export {
    FileInfo,
    PresignedUploadResult,
    StorageConfig,
    StorageProviderType,
    UploadOptions,
} from "./storage.js";

export {
    binaryMessage,
    ConnectEventData,
    Connection,
    ConnectionId,
    ConnectionState,
    closeMessage,
    DisconnectEventData,
    ErrorEventData,
    isBinaryMessage,
    isTextMessage,
    MessageEventData,
    ReconnectEventData,
    ReconnectFailedEventData,
    RoomId,
    textMessage,
    WebSocketClientConfig,
    WebSocketEventHandler,
    WebSocketEventType,
    WebSocketMessage,
    WebSocketMessageType,
    WebSocketServerConfig,
} from "./websocket.js";
