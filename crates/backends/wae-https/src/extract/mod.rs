#![doc = include_str!("readme.md")]

use http::{Method, Uri, Version, header::HeaderMap};
use serde::de::DeserializeOwned;
use std::fmt;

/// 请求上下文
///
/// 封装 HTTP 请求的原始状态，用于提取器访问请求数据。
#[derive(Debug)]
pub struct RequestParts {
    /// HTTP 方法
    pub method: Method,
    /// 请求 URI
    pub uri: Uri,
    /// HTTP 版本
    pub version: Version,
    /// 请求头
    pub headers: HeaderMap,
    /// 路径参数
    pub path_params: Vec<(String, String)>,
}

impl RequestParts {
    /// 创建新的请求上下文
    pub fn new(method: Method, uri: Uri, version: Version, headers: HeaderMap) -> Self {
        Self { method, uri, version, headers, path_params: Vec::new() }
    }
}

/// 从请求中提取数据的 trait
///
/// 类似于 Axum 的 FromRequestParts trait，用于从 HTTP 请求中提取数据。
pub trait FromRequestParts<S>: Sized {
    /// 提取过程中可能发生的错误
    type Error;

    /// 从请求中提取数据
    ///
    /// # 参数
    ///
    /// * `parts` - 请求上下文
    /// * `state` - 应用状态
    fn from_request_parts(parts: &RequestParts, state: &S) -> Result<Self, Self::Error>;
}

/// Extractor 错误类型
///
/// 统一封装所有提取器可能产生的错误，便于错误处理和响应。
#[derive(Debug)]
pub enum ExtractorError {
    /// 路径参数提取错误
    PathRejection(String),

    /// 查询参数提取错误
    QueryRejection(String),

    /// JSON 提取错误
    JsonRejection(String),

    /// 表单数据提取错误
    FormRejection(String),

    /// 扩展数据提取错误
    ExtensionRejection(String),

    /// Host 提取错误
    HostRejection(String),

    /// TypedHeader 提取错误
    TypedHeaderRejection(String),

    /// Cookie 提取错误
    CookieRejection(String),

    /// Multipart 提取错误
    MultipartRejection(String),

    /// WebSocketUpgrade 提取错误
    WebSocketUpgradeRejection(String),

    /// Bytes 提取错误
    BytesRejection(String),

    /// Stream 提取错误
    StreamRejection(String),

    /// 自定义错误消息
    Custom(String),
}

impl fmt::Display for ExtractorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractorError::PathRejection(e) => write!(f, "Path extraction failed: {}", e),
            ExtractorError::QueryRejection(e) => write!(f, "Query extraction failed: {}", e),
            ExtractorError::JsonRejection(e) => write!(f, "Json extraction failed: {}", e),
            ExtractorError::FormRejection(e) => write!(f, "Form extraction failed: {}", e),
            ExtractorError::ExtensionRejection(e) => write!(f, "Extension extraction failed: {}", e),
            ExtractorError::HostRejection(e) => write!(f, "Host extraction failed: {}", e),
            ExtractorError::TypedHeaderRejection(e) => write!(f, "TypedHeader extraction failed: {}", e),
            ExtractorError::CookieRejection(e) => write!(f, "Cookie extraction failed: {}", e),
            ExtractorError::MultipartRejection(e) => write!(f, "Multipart extraction failed: {}", e),
            ExtractorError::WebSocketUpgradeRejection(e) => write!(f, "WebSocketUpgrade extraction failed: {}", e),
            ExtractorError::BytesRejection(e) => write!(f, "Bytes extraction failed: {}", e),
            ExtractorError::StreamRejection(e) => write!(f, "Stream extraction failed: {}", e),
            ExtractorError::Custom(msg) => write!(f, "Extractor error: {}", msg),
        }
    }
}

impl std::error::Error for ExtractorError {}

/// 路径参数提取器
///
/// 用于从 URL 路径中提取命名参数。
///
/// # 示例
///
/// ```ignore
/// use wae_https::extract::Path;
///
/// async fn handler(Path(user_id): Path<u64>) -> String {
///     format!("User ID: {}", user_id)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Path<T>(pub T);

impl<T, S> FromRequestParts<S> for Path<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    type Error = ExtractorError;

    fn from_request_parts(parts: &RequestParts, _state: &S) -> Result<Self, Self::Error> {
        if let Some((_, value)) = parts.path_params.first() {
            T::from_str(value)
                .map(Path)
                .map_err(|e| ExtractorError::PathRejection(format!("Failed to parse path parameter: {}", e)))
        }
        else {
            Err(ExtractorError::PathRejection("No path parameters found".to_string()))
        }
    }
}

/// 查询参数提取器
///
/// 用于从 URL 查询字符串中提取参数。
///
/// # 示例
///
/// ```ignore
/// use wae_https::extract::Query;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Pagination {
///     page: u32,
///     limit: u32,
/// }
///
/// async fn handler(Query(pagination): Query<Pagination>) -> String {
///     format!("Page: {}, Limit: {}", pagination.page, pagination.limit)
/// }
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Query<T>(pub T);

impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned,
{
    type Error = ExtractorError;

    fn from_request_parts(parts: &RequestParts, _state: &S) -> Result<Self, Self::Error> {
        let query = parts.uri.query().unwrap_or_default();
        serde_urlencoded::from_str(query)
            .map(Query)
            .map_err(|e| ExtractorError::QueryRejection(format!("Failed to parse query parameters: {}", e)))
    }
}

