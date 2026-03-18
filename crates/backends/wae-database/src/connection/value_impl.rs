//! 从数据库值转换的 trait 实现

use crate::connection::config::DatabaseResult;
use wae_types::WaeError;

/// 从数据库值转换的 trait
pub trait FromDatabaseValue: Sized {
    /// 从 Limbo 数据库值转换
    #[cfg(feature = "limbo")]
    fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self>;

    /// 从 PostgreSQL 行获取值
    #[cfg(feature = "postgres")]
    fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self>;

    /// 从 MySQL 行获取值
    #[cfg(feature = "mysql")]
    fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self>;
}

#[cfg(all(feature = "limbo", not(feature = "postgres"), not(feature = "mysql")))]
mod database_value_impl {
    use super::*;

    macro_rules! impl_from_value {
        ($type:ty, $pattern:pat => $expr:expr) => {
            impl FromDatabaseValue for $type {
                fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
                    match value {
                        $pattern => Ok($expr),
                        _ => Err(WaeError::invalid_format(stringify!($type), format!("{:?}", value))),
                    }
                }
            }
        };
    }

    impl_from_value!(i64, limbo::Value::Integer(i) => i);
    impl_from_value!(i32, limbo::Value::Integer(i) => i as i32);
    impl_from_value!(i16, limbo::Value::Integer(i) => i as i16);
    impl_from_value!(u64, limbo::Value::Integer(i) => i as u64);
    impl_from_value!(u32, limbo::Value::Integer(i) => i as u32);
    impl_from_value!(Vec<u8>, limbo::Value::Blob(b) => b);

    impl FromDatabaseValue for f64 {
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Integer(i) => Ok(i as f64),
                limbo::Value::Text(s) => {
                    s.parse().map_err(|_| WaeError::invalid_format("f64", format!("Cannot parse '{}' as float", s)))
                }
                _ => Err(WaeError::invalid_format("f64", format!("{:?}", value))),
            }
        }
    }

    impl FromDatabaseValue for String {
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Text(s) => Ok(s),
                limbo::Value::Integer(i) => Ok(i.to_string()),
                _ => Err(WaeError::invalid_format("String", format!("{:?}", value))),
            }
        }
    }

    impl FromDatabaseValue for bool {
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Integer(i) => Ok(i != 0),
                _ => Err(WaeError::invalid_format("bool", format!("{:?}", value))),
            }
        }
    }

    impl<T: FromDatabaseValue> FromDatabaseValue for Option<T> {
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => T::from_limbo_value(v).map(Some),
            }
        }
    }
}

#[cfg(all(feature = "postgres", not(feature = "limbo"), not(feature = "mysql")))]
mod database_value_impl {
    use super::*;

    macro_rules! impl_from_value {
        ($type:ty) => {
            impl FromDatabaseValue for $type {
                fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
                    row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
                }
            }
        };
    }

    impl_from_value!(i64);
    impl_from_value!(i32);
    impl_from_value!(i16);
    impl_from_value!(f64);
    impl_from_value!(String);
    impl_from_value!(Vec<u8>);
    impl_from_value!(bool);

    impl FromDatabaseValue for u64 {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            let i: i64 =
                row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))?;
            Ok(i as u64)
        }
    }

    impl FromDatabaseValue for u32 {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            let i: i32 =
                row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))?;
            Ok(i as u32)
        }
    }

    impl FromDatabaseValue for Option<i64> {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }
    }

    impl FromDatabaseValue for Option<i32> {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }
    }

    impl FromDatabaseValue for Option<i16> {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }
    }

    impl FromDatabaseValue for Option<u64> {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            let opt: Option<i64> =
                row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))?;
            Ok(opt.map(|i| i as u64))
        }
    }

    impl FromDatabaseValue for Option<u32> {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            let opt: Option<i32> =
                row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))?;
            Ok(opt.map(|i| i as u32))
        }
    }

    impl FromDatabaseValue for Option<f64> {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }
    }

    impl FromDatabaseValue for Option<String> {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }
    }

    impl FromDatabaseValue for Option<Vec<u8>> {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }
    }

    impl FromDatabaseValue for Option<bool> {
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }
    }
}

#[cfg(all(feature = "mysql", not(feature = "limbo"), not(feature = "postgres")))]
mod database_value_impl {
    use super::*;
    use mysql_async::prelude::FromValue;

