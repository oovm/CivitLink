#![warn(missing_docs)]

//! GG 引擎过程宏模块
//! 提供 `#[derive(Reflect)]` 等派生宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// 为结构体派生 `PartialReflect` trait
///
/// 要求：
/// - 仅支持具名字段的结构体
/// - 所有字段类型必须实现 `PartialReflect`
/// - 结构体必须实现 `Clone`
///
/// # 示例
///
/// ```ignore
/// #[derive(Reflect, Clone)]
/// struct MyStruct {
///     name: String,
///     value: i32,
/// }
/// ```
#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return syn::Error::new_spanned(&input.ident, "Reflect only supports structs with named fields")
                .to_compile_error()
                .into();
        }
    };

    let fields = match &data.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return syn::Error::new_spanned(&input.ident, "Reflect only supports structs with named fields")
                .to_compile_error()
                .into();
        }
    };

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_name_strs: Vec<_> = field_names.iter().map(|name| name.as_ref().unwrap().to_string()).collect();

    let field_match_arms: Vec<_> = field_names
        .iter()
        .map(|name| {
            let name_str = name.as_ref().unwrap().to_string();
            quote! { #name_str => Some(&self.#name as &dyn gg_reflection::PartialReflect) }
        })
        .collect();

    let field_match_arms_mut: Vec<_> = field_names
        .iter()
        .map(|name| {
            let name_str = name.as_ref().unwrap().to_string();
            quote! { #name_str => Some(&mut self.#name as &mut dyn gg_reflection::PartialReflect) }
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics gg_reflection::PartialReflect for #name #ty_generics #where_clause {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn type_name(&self) -> &'static str {
                std::any::type_name::<Self>()
            }

            fn clone_reflect(&self) -> Box<dyn gg_reflection::PartialReflect> {
                Box::new(self.clone())
            }

            fn field_names(&self) -> &[&str] {
                &[#(#field_name_strs),*]
            }

            fn field(&self, name: &str) -> Option<&dyn gg_reflection::PartialReflect> {
                match name {
                    #(#field_match_arms,)*
                    _ => None,
                }
            }

            fn field_mut(&mut self, name: &str) -> Option<&mut dyn gg_reflection::PartialReflect> {
                match name {
                    #(#field_match_arms_mut,)*
                    _ => None,
                }
            }

            fn try_assign(&mut self, source: &dyn gg_reflection::PartialReflect) -> Result<(), String> {
                if let Some(val) = source.as_any().downcast_ref::<Self>() {
                    *self = val.clone();
                    Ok(())
                } else {
                    Err(format!(
                        "type mismatch: expected {}, got {}",
                        self.type_name(),
                        source.type_name()
                    ))
                }
            }
        }
    };

    TokenStream::from(expanded)
}
