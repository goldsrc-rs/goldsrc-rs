//! Attribute model and parsing for `#[plugin(...)]` and stacked helper attributes.

use crate::defs::PluginAttr;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprArray, ExprLit, ItemImpl, Lit, Meta, Token};

/// Parses both the `#[plugin(...)]` attribute parameters and stacked helper attributes on `impl MyPlugin`.
pub fn parse_plugin_and_helpers(
    attr: proc_macro2::TokenStream,
    input_impl: &mut ItemImpl,
) -> syn::Result<PluginAttr> {
    let cargo_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "Unknown".to_string());
    let cargo_version = std::env::var("CARGO_PKG_VERSION")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "1.0.0".to_string());
    let cargo_authors = std::env::var("CARGO_PKG_AUTHORS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Unknown".to_string());
    let cargo_desc = std::env::var("CARGO_PKG_DESCRIPTION")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "No description provided".to_string());
    let cargo_license = std::env::var("CARGO_PKG_LICENSE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Not Stated".to_string());
    let cargo_url = std::env::var("CARGO_PKG_HOMEPAGE")
        .or_else(|_| std::env::var("CARGO_PKG_REPOSITORY"))
        .or_else(|_| std::env::var("CARGO_PKG_DOCUMENTATION"))
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "N/A".to_string());

    let mut out = PluginAttr {
        name: cargo_name,
        version: cargo_version,
        author: cargo_authors,
        description: cargo_desc,
        url: cargo_url,
        license: cargo_license,
        bundle: None,
        require: Vec::new(),
        permissions: Vec::new(),
        load_time: "anytime".to_string(),
        unload_time: "anytime".to_string(),
    };

    // 1. Parse arguments inside `#[plugin(...)]`
    if !attr.is_empty() {
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
                    if ident == "require" {
                        parse_string_list_expr(&nv.value, &mut out.require)?;
                        continue;
                    } else if ident == "permissions" || ident == "permission" {
                        parse_string_list_expr(&nv.value, &mut out.permissions)?;
                        continue;
                    }
                    let value = parse_str_lit(&nv.value, &ident)?;
                    apply_kv_meta(&ident, value, &mut out, &nv)?;
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unsupported #[plugin] attribute; supported: name, version, author, description, url/repository, license, bundle, require, permissions",
                    ));
                }
            }
        }
    }

    // 2. Scan and consume stacked helper attributes on the `impl` block
    let mut helper_error = None;
    input_impl.attrs.retain(|attr| {
        if attr.path().is_ident("bundle") {
            if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
                out.bundle = Some(s.value());
            } else if let Ok(meta_list) = attr.meta.require_list() {
                let _ = meta_list.parse_nested_meta(|meta| {
                    if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                        out.bundle = Some(s.value());
                    }
                    Ok(())
                });
            }
            false
        } else if attr.path().is_ident("require") {
            if let Ok(exprs) = attr.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            {
                for expr in exprs {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = expr
                    {
                        out.require.push(s.value());
                    }
                }
            } else if let Ok(meta_list) = attr.meta.require_list() {
                let _ = meta_list.parse_nested_meta(|meta| {
                    if let Some(id) = meta.path.get_ident() {
                        out.require.push(id.to_string());
                    }
                    Ok(())
                });
            }
            false
        } else if attr.path().is_ident("permissions") || attr.path().is_ident("permission") {
            if let Ok(exprs) = attr.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            {
                for expr in exprs {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = expr
                    {
                        out.permissions.push(s.value());
                    }
                }
            }
            false
        } else if attr.path().is_ident("description") {
            if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
                out.description = s.value();
            }
            false
        } else if attr.path().is_ident("url") {
            if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
                out.url = s.value();
            }
            false
        } else if attr.path().is_ident("license") {
            if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
                out.license = s.value();
            }
            false
        } else if attr.path().is_ident("lifecycle") {
            if let Ok(meta_list) = attr.meta.require_list() {
                let res = meta_list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("load") {
                        if let Ok(val) = meta.value() {
                            if let Ok(Lit::Str(s)) = val.parse::<Lit>() {
                                out.load_time = s.value();
                            } else if let Ok(syn::Path { segments, .. }) = val.parse::<syn::Path>()
                            {
                                if let Some(seg) = segments.last() {
                                    out.load_time = seg.ident.to_string();
                                }
                            }
                        }
                    } else if meta.path.is_ident("unload") {
                        if let Ok(val) = meta.value() {
                            if let Ok(Lit::Str(s)) = val.parse::<Lit>() {
                                out.unload_time = s.value();
                            } else if let Ok(syn::Path { segments, .. }) = val.parse::<syn::Path>()
                            {
                                if let Some(seg) = segments.last() {
                                    out.unload_time = seg.ident.to_string();
                                }
                            }
                        }
                    }
                    Ok(())
                });
                if let Err(e) = res {
                    helper_error = Some(e);
                }
            }
            false
        } else {
            true
        }
    });

    if let Some(err) = helper_error {
        return Err(err);
    }

    Ok(out)
}

fn parse_str_lit(expr: &Expr, ident: &str) -> syn::Result<String> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(s) => Ok(s.value()),
            _ => Err(syn::Error::new_spanned(
                expr,
                format!("#[plugin({ident} = ...)] expects a string literal"),
            )),
        },
        _ => Err(syn::Error::new_spanned(
            expr,
            format!("#[plugin({ident} = ...)] expects a string literal"),
        )),
    }
}

fn parse_string_list_expr(expr: &Expr, target: &mut Vec<String>) -> syn::Result<()> {
    match expr {
        Expr::Array(ExprArray { elems, .. }) => {
            for elem in elems {
                match elem {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) => target.push(s.value()),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            elem,
                            "expects an array of string literals",
                        ));
                    }
                }
            }
            Ok(())
        }
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => {
            target.push(s.value());
            Ok(())
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            "expects an array of string literals or single string",
        )),
    }
}

fn apply_kv_meta(
    ident: &str,
    value: String,
    out: &mut PluginAttr,
    nv: &syn::MetaNameValue,
) -> syn::Result<()> {
    if ident == "name" {
        out.name = value;
    } else if ident == "version" {
        out.version = value;
    } else if ident == "author" || ident == "authors" {
        out.author = value;
    } else if ident == "description" {
        out.description = value;
    } else if ident == "url" || ident == "repository" || ident == "homepage" {
        out.url = value;
    } else if ident == "license" {
        out.license = value;
    } else if ident == "bundle" {
        if value.is_empty()
            || value.contains("..")
            || value.starts_with('/')
            || value.starts_with('\\')
            || value.contains(':')
            || !value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/')
        {
            return Err(syn::Error::new_spanned(
                &nv.value,
                format!(
                    "invalid #[plugin(bundle = \"{value}\")]: must be non-empty, relative path without '..' or ':', using only [a-zA-Z0-9_/-]"
                ),
            ));
        }
        out.bundle = Some(value);
    } else {
        return Err(syn::Error::new_spanned(
            &nv.path,
            format!(
                "unknown #[plugin] attribute '{ident}'; supported: name, version, author, description, url/repository, license, bundle, require, permissions"
            ),
        ));
    }
    Ok(())
}
