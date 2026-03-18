//! SQL 查询宏实现
//!
//! 提供类似 sqlx 风格的查询宏，但底层使用 wae-database。

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, Lit, parse_macro_input};

/// 解析宏输入参数
struct QueryInput {
    conn: syn::Expr,
    sql: String,
    params: Vec<syn::Expr>,
}

/// 解析带类型参数的宏输入 (query_as, query_scalar)
struct TypedQueryInput {
    result_type: syn::Type,
    conn: syn::Expr,
    sql: String,
    params: Vec<syn::Expr>,
}

mod parsing {
    use syn::{
        Expr, Lit, Token, Type,
        parse::{Parse, ParseStream},
    };

    use super::{QueryInput, TypedQueryInput};

    impl Parse for QueryInput {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let conn: Expr = input.parse()?;
            input.parse::<Token![,]>()?;

            let sql_lit: Lit = input.parse()?;
            let sql = match sql_lit {
                Lit::Str(s) => s.value(),
                _ => return Err(syn::Error::new(input.span(), "SQL must be a string literal")),
            };

            let mut params = Vec::new();
            while input.parse::<Token![,]>().is_ok() {
                if input.is_empty() {
                    break;
                }
                params.push(input.parse()?);
            }

            Ok(QueryInput { conn, sql, params })
        }
    }

    impl Parse for TypedQueryInput {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let result_type: Type = input.parse()?;
            input.parse::<Token![,]>()?;

            let conn: Expr = input.parse()?;
            input.parse::<Token![,]>()?;

            let sql_lit: Lit = input.parse()?;
            let sql = match sql_lit {
                Lit::Str(s) => s.value(),
                _ => return Err(syn::Error::new(input.span(), "SQL must be a string literal")),
            };

            let mut params = Vec::new();
            while input.parse::<Token![,]>().is_ok() {
                if input.is_empty() {
                    break;
                }
                params.push(input.parse()?);
            }

            Ok(TypedQueryInput { result_type, conn, sql, params })
        }
    }
}

/// 将表达式转换为 wae_types::Value
fn expr_to_value(expr: &syn::Expr) -> TokenStream2 {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(i) => {
                let value = i.base10_parse::<i64>().unwrap_or(0);
                quote! { wae_types::Value::Integer(#value) }
            }
            Lit::Float(f) => {
                let value: f64 = f.base10_parse().unwrap_or(0.0);
                quote! { wae_types::Value::Float(#value) }
            }
            Lit::Bool(b) => {
                let value = b.value;
                quote! { wae_types::Value::Bool(#value) }
            }
            Lit::Str(s) => {
                let value = s.value();
                quote! { wae_types::Value::String(#value.to_string()) }
            }
            Lit::ByteStr(b) => {
                let bytes = b.value();
                quote! { wae_types::Value::Bytes(vec![#(#bytes),*]) }
            }
            _ => quote! { wae_types::Value::Null },
        },
        Expr::Path(_) => {
            quote! { <_ as Into<wae_types::Value>>::into(#expr) }
        }
        _ => {
            quote! { <_ as Into<wae_types::Value>>::into(#expr) }
        }
    }
}

/// 展开参数列表
fn expand_params(params: &[syn::Expr]) -> TokenStream2 {
    if params.is_empty() {
        return quote! { vec![] };
    }

    let values: Vec<TokenStream2> = params.iter().map(expr_to_value).collect();
    quote! { vec![#(#values),*] }
}

/// 展开 query! 宏
pub fn expand_query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryInput);
    let conn = &input.conn;
    let sql = &input.sql;
    let params = expand_params(&input.params);

    let expanded = quote! {
        async {
            let conn = &#conn;
            conn.query_with(#sql, #params).await
        }
    };

    TokenStream::from(expanded)
}

/// 展开 query_as! 宏
pub fn expand_query_as(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TypedQueryInput);
    let result_type = &input.result_type;
    let conn = &input.conn;
    let sql = &input.sql;
    let params = expand_params(&input.params);

    let expanded = quote! {
        async {
            let conn = &#conn;
            let mut rows = conn.query_with(#sql, #params).await?;
            let mut results = Vec::new();
            while let Some(row) = rows.next().await? {
                results.push(<#result_type as wae_database::FromRow>::from_row(&row)?);
            }
            Ok::<Vec<#result_type>, wae_database::DatabaseError>(results)
        }
    };

    TokenStream::from(expanded)
}

/// 展开 execute! 宏
pub fn expand_execute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryInput);
    let conn = &input.conn;
    let sql = &input.sql;
    let params = expand_params(&input.params);

    let expanded = quote! {
        async {
            let conn = &#conn;
            conn.execute_with(#sql, #params).await
        }
    };

    TokenStream::from(expanded)
}

/// 展开 query_scalar! 宏
pub fn expand_query_scalar(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TypedQueryInput);
    let result_type = &input.result_type;
    let conn = &input.conn;
    let sql = &input.sql;
    let params = expand_params(&input.params);

    let expanded = quote! {
        async {
            let conn = &#conn;
            let mut rows = conn.query_with(#sql, #params).await?;
            match rows.next().await? {
                Some(row) => {
                    let value = row.get::<#result_type>(0)?;
                    Ok::<#result_type, wae_database::DatabaseError>(value)
                }
                None => Err(wae_database::DatabaseError::NotFound("No result returned".to_string())),
            }
        }
    };

    TokenStream::from(expanded)
}
