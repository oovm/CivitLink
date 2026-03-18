//! 动态值类型定义

use crate::WaeError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

/// 计费维度：四元矩阵 (输入文本, 输出文本, 输入像素, 输出像素)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BillingDimensions {
    /// 输入文本长度 (如字符数或 Token 数)
    pub input_text: u64,
    /// 输出文本长度
    pub output_text: u64,
    /// 输入像素数 (宽 * 高)
    pub input_pixels: u64,
    /// 输出像素数
    pub output_pixels: u64,
}

/// 文本成本定义 (每百万 Tokens/字符)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCost {
    /// 每百万单位的价格
    pub per_million: crate::Decimal,
}

/// 图像成本定义 (每百万像素)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCost {
    /// 每百万像素的价格
    pub per_million: crate::Decimal,
}

/// 四元矩阵成本配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingCostConfig {
    /// 输入文本成本 (每百万 Tokens)
    pub input_text: TextCost,
    /// 输出文本成本 (每百万 Tokens)
    pub output_text: TextCost,
    /// 输入图像成本 (每百万像素)
    pub input_pixels: ImageCost,
    /// 输出图像成本 (每百万像素)
    pub output_pixels: ImageCost,
}

impl BillingCostConfig {
    /// 计算总成本 (单位：Decimal)
    ///
    /// 计算公式: (消耗数量 / 1,000,000) * 每百万单价
    pub fn calculate_total_cost(&self, usage: &BillingDimensions) -> crate::Decimal {
        let million = crate::dec!(1_000_000);
        let mut total = crate::Decimal::ZERO;

        if usage.input_text > 0 {
            total += (crate::Decimal::from(usage.input_text) / million) * self.input_text.per_million;
        }
        if usage.output_text > 0 {
            total += (crate::Decimal::from(usage.output_text) / million) * self.output_text.per_million;
        }
        if usage.input_pixels > 0 {
            total += (crate::Decimal::from(usage.input_pixels) / million) * self.input_pixels.per_million;
        }
        if usage.output_pixels > 0 {
            total += (crate::Decimal::from(usage.output_pixels) / million) * self.output_pixels.per_million;
        }

        total
    }
}

/// 动态值类型
///
/// 提供类似 JSON 的动态类型支持，用于：
/// - 配置文件解析
/// - API 响应处理
/// - 数据库查询结果
/// - 通用数据交换
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 空值
    Null,
    /// 布尔值
    Bool(bool),
    /// 64位整数
    Integer(i64),
    /// 64位浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 字节数组
    Bytes(Vec<u8>),
    /// 数组
    Array(Vec<Value>),
    /// 对象/映射
    Object(HashMap<String, Value>),
}

impl Value {
    /// 创建空值
    pub fn null() -> Self {
        Value::Null
    }

    /// 创建布尔值
    pub fn bool(v: bool) -> Self {
        Value::Bool(v)
    }

    /// 创建整数值
    pub fn integer(v: i64) -> Self {
        Value::Integer(v)
    }

    /// 创建浮点数值
    pub fn float(v: f64) -> Self {
        Value::Float(v)
    }

    /// 创建字符串值
    pub fn string(v: impl Into<String>) -> Self {
        Value::String(v.into())
    }

    /// 创建字节数组值
    pub fn bytes(v: Vec<u8>) -> Self {
        Value::Bytes(v)
    }

    /// 创建数组值
    pub fn array(v: Vec<Value>) -> Self {
        Value::Array(v)
    }

    /// 创建对象值
    pub fn object(v: HashMap<String, Value>) -> Self {
        Value::Object(v)
    }

