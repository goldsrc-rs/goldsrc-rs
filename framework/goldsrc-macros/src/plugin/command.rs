//! Console and chat command handler parsing and code generation.

use crate::defs::CommandDefInfo;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprArray, ExprLit, Lit};

/// Parses a `#[command]` attribute on a method.
pub fn parse_command(attr: &syn::Attribute) -> syn::Result<CommandDefInfo> {
    let mut cmd_name = None;
    let mut cmd_aliases = Vec::new();
    let mut cmd_capability: Option<String> = None;
    let mut cmd_description: Option<String> = None;
    let mut cmd_usage: Option<String> = None;
    let mut cmd_requires: Vec<String> = Vec::new();

    let meta_list = attr.meta.require_list()?;
    meta_list.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                cmd_name = Some(s.value());
            }
        } else if meta.path.is_ident("capability") {
            if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                let cap_val = s.value();
                if let Err(err) = ::goldsrc_api::auth::CapExpr::parse(&cap_val) {
                    return Err(
                        meta.error(format!("invalid capability expression '{cap_val}': {err}"))
                    );
                }
                cmd_capability = Some(cap_val);
            }
        } else if meta.path.is_ident("description") {
            if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                cmd_description = Some(s.value());
            }
        } else if meta.path.is_ident("usage") {
            if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                cmd_usage = Some(s.value());
            }
        } else if meta.path.is_ident("requires") {
            if let Ok(ExprArray { elems, .. }) = meta.value()?.parse::<ExprArray>() {
                for elem in elems {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = elem
                    {
                        cmd_requires.push(s.value());
                    }
                }
            } else if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                cmd_requires.push(s.value());
            }
        } else if meta.path.is_ident("aliases") {
            if let Ok(ExprArray { elems, .. }) = meta.value()?.parse::<ExprArray>() {
                for elem in elems {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = elem
                    {
                        cmd_aliases.push(s.value());
                    }
                }
            }
        } else {
            return Err(meta.error(format!(
                "unknown #[command] key '{}'",
                meta.path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default()
            )));
        }
        Ok(())
    })?;

    let name = cmd_name
        .ok_or_else(|| syn::Error::new_spanned(attr, "missing 'name' parameter in #[command]"))?;

    Ok(CommandDefInfo {
        name,
        description: cmd_description.unwrap_or_default(),
        usage: cmd_usage.unwrap_or_default(),
        aliases: cmd_aliases,
        capability: cmd_capability,
        requires: cmd_requires,
    })
}

