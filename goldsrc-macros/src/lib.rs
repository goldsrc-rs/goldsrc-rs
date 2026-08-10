extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_attribute]
pub fn plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        #input

        #[no_mangle]
        pub extern "C" fn __goldsrc_plugin_metadata() -> *const u8 {
            let meta = concat!(r#"{"name": ""#, stringify!(#name), r#"", "version": "1.0.0", "systems": []}"#, "\0");
            meta.as_ptr()
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