    /// 检查是否为空值
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// 检查是否为布尔值
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// 检查是否为整数
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    /// 检查是否为浮点数
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// 检查是否为数值 (整数或浮点)
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Float(_))
    }

    /// 检查是否为字符串
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// 检查是否为数组
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// 检查是否为对象
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// 获取布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// 获取整数值
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(v) => Some(*v),
            _ => None,
        }
    }

    /// 获取浮点数值
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(v) => Some(*v),
            Value::Integer(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// 获取字符串引用
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    /// 获取数组引用
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    /// 获取可变数组引用
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    /// 获取对象引用
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(v) => Some(v),
            _ => None,
        }
    }

    /// 获取可变对象引用
    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, Value>> {
        match self {
            Value::Object(v) => Some(v),
            _ => None,
        }
    }

    /// 从对象中获取字段值
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// 从数组中获取索引值
    pub fn get_index(&self, index: usize) -> Option<&Value> {
        match self {
            Value::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    /// 转换为 JSON 字符串
    pub fn to_json_string(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Integer(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::String(v) => format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\"")),
            Value::Bytes(v) => {
                let encoded = base64_encode(v);
                format!("\"{}\"", encoded)
            }
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_json_string()).collect();
                format!("[{}]", items.join(","))
            }
            Value::Object(map) => {
                let items: Vec<String> = map.iter().map(|(k, v)| format!("\"{}\":{}", k, v.to_json_string())).collect();
                format!("{{{}}}", items.join(","))
            }
        }
    }

    /// 从 JSON 字符串解析
    pub fn from_json_str(s: &str) -> Result<Value, WaeError> {
        parse_json_value(s.trim())
    }

    /// 深度克隆
    pub fn deep_clone(&self) -> Value {
        self.clone()
    }

    /// 合并两个值 (用于配置合并)
    pub fn merge(&mut self, other: Value) {
        match (self, other) {
            (Value::Object(a), Value::Object(b)) => {
                for (k, v) in b {
                    if let Some(existing) = a.get_mut(&k) {
                        existing.merge(v);
                    }
                    else {
                        a.insert(k, v);
                    }
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                a.extend(b);
            }
            (self_val, other_val) => {
                *self_val = other_val;
            }
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Integer(v)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(v: HashMap<String, Value>) -> Self {
        Value::Object(v)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null => serializer.serialize_none(),
            Value::Bool(v) => serializer.serialize_bool(*v),
            Value::Integer(v) => serializer.serialize_i64(*v),
            Value::Float(v) => serializer.serialize_f64(*v),
            Value::String(v) => serializer.serialize_str(v),
            Value::Bytes(v) => serializer.serialize_bytes(v),
            Value::Array(v) => v.serialize(serializer),
            Value::Object(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a valid value")
            }

            fn visit_none<E>(self) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Null)
            }

            fn visit_unit<E>(self) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Null)
            }

            fn visit_bool<E>(self, v: bool) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(v as i64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Float(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v.to_string()))
            }

            fn visit_string<E>(self, v: String) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Bytes(v.to_vec()))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Bytes(v))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut arr = Vec::new();
                while let Some(v) = seq.next_element()? {
                    arr.push(v);
                }
                Ok(Value::Array(arr))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut obj = HashMap::new();
                while let Some((k, v)) = map.next_entry()? {
                    obj.insert(k, v);
                }
                Ok(Value::Object(obj))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

/// JSON 解析器
fn parse_json_value(s: &str) -> Result<Value, WaeError> {
    let (value, remaining) = parse_value(s.trim())?;
    if !remaining.trim().is_empty() {
        return Err(WaeError::parse_error("json", format!("Unexpected characters after JSON: {}", remaining)));
    }
    Ok(value)
}

fn parse_value(s: &str) -> Result<(Value, &str), WaeError> {
    let s = s.trim_start();

    if s.is_empty() {
        return Err(WaeError::parse_error("json", "Unexpected end of input"));
    }

    if let Some(stripped) = s.strip_prefix("null") {
        return Ok((Value::Null, stripped));
    }

    if let Some(stripped) = s.strip_prefix("true") {
        return Ok((Value::Bool(true), stripped));
    }

    if let Some(stripped) = s.strip_prefix("false") {
        return Ok((Value::Bool(false), stripped));
    }

    if s.starts_with('"') {
        return parse_string(s);
    }

    if s.starts_with('[') {
        return parse_array(s);
    }

    if s.starts_with('{') {
        return parse_object(s);
    }

    if s.starts_with('-') || s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return parse_number(s);
    }

    Err(WaeError::parse_error("json", format!("valid JSON character, got '{}'", s.chars().next().unwrap_or(' '))))
}

fn parse_number(s: &str) -> Result<(Value, &str), WaeError> {
    let mut i = 0;
    let mut has_dot = false;
    let mut has_exp = false;

    if s.starts_with('-') {
        i += 1;
    }

    while i < s.len() {
        let c = s.chars().nth(i).unwrap();
        if c.is_ascii_digit() {
            i += 1;
        }
        else if c == '.' && !has_dot && !has_exp {
            has_dot = true;
            i += 1;
        }
        else if (c == 'e' || c == 'E') && !has_exp {
            has_exp = true;
            i += 1;
            if i < s.len() && (s.chars().nth(i).unwrap() == '+' || s.chars().nth(i).unwrap() == '-') {
                i += 1;
            }
        }
        else {
            break;
        }
    }

    let num_str = &s[..i];
    if has_dot || has_exp {
        let v: f64 = num_str.parse().map_err(|e| WaeError::parse_error("number", format!("valid float: {}", e)))?;
        Ok((Value::Float(v), &s[i..]))
    }
    else {
        let v: i64 = num_str.parse().map_err(|e| WaeError::parse_error("number", format!("valid integer: {}", e)))?;
        Ok((Value::Integer(v), &s[i..]))
    }
}

