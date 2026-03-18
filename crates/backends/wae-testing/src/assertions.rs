//! 断言宏和工具模块

use std::time::Duration;
use wae_types::{WaeError, WaeErrorKind, WaeResult as TestingResult};

/// 断言 Result 是 Ok
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("{}: Expected Ok, got Err: {:?}", $msg, e),
        }
    };
}

/// 断言 Result 是 Err
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Err(e) => e,
            Ok(v) => panic!("Expected Err, got Ok: {:?}", v),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Err(e) => e,
            Ok(v) => panic!("{}: Expected Err, got Ok: {:?}", $msg, v),
        }
    };
}

/// 断言 Option 是 Some
#[macro_export]
macro_rules! assert_some {
    ($expr:expr) => {
        match $expr {
            Some(v) => v,
            None => panic!("Expected Some, got None"),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Some(v) => v,
            None => panic!("{}: Expected Some, got None", $msg),
        }
    };
}

/// 断言 Option 是 None
#[macro_export]
macro_rules! assert_none {
    ($expr:expr) => {
        match $expr {
            None => (),
            Some(v) => panic!("Expected None, got Some: {:?}", v),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            None => (),
            Some(v) => panic!("{}: Expected None, got Some: {:?}", $msg, v),
        }
    };
}

/// 断言集合包含元素
#[macro_export]
macro_rules! assert_contains {
    ($container:expr, $item:expr) => {
        if !$container.contains(&$item) {
            panic!("Expected container to contain item: {:?}", $item);
        }
    };
    ($container:expr, $item:expr, $msg:expr) => {
        if !$container.contains(&$item) {
            panic!("{}: Expected container to contain item: {:?}", $msg, $item);
        }
    };
}

/// 断言集合不包含元素
#[macro_export]
macro_rules! assert_not_contains {
    ($container:expr, $item:expr) => {
        if $container.contains(&$item) {
            panic!("Expected container to NOT contain item: {:?}", $item);
        }
    };
    ($container:expr, $item:expr, $msg:expr) => {
        if $container.contains(&$item) {
            panic!("{}: Expected container to NOT contain item: {:?}", $msg, $item);
        }
    };
}

/// 断言两个值近似相等 (用于浮点数比较)
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let left = $left;
        let right = $right;
        let diff = if left > right { left - right } else { right - left };
        if diff > $epsilon {
            panic!("Values are not approximately equal: left={}, right={}, diff={}", left, right, diff);
        }
    };
}

/// 异步断言工具
pub struct AsyncAssert;

impl AsyncAssert {
    /// 等待条件满足
    pub async fn eventually<F, Fut>(condition: F, timeout_duration: Duration, check_interval: Duration) -> TestingResult<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout_duration {
            if condition().await {
                return Ok(());
            }
            tokio::time::sleep(check_interval).await;
        }

        Err(WaeError::new(WaeErrorKind::OperationTimeout {
            operation: "eventually".to_string(),
            timeout_ms: timeout_duration.as_millis() as u64,
        }))
    }

    /// 等待条件满足并返回值
    pub async fn eventually_with_value<F, Fut, T>(
        condition: F,
        timeout_duration: Duration,
        check_interval: Duration,
    ) -> TestingResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Option<T>>,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout_duration {
            if let Some(value) = condition().await {
                return Ok(value);
            }
            tokio::time::sleep(check_interval).await;
        }

        Err(WaeError::new(WaeErrorKind::OperationTimeout {
            operation: "eventually_with_value".to_string(),
            timeout_ms: timeout_duration.as_millis() as u64,
        }))
    }
}

/// 断言异步条件最终满足
pub async fn assert_eventually<F, Fut>(condition: F, timeout_duration: Duration, check_interval: Duration) -> TestingResult<()>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    AsyncAssert::eventually(condition, timeout_duration, check_interval).await
}

/// 断言两个字符串匹配正则表达式
pub fn assert_matches_regex(text: &str, pattern: &str) -> TestingResult<()> {
    let re = regex::Regex::new(pattern)
        .map_err(|e| WaeError::new(WaeErrorKind::AssertionFailed { message: format!("Invalid regex: {}", e) }))?;

    if !re.is_match(text) {
        return Err(WaeError::new(WaeErrorKind::AssertionFailed {
            message: format!("Text '{}' does not match pattern '{}'", text, pattern),
        }));
    }

    Ok(())
}

/// 断言 JSON 值包含指定字段
pub fn assert_json_contains(json: &serde_json::Value, path: &str) -> TestingResult<()> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for part in parts {
        if let serde_json::Value::Object(map) = current {
            current = map.get(part).ok_or_else(|| {
                WaeError::new(WaeErrorKind::AssertionFailed { message: format!("JSON path '{}' not found", path) })
            })?;
        }
        else {
            return Err(WaeError::new(WaeErrorKind::AssertionFailed { message: format!("JSON path '{}' not found", path) }));
        }
    }

    Ok(())
}
