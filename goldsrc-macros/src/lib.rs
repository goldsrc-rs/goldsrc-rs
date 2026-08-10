extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Expr, Lit, Meta, Token, parse_macro_input};

#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    let mut plugin_name = struct_name.to_string();
    let mut plugin_version = "1.0.0".to_string();
    let mut systems: Vec<String> = Vec::new();

    if !attr.is_empty() {
        let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
        if let Ok(metas) = parser.parse(attr) {
            for meta in metas {
                match meta {
                    Meta::NameValue(nv) => {
                        if nv.path.is_ident("name") {
                            if let Expr::Lit(expr_lit) = &nv.value {
                                if let Lit::Str(s) = &expr_lit.lit {
                                    plugin_name = s.value();
                                }
                            }
                        } else if nv.path.is_ident("version") {
                            if let Expr::Lit(expr_lit) = &nv.value {
                                if let Lit::Str(s) = &expr_lit.lit {
                                    plugin_version = s.value();
                                }
                            }
                        } else if nv.path.is_ident("systems") {
                            if let Expr::Array(arr) = &nv.value {
                                for elem in &arr.elems {
                                    if let Expr::Lit(expr_lit) = elem {
                                        if let Lit::Str(s) = &expr_lit.lit {
                                            systems.push(s.value());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let systems_json = format!(
        "[{}]",
        systems
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",")
    );

    let meta_json = format!(
        "{{\"name\":\"{}\",\"version\":\"{}\",\"systems\":{}}}",
        plugin_name, plugin_version, systems_json
    );

    let expanded = quote! {
        #input

        #[unsafe(no_mangle)]
        pub extern "C" fn __goldsrc_plugin_metadata() -> *const u8 {
            let meta = concat!(#meta_json, "\0");
            meta.as_ptr()
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn __goldsrc_alloc(size: usize) -> *mut u8 {
            let mut buf = vec![0u8; size];
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            ptr
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn __goldsrc_dealloc(ptr: *mut u8, size: usize) {
            if !ptr.is_null() {
                let _ = unsafe { Vec::from_raw_parts(ptr, size, size) };
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
