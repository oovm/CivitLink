#![warn(missing_docs)]

//! GG 引擎过程宏模块
//!
//! 提供 `#[derive(Reflect)]`、`#[derive(Component)]` 和 `#[derive(Resource)]` 等派生宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, parse_macro_input};

/// 为结构体或枚举派生 `PartialReflect` trait
///
/// 支持具名字段结构体、元组结构体、单元结构体和枚举类型。
/// 所有字段类型必须实现 `PartialReflect`，类型必须实现 `Clone`。
/// 对于枚举类型，同时生成 `EnumReflect` trait 的实现。
#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match &input.data {
        Data::Struct(data) => {
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
        Data::Enum(data) => derive_reflect_enum(name, data, &impl_generics, &ty_generics, where_clause),
        _ => syn::Error::new_spanned(&input.ident, "Reflect only supports structs and enums").to_compile_error().into(),
    }
}

fn derive_reflect_enum(
    name: &syn::Ident,
    data: &DataEnum,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
) -> TokenStream {
    let variant_names: Vec<String> = data.variants.iter().map(|v| v.ident.to_string()).collect();

    let variant_name_match_arms: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let name_str = ident.to_string();
            match &variant.fields {
                Fields::Named(_) => quote! { #name::#ident { .. } => #name_str },
                Fields::Unnamed(_) => quote! { #name::#ident(..) => #name_str },
                Fields::Unit => quote! { #name::#ident => #name_str },
            }
        })
        .collect();

    let field_count_match_arms: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let count = match &variant.fields {
                Fields::Named(f) => f.named.len(),
                Fields::Unnamed(f) => f.unnamed.len(),
                Fields::Unit => 0,
            };
            match &variant.fields {
                Fields::Named(_) => quote! { #name::#ident { .. } => #count },
                Fields::Unnamed(_) => quote! { #name::#ident(..) => #count },
                Fields::Unit => quote! { #name::#ident => #count },
            }
        })
        .collect();

    let field_at_match_arms: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            match &variant.fields {
                Fields::Named(f) => {
                    let field_idents: Vec<_> = f.named.iter().map(|field| &field.ident).collect();
                    let index_match: Vec<_> = field_idents
                        .iter()
                        .enumerate()
                        .map(|(i, field_ident)| {
                            quote! { #i => Some(&#field_ident as &dyn gg_reflection::PartialReflect) }
                        })
                        .collect();
                    quote! {
                        #name::#ident { #(#field_idents),* } => match index {
                            #(#index_match,)*
                            _ => None,
                        }
                    }
                }
                Fields::Unnamed(f) => {
                    let field_count = f.unnamed.len();
                    let field_indices: Vec<_> = (0..field_count).collect::<Vec<usize>>();
                    let index_match: Vec<_> = field_indices
                        .iter()
                        .map(|i| {
                            let idx = syn::Index::from(*i);
                            quote! { #i => Some(&#idx as &dyn gg_reflection::PartialReflect) }
                        })
                        .collect();
                    let bindings: Vec<_> = field_indices
                        .iter()
                        .map(|i| syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site()))
                        .collect();
                    quote! {
                        #name::#ident(#(#bindings),*) => match index {
                            #(#index_match,)*
                            _ => None,
                        }
                    }
                }
                Fields::Unit => {
                    quote! { #name::#ident => None }
                }
            }
        })
        .collect();

    let field_at_mut_match_arms: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            match &variant.fields {
                Fields::Named(f) => {
                    let field_idents: Vec<_> = f.named.iter().map(|field| &field.ident).collect();
                    let index_match: Vec<_> = field_idents
                        .iter()
                        .enumerate()
                        .map(|(i, field_ident)| {
                            quote! { #i => Some(&mut #field_ident as &mut dyn gg_reflection::PartialReflect) }
                        })
                        .collect();
                    quote! {
                        #name::#ident { #(#field_idents),* } => match index {
                            #(#index_match,)*
                            _ => None,
                        }
                    }
                }
                Fields::Unnamed(f) => {
                    let field_count = f.unnamed.len();
                    let field_indices: Vec<_> = (0..field_count).collect::<Vec<usize>>();
                    let index_match: Vec<_> = field_indices
                        .iter()
                        .map(|i| {
                            let idx = syn::Index::from(*i);
                            quote! { #i => Some(&mut #idx as &mut dyn gg_reflection::PartialReflect) }
                        })
                        .collect();
                    let bindings: Vec<_> = field_indices
                        .iter()
                        .map(|i| syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site()))
                        .collect();
                    quote! {
                        #name::#ident(#(#bindings),*) => match index {
                            #(#index_match,)*
                            _ => None,
                        }
                    }
                }
                Fields::Unit => {
                    quote! { #name::#ident => None }
                }
            }
        })
        .collect();

    let set_variant_match_arms: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let name_str = ident.to_string();
            match &variant.fields {
                Fields::Named(f) => {
                    let field_idents: Vec<_> = f.named.iter().map(|field| &field.ident).collect();
                    let field_defaults: Vec<_> = f.named.iter().map(|_| quote! { Default::default() }).collect();
                    quote! {
                        #name_str => {
                            *self = #name::#ident { #(#field_idents: #field_defaults),* };
                            Ok(())
                        }
                    }
                }
                Fields::Unnamed(f) => {
                    let field_defaults: Vec<_> = f.unnamed.iter().map(|_| quote! { Default::default() }).collect();
                    quote! {
                        #name_str => {
                            *self = #name::#ident(#(#field_defaults),*);
                            Ok(())
                        }
                    }
                }
                Fields::Unit => {
                    quote! {
                        #name_str => {
                            *self = #name::#ident;
                            Ok(())
                        }
                    }
                }
            }
        })
        .collect();

    let static_variants_name =
        syn::Ident::new(&format!("{}_REFLECT_VARIANTS", name.to_string().to_uppercase()), proc_macro2::Span::call_site());

    let expanded = quote! {
        static #static_variants_name: &[&str] = &[#(#variant_names),*];

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

        impl #impl_generics gg_reflection::EnumReflect for #name #ty_generics #where_clause {
            fn variants(&self) -> &[&str] {
                #static_variants_name
            }

            fn variant_name(&self) -> &str {
                match self {
                    #(#variant_name_match_arms,)*
                }
            }

            fn field_at(&self, index: usize) -> Option<&dyn gg_reflection::PartialReflect> {
                match self {
                    #(#field_at_match_arms,)*
                }
            }

            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn gg_reflection::PartialReflect> {
                match self {
                    #(#field_at_mut_match_arms,)*
                }
            }

            fn field_count(&self) -> usize {
                match self {
                    #(#field_count_match_arms,)*
                }
            }

            fn set_variant(&mut self, name: &str) -> Result<(), String> {
                match name {
                    #(#set_variant_match_arms)*
                    _ => Err(format!("unknown variant: {}", name)),
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

/// 为类型派生 `Resource` trait
///
/// 由于 `Resource` 使用 blanket impl（任何满足 `Any + Send + Sync` 的类型自动成为资源），
/// 此宏不生成 impl 块，仅作为语义标记用于文档目的和未来扩展性。
#[proc_macro_derive(Resource)]
pub fn derive_resource(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
