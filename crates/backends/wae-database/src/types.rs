//! 类型转换模块

#[cfg(feature = "limbo")]
use limbo::Value as LimboValue;
#[cfg(any(feature = "limbo", feature = "mysql"))]
use wae_types::Value;

#[cfg(feature = "limbo")]
/// 将 wae_types::Value 转换为 limbo::Value
pub fn from_wae_values(values: Vec<Value>) -> Vec<LimboValue> {
    values.into_iter().map(from_wae_value).collect()
}

#[cfg(feature = "limbo")]
/// 将单个 wae_types::Value 转换为 limbo::Value
pub(crate) fn from_wae_value(value: Value) -> LimboValue {
    match value {
        Value::Null => LimboValue::Null,
        Value::Bool(b) => LimboValue::Integer(if b { 1 } else { 0 }),
        Value::Integer(i) => LimboValue::Integer(i),
        Value::Float(f) => LimboValue::Text(f.to_string()),
        Value::String(s) => LimboValue::Text(s),
        Value::Bytes(b) => LimboValue::Blob(b),
        Value::Array(l) => LimboValue::Text(serde_json::to_string(&l).unwrap_or_default()),
        Value::Object(m) => LimboValue::Text(serde_json::to_string(&m).unwrap_or_default()),
    }
}

#[cfg(feature = "mysql")]
use mysql_async::Value as MySqlValue;

#[cfg(feature = "mysql")]
/// 将 wae_types::Value 列表转换为 MySQL 参数
pub fn to_mysql_params(values: Vec<Value>) -> Vec<MySqlValue> {
    values.into_iter().map(from_wae_to_mysql).collect()
}

#[cfg(feature = "mysql")]
/// 将单个 wae_types::Value 转换为 MySQL Value
pub(crate) fn from_wae_to_mysql(value: Value) -> MySqlValue {
    match value {
        Value::Null => MySqlValue::NULL,
        Value::Bool(b) => MySqlValue::Int(if b { 1 } else { 0 }),
        Value::Integer(i) => MySqlValue::Int(i),
        Value::Float(f) => MySqlValue::Double(f),
        Value::String(s) => MySqlValue::Bytes(s.into_bytes()),
        Value::Bytes(b) => MySqlValue::Bytes(b),
        Value::Array(l) => MySqlValue::Bytes(serde_json::to_string(&l).unwrap_or_default().into_bytes()),
        Value::Object(m) => MySqlValue::Bytes(serde_json::to_string(&m).unwrap_or_default().into_bytes()),
    }
}

#[cfg(feature = "mysql")]
/// 将 MySQL Value 转换为 wae_types::Value
pub fn mysql_value_to_wae(value: MySqlValue) -> Value {
    match value {
        MySqlValue::NULL => Value::Null,
        MySqlValue::Bytes(b) => String::from_utf8(b.clone())
            .map(Value::String)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&b).into_owned())),
        MySqlValue::Int(i) => Value::Integer(i),
        MySqlValue::UInt(u) => Value::Integer(u as i64),
        MySqlValue::Float(f) => Value::Float(f as f64),
        MySqlValue::Double(d) => Value::Float(d),
        MySqlValue::Date(y, m, d, h, min, s, us) => {
            Value::String(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}", y, m, d, h, min, s, us))
        }
        MySqlValue::Time(neg, d, h, m, s, us) => {
            let sign = if neg { "-" } else { "" };
            Value::String(format!("{}{} {:02}:{:02}:{:02}.{:06}", sign, d, h, m, s, us))
        }
    }
}

#[cfg(feature = "postgres")]
use tokio_postgres::types::ToSql;

#[cfg(feature = "postgres")]
#[allow(dead_code)]
pub fn to_postgres_params(values: Vec<Value>) -> Vec<Box<dyn ToSql + Sync>> {
    values.into_iter().map(from_wae_to_postgres).collect()
}

#[cfg(feature = "postgres")]
#[allow(dead_code)]
pub(crate) fn from_wae_to_postgres(value: Value) -> Box<dyn ToSql + Sync> {
    match value {
        Value::Null => Box::new(Option::<i32>::None),
        Value::Bool(b) => Box::new(b),
        Value::Integer(i) => Box::new(i),
        Value::Float(f) => Box::new(f),
        Value::String(s) => Box::new(s),
        Value::Bytes(b) => Box::new(b),
        Value::Array(l) => Box::new(serde_json::to_string(&l).unwrap_or_default()),
        Value::Object(m) => Box::new(serde_json::to_string(&m).unwrap_or_default()),
    }
}
