# Extractor 模块

提供请求参数提取器的统一封装，基于 axum extractors 进行扩展。

## 支持的 Extractors

- [`Path`] - 路径参数提取
- [`Query`] - 查询参数提取
- [`Json`] - JSON 请求体提取
- [`Form`] - 表单数据提取
- [`Header`] - 请求头提取
- [`State`] - 应用状态提取
- [`Extension`] - 扩展数据提取
- [`Body`] - 原始请求体
- [`Method`] - HTTP 方法
- [`Uri`] - 请求 URI
- [`Version`] - HTTP 版本

## 错误处理

所有提取器错误统一转换为 [`ExtractorError`]，便于统一处理。