/// JSON 请求体提取器
///
/// 用于从 JSON 格式的请求体中提取数据。
///
/// # 示例
///
/// ```ignore
/// use wae_https::extract::Json;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// async fn handler(Json(user): Json<User>) -> String {
///     format!("Name: {}, Age: {}", user.name, user.age)
/// }
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Json<T>(pub T);

/// 表单数据提取器
///
/// 用于从表单格式的请求体中提取数据。
///
/// # 示例
///
/// ```ignore
/// use wae_https::extract::Form;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct LoginForm {
///     username: String,
///     password: String,
/// }
///
/// async fn handler(Form(form): Form<LoginForm>) -> String {
///     format!("Username: {}", form.username)
/// }
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Form<T>(pub T);

/// 请求头提取器
///
/// 用于从请求中提取指定名称的请求头值。
///
/// # 示例
///
/// ```ignore
/// use wae_https::extract::Header;
///
/// async fn handler(Header(content_type): Header<String>) -> String {
///     content_type
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Header<T>(pub T);

/// HTTP 方法提取器
///
/// 获取当前请求的 HTTP 方法（GET、POST、PUT、DELETE 等）。
pub type HttpMethod = Method;

impl<S> FromRequestParts<S> for HttpMethod {
    type Error = ExtractorError;

    fn from_request_parts(parts: &RequestParts, _state: &S) -> Result<Self, Self::Error> {
        Ok(parts.method.clone())
    }
}

/// 请求 URI 提取器
///
/// 获取当前请求的完整 URI。
pub type RequestUri = Uri;

impl<S> FromRequestParts<S> for RequestUri {
    type Error = ExtractorError;

    fn from_request_parts(parts: &RequestParts, _state: &S) -> Result<Self, Self::Error> {
        Ok(parts.uri.clone())
    }
}

impl<S> FromRequestParts<S> for RequestParts {
    type Error = ExtractorError;

    fn from_request_parts(parts: &RequestParts, _state: &S) -> Result<Self, Self::Error> {
        Ok(parts.clone())
    }
}

impl Clone for RequestParts {
    fn clone(&self) -> Self {
        Self {
            method: self.method.clone(),
            uri: self.uri.clone(),
            version: self.version,
            headers: self.headers.clone(),
            path_params: self.path_params.clone(),
        }
    }
}

/// HTTP 版本提取器
///
/// 获取当前请求的 HTTP 版本（HTTP/1.0、HTTP/1.1、HTTP/2.0 等）。
pub type HttpVersion = Version;

impl<S> FromRequestParts<S> for HttpVersion {
    type Error = ExtractorError;

    fn from_request_parts(parts: &RequestParts, _state: &S) -> Result<Self, Self::Error> {
        Ok(parts.version)
    }
}

/// 请求头映射提取器
///
/// 获取所有请求头的键值对映射。
pub type Headers = HeaderMap;

impl<S> FromRequestParts<S> for Headers {
    type Error = ExtractorError;

    fn from_request_parts(parts: &RequestParts, _state: &S) -> Result<Self, Self::Error> {
        Ok(parts.headers.clone())
    }
}

/// 扩展数据提取器
#[derive(Debug, Clone)]
pub struct Extension<T>(pub T);

/// 多部分表单数据提取器
#[derive(Debug, Clone)]
pub struct Multipart;

/// 原始 URI 提取器
pub type OriginalUri = http::Uri;

/// 状态提取器
///
/// 用于从应用状态中提取数据。
///
/// # 示例
///
/// ```ignore
/// use wae_https::extract::State;
///
/// async fn handler(State(db_pool): State<DatabasePool>) -> String {
///     // 使用 db_pool
/// }
/// ```
#[derive(Debug, Clone)]
pub struct State<T>(pub T);

impl<T, S> FromRequestParts<S> for State<T>
where
    T: Clone + Send + Sync + 'static,
    S: std::ops::Deref<Target = T>,
{
    type Error = ExtractorError;

    fn from_request_parts(_parts: &RequestParts, state: &S) -> Result<Self, Self::Error> {
        Ok(State(state.deref().clone()))
    }
}

/// WebSocket 升级提取器
#[derive(Debug, Clone)]
pub struct WebSocketUpgrade;

/// 流式请求体提取器
///
/// 用于从请求中提取流式数据。
pub type Stream = crate::Body;

/// 元组提取器支持（最多支持 16 个参数）
macro_rules! impl_from_request_parts_tuple {
    ($($ty:ident),*) => {
        #[allow(non_snake_case, unused_variables)]
        impl<S, $($ty,)*> FromRequestParts<S> for ($($ty,)*)
        where
            $($ty: FromRequestParts<S, Error = ExtractorError>,)*
        {
            type Error = ExtractorError;

            fn from_request_parts(parts: &RequestParts, state: &S) -> Result<Self, Self::Error> {
                Ok(($(
                    $ty::from_request_parts(parts, state)?,
                )*))
            }
        }
    };
}

impl_from_request_parts_tuple!();
impl_from_request_parts_tuple!(T1);
impl_from_request_parts_tuple!(T1, T2);
impl_from_request_parts_tuple!(T1, T2, T3);
impl_from_request_parts_tuple!(T1, T2, T3, T4);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_from_request_parts_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
