//! Menu action handler parsing and code generation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Lit;

/// Extracted menu action handler information.
pub struct MenuActionHandler {
    /// Explicit or hashed action ID.
    pub id: Option<u32>,
    /// Action name string identifier.
    pub action_name: Option<String>,
    /// Handler function identifier.
    pub ident: syn::Ident,
    /// Number of function parameters.
    pub inputs_len: usize,
}

/// Parses a `#[menu_action]` attribute on a method.
pub fn parse_menu_action(
    attr: &syn::Attribute,
    sig: &syn::Signature,
) -> syn::Result<MenuActionHandler> {
    let mut action_id = None;
    let mut action_str = None;

    if let Ok(Lit::Int(i)) = attr.parse_args::<Lit>() {
        if let Ok(val) = i.base10_parse::<u32>() {
            action_id = Some(val);
        }
    } else if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
        action_str = Some(s.value());
    } else if let Ok(meta_list) = attr.meta.require_list() {
        meta_list.parse_nested_meta(|meta| {
            if meta.path.is_ident("id") {
                if let Ok(Lit::Int(i)) = meta.value()?.parse::<Lit>() {
                    if let Ok(val) = i.base10_parse::<u32>() {
                        action_id = Some(val);
                    }
                }
            } else if meta.path.is_ident("action") || meta.path.is_ident("name") {
                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                    action_str = Some(s.value());
                }
            }
            Ok(())
        })?;
    }

    if let Some(id) = action_id {
        Ok(MenuActionHandler {
            id: Some(id),
            action_name: None,
            ident: sig.ident.clone(),
            inputs_len: sig.inputs.len(),
        })
    } else if let Some(act) = action_str {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&act, &mut hasher);
        let calculated_id = (std::hash::Hasher::finish(&hasher) & 0x7FFF_FFFF) as u32;
        Ok(MenuActionHandler {
            id: Some(calculated_id),
            action_name: Some(act),
            ident: sig.ident.clone(),
            inputs_len: sig.inputs.len(),
        })
    } else {
        let method_name = sig.ident.to_string();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&method_name, &mut hasher);
        let calculated_id = (std::hash::Hasher::finish(&hasher) & 0x7FFF_FFFF) as u32;
        Ok(MenuActionHandler {
            id: Some(calculated_id),
            action_name: Some(method_name),
            ident: sig.ident.clone(),
            inputs_len: sig.inputs.len(),
        })
    }
}

/// Generates menu action registrations for `on_load`.
pub fn generate_menu_registrations(
    struct_name: &syn::Type,
    handlers: &[MenuActionHandler],
) -> Vec<TokenStream> {
    let mut registrations = Vec::new();
    for h in handlers {
        let fn_name = &h.ident;
        let call = match h.inputs_len {
            0 => quote! { |_player, _action| { #struct_name::#fn_name(); } },
            1 => quote! { |mut player, _action| { #struct_name::#fn_name(&mut player); } },
            _ => {
                let act_str = h.action_name.clone().unwrap_or_default();
                quote! { |mut player, _action| { #struct_name::#fn_name(&mut player, #act_str); } }
            }
        };
        if let Some(id) = h.id {
            registrations.push(quote! {
                ::goldsrc::menu::register_menu_action_id(#id, #call);
            });
        }
        if let Some(act) = &h.action_name {
            registrations.push(quote! {
                ::goldsrc::menu::register_menu_action_name(#act, #call);
            });
        }
    }
    registrations
}
