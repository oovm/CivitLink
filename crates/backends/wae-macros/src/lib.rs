//! WAE Macros - 过程宏模块
//!
//! 提供用于简化开发的过程宏：
//! - `#[derive(ToSchema)]` - 自动生成 Schema 定义
//! - `query!` - 编译时 SQL 查询宏
//! - `query_as!` - 编译时 SQL 查询宏（自动映射到结构体）

#![warn(missing_docs)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Fields, GenericArgument, Ident, Lit, Meta, PathArguments, Type, parse_macro_input};

/// 获取类型的 Schema 定义
///
/// 正确处理基础类型、`Vec<T>`、`Option<T>` 等泛型类型。
fn get_type_schema(ty: &Type) -> TokenStream2 {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last();
            if let Some(segment) = last_segment {
                let ident = &segment.ident;

                match &segment.arguments {
                    PathArguments::AngleBracketed(angle_bracketed) => {
                        if let Some(GenericArgument::Type(inner_ty)) = angle_bracketed.args.first() {
                            let inner_schema = get_type_schema(inner_ty);
                            if ident == "Vec" {
                                return quote! { wae_schema::Schema::array(#inner_schema) };
                            }
                            else if ident == "Option" {
                                return quote! { #inner_schema.nullable(true) };
                            }
                        }
                    }
                    _ => {}
                }

                let ident_str = ident.to_string();
                if ident_str == "String" || ident_str == "&str" {
                    return quote! { wae_schema::Schema::string() };
                }
                else if matches!(
                    ident_str.as_str(),
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                ) {
                    return quote! { wae_schema::Schema::integer() };
                }
                else if ident_str == "f32" || ident_str == "f64" {
                    return quote! { wae_schema::Schema::number() };
                }
                else if ident_str == "bool" {
                    return quote! { wae_schema::Schema::boolean() };
                }
            }
        }
        Type::Reference(type_ref) => {
            return get_type_schema(&type_ref.elem);
        }
        _ => {}
    }
    quote! { <#ty as wae_schema::ToSchema>::schema() }
}

/// 提取文档注释
///
/// 合并多行文档注释为单个字符串。
fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut docs = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(meta) = &attr.meta {
                if let Expr::Lit(expr_lit) = &meta.value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        let doc = lit_str.value().trim().to_string();
                        if !doc.is_empty() {
                            docs.push(doc);
                        }
                    }
                }
            }
        }
    }
    if docs.is_empty() { None } else { Some(docs.join("\n")) }
}

