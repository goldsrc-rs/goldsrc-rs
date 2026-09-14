//! Modular plugin code generator submodules for `#[plugin]`.

pub mod attr;
pub mod command;
pub mod event;
pub mod manifest;
pub mod menu;
pub mod system;

use crate::defs::{CommandDefInfo, PluginAttr, SystemDefInfo};
use crate::utils::check_handler_args;
use command::{generate_command_registration, parse_command};
use event::{EventHandler, generate_event_registrations, parse_event};
use manifest::generate_manifest_toml;
use menu::{MenuActionHandler, generate_menu_registrations, parse_menu_action};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, ImplItem, ItemImpl, Lit};
use system::{generate_system_registrations, parse_system};

pub fn expand_plugin(mut attr: PluginAttr, mut input_impl: ItemImpl) -> TokenStream {
    let struct_name = &input_impl.self_ty;

    let mut on_load_fn = quote! {};
    let mut on_unload_fn = quote! {};
    let mut on_frame_fn = quote! {};
    let mut event_handlers: Vec<EventHandler> = Vec::new();
    let mut registered_events: std::collections::HashSet<Option<String>> =
        std::collections::HashSet::new();

    let mut command_registrations = Vec::new();
    let mut command_defs: Vec<CommandDefInfo> = Vec::new();
    let mut menu_action_matchers: Vec<MenuActionHandler> = Vec::new();
    let mut system_handlers: Vec<SystemDefInfo> = Vec::new();

    // Iterate over the items in the impl block to find our marker attributes
    for item in &mut input_impl.items {
        if let ImplItem::Fn(method) = item {
            let mut is_on_load = false;
            let mut is_on_unload = false;
            let mut is_on_frame = false;
            let mut current_cmd_def = None;
            let mut macro_error: Option<syn::Error> = None;

            let sig = &mut method.sig;
            // Retain attributes that are NOT our custom ones
            method.attrs.retain(|fn_attr| {
                if fn_attr.path().is_ident(crate::defs::markers::ON_LOAD) {
                    is_on_load = true;
                    false
                } else if fn_attr.path().is_ident(crate::defs::markers::ON_UNLOAD) {
                    is_on_unload = true;
                    false
                } else if fn_attr.path().is_ident(crate::defs::markers::ON_FRAME) {
                    is_on_frame = true;
                    false
                } else if fn_attr.path().is_ident(crate::defs::markers::PERMISSIONS)
                    || fn_attr.path().is_ident(crate::defs::markers::PERMISSION)
                {
                    if let Ok(exprs) = fn_attr.parse_args_with(
                        syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated,
                    ) {
                        for expr in exprs {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(s), ..
                            }) = expr
                            {
                                attr.permissions.push(s.value());
                            }
                        }
                    }
                    false
                } else if fn_attr.path().is_ident(crate::defs::markers::EVENT) {
                    match parse_event(fn_attr, sig) {
                        Ok(handler) => {
                            if !registered_events.insert(handler.name.clone()) {
                                macro_error = Some(syn::Error::new_spanned(
                                    fn_attr,
                                    format!("duplicate handler for event {:?}", handler.name),
                                ));
                            } else {
                                event_handlers.push(handler);
                            }
                        }
                        Err(e) => macro_error = Some(e),
                    }
                    false
                } else if fn_attr.path().is_ident(crate::defs::markers::COMMAND) {
                    match parse_command(fn_attr) {
                        Ok(cmd_def) => {
                            current_cmd_def = Some(cmd_def);
                        }
                        Err(e) => macro_error = Some(e),
                    }
                    false
                } else if fn_attr.path().is_ident(crate::defs::markers::SYSTEM) {
                    match parse_system(fn_attr, sig) {
                        Ok(sys) => system_handlers.push(sys),
                        Err(e) => macro_error = Some(e),
                    }
                    false
                } else if fn_attr.path().is_ident(crate::defs::markers::MENU_ACTION) {
                    match parse_menu_action(fn_attr, sig) {
                        Ok(action) => menu_action_matchers.push(action),
                        Err(e) => macro_error = Some(e),
                    }
                    false
                } else {
                    true
                }
            });

            if let Some(err) = macro_error {
                return err.to_compile_error().into();
            }

            let fn_name = &method.sig.ident;

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
            if let Some(cmd_def) = current_cmd_def {
                match generate_command_registration(struct_name, &method.sig, &cmd_def) {
                    Ok(reg) => {
                        command_registrations.push(reg);
                        command_defs.push(cmd_def);
                    }
                    Err(e) => return e.to_compile_error().into(),
                }
            }
        }
    }

    let on_command_fn = quote! {
        ::goldsrc::command::dispatch_command(&name, caller, &args)
    };

    let event_registrations = generate_event_registrations(struct_name, &event_handlers);
    let menu_registrations = generate_menu_registrations(struct_name, &menu_action_matchers);
    let system_registrations = generate_system_registrations(struct_name, &system_handlers);

    let manifest_toml = generate_manifest_toml(&attr, &command_defs);

    let expanded = quote! {
        #input_impl

        impl ::goldsrc::bindings::Guest for #struct_name {
            fn get_metadata() -> String {
                #manifest_toml.to_string()
            }

            fn on_load() {
                ::goldsrc::init_guest_logger();
                #(#system_registrations)*
                #(#command_registrations)*
                #(#event_registrations)*
                #(#menu_registrations)*
                #on_load_fn
            }

            fn on_unload() {
                #on_unload_fn
            }

            fn on_frame() {
                ::goldsrc::__plugin_frame_dispatch();
                #on_frame_fn
            }

            fn on_event(name: String, payload: Vec<u8>) {
                if name == "menu_select" && payload.len() >= 8 {
                    let caller = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let slot = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
                    ::goldsrc::__plugin_dispatch_menu_select(caller, slot);
                }
                ::goldsrc::event::dispatch_event(&name, &payload);
            }

            fn on_command(name: String, caller: i32, args: String) -> bool {
                #on_command_fn
            }

            fn on_placeholder(name: String, caller: i32, param: String) -> Option<String> {
                ::goldsrc::placeholders::dispatch_local_placeholder(&name, caller, &param)
            }

            fn on_chat(sender: i32, text: String, is_team: bool) -> Option<String> {
                ::goldsrc::chat::dispatch_local_chat(sender, &text, is_team)
            }
        }

        #[cfg(target_arch = "wasm32")]
        const _: () = {
            #[allow(unsafe_attributes)]
            ::goldsrc::bindings::export!(#struct_name with_types_in ::goldsrc::bindings);

            #[unsafe(no_mangle)]
            #[doc(hidden)]
            pub static _KEEP_WIT_COMPONENT_TYPE: &[u8] = &::goldsrc::bindings::__WIT_BINDGEN_COMPONENT_TYPE;
        };
    };

    expanded.into()
}
