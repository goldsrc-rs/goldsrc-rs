#![allow(clippy::collapsible_if, clippy::single_match)]

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
            let actual_size = size.max(1);
            let buf = vec![0u8; actual_size].into_boxed_slice();
            let ptr = buf.as_ptr() as *mut u8;
            std::mem::forget(buf);
            ptr
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn __goldsrc_dealloc(ptr: *mut u8, size: usize) {
            if !ptr.is_null() {
                let actual_size = size.max(1);
                let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr, actual_size)) };
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as syn::ItemFn);
    let fn_name = &input_fn.sig.ident;
    let mut cmd_name = fn_name.to_string();

    if !attr.is_empty() {
        let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
        if let Ok(metas) = parser.parse(attr) {
            for meta in metas {
                if let Meta::NameValue(nv) = meta {
                    if nv.path.is_ident("name") {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(s) = &expr_lit.lit {
                                cmd_name = s.value();
                            }
                        }
                    }
                }
            }
        }
    }

    let expanded = quote! {
        #input_fn

        #[unsafe(no_mangle)]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub unsafe extern "C" fn on_command(
            cmd_ptr: *const u8,
            cmd_len: usize,
            args_ptr: *const u8,
            args_len: usize,
        ) {
            let cmd_slice = unsafe { std::slice::from_raw_parts(cmd_ptr, cmd_len) };
            let args_slice = unsafe { std::slice::from_raw_parts(args_ptr, args_len) };

            if let (Ok(cmd_str), Ok(args_str)) = (
                std::str::from_utf8(cmd_slice),
                std::str::from_utf8(args_slice),
            ) {
                if cmd_str == #cmd_name {
                    #fn_name(cmd_str, args_str);
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as syn::ItemFn);
    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        #[unsafe(no_mangle)]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub unsafe extern "C" fn on_event(
            name_ptr: *const u8,
            name_len: usize,
            data_ptr: *const u8,
            data_len: usize,
        ) {
            let name_slice = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
            let data_slice = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };

            if let (Ok(event_name), Ok(event_data)) = (
                std::str::from_utf8(name_slice),
                std::str::from_utf8(data_slice),
            ) {
                #fn_name(event_name, event_data);
            }
        }
    };

    TokenStream::from(expanded)
}
