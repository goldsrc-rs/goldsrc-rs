#![allow(clippy::collapsible_if, clippy::single_match)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprArray, ImplItem, ImplItemFn, ItemImpl, Lit, Meta, Token, parse_macro_input};

/// Escapes a string for embedding inside a TOML double-quoted literal.
fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Parsed `#[plugin(...)]` attribute values.
struct PluginAttr {
    name: String,
    version: String,
    author: String,
    description: String,
    dependencies: Vec<String>,
}

fn parse_plugin_attr(attr: proc_macro2::TokenStream) -> syn::Result<PluginAttr> {
    let mut out = PluginAttr {
        name: "Unknown".to_string(),
        version: "1.0.0".to_string(),
        author: "Unknown".to_string(),
        description: String::new(),
        dependencies: Vec::new(),
    };
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    for meta in parser.parse2(attr)? {
        let ident = match meta.path().get_ident() {
            Some(id) => id.to_string(),
            None => {
                return Err(syn::Error::new_spanned(
                    meta.path(),
                    "unsupported #[plugin] attribute",
                ));
            }
        };
        match meta {
            Meta::NameValue(nv) => {
                if ident == "dependencies" {
                    let array: ExprArray = match &nv.value {
                        Expr::Array(arr) => arr.clone(),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "#[plugin(dependencies = ...)] expects an array of string literals",
                            ));
                        }
                    };
                    for expr in &array.elems {
                        match expr {
                            Expr::Lit(expr_lit) => match &expr_lit.lit {
                                Lit::Str(s) => out.dependencies.push(s.value()),
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        expr,
                                        "dependencies expects a list of string literals like \"name@>=1.0\"",
                                    ));
                                }
                            },
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    expr,
                                    "dependencies expects a list of string literals like \"name@>=1.0\"",
                                ));
                            }
                        }
                    }
                    continue;
                }
                let value = match &nv.value {
                    Expr::Lit(expr_lit) => match &expr_lit.lit {
                        Lit::Str(s) => s.value(),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                format!("#[plugin({ident} = ...)] expects a string literal"),
                            ));
                        }
                    },
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            format!("#[plugin({ident} = ...)] expects a string literal"),
                        ));
                    }
                };
                if ident == "name" {
                    out.name = value;
                } else if ident == "version" {
                    out.version = value;
                } else if ident == "author" {
                    out.author = value;
                } else if ident == "description" {
                    out.description = value;
                } else {
                    return Err(syn::Error::new_spanned(
                        nv.path,
                        format!(
                            "unknown #[plugin] attribute '{ident}'; supported: name, version, author, description, dependencies"
                        ),
                    ));
                }
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "unsupported #[plugin] attribute; supported: name, version, author, description, dependencies",
                ));
            }
        }
    }
    Ok(out)
}

/// Validates a handler method's argument count against expectations.
fn check_handler_args(method: &ImplItemFn, attr_name: &str, expected: &[usize]) -> syn::Result<()> {
    let actual = method.sig.inputs.len();
    if expected.contains(&actual) {
        Ok(())
    } else {
        let expected_str = if expected.len() == 1 {
            format!("{} argument", expected[0])
        } else {
            let list = expected
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" or ");
            format!("{list} arguments")
        };
        Err(syn::Error::new_spanned(
            &method.sig,
            format!("#[{attr_name}] handler must take {expected_str}, got {actual}"),
        ))
    }
}