    macro_rules! impl_from_value {
        ($type:ty) => {
            impl FromDatabaseValue for $type {
                fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
                    row.get_opt(index)
                        .transpose()
                        .map_err(|e| WaeError::internal(format!("Failed to get column {}: {:?}", index, e)))?
                        .ok_or_else(|| WaeError::internal(format!("Column {} is NULL", index)))
                }
            }
        };
    }

    impl_from_value!(i64);
    impl_from_value!(i32);
    impl_from_value!(i16);
    impl_from_value!(f64);
    impl_from_value!(String);
    impl_from_value!(Vec<u8>);
    impl_from_value!(bool);

    impl FromDatabaseValue for u64 {
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            let i: i64 = row
                .get_opt(index)
                .transpose()
                .map_err(|e| WaeError::internal(format!("Failed to get column {}: {:?}", index, e)))?
                .ok_or_else(|| WaeError::internal(format!("Column {} is NULL", index)))?;
            Ok(i as u64)
        }
    }

    impl FromDatabaseValue for u32 {
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            let i: i32 = row
                .get_opt(index)
                .transpose()
                .map_err(|e| WaeError::internal(format!("Failed to get column {}: {:?}", index, e)))?
                .ok_or_else(|| WaeError::internal(format!("Column {} is NULL", index)))?;
            Ok(i as u32)
        }
    }

    impl<T: FromDatabaseValue + FromValue> FromDatabaseValue for Option<T> {
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<T>, usize>(index) {
                Some(Ok(val)) => Ok(val),
                Some(Err(e)) => Err(WaeError::internal(format!("Failed to get column {}: {:?}", index, e))),
                None => Ok(None),
            }
        }
    }
}

#[cfg(any(
    all(feature = "limbo", feature = "postgres"),
    all(feature = "limbo", feature = "mysql"),
    all(feature = "postgres", feature = "mysql"),
    all(feature = "limbo", feature = "postgres", feature = "mysql")
))]
mod database_value_impl {
    use super::*;

