# HTTP 中间件模块

提供常用的 HTTP 中间件实现。

## 中间件列表

- **CORS** - 跨域资源共享配置
- **Compression** - 响应压缩
- **Request ID** - 请求追踪 ID
- **Tracing** - 请求追踪与日志

## 重新导出

本模块还重新导出 tower-http 的常用中间件：

- CatchPanicLayer
- NormalizePathLayer
- MakeRequestUuid
- SetRequestHeaderLayer
- SetResponseHeaderLayer
- TimeoutLayer
