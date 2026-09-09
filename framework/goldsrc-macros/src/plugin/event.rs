//! Event subscriber handler parsing and code generation.

use crate::utils::check_sig_args;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Lit;

/// Extracted event handler information.
pub struct EventHandler {
    /// Target event name string.
    pub name: Option<String>,
    /// Handler function identifier.
    pub ident: syn::Ident,
    /// Number of parameters.
    pub inputs_len: usize,
}

/// Parses a `#[event]` attribute on a method.
pub fn parse_event(attr: &syn::Attribute, sig: &syn::Signature) -> syn::Result<EventHandler> {
    check_sig_args(sig, "event", &[0, 1, 2])?;

    let mut event_name = None;
    if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
        event_name = Some(s.value());
    } else if let Ok(meta_list) = attr.meta.require_list() {
        let _ = meta_list.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                    event_name = Some(s.value());
                }
            }
            Ok(())
        });
    }

    Ok(EventHandler {
        name: event_name,
        ident: sig.ident.clone(),
        inputs_len: sig.inputs.len(),
    })
}

/// Generates event subscriber registrations for `on_load`.
pub fn generate_event_registrations(
    struct_name: &syn::Type,
    handlers: &[EventHandler],
) -> Vec<TokenStream> {
    let mut registrations = Vec::new();
    for h in handlers {
        let fn_name = &h.ident;
        let call = match h.inputs_len {
            0 => quote! { |_payload| { #struct_name::#fn_name(); } },
            1 => quote! { |payload| { #struct_name::#fn_name(payload.to_vec()); } },
            _ => {
                let n_str = h.name.clone().unwrap_or_default();
                quote! { |payload| { #struct_name::#fn_name(#n_str.to_string(), payload.to_vec()); } }
            }
        };
        let ev_name = h.name.clone().unwrap_or_default();
        registrations.push(quote! {
            ::goldsrc::event::Event::subscriber(#ev_name)
                .subscribe(#call);
        });
    }
    registrations
}