fn parse_string(s: &str) -> Result<(Value, &str), WaeError> {
    if !s.starts_with('"') {
        return Err(WaeError::parse_error("string", "expected '\"'"));
    }

    let mut result = String::new();
    let mut i = 1;
    let chars: Vec<char> = s.chars().collect();

    while i < chars.len() {
        let c = chars[i];
        if c == '"' {
            return Ok((Value::String(result), &s[i + 1..]));
        }
        if c == '\\' {
            i += 1;
            if i >= chars.len() {
                return Err(WaeError::parse_error("string", "Unexpected end in escape"));
            }
            let escaped = chars[i];
            match escaped {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                '/' => result.push('/'),
                'u' => {
                    if i + 4 >= chars.len() {
                        return Err(WaeError::parse_error("string", "Invalid unicode escape"));
                    }
                    let hex: String = chars[i + 1..i + 5].iter().collect();
                    let code = u16::from_str_radix(&hex, 16)
                        .map_err(|e| WaeError::parse_error("string", format!("Invalid unicode: {}", e)))?;
                    if let Some(c) = char::from_u32(code as u32) {
                        result.push(c);
                    }
                    i += 4;
                }
                _ => result.push(escaped),
            }
        }
        else {
            result.push(c);
        }
        i += 1;
    }

    Err(WaeError::parse_error("string", "Unterminated string"))
}

fn parse_array(s: &str) -> Result<(Value, &str), WaeError> {
    if !s.starts_with('[') {
        return Err(WaeError::parse_error("array", "expected '['"));
    }

    let mut result = Vec::new();
    let mut s = &s[1..];

    s = s.trim_start();
    if let Some(stripped) = s.strip_prefix(']') {
        return Ok((Value::Array(result), stripped));
    }

    loop {
        let (value, remaining) = parse_value(s)?;
        result.push(value);
        s = remaining.trim_start();

        if let Some(stripped) = s.strip_prefix(']') {
            return Ok((Value::Array(result), stripped));
        }

        if !s.starts_with(',') {
            return Err(WaeError::parse_error("array", "expected ',' or ']'"));
        }
        s = s[1..].trim_start();
    }
}

fn parse_object(s: &str) -> Result<(Value, &str), WaeError> {
    if !s.starts_with('{') {
        return Err(WaeError::parse_error("object", "expected '{'"));
    }

    let mut result = HashMap::new();
    let mut s = &s[1..];

    s = s.trim_start();
    if let Some(stripped) = s.strip_prefix('}') {
        return Ok((Value::Object(result), stripped));
    }

    loop {
        s = s.trim_start();
        let (key, remaining) = parse_string(s)?;
        let key = key.as_str().ok_or_else(|| WaeError::parse_error("object_key", "Object key must be a string"))?.to_string();
        s = remaining.trim_start();

        if !s.starts_with(':') {
            return Err(WaeError::parse_error("object", "expected ':'"));
        }
        s = s[1..].trim_start();

        let (value, remaining) = parse_value(s)?;
        result.insert(key, value);
        s = remaining.trim_start();

        if let Some(stripped) = s.strip_prefix('}') {
            return Ok((Value::Object(result), stripped));
        }

        if !s.starts_with(',') {
            return Err(WaeError::parse_error("object", "expected ',' or '}'"));
        }
        s = s[1..].trim_start();
    }
}

/// Base64 编码
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let a = data[i] as usize;
        let b = if i + 1 < data.len() { data[i + 1] as usize } else { 0 };
        let c = if i + 2 < data.len() { data[i + 2] as usize } else { 0 };

        result.push(ALPHABET[a >> 2] as char);
        result.push(ALPHABET[((a & 0x03) << 4) | (b >> 4)] as char);

        if i + 1 < data.len() {
            result.push(ALPHABET[((b & 0x0f) << 2) | (c >> 6)] as char);
        }
        else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(ALPHABET[c & 0x3f] as char);
        }
        else {
            result.push('=');
        }

        i += 3;
    }

    result
}
