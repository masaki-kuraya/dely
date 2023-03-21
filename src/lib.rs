// #![allow(dead_code)]
pub mod domain;
pub mod infrastructure;

// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, DeriveInput};

// #[proc_macro_derive(Id)]
// pub fn my_macro(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = input.ident;
//     let expanded = quote! {
//         impl dely::domain::Id for #name {}
//     };
//     TokenStream::from(expanded)
// }
