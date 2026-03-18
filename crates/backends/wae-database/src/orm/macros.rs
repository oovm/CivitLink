//! 宏定义模块
//!
//! 提供实体字段定义和自动实现 trait 的宏

/// 实体字段定义宏辅助
#[macro_export]
macro_rules! entity_fields {
    ($($name:ident: $type:ty),* $(,)?) => {
        $(pub $name: $type,)*
    };
}

/// 自动实现 ToRow 的宏
#[macro_export]
macro_rules! impl_to_row {
    ($type:ty { $($col:literal => $field:ident),* $(,)? }) => {
        impl ToRow for $type {
            fn to_row(&self) -> Vec<(&'static str, wae_types::Value)> {
                vec![
                    $((col, self.$field.clone().into()),)*
                ]
            }
        }
    };
}

/// 自动实现 FromRow 的宏
#[macro_export]
macro_rules! impl_from_row {
    ($type:ty { $($idx:literal => $field:ident: $ftype:ty),* $(,)? }) => {
        impl FromRow for $type {
            fn from_row(row: &$crate::DatabaseRow) -> $crate::DatabaseResult<Self> {
                Ok(Self {
                    $($field: row.get::<$ftype>($idx)?,)*
                })
            }
        }
    };
}