    macro_rules! impl_from_value_multi {
        ($type:ty) => {
            impl FromDatabaseValue for $type {
                #[cfg(feature = "limbo")]
                fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
                    match value {
                        limbo::Value::Integer(i) => Ok(i as $type),
                        limbo::Value::Text(s) => s.parse().map_err(|_| {
                            WaeError::invalid_format(stringify!($type), format!("Cannot parse as {}", stringify!($type)))
                        }),
                        _ => Err(WaeError::invalid_format(stringify!($type), format!("{:?}", value))),
                    }
                }

                #[cfg(feature = "postgres")]
                fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
                    row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
                }

                #[cfg(feature = "mysql")]
                fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
                    row.get_opt(index)
                        .transpose()
                        .map_err(|e| WaeError::internal(format!("Failed to get column {}: {:?}", index, e)))?
                        .ok_or_else(|| WaeError::internal(format!("Column {} is NULL", index)))
                }
            }
        };
    }

    impl_from_value_multi!(i64);
    impl_from_value_multi!(i32);
    impl_from_value_multi!(i16);
    impl_from_value_multi!(f64);

    impl FromDatabaseValue for String {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Text(s) => Ok(s),
                limbo::Value::Integer(i) => Ok(i.to_string()),
                _ => Err(WaeError::invalid_format("String", format!("{:?}", value))),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            row.get_opt(index)
                .transpose()
                .map_err(|e| WaeError::internal(format!("Failed to get column {}: {:?}", index, e)))?
                .ok_or_else(|| WaeError::internal(format!("Column {} is NULL", index)))
        }
    }

    impl FromDatabaseValue for Vec<u8> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Blob(b) => Ok(b),
                _ => Err(WaeError::invalid_format("Vec<u8>", format!("{:?}", value))),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            row.get_opt(index)
                .transpose()
                .map_err(|e| WaeError::internal(format!("Failed to get column {}: {:?}", index, e)))?
                .ok_or_else(|| WaeError::internal(format!("Column {} is NULL", index)))
        }
    }

    impl FromDatabaseValue for bool {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Integer(i) => Ok(i != 0),
                _ => Err(WaeError::invalid_format("bool", format!("{:?}", value))),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            row.get_opt(index)
                .transpose()
                .map_err(|e| WaeError::internal(format!("Failed to get column {}: {:?}", index, e)))?
                .ok_or_else(|| WaeError::internal(format!("Column {} is NULL", index)))
        }
    }

    impl FromDatabaseValue for u64 {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Integer(i) => Ok(i as u64),
                _ => Err(WaeError::invalid_format("u64", format!("{:?}", value))),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            let i: i64 =
                row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))?;
            Ok(i as u64)
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            let i: i64 = row
                .get_opt(index)
                .transpose()
                .map_err(|e| WaeError::internal(format!("Failed to get column {}: {:?}", index, e)))?
                .ok_or_else(|| WaeError::internal(format!("Column {} is NULL", index)))?;
            Ok(i as u64)
        }
    }

    impl FromDatabaseValue for u32 {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Integer(i) => Ok(i as u32),
                _ => Err(WaeError::invalid_format("u32", format!("{:?}", value))),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            let i: i32 =
                row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))?;
            Ok(i as u32)
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            let i: i32 = row
                .get_opt(index)
                .transpose()
                .map_err(|e| WaeError::internal(format!("Failed to get column {}: {:?}", index, e)))?
                .ok_or_else(|| WaeError::internal(format!("Column {} is NULL", index)))?;
            Ok(i as u32)
        }
    }

    impl FromDatabaseValue for Option<i64> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => i64::from_limbo_value(v).map(Some),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<i64>, usize>(index) {
                Some(Ok(val)) => Ok(val),
                Some(Err(_)) => Ok(None),
                None => Ok(None),
            }
        }
    }

    impl FromDatabaseValue for Option<i32> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => i32::from_limbo_value(v).map(Some),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<i32>, usize>(index) {
                Some(Ok(val)) => Ok(val),
                Some(Err(_)) => Ok(None),
                None => Ok(None),
            }
        }
    }

    impl FromDatabaseValue for Option<i16> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => i16::from_limbo_value(v).map(Some),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<i16>, usize>(index) {
                Some(Ok(val)) => Ok(val),
                Some(Err(_)) => Ok(None),
                None => Ok(None),
            }
        }
    }

    impl FromDatabaseValue for Option<u64> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => u64::from_limbo_value(v).map(Some),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            let opt: Option<i64> =
                row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))?;
            Ok(opt.map(|i| i as u64))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<i64>, usize>(index) {
                Some(Ok(val)) => Ok(val.map(|i| i as u64)),
                Some(Err(_)) => Ok(None),
                None => Ok(None),
            }
        }
    }

    impl FromDatabaseValue for Option<u32> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => u32::from_limbo_value(v).map(Some),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            let opt: Option<i32> =
                row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))?;
            Ok(opt.map(|i| i as u32))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<i32>, usize>(index) {
                Some(Ok(val)) => Ok(val.map(|i| i as u32)),
                Some(Err(_)) => Ok(None),
                None => Ok(None),
            }
        }
    }

    impl FromDatabaseValue for Option<f64> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => f64::from_limbo_value(v).map(Some),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<f64>, usize>(index) {
                Some(Ok(val)) => Ok(val),
                Some(Err(_)) => Ok(None),
                None => Ok(None),
            }
        }
    }

    impl FromDatabaseValue for Option<String> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => String::from_limbo_value(v).map(Some),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<String>, usize>(index) {
                Some(Ok(val)) => Ok(val),
                Some(Err(_)) => Ok(None),
                None => Ok(None),
            }
        }
    }

    impl FromDatabaseValue for Option<Vec<u8>> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => Vec::<u8>::from_limbo_value(v).map(Some),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<Vec<u8>>, usize>(index) {
                Some(Ok(val)) => Ok(val),
                Some(Err(_)) => Ok(None),
                None => Ok(None),
            }
        }
    }

    impl FromDatabaseValue for Option<bool> {
        #[cfg(feature = "limbo")]
        fn from_limbo_value(value: limbo::Value) -> DatabaseResult<Self> {
            match value {
                limbo::Value::Null => Ok(None),
                v => bool::from_limbo_value(v).map(Some),
            }
        }

        #[cfg(feature = "postgres")]
        fn from_postgres_row(row: &tokio_postgres::Row, index: usize) -> DatabaseResult<Self> {
            row.try_get(index).map_err(|e| WaeError::internal(format!("Failed to get column {}: {}", index, e)))
        }

        #[cfg(feature = "mysql")]
        fn from_mysql_row(row: &mysql_async::Row, index: usize) -> DatabaseResult<Self> {
            match row.get_opt::<Option<bool>, usize>(index) {
                Some(Ok(val)) => Ok(val),
                Some(Err(_)) => Ok(None),
                None => Ok(None),
            }
        }
    }
}
