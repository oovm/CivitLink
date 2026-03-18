//! GWG Engine 过程宏
//!
//! 提供 Component、Resource 等派生宏。

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Component 派生宏
///
/// 为结构体自动实现 Component trait。
#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl gwg_types::prelude::Component for #name {}
    };
    
    expanded.into()
}

/// Resource 派生宏
///
/// 为结构体自动实现 Resource trait。
#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl gwg_types::prelude::Resource for #name {}
    };
    
    expanded.into()
}

/// Reflect 派生宏
///
/// 为结构体自动实现 Reflect trait。
#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let type_name = format!("{}::{}", module_path!(), name);
    
    let expanded = quote! {
        impl gwg_reflection::prelude::Reflect for #name {
            fn type_name(&self) -> &'static str {
                #type_name
            }
            
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            
            fn clone_value(&self) -> Box<dyn gwg_reflection::prelude::Reflect> {
                Box::new(self.clone())
            }
        }
    };
    
    expanded.into()
}
