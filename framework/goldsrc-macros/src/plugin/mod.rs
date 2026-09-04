//! Modular plugin code generator submodules for `#[plugin]`.

pub mod attr;

use crate::defs::{CommandDefInfo, PluginAttr, SystemDefInfo};
use crate::utils::{check_handler_args, toml_escape};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprArray, ExprLit, ImplItem, ItemImpl, Lit};

pub fn expand_plugin(mut attr: PluginAttr, mut input_impl: ItemImpl) -> TokenStream {
    let struct_name = &input_impl.self_ty;

    let plugin_name = attr.name;
    let plugin_version = attr.version;
    let plugin_author = attr.author;

    let mut on_load_fn = quote! {};
    let mut on_unload_fn = quote! {};
    let mut on_frame_fn = quote! {};
    let mut event_handlers: Vec<(Option<String>, syn::Ident, usize)> = Vec::new();
    let mut registered_events: std::collections::HashSet<Option<String>> =
        std::collections::HashSet::new();

    let mut command_registrations = Vec::new();
    let mut command_defs: Vec<CommandDefInfo> = Vec::new();
    let mut menu_action_matchers: Vec<(Option<u32>, Option<String>, syn::Ident, usize)> =
        Vec::new();
    let mut system_handlers: Vec<SystemDefInfo> = Vec::new();

    // Iterate over the items in the impl block to find our marker attributes
    for item in &mut input_impl.items {
        if let ImplItem::Fn(method) = item {
            let mut is_on_load = false;
            let mut is_on_unload = false;
            let mut is_on_frame = false;
            let mut is_on_event = false;
            let mut event_name: Option<String> = None;
            let mut cmd_name = None;
            let mut cmd_aliases = Vec::new();
            let mut cmd_capability: Option<String> = None;
            let mut cmd_description: Option<String> = None;
            let mut cmd_usage: Option<String> = None;
            let mut cmd_requires: Vec<String> = Vec::new();
            let mut macro_error: Option<syn::Error> = None;

            // Retain attributes that are NOT our custom ones
            method.attrs.retain(|fn_attr| {
                if fn_attr.path().is_ident("on_load") {
                    is_on_load = true;
                    false
                } else if fn_attr.path().is_ident("on_unload") {
                    is_on_unload = true;
                    false
                } else if fn_attr.path().is_ident("on_frame") {
                    is_on_frame = true;
                    false
                } else if fn_attr.path().is_ident("permissions")
                    || fn_attr.path().is_ident("permission")
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
                } else if fn_attr.path().is_ident("event") {
                    is_on_event = true;
                    if let Ok(Lit::Str(s)) = fn_attr.parse_args::<Lit>() {
                        event_name = Some(s.value());
                    } else if let Ok(meta_list) = fn_attr.meta.require_list() {
                        let _ = meta_list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("name") {
                                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                                    event_name = Some(s.value());
                                }
                            }
                            Ok(())
                        });
                    }
                    false
                } else if fn_attr.path().is_ident("command") {
                    if let Ok(meta_list) = fn_attr.meta.require_list() {
                        let res = meta_list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("name") {
                                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                                    cmd_name = Some(s.value());
                                }
                            } else if meta.path.is_ident("capability") {
                                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                                    let cap_val = s.value();
                                    if let Err(err) = ::goldsrc_api::auth::CapExpr::parse(&cap_val)
                                    {
                                        return Err(meta.error(format!(
                                            "invalid capability expression '{cap_val}': {err}"
                                        )));
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
                                if let Ok(ExprArray { elems, .. }) =
                                    meta.value()?.parse::<ExprArray>()
                                {
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
                                if let Ok(ExprArray { elems, .. }) =
                                    meta.value()?.parse::<ExprArray>()
                                {
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
                        });
                        if let Err(e) = res {
                            macro_error = Some(e);
                        }
                    }
                    false
                } else if fn_attr.path().is_ident("system") {
                    let mut stage_name = "frame".to_string();
                    let mut phase_name = "execute".to_string();
                    let mut before_list: Vec<String> = Vec::new();
                    let mut after_list: Vec<String> = Vec::new();
                    if let Ok(Lit::Str(s)) = fn_attr.parse_args::<Lit>() {
                        stage_name = s.value();
                    } else if let Ok(meta_list) = fn_attr.meta.require_list() {
                        let res = meta_list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("stage") {
                                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                                    stage_name = s.value();
                                }
                            } else if meta.path.is_ident("phase") {
                                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                                    phase_name = s.value();
                                }
                            } else if meta.path.is_ident("before") {
                                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                                    before_list.push(s.value());
                                } else if let Ok(ExprArray { elems, .. }) =
                                    meta.value()?.parse::<ExprArray>()
                                {
                                    for elem in elems {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Str(s), ..
                                        }) = elem
                                        {
                                            before_list.push(s.value());
                                        }
                                    }
                                }
                            } else if meta.path.is_ident("after") {
                                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                                    after_list.push(s.value());
                                } else if let Ok(ExprArray { elems, .. }) =
                                    meta.value()?.parse::<ExprArray>()
                                {
                                    for elem in elems {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Str(s), ..
                                        }) = elem
                                        {
                                            after_list.push(s.value());
                                        }
                                    }
                                }
                            }
                            Ok(())
                        });
                        if let Err(e) = res {
                            macro_error = Some(e);
                        }
                    }
                    let mut takes_player = false;
                    if let Some(syn::FnArg::Typed(pat_type)) = method.sig.inputs.first() {
                        let ty_str = quote!(#pat_type).to_string();
                        if ty_str.contains("Player") {
                            takes_player = true;
                        }
                    }

                    system_handlers.push(SystemDefInfo {
                        stage: stage_name,
                        phase: phase_name,
                        before: before_list,
                        after: after_list,
                        ident: method.sig.ident.clone(),
                        inputs_len: method.sig.inputs.len(),
                        takes_player,
                    });
                    false
                } else if fn_attr.path().is_ident("menu_action") {
                    let mut action_id = None;
                    let mut action_str = None;
                    if let Ok(Lit::Int(i)) = fn_attr.parse_args::<Lit>() {
                        if let Ok(val) = i.base10_parse::<u32>() {
                            action_id = Some(val);
                        }
                    } else if let Ok(Lit::Str(s)) = fn_attr.parse_args::<Lit>() {
                        action_str = Some(s.value());
                    } else if let Ok(meta_list) = fn_attr.meta.require_list() {
                        let res = meta_list.parse_nested_meta(|meta| {
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
                        });
                        if let Err(e) = res {
                            macro_error = Some(e);
                        }
                    }
                    if let Some(id) = action_id {
                        menu_action_matchers.push((
                            Some(id),
                            None,
                            method.sig.ident.clone(),
                            method.sig.inputs.len(),
                        ));
                    } else if let Some(act) = action_str {
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        std::hash::Hash::hash(&act, &mut hasher);
                        let calculated_id =
                            (std::hash::Hasher::finish(&hasher) & 0x7FFF_FFFF) as u32;
                        menu_action_matchers.push((
                            Some(calculated_id),
                            Some(act),
                            method.sig.ident.clone(),
                            method.sig.inputs.len(),
                        ));
                    } else {
                        let method_name = method.sig.ident.to_string();
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        std::hash::Hash::hash(&method_name, &mut hasher);
                        let calculated_id =
                            (std::hash::Hasher::finish(&hasher) & 0x7FFF_FFFF) as u32;
                        menu_action_matchers.push((
                            Some(calculated_id),
                            Some(method_name),
                            method.sig.ident.clone(),
                            method.sig.inputs.len(),
                        ));
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
                if !registered_events.insert(event_name.clone()) {
                    return syn::Error::new_spanned(
                        method,
                        format!("duplicate handler for event {:?}", event_name),
                    )
                    .to_compile_error()
                    .into();
                }
                event_handlers.push((event_name, fn_name.clone(), inputs_len));
            }
            if let Some(cmd) = cmd_name {
                command_defs.push(CommandDefInfo {
                    name: cmd.clone(),
                    description: cmd_description.clone().unwrap_or_default(),
                    usage: cmd_usage.clone().unwrap_or_default(),
                    aliases: cmd_aliases.clone(),
                    capability: cmd_capability.clone(),
                    requires: cmd_requires.clone(),
                });

                let is_raw_signature = match inputs_len {
                    0 => true,
                    1 => {
                        if let Some(syn::FnArg::Typed(pat_type)) = method.sig.inputs.first() {
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
                        for input in &method.sig.inputs {
                            if let syn::FnArg::Typed(pat_type) = input {
                                if let syn::Type::Path(type_path) = &*pat_type.ty {
                                    if !type_path.path.is_ident("String")
                                        && !type_path.path.is_ident("str")
                                    {
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
                    let total_non_context_params = method
                        .sig
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

                    for input in &method.sig.inputs {
                        if let syn::FnArg::Typed(pat_type) = input {
                            let ident = match &*pat_type.pat {
                                syn::Pat::Ident(p) => &p.ident,
                                _ => {
                                    return syn::Error::new_spanned(
                                        pat_type,
                                        "unsupported parameter pattern",
                                    )
                                    .to_compile_error()
                                    .into();
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

                let cap_builder = if let Some(cap) = &cmd_capability {
                    quote! { .capability(#cap) }
                } else {
                    quote! {}
                };
                let desc_builder = if !cmd_description.as_deref().unwrap_or_default().is_empty() {
                    let d = cmd_description.as_deref().unwrap();
                    quote! { .description(#d) }
                } else {
                    quote! {}
                };
                let usage_builder = if !cmd_usage.as_deref().unwrap_or_default().is_empty() {
                    let u = cmd_usage.as_deref().unwrap();
                    quote! { .usage(#u) }
                } else {
                    quote! {}
                };

                command_registrations.push(quote! {
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
                });
            }
        }
    }

    let on_command_fn = quote! {
        ::goldsrc::command::dispatch_command(&name, caller, &args)
    };

    let mut event_registrations = Vec::new();
    for (name, fn_name, inputs_len) in event_handlers {
        let call = match inputs_len {
            0 => quote! { |_payload| { #struct_name::#fn_name(); } },
            1 => quote! { |payload| { #struct_name::#fn_name(payload.to_vec()); } },
            _ => {
                let n_str = name.clone().unwrap_or_default();
                quote! { |payload| { #struct_name::#fn_name(#n_str.to_string(), payload.to_vec()); } }
            }
        };
        let ev_name = name.unwrap_or_default();
        event_registrations.push(quote! {
            ::goldsrc::event::Event::subscriber(#ev_name)
                .subscribe(#call);
        });
    }

    let mut menu_registrations = Vec::new();
    for (id_opt, act_opt, fn_name, inputs_len) in menu_action_matchers {
        let call = match inputs_len {
            0 => quote! { |_player, _action| { #struct_name::#fn_name(); } },
            1 => quote! { |mut player, _action| { #struct_name::#fn_name(&mut player); } },
            _ => {
                let act_str = act_opt.clone().unwrap_or_default();
                quote! { |mut player, _action| { #struct_name::#fn_name(&mut player, #act_str); } }
            }
        };
        if let Some(id) = id_opt {
            menu_registrations.push(quote! {
                ::goldsrc::menu::register_menu_action_id(#id, #call);
            });
        }
        if let Some(act) = act_opt {
            menu_registrations.push(quote! {
                ::goldsrc::menu::register_menu_action_name(#act, #call);
            });
        }
    }

    let mut system_registrations = Vec::new();
    for sys in system_handlers {
        let sys_ident = sys.ident;
        let sys_name = sys_ident.to_string();
        let stage_str = sys.stage;
        let phase_str = sys.phase;
        let before_strs = sys.before;
        let after_strs = sys.after;

        let runner_fn = if sys.inputs_len == 0 {
            quote! { |_world: &mut ::goldsrc::ecs::World, _target: Option<::goldsrc::ecs::EntityId>| {
                #struct_name::#sys_ident();
            }}
        } else if sys.inputs_len == 1 {
            if sys.takes_player {
                quote! { |_world: &mut ::goldsrc::ecs::World, target: Option<::goldsrc::ecs::EntityId>| {
                    if let Some(target_id) = target {
                        let mut p = ::goldsrc::Player::new(target_id.0 as i32);
                        #struct_name::#sys_ident(&mut p);
                    }
                }}
            } else {
                quote! { |world: &mut ::goldsrc::ecs::World, _target: Option<::goldsrc::ecs::EntityId>| {
                    #struct_name::#sys_ident(world);
                }}
            }
        } else {
            quote! { |world: &mut ::goldsrc::ecs::World, target: Option<::goldsrc::ecs::EntityId>| {
                #struct_name::#sys_ident(world, target);
            }}
        };

        system_registrations.push(quote! {
            ::goldsrc::ecs::System::builder(#sys_name)
                .stage(#stage_str.parse::<::goldsrc::ecs::Stage>().unwrap_or(::goldsrc::ecs::Stage::Frame))
                .phase(#phase_str.parse::<::goldsrc::ecs::SystemPhase>().unwrap_or(::goldsrc::ecs::SystemPhase::Execute))
                .before(vec![#(#before_strs),*])
                .after(vec![#(#after_strs),*])
                .register(#runner_fn);
        });
    }

    let mut requires_toml = String::new();
    if !attr.requires.is_empty() {
        let reqs: Vec<String> = attr
            .requires
            .iter()
            .map(|d| format!("\"{}\"", toml_escape(d)))
            .collect();
        requires_toml = format!("requires = [{}]\n", reqs.join(", "));
    }

    let mut permissions_toml = String::new();
    if !attr.permissions.is_empty() {
        let perms: Vec<String> = attr
            .permissions
            .iter()
            .map(|p| format!("\"{}\"", toml_escape(p)))
            .collect();
        permissions_toml = format!("permissions = [{}]\n", perms.join(", "));
    }

    let mut commands_toml = String::new();
    if !command_defs.is_empty() {
        commands_toml.push('\n');
        for cmd in &command_defs {
            commands_toml.push_str("[[commands]]\n");
            commands_toml.push_str(&format!("name = \"{}\"\n", toml_escape(&cmd.name)));
            if !cmd.description.is_empty() {
                commands_toml.push_str(&format!(
                    "description = \"{}\"\n",
                    toml_escape(&cmd.description)
                ));
            }
            if !cmd.usage.is_empty() {
                commands_toml.push_str(&format!("usage = \"{}\"\n", toml_escape(&cmd.usage)));
            }
            if !cmd.aliases.is_empty() {
                let aliases_str: Vec<String> = cmd
                    .aliases
                    .iter()
                    .map(|a| format!("\"{}\"", toml_escape(a)))
                    .collect();
                commands_toml.push_str(&format!("aliases = [{}]\n", aliases_str.join(", ")));
            }
            if let Some(cap) = &cmd.capability {
                commands_toml.push_str(&format!("capability = \"{}\"\n", toml_escape(cap)));
            }
            if !cmd.requires.is_empty() {
                let req_str: Vec<String> = cmd
                    .requires
                    .iter()
                    .map(|r| format!("\"{}\"", toml_escape(r)))
                    .collect();
                commands_toml.push_str(&format!("requires = [{}]\n", req_str.join(", ")));
            }
            commands_toml.push('\n');
        }
    }

    let bundle_field = match &attr.bundle {
        Some(b) => format!("bundle = \"{}\"\n", toml_escape(b)),
        None => String::new(),
    };

    let lifecycle_toml = format!(
        "[lifecycle]\nload = \"{}\"\nunload = \"{}\"\n",
        toml_escape(&attr.load_time),
        toml_escape(&attr.unload_time)
    );

    let manifest_toml = format!(
        "[plugin]\nname = \"{}\"\nversion = \"{}\"\nauthor = \"{}\"\ndescription = \"{}\"\nurl = \"{}\"\nlicense = \"{}\"\n{}{}{}{}{}",
        toml_escape(&plugin_name),
        toml_escape(&plugin_version),
        toml_escape(&plugin_author),
        toml_escape(&attr.description),
        toml_escape(&attr.url),
        toml_escape(&attr.license),
        bundle_field,
        requires_toml,
        permissions_toml,
        lifecycle_toml,
        commands_toml
    );

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
                ::goldsrc::ecs::run_frame_systems();
                #on_frame_fn
            }

            fn on_event(name: String, payload: Vec<u8>) {
                if name == "menu_select" && payload.len() >= 8 {
                    let caller = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let item_id = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
                    ::goldsrc::menu::dispatch_menu_action(::goldsrc::Player::new(caller), Some(item_id), None);
                }
                ::goldsrc::event::dispatch_event(&name, &payload);
            }

            fn on_command(name: String, caller: i32, args: String) -> bool {
                #on_command_fn
            }

            fn on_placeholder(name: String, caller: i32, param: String) -> Option<String> {
                ::goldsrc::placeholders::dispatch_local_placeholder(&name, caller, &param)
            }

            fn on_chat(_sender: i32, _text: String, _is_team: bool) -> Option<String> {
                None
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
