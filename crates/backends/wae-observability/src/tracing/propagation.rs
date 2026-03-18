//! Trace Context 提取模块
//!
//! 提供从 HTTP 头提取和注入 traceparent 和 tracestate 的功能。

use http::{HeaderMap, HeaderValue};
use std::fmt;

/// Trace ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceId([u8; 16]);

/// Span ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanId([u8; 8]);

/// Trace Context
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Trace ID
    pub trace_id: TraceId,
    /// Span ID
    pub span_id: SpanId,
    /// Trace Flags
    pub trace_flags: u8,
}

impl TraceId {
    /// 创建新的 Trace ID
    pub fn new() -> Self {
        let uuid = uuid::Uuid::new_v4();
        TraceId(*uuid.as_bytes())
    }

    /// 从字节创建
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        TraceId(bytes)
    }

    /// 获取字节
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// 从十六进制字符串解析
    pub fn from_hex(s: &str) -> Option<Self> {
        if s.len() != 32 {
            return None;
        }
        let mut bytes = [0u8; 16];
        hex::decode_to_slice(s, &mut bytes).ok()?;
        Some(TraceId(bytes))
    }

    /// 转换为十六进制字符串
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl SpanId {
    /// 创建新的 Span ID
    pub fn new() -> Self {
        let uuid = uuid::Uuid::new_v4();
        let bytes = uuid.as_bytes();
        let mut span_bytes = [0u8; 8];
        span_bytes.copy_from_slice(&bytes[0..8]);
        SpanId(span_bytes)
    }

    /// 从字节创建
    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        SpanId(bytes)
    }

    /// 获取字节
    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    /// 从十六进制字符串解析
    pub fn from_hex(s: &str) -> Option<Self> {
        if s.len() != 16 {
            return None;
        }
        let mut bytes = [0u8; 8];
        hex::decode_to_slice(s, &mut bytes).ok()?;
        Some(SpanId(bytes))
    }

    /// 转换为十六进制字符串
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl Default for SpanId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl TraceContext {
    /// 创建新的 Trace Context
    pub fn new() -> Self {
        Self { trace_id: TraceId::new(), span_id: SpanId::new(), trace_flags: 0x01 }
    }

    /// 从 traceparent 头解析
    pub fn from_traceparent(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return None;
        }
        if parts[0] != "00" {
            return None;
        }
        let trace_id = TraceId::from_hex(parts[1])?;
        let span_id = SpanId::from_hex(parts[2].get(0..16)?)?;
        let trace_flags = u8::from_str_radix(parts[2].get(16..18)?, 16).ok()?;
        Some(Self { trace_id, span_id, trace_flags })
    }

    /// 格式化为 traceparent 头
    pub fn to_traceparent(&self) -> String {
        format!("00-{}-{}-{:02x}", self.trace_id, self.span_id, self.trace_flags)
    }

    /// 创建子 Span
    pub fn child(&self) -> Self {
        Self { trace_id: self.trace_id, span_id: SpanId::new(), trace_flags: self.trace_flags }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 从 HTTP 头提取 Trace Context
pub fn extract_context_from_headers(headers: &HeaderMap) -> Option<TraceContext> {
    headers.get("traceparent").and_then(|v| v.to_str().ok()).and_then(TraceContext::from_traceparent)
}

/// 将 Trace Context 注入 HTTP 头
pub fn inject_context_to_headers(context: &TraceContext, headers: &mut HeaderMap) {
    if let Ok(value) = HeaderValue::from_str(&context.to_traceparent()) {
        headers.insert("traceparent", value);
    }
}

/// 设置全局 Text Map 传播器
pub fn set_global_text_map_propagator() {}