/// Attribute macro that marks a struct implementation as a GoldSrc WASM plugin.
///
/// Generates the WIT Component Model export bindings, event dispatcher, command registry,
/// and metadata header for the WASM runtime.
#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = match parse_plugin_attr(attr.into()) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut input_impl = parse_macro_input!(item as ItemImpl);
    let struct_name = &input_impl.self_ty;

    let plugin_name = attr.name;
    let plugin_version = attr.version;
    let plugin_author = attr.author;

    let mut deps_toml = String::new();
    if !attr.dependencies.is_empty() {
        let deps: Vec<String> = attr
            .dependencies
            .iter()
            .map(|d| format!("\"{}\"", toml_escape(d)))
            .collect();
        deps_toml = format!("dependencies = [{}]\n", deps.join(", "));
    }

    let mut on_load_fn = quote! {};
    let mut on_unload_fn = quote! {};
    let mut on_frame_fn = quote! {};
    let mut on_event_fn = quote! {};
    let mut on_command_fn = quote! {};

    let mut command_matchers = Vec::new();
    let mut plugin_commands: Vec<String> = Vec::new();

    // Iterate over the items in the impl block to find our marker attributes
    for item in &mut input_impl.items {
        if let ImplItem::Fn(method) = item {
            let mut is_on_load = false;
            let mut is_on_unload = false;
            let mut is_on_frame = false;
            let mut is_on_event = false;
            let mut cmd_name = None;

            // Retain attributes that are NOT our custom ones
            method.attrs.retain(|attr| {
                if attr.path().is_ident("on_load") {
                    is_on_load = true;
                    false
                } else if attr.path().is_ident("on_unload") {
                    is_on_unload = true;
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
                if let Err(e) = check_handler_args(method, "on_load", &[0]) {
                    return e.to_compile_error().into();
                }
                on_load_fn = quote! { #struct_name::#fn_name(); };
            }
            if is_on_unload {
                if let Err(e) = check_handler_args(method, "on_unload", &[0]) {
                    return e.to_compile_error().into();
                }
                on_unload_fn = quote! { #struct_name::#fn_name(); };
            }
            if is_on_frame {
                if let Err(e) = check_handler_args(method, "on_frame", &[0]) {
                    return e.to_compile_error().into();
                }
                on_frame_fn = quote! { #struct_name::#fn_name(); };
            }
            if is_on_event {
                if let Err(e) = check_handler_args(method, "event", &[0, 1, 2]) {
                    return e.to_compile_error().into();
                }
                let call_expr = match inputs_len {
                    0 => quote! { #struct_name::#fn_name() },
                    1 => quote! { #struct_name::#fn_name(name) },
                    _ => quote! { #struct_name::#fn_name(name, payload) },
                };
                on_event_fn = quote! { #call_expr; };
            }
            if let Some(cmd) = cmd_name {
                if let Err(e) = check_handler_args(method, "command", &[0, 1, 2]) {
                    return e.to_compile_error().into();
                }
                plugin_commands.push(cmd.clone());
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
                _ => return false,
            }
            true
        };
    } else {
        on_command_fn = quote! { false };
    }

    // Commands are discovered from #[command] markers on handler methods.
    let mut commands_toml = String::new();
    if !plugin_commands.is_empty() {
        let cmds: Vec<String> = plugin_commands
            .iter()
            .map(|c| format!("\"{}\"", toml_escape(c)))
            .collect();
        commands_toml = format!("commands = [{}]\n", cmds.join(", "));
    }

    let desc_toml = if !attr.description.is_empty() {
        format!("description = \"{}\"\n", toml_escape(&attr.description))
    } else {
        String::new()
    };

    let meta_toml = format!(
        "name = \"{}\"\nversion = \"{}\"\nauthor = \"{}\"\n{}{}{}",
        toml_escape(&plugin_name),
        toml_escape(&plugin_version),
        toml_escape(&plugin_author),
        desc_toml,
        deps_toml,
        commands_toml
    );

    let expanded = quote! {
        #input_impl

        impl ::goldsrc::goldsrc_api::bindings::Guest for #struct_name {
            fn get_metadata() -> String {
                #meta_toml.to_string()
            }

            fn on_load() {
                #on_load_fn
            }

            fn on_unload() {
                #on_unload_fn
            }

            fn on_frame() {
                #on_frame_fn
            }

            fn on_event(name: String, payload: Vec<u8>) {
                #on_event_fn
            }

            fn on_command(name: String, args: String) -> bool {
                #on_command_fn
            }
        }

        #[cfg(target_arch = "wasm32")]
        const _: () = {
            #[allow(unsafe_attributes)]
            ::goldsrc::goldsrc_api::bindings::export!(#struct_name with_types_in ::goldsrc::goldsrc_api::bindings);

            #[unsafe(no_mangle)]
            #[doc(hidden)]
            pub static _KEEP_WIT_COMPONENT_TYPE: &[u8] = &::goldsrc::goldsrc_api::bindings::__WIT_BINDGEN_COMPONENT_TYPE;
        };
    };

    TokenStream::from(expanded)
}

/// Error raised when a marker attribute is used outside a `#[plugin]` impl.
fn marker_outside_plugin(name: &str) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("#[{name}] can only be used on methods inside a #[plugin(...)] impl block"),
    )
    .to_compile_error()
    .into()
}

/// Marker attribute for the plugin's `on_load` lifecycle hook.
#[proc_macro_attribute]
pub fn on_load(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    // Expanded by the compiler only when NOT inside #[plugin] (which strips it).
    marker_outside_plugin("on_load")
}

/// Marker attribute for the plugin's `on_unload` lifecycle hook.
#[proc_macro_attribute]
pub fn on_unload(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("on_unload")
}

/// Marker attribute for the plugin's per-frame `on_frame` hook.
#[proc_macro_attribute]
pub fn on_frame(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("on_frame")
}

/// Marker attribute for event handlers (`#[event("event_name")]`).
#[proc_macro_attribute]
pub fn event(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("event")
}

/// Marker attribute for console / client command handlers (`#[command("cmd_name")]`).
#[proc_macro_attribute]
pub fn command(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("command")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> syn::Result<PluginAttr> {
        let ts: proc_macro2::TokenStream = s.parse().unwrap();
        parse_plugin_attr(ts)
    }

    #[test]
    fn parses_all_attrs() {
        let a = parse(
            r#"name = "x", version = "2.0", author = "A", description = "Test Desc", dependencies = ["b@>=1", "c@1.0"]"#,
        )
        .unwrap();
        assert_eq!(a.name, "x");
        assert_eq!(a.version, "2.0");
        assert_eq!(a.author, "A");
        assert_eq!(a.description, "Test Desc");
        assert_eq!(a.dependencies, vec!["b@>=1", "c@1.0"]);
    }

    #[test]
    fn rejects_unknown_attr() {
        assert!(parse("banana = 1").is_err());
    }

    #[test]
    fn rejects_non_string_value() {
        assert!(parse("name = 42").is_err());
    }

    #[test]
    fn toml_escape_quotes_and_slashes() {
        assert_eq!(toml_escape("a\"b\\c"), "a\\\"b\\\\c");
        assert_eq!(toml_escape("line\nbreak"), "line\\nbreak");
    }
}