/// 检查类型是否为 Option 类型
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// 生成结构体的 Schema 实现
fn generate_struct_schema(name: &Ident, fields: &Fields, attrs: &[syn::Attribute]) -> TokenStream2 {
    let mut properties = Vec::new();
    let mut required_fields = Vec::new();
    let type_doc = extract_doc_comment(attrs);

    match fields {
        Fields::Named(fields_named) => {
            for field in &fields_named.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                let field_schema = get_type_schema(&field.ty);

                let doc_comment = extract_doc_comment(&field.attrs);
                let property = if let Some(doc) = doc_comment {
                    quote! {
                        .property(#field_name_str, #field_schema.description(#doc))
                    }
                }
                else {
                    quote! {
                        .property(#field_name_str, #field_schema)
                    }
                };
                properties.push(property);

                if !is_option_type(&field.ty) {
                    required_fields.push(field_name_str);
                }
            }
        }
        Fields::Unnamed(fields_unnamed) => {
            for (i, field) in fields_unnamed.unnamed.iter().enumerate() {
                let field_name_str = format!("field_{}", i);
                let field_schema = get_type_schema(&field.ty);
                let property = quote! {
                    .property(#field_name_str, #field_schema)
                };
                properties.push(property);
            }
        }
        Fields::Unit => {}
    }

    let required = if required_fields.is_empty() {
        quote! {}
    }
    else {
        quote! { .required(vec![#(#required_fields.to_string()),*]) }
    };

    let description = if let Some(doc) = type_doc {
        quote! { .description(#doc) }
    }
    else {
        quote! {}
    };

    quote! {
        impl wae_schema::ToSchema for #name {
            fn schema() -> wae_schema::Schema {
                wae_schema::Schema::object()
                    #description
                    #(#properties)*
                    #required
            }
        }
    }
}

/// 生成枚举的 Schema 实现
fn generate_enum_schema(name: &Ident, data: &Data, attrs: &[syn::Attribute]) -> TokenStream2 {
    let variants = match data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => return quote! {},
    };

    let mut enum_values = Vec::new();
    for variant in variants {
        let variant_name = variant.ident.to_string();
        enum_values.push(quote! { serde_json::Value::String(#variant_name.to_string()) });
    }

    let type_doc = extract_doc_comment(attrs);
    let description = if let Some(doc) = type_doc {
        quote! { .description(#doc) }
    }
    else {
        quote! {}
    };

    quote! {
        impl wae_schema::ToSchema for #name {
            fn schema() -> wae_schema::Schema {
                wae_schema::Schema::string()
                    #description
                    .enum_values(vec![#(#enum_values),*])
            }
        }
    }
}

/// 自动生成 Schema 的派生宏
///
/// 为结构体或枚举自动生成 `ToSchema` trait 实现。
///
/// # Example
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use wae_schema::{Schema, ToSchema};
///
/// /// 用户信息
/// #[derive(Debug, Serialize, Deserialize, ToSchema)]
/// pub struct User {
///     /// 用户 ID
///     pub id: u64,
///     /// 用户名
///     pub name: String,
///     /// 邮箱（可选）
///     pub email: Option<String>,
/// }
/// ```
#[proc_macro_derive(ToSchema)]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let attrs = &input.attrs;

    let expanded = match &input.data {
        Data::Struct(data_struct) => generate_struct_schema(name, &data_struct.fields, attrs),
        Data::Enum(_) => generate_enum_schema(name, &input.data, attrs),
        Data::Union(_) => {
            return syn::Error::new_spanned(name, "Unions are not supported").to_compile_error().into();
        }
    };

    TokenStream::from(expanded)
}

/// 生成 OpenAPI 路由文档的宏
///
/// # Example
///
/// ```rust,ignore
/// use wae_macros::api_doc;
///
/// let doc = api_doc! {
///     "/users" => {
///         GET => {
///             summary: "获取用户列表",
///             response: 200 => "成功",
///         },
///         POST => {
///             summary: "创建用户",
///             body: "User",
///             response: 201 => "创建成功",
///         },
///     },
/// };
/// ```
#[proc_macro]
pub fn api_doc(input: TokenStream) -> TokenStream {
    let _ = input;
    TokenStream::from(quote! {
        wae_schema::OpenApiDoc::new("API", "1.0.0")
    })
}

mod query;

/// SQL 查询宏 - 返回原始行数据
///
/// 执行 SQL 查询并返回 `DatabaseRows`，需要手动迭代处理结果。
///
/// # Example
///
/// ```rust,ignore
/// use wae_macros::query;
/// use wae_database::{DatabaseConnection, DatabaseRow};
///
/// async fn get_users(conn: &dyn DatabaseConnection) -> Result<Vec<DatabaseRow>, Box<dyn std::error::Error>> {
///     let mut rows = query!(conn, "SELECT id, name, email FROM users WHERE active = ?", true).await?;
///     let mut results = Vec::new();
///     while let Some(row) = rows.next().await? {
///         results.push(row);
///     }
///     Ok(results)
/// }
/// ```
#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    query::expand_query(input)
}

/// SQL 查询宏 - 自动映射到结构体
///
/// 执行 SQL 查询并自动将结果映射到指定结构体类型。
/// 结构体需要实现 `FromRow` trait。
///
/// # Example
///
/// ```rust,ignore
/// use wae_macros::query_as;
/// use wae_database::{Entity, FromRow, DatabaseRow, DatabaseResult};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct User {
///     id: i64,
///     name: String,
///     email: String,
/// }
///
/// impl FromRow for User {
///     fn from_row(row: &DatabaseRow) -> DatabaseResult<Self> {
///         Ok(Self {
///             id: row.get(0)?,
///             name: row.get(1)?,
///             email: row.get(2)?,
///         })
///     }
/// }
///
/// async fn get_users(conn: &dyn DatabaseConnection) -> Result<Vec<User>, Box<dyn std::error::Error>> {
///     let users = query_as!(User, conn, "SELECT id, name, email FROM users WHERE active = ?", true).await?;
///     Ok(users)
/// }
/// ```
#[proc_macro]
pub fn query_as(input: TokenStream) -> TokenStream {
    query::expand_query_as(input)
}

/// 执行宏 - 执行 INSERT/UPDATE/DELETE 等 SQL 语句
///
/// 执行 SQL 语句并返回影响的行数。
///
/// # Example
///
/// ```rust,ignore
/// use wae_macros::execute;
/// use wae_database::DatabaseConnection;
///
/// async fn insert_user(conn: &dyn DatabaseConnection) -> Result<u64, Box<dyn std::error::Error>> {
///     let affected = execute!(conn, "INSERT INTO users (name, email) VALUES (?, ?)", "Alice", "alice@example.com").await?;
///     Ok(affected)
/// }
/// ```
#[proc_macro]
pub fn execute(input: TokenStream) -> TokenStream {
    query::expand_execute(input)
}

/// 标量查询宏 - 返回单个值
///
/// 执行查询并返回单个值，适用于 COUNT、SUM 等聚合查询。
///
/// # Example
///
/// ```rust,ignore
/// use wae_macros::query_scalar;
/// use wae_database::DatabaseConnection;
///
/// async fn count_users(conn: &dyn DatabaseConnection) -> Result<i64, Box<dyn std::error::Error>> {
///     let count = query_scalar!(i64, conn, "SELECT COUNT(*) FROM users").await?;
///     Ok(count)
/// }
/// ```
#[proc_macro]
pub fn query_scalar(input: TokenStream) -> TokenStream {
    query::expand_query_scalar(input)
}

/// 使用效果宏 - 获取依赖的便捷宏
///
/// 支持多种语法：
/// - `use_effect!(effectful, MyType)` - 按类型获取依赖
/// - `use_effect!(effectful, "name", MyType)` - 按字符串键和类型获取依赖
/// - `use_effect!(effectful, config, MyConfig)` - 便捷地获取配置
/// - `use_effect!(effectful, auth, MyAuthService)` - 便捷地获取认证服务
///
/// # Example
///
/// ```rust,ignore
/// use wae_macros::use_effect;
///
/// async fn handler(effectful: Effectful) -> WaeResult<()> {
///     let config: MyConfig = use_effect!(effectful, MyConfig)?;
///     let auth: Arc<dyn AuthService> = use_effect!(effectful, auth, Arc<dyn AuthService>)?;
///     Ok(())
/// }
/// ```
#[proc_macro]
pub fn use_effect(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as UseEffectInput);

    let expanded = match parsed {
        UseEffectInput::TypeOnly { effectful, ty } => {
            quote! {
                #effectful.get_type::<#ty>()
            }
        }
        UseEffectInput::Named { effectful, name, ty } => {
            quote! {
                #effectful.get::<#ty>(#name)
            }
        }
        UseEffectInput::Config { effectful, ty } => {
            quote! {
                #effectful.use_config::<#ty>()
            }
        }
        UseEffectInput::Auth { effectful, ty } => {
            quote! {
                #effectful.use_auth::<#ty>()
            }
        }
    };

    TokenStream::from(expanded)
}

enum UseEffectInput {
    TypeOnly { effectful: syn::Expr, ty: syn::Type },
    Named { effectful: syn::Expr, name: syn::LitStr, ty: syn::Type },
    Config { effectful: syn::Expr, ty: syn::Type },
    Auth { effectful: syn::Expr, ty: syn::Type },
}

impl syn::parse::Parse for UseEffectInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let effectful: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        if input.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            let _: syn::Token![,] = input.parse()?;
            let ty: syn::Type = input.parse()?;

            if ident == "config" {
                Ok(UseEffectInput::Config { effectful, ty })
            }
            else if ident == "auth" {
                Ok(UseEffectInput::Auth { effectful, ty })
            }
            else {
                Err(syn::Error::new_spanned(ident, "Expected 'config' or 'auth'"))
            }
        }
        else if input.peek(syn::LitStr) {
            let name: syn::LitStr = input.parse()?;
            let _: syn::Token![,] = input.parse()?;
            let ty: syn::Type = input.parse()?;
            Ok(UseEffectInput::Named { effectful, name, ty })
        }
        else {
            let ty: syn::Type = input.parse()?;
            Ok(UseEffectInput::TypeOnly { effectful, ty })
        }
    }
}
