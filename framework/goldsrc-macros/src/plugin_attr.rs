//! Attribute model and parsing for `#[plugin(...)]`.

use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprArray, Lit, Meta, Token};

/// Parsed `#[plugin(...)]` attribute values.
pub struct PluginAttr {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub url: String,
    pub license: String,
    pub bundle: Option<String>,
    pub require: Vec<String>,
}

pub fn parse_plugin_attr(attr: proc_macro2::TokenStream) -> syn::Result<PluginAttr> {
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
                if ident == "require" {
                    let array: ExprArray = match &nv.value {
                        Expr::Array(arr) => arr.clone(),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "#[plugin(require = ...)] expects an array of string literals",
                            ));
                        }
                    };
                    for expr in &array.elems {
                        match expr {
                            Expr::Lit(expr_lit) => match &expr_lit.lit {
                                Lit::Str(s) => out.require.push(s.value()),
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        expr,
                                        "require expects a list of string literals like \"plugin:name@>=1.0\"",
                                    ));
                                }
                            },
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    expr,
                                    "require expects a list of string literals like \"plugin:name@>=1.0\"",
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
                        nv.path,
                        format!(
                            "unknown #[plugin] attribute '{ident}'; supported: name, version, author, description, url/repository, license, bundle, require"
                        ),
                    ));
                }
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "unsupported #[plugin] attribute; supported: name, version, author, description, url/repository, license, bundle, require",
                ));
            }
        }
    }
    Ok(out)
}