/// Generates the `::goldsrc::Command::builder(...).register(...)` token stream for a command.
pub fn generate_command_registration(
    struct_name: &syn::Type,
    sig: &syn::Signature,
    cmd_def: &CommandDefInfo,
) -> syn::Result<TokenStream> {
    let fn_name = &sig.ident;
    let inputs_len = sig.inputs.len();
    let cmd = &cmd_def.name;
    let cmd_aliases = &cmd_def.aliases;
    let cmd_capability = &cmd_def.capability;
    let cmd_description = &cmd_def.description;
    let cmd_usage = &cmd_def.usage;

    let is_raw_signature = match inputs_len {
        0 => true,
        1 => {
            if let Some(syn::FnArg::Typed(pat_type)) = sig.inputs.first() {
                if let syn::Type::Path(type_path) = &*pat_type.ty {
                    type_path.path.is_ident("String") || type_path.path.is_ident("str")
                } else {
                    false
                }
            } else {
                false
            }
        }
        2 => {
            let mut all_string = true;
            for input in &sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    if let syn::Type::Path(type_path) = &*pat_type.ty {
                        if !type_path.path.is_ident("String") && !type_path.path.is_ident("str") {
                            all_string = false;
                        }
                    } else {
                        all_string = false;
                    }
                } else {
                    all_string = false;
                }
            }
            all_string
        }
        _ => false,
    };

    let call_expr = if is_raw_signature {
        match inputs_len {
            0 => quote! {
                #struct_name::#fn_name();
                true
            },
            1 => quote! {
                #struct_name::#fn_name(args.to_string());
                true
            },
            _ => quote! {
                #struct_name::#fn_name(#cmd.to_string(), args.to_string());
                true
            },
        }
    } else {
        let total_non_context_params = sig
            .inputs
            .iter()
            .filter(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let syn::Pat::Ident(p) = &*pat_type.pat {
                        return p.ident != "caller" && p.ident != "player";
                    }
                }
                false
            })
            .count();

        let mut param_bindings = Vec::new();
        let mut param_idents = Vec::new();
        let mut current_non_context_idx = 0;

        for input in &sig.inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                let ident = match &*pat_type.pat {
                    syn::Pat::Ident(p) => &p.ident,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            pat_type,
                            "unsupported parameter pattern",
                        ));
                    }
                };
                let ty = &pat_type.ty;
                param_idents.push(ident.clone());
                if ident == "caller" {
                    param_bindings.push(quote! {
                        let mut #ident: #ty = caller;
                    });
                } else if ident == "player" {
                    param_bindings.push(quote! {
                        let caller_token = caller.to_string();
                        let target_token = if #total_non_context_params == 0 {
                            if let Some(arg) = parsed_args.first().filter(|s| !s.is_empty()) {
                                arg.as_str()
                            } else {
                                &caller_token
                            }
                        } else {
                            &caller_token
                        };
                        let mut #ident: #ty = match ::goldsrc::FromArg::from_arg(target_token) {
                            Ok(val) => val,
                            Err(err) => {
                                ::goldsrc::log_warn!("[Command Error] Failed to bind 'player' context for '{target_token}' (caller {caller}): {err}");
                                return false;
                            }
                        };
                    });
                } else {
                    let param_idx = current_non_context_idx;
                    current_non_context_idx += 1;
                    let is_tail = param_idx == total_non_context_params - 1;
                    let ident_str = ident.to_string();

                    if is_tail {
                        param_bindings.push(quote! {
                            let raw_arg = if #param_idx < parsed_args.len() {
                                parsed_args[#param_idx..].join(" ")
                            } else {
                                String::new()
                            };
                            let mut #ident: #ty = match ::goldsrc::FromArg::from_arg(&raw_arg) {
                                Ok(val) => val,
                                Err(err) => {
                                    ::goldsrc::log_warn!("[Command Error] Failed to parse tail argument '{}' for parameter '{}': {err}", raw_arg, #ident_str);
                                    return false;
                                }
                            };
                        });
                    } else {
                        param_bindings.push(quote! {
                            let raw_arg = parsed_args.get(#param_idx).cloned().unwrap_or_default();
                            let mut #ident: #ty = match ::goldsrc::FromArg::from_arg(&raw_arg) {
                                Ok(val) => val,
                                Err(err) => {
                                    ::goldsrc::log_warn!("[Command Error] Failed to parse argument '{}' for parameter '{}' at index {}: {err}", raw_arg, #ident_str, #param_idx);
                                    return false;
                                }
                            };
                        });
                    }
                }
            }
        }

        quote! {
            let parsed_args = ::goldsrc::split_command_args(args.as_ref());
            #(#param_bindings)*
            #struct_name::#fn_name(#(#param_idents),*);
            true
        }
    };

    let cap_builder = if let Some(cap) = cmd_capability {
        quote! { .capability(#cap) }
    } else {
        quote! {}
    };
    let desc_builder = if !cmd_description.is_empty() {
        quote! { .description(#cmd_description) }
    } else {
        quote! {}
    };
    let usage_builder = if !cmd_usage.is_empty() {
        quote! { .usage(#cmd_usage) }
    } else {
        quote! {}
    };

    Ok(quote! {
        {
            let aliases: Vec<&'static str> = vec![#(#cmd_aliases),*];
            ::goldsrc::Command::builder(#cmd)
                .aliases(aliases)
                #cap_builder
                #desc_builder
                #usage_builder
                .register(|caller, args| {
                    #call_expr
                });
        }
    })
}
