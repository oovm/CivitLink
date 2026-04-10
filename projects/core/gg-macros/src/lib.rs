#![warn(missing_docs)]

//! GG 引擎过程宏模块
//! 提供 `#[derive(Reflect)]` 和 `#[derive(Component)]` 等派生宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// 为结构体派生 `PartialReflect` trait
///
/// 支持具名字段结构体、元组结构体和单元结构体。
/// 所有字段类型必须实现 `PartialReflect`，结构体必须实现 `Clone`。
#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return syn::Error::new_spanned(&input.ident, "Reflect only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let (field_names_impl, field_impl, field_mut_impl) = match &data.fields {
        Fields::Named(fields) => {
            let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
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

            let names = quote! { &[#(#field_name_strs),*] };
            let get_field = quote! {
                match name {
                    #(#field_match_arms,)*
                    _ => None,
                }
            };
            let get_field_mut = quote! {
                match name {
                    #(#field_match_arms_mut,)*
                    _ => None,
                }
            };

            (names, get_field, get_field_mut)
        }
        Fields::Unnamed(fields) => {
            let field_count = fields.unnamed.len();
            let field_name_strs: Vec<String> = (0..field_count).map(|i| i.to_string()).collect();
            let field_indices: Vec<_> = (0..field_count).collect::<Vec<usize>>();

            let field_match_arms: Vec<_> = field_indices
                .iter()
                .zip(field_name_strs.iter())
                .map(|(idx, name_str)| {
                    let idx = syn::Index::from(*idx);
                    quote! { #name_str => Some(&self.#idx as &dyn gg_reflection::PartialReflect) }
                })
                .collect();

            let field_match_arms_mut: Vec<_> = field_indices
                .iter()
                .zip(field_name_strs.iter())
                .map(|(idx, name_str)| {
                    let idx = syn::Index::from(*idx);
                    quote! { #name_str => Some(&mut self.#idx as &mut dyn gg_reflection::PartialReflect) }
                })
                .collect();

            let names = quote! { &[#(#field_name_strs),*] };
            let get_field = quote! {
                match name {
                    #(#field_match_arms,)*
                    _ => None,
                }
            };
            let get_field_mut = quote! {
                match name {
                    #(#field_match_arms_mut,)*
                    _ => None,
                }
            };

            (names, get_field, get_field_mut)
        }
        Fields::Unit => {
            let names = quote! { &[] };
            let get_field = quote! { None };
            let get_field_mut = quote! { None };

            (names, get_field, get_field_mut)
        }
    };

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
                #field_names_impl
            }

            fn field(&self, name: &str) -> Option<&dyn gg_reflection::PartialReflect> {
                #field_impl
            }

            fn field_mut(&mut self, name: &str) -> Option<&mut dyn gg_reflection::PartialReflect> {
                #field_mut_impl
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

/// 为类型派生 `Component` trait
///
/// 由于 `Component` 使用 blanket impl（任何满足 `Any + Send + Sync` 的类型自动成为组件），
/// 此宏不生成 impl 块，仅作为语义标记用于文档目的和未来扩展性。
#[proc_macro_derive(Component)]
pub fn derive_component(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
