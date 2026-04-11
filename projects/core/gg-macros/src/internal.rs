//! 过程宏内部实现

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Component 派生宏的实现
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::gg_ecs::component::Component for #name #ty_generics #where_clause {
            type Storage = ::gg_ecs::storage::VecStorage;
        }
    };

    TokenStream::from(expanded)
}

/// Resource 派生宏的实现
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::gg_ecs::resource::Resource for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}
