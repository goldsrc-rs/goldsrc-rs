#![allow(clippy::collapsible_if, clippy::single_match)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Expr, ImplItem, ItemImpl, Lit, Meta, Token};

#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_impl = parse_macro_input!(item as ItemImpl);
    let struct_name = &input_impl.self_ty;

    let mut plugin_name = "Unknown".to_string();
    let mut plugin_version = "1.0.0".to_string();
    let mut plugin_author = "Unknown".to_string();

    if !attr.is_empty() {
        let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
        if let Ok(metas) = parser.parse(attr) {
            for meta in metas {
                if let Meta::NameValue(nv) = meta {
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
                    } else if nv.path.is_ident("author") {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(s) = &expr_lit.lit {
                                plugin_author = s.value();
                            }
                        }
                    }
                }
            }
        }
    }

    let meta_toml = format!(
        "name = \"{}\"\nversion = \"{}\"\nauthor = \"{}\"\n",
        plugin_name, plugin_version, plugin_author
    );

    let mut on_load_fn = quote! {};
    let mut on_frame_fn = quote! {};
    let mut on_event_fn = quote! {};
    let mut on_command_fn = quote! {};

    let mut command_matchers = Vec::new();

    // Iterate over the items in the impl block to find our marker attributes
    for item in &mut input_impl.items {
        if let ImplItem::Fn(method) = item {
            let mut is_on_load = false;
            let mut is_on_frame = false;
            let mut is_on_event = false;
            let mut cmd_name = None;

            // Retain attributes that are NOT our custom ones
            method.attrs.retain(|attr| {
                if attr.path().is_ident("on_load") {
                    is_on_load = true;
                    false
                } else if attr.path().is_ident("on_frame") {
                    is_on_frame = true;
                    false
                } else if attr.path().is_ident("event") {
                    is_on_event = true;
                    false
                } else if attr.path().is_ident("command") {
                    if let Ok(meta_list) = attr.meta.require_list() {
                        let _ = meta_list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("name") {
                                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                                    cmd_name = Some(s.value());
                                }
                            }
                            Ok(())
                        });
                    }
                    false
                } else {
                    true
                }
            });

            let fn_name = &method.sig.ident;
            let inputs_len = method.sig.inputs.len();

            if is_on_load {
                on_load_fn = quote! { #struct_name::#fn_name(); };
            }
            if is_on_frame {
                on_frame_fn = quote! { #struct_name::#fn_name(); };
            }
            if is_on_event {
                let call_expr = match inputs_len {
                    0 => quote! { #struct_name::#fn_name() },
                    1 => quote! { #struct_name::#fn_name(name) },
                    _ => quote! { #struct_name::#fn_name(name, payload) },
                };
                on_event_fn = quote! { #call_expr; };
            }
            if let Some(cmd) = cmd_name {
                let call_expr = match inputs_len {
                    0 => quote! { #struct_name::#fn_name() },
                    1 => quote! { #struct_name::#fn_name(args) },
                    _ => quote! { #struct_name::#fn_name(name, args) },
                };
                command_matchers.push(quote! {
                    #cmd => { #call_expr; },
                });
            }
        }
    }

    if !command_matchers.is_empty() {
        on_command_fn = quote! {
            match name.as_str() {
                #(#command_matchers)*
                _ => {}
            }
        };
    }

    let expanded = quote! {
        #input_impl

        impl ::goldsrc::goldsrc_api::bindings::Guest for #struct_name {
            fn get_metadata() -> String {
                #meta_toml.to_string()
            }

            fn on_load() {
                #on_load_fn
            }

            fn on_frame() {
                #on_frame_fn
            }

            fn on_event(name: String, payload: Vec<u8>) {
                #on_event_fn
            }

            fn on_command(name: String, args: String) {
                #on_command_fn
            }
        }

        #[allow(unsafe_attributes)]
        ::goldsrc::goldsrc_api::bindings::export!(#struct_name with_types_in ::goldsrc::goldsrc_api::bindings);

        #[unsafe(no_mangle)]
        #[doc(hidden)]
        pub static _KEEP_WIT_COMPONENT_TYPE: &[u8] = &::goldsrc::goldsrc_api::bindings::__WIT_BINDGEN_COMPONENT_TYPE;
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn on_load(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item // Passed through to be parsed by #[plugin]
}

#[proc_macro_attribute]
pub fn on_frame(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
