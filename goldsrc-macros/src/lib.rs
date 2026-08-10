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
                        }
                    }
                    Meta::List(list) if list.path.is_ident("systems") => {
                        let inner_parser = Punctuated::<Expr, Token![,]>::parse_terminated;
                        if let Ok(exprs) = list.parse_args_with(inner_parser) {
                            for expr in exprs {
                                if let Expr::Lit(expr_lit) = expr {
                                    if let Lit::Str(s) = expr_lit.lit {
                                        systems.push(s.value());
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

        #[no_mangle]
        pub extern "C" fn __goldsrc_plugin_metadata() -> *const u8 {
            let meta = concat!(#meta_json, "\0");
            meta.as_ptr()
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
