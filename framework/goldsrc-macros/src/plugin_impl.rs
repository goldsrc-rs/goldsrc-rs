//! Plugin implementation generator for `#[plugin]`.

use crate::command_def::CommandDefInfo;
use crate::plugin_attr::PluginAttr;
use crate::system_def::SystemDefInfo;
use crate::utils::{check_handler_args, toml_escape};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprArray, ExprLit, ImplItem, ItemImpl, Lit};

pub fn expand_plugin(attr: PluginAttr, mut input_impl: ItemImpl) -> TokenStream {
    let struct_name = &input_impl.self_ty;

    let plugin_name = attr.name;
    let plugin_version = attr.version;
    let plugin_author = attr.author;

    let mut require_toml = String::new();
    if !attr.require.is_empty() {
        let reqs: Vec<String> = attr
            .require
            .iter()
            .map(|d| format!("\"{}\"", toml_escape(d)))
            .collect();
        require_toml = format!("require = [{}]\n", reqs.join(", "));
    }

    let mut on_load_fn = quote! {};
    let mut on_unload_fn = quote! {};
    let mut on_frame_fn = quote! {};
    let mut on_command_fn = quote! {};
    let mut event_handlers: Vec<(Option<String>, syn::Ident, usize)> = Vec::new();
    let mut registered_events: std::collections::HashSet<Option<String>> =
        std::collections::HashSet::new();

    let mut command_matchers = Vec::new();
    let mut plugin_commands: Vec<String> = Vec::new();
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
            let mut macro_error: Option<syn::Error> = None;

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
                    false
                } else if attr.path().is_ident("command") {
                    if let Ok(meta_list) = attr.meta.require_list() {
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
                } else if attr.path().is_ident("system") {
                    let mut stage_name = "frame".to_string();
                    let mut phase_name = "execute".to_string();
                    let mut before_list: Vec<String> = Vec::new();
                    let mut after_list: Vec<String> = Vec::new();
                    if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
                        stage_name = s.value();
                    } else if let Ok(meta_list) = attr.meta.require_list() {
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
                    system_handlers.push(SystemDefInfo {
                        stage: stage_name,
                        phase: phase_name,
                        before: before_list,
                        after: after_list,
                        ident: method.sig.ident.clone(),
                        inputs_len: method.sig.inputs.len(),
                    });
                    false
                } else if attr.path().is_ident("menu_action") {
                    let mut action_id = None;
                    let mut action_str = None;
                    if let Ok(Lit::Int(i)) = attr.parse_args::<Lit>() {
                        if let Ok(val) = i.base10_parse::<u32>() {
                            action_id = Some(val);
                        }
                    } else if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
                        action_str = Some(s.value());
                    } else if let Ok(meta_list) = attr.meta.require_list() {
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
                        // Bare #[menu_action] without attributes: default action name is function name!
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
                plugin_commands.push(cmd.clone());
                for alias in &cmd_aliases {
                    plugin_commands.push(alias.clone());
                }
                command_defs.push(CommandDefInfo {
                    name: cmd.clone(),
                    description: cmd_description.unwrap_or_default(),
                    usage: cmd_usage.unwrap_or_default(),
                    aliases: cmd_aliases.clone(),
                    capability: cmd_capability.clone(),
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
                        0 => quote! { #struct_name::#fn_name(); },
                        1 => quote! { #struct_name::#fn_name(args); },
                        _ => quote! { #struct_name::#fn_name(name, args); },
                    }
                } else {
                    let mut param_bindings = Vec::new();
                    let mut param_idents = Vec::new();

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
                            if ident == "caller" || ident == "caller_idx" {
                                param_bindings.push(quote! {
                                    let mut #ident: #ty = caller;
                                });
                            } else {
                                param_bindings.push(quote! {
                                    let __next_tok = __iter.next();
                                    let __tok_str = match __next_tok {
                                        Some(t) if !t.is_empty() => t.to_string(),
                                        _ if caller > 0 => caller.to_string(),
                                        _ => String::new(),
                                    };
                                    let mut #ident: #ty = match ::goldsrc::FromArg::from_arg(&__tok_str) {
                                        Ok(val) => val,
                                        Err(err) => {
                                            ::goldsrc::log_warn!("[Command '{}'] Parameter '{}' invalid: {}", name, stringify!(#ident), err);
                                            return true;
                                        }
                                    };
                                });
                            }
                        }
                    }

                    let is_result = match &method.sig.output {
                        syn::ReturnType::Default => false,
                        syn::ReturnType::Type(_, _) => true,
                    };

                    if is_result {
                        quote! {
                            let mut __iter = args.split_whitespace();
                            #(#param_bindings)*
                            if let Err(err) = #struct_name::#fn_name(#(#param_idents),*) {
                                ::goldsrc::log_warn!("[Command '{}'] Error: {}", name, err);
                            }
                        }
                    } else {
                        quote! {
                            let mut __iter = args.split_whitespace();
                            #(#param_bindings)*
                            #struct_name::#fn_name(#(#param_idents),*);
                        }
                    }
                };

                let handler_body = if let Some(ref cap_str) = cmd_capability {
                    quote! {
                        if caller > 0 {
                            let allowed = if let Ok(ast) = ::goldsrc::CapExpr::parse(#cap_str) {
                                ast.evaluate(&|c| ::goldsrc::Auth::has_capability(caller, c))
                            } else {
                                false
                            };
                            if !allowed {
                                ::goldsrc::log_warn!("[Auth] Access denied for player #{}: requires '{}'", caller, #cap_str);
                                return true;
                            }
                        }
                        #call_expr
                    }
                } else {
                    quote! { #call_expr }
                };

                let mut all_match_names = vec![cmd.clone()];
                all_match_names.extend(cmd_aliases);

                for match_name in all_match_names {
                    command_matchers.push(quote! {
                        #match_name => { #handler_body },
                    });
                }
            }
        }
    }

    let mut event_arms = Vec::new();
    let mut fallback_event = quote! {};
    for (evt_name, f_name, in_len) in event_handlers {
        let call_expr = match in_len {
            0 => quote! { #struct_name::#f_name() },
            1 => quote! { #struct_name::#f_name(name.clone()) },
            _ => quote! { #struct_name::#f_name(name.clone(), payload.clone()) },
        };
        if let Some(n) = evt_name {
            event_arms.push(quote! {
                #n => { #call_expr; }
            });
        } else {
            fallback_event = quote! { #call_expr; };
        }
    }

    // Sort systems by Phase + topological DAG dependencies (before / after)
    let phase_val = |p: &str| -> i32 {
        match p.to_ascii_lowercase().as_str() {
            "validate" => 0,
            "modify" => 10,
            "execute" => 20,
            "react" => 30,
            "monitor" => 40,
            _ => 20,
        }
    };

    system_handlers.sort_by_key(|s| (s.stage.clone(), phase_val(&s.phase)));

    // Intra-phase topological DAG sort
    let n = system_handlers.len();
    if n > 1 {
        let mut name_to_idx = std::collections::HashMap::new();
        for (idx, sys) in system_handlers.iter().enumerate() {
            name_to_idx.insert(sys.ident.to_string(), idx);
        }

        let mut in_degree = vec![0; n];
        let mut adj = vec![Vec::new(); n];

        for (u, sys) in system_handlers.iter().enumerate() {
            for after_name in &sys.after {
                if let Some(&v) = name_to_idx.get(after_name) {
                    adj[v].push(u);
                    in_degree[u] += 1;
                }
            }
            for before_name in &sys.before {
                if let Some(&v) = name_to_idx.get(before_name) {
                    adj[u].push(v);
                    in_degree[v] += 1;
                }
            }
        }

        let mut queue = std::collections::VecDeque::new();
        for (idx, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(idx);
            }
        }

        let mut sorted = Vec::with_capacity(n);
        while let Some(u) = queue.pop_front() {
            sorted.push(system_handlers[u].clone());
            for &v in &adj[u] {
                in_degree[v] -= 1;
                if in_degree[v] == 0 {
                    queue.push_back(v);
                }
            }
        }

        if sorted.len() == n {
            system_handlers = sorted;
        }
    }

    let mut system_frame_calls = Vec::new();
    let mut system_think_calls = Vec::new();
    let mut system_connect_calls = Vec::new();
    let mut system_disconnect_calls = Vec::new();

    for sys in &system_handlers {
        let f_name = &sys.ident;
        let in_len = sys.inputs_len;
        match sys.stage.as_str() {
            "frame" => {
                let call = match in_len {
                    0 => quote! { #struct_name::#f_name(); },
                    _ => quote! { #struct_name::#f_name(&mut __world); },
                };
                system_frame_calls.push(call);
            }
            "post_think" => {
                let call = match in_len {
                    0 => quote! { #struct_name::#f_name(); },
                    1 => quote! { #struct_name::#f_name(&mut __player); },
                    _ => quote! { #struct_name::#f_name(&mut __player, &mut __world); },
                };
                system_think_calls.push(call);
            }
            "player_connect" => {
                let call = match in_len {
                    0 => quote! { #struct_name::#f_name(); },
                    1 => quote! { #struct_name::#f_name(&mut __player); },
                    _ => quote! { #struct_name::#f_name(&mut __player, &mut __world); },
                };
                system_connect_calls.push(call);
            }
            "player_disconnect" => {
                let call = match in_len {
                    0 => quote! { #struct_name::#f_name(); },
                    1 => quote! { #struct_name::#f_name(&mut __player); },
                    _ => quote! { #struct_name::#f_name(&mut __player, &mut __world); },
                };
                system_disconnect_calls.push(call);
            }
            _ => {}
        }
    }

    if !system_frame_calls.is_empty() {
        let prev_frame = on_frame_fn;
        on_frame_fn = quote! {
            #prev_frame
            let mut __world = ::goldsrc::ecs::World::new();
            #(#system_frame_calls)*
        };
    }

    if !system_think_calls.is_empty() {
        event_arms.push(quote! {
            "player_post_think" => {
                if payload.len() >= 4 {
                    let __idx = i32::from_le_bytes(payload[0..4].try_into().unwrap_or_default());
                    let mut __player = ::goldsrc::Player::new(__idx);
                    let mut __world = ::goldsrc::ecs::World::new();
                    #(#system_think_calls)*
                }
            }
        });
    }

    if !system_connect_calls.is_empty() {
        event_arms.push(quote! {
            "client_connect" => {
                if payload.len() >= 4 {
                    let __idx = i32::from_le_bytes(payload[0..4].try_into().unwrap_or_default());
                    let mut __player = ::goldsrc::Player::new(__idx);
                    let mut __world = ::goldsrc::ecs::World::new();
                    #(#system_connect_calls)*
                }
            }
        });
    }

    if !system_disconnect_calls.is_empty() {
        event_arms.push(quote! {
            "client_disconnect" => {
                if payload.len() >= 4 {
                    let __idx = i32::from_le_bytes(payload[0..4].try_into().unwrap_or_default());
                    let mut __player = ::goldsrc::Player::new(__idx);
                    let mut __world = ::goldsrc::ecs::World::new();
                    #(#system_disconnect_calls)*
                }
            }
        });
    }

    if !menu_action_matchers.is_empty() {
        let mut id_branches = Vec::new();
        let mut name_branches = Vec::new();

        for (id_opt, name_opt, f_name, in_len) in menu_action_matchers {
            let invoker = match in_len {
                0 => quote! { #struct_name::#f_name(); },
                1 => quote! { #struct_name::#f_name(&mut __player); },
                _ => quote! { #struct_name::#f_name(&mut __player, __action_id); },
            };

            if let Some(id) = id_opt {
                id_branches.push(quote! {
                    #id => { #invoker }
                });
            }
            if let Some(name_str) = name_opt {
                name_branches.push(quote! {
                    #name_str => { #invoker }
                });
            }
        }

        event_arms.push(quote! {
            "menu_select" => {
                if payload.len() >= 8 {
                    let __player_idx = i32::from_le_bytes(payload[0..4].try_into().unwrap_or_default());
                    let __slot_or_id = u32::from_le_bytes(payload[4..8].try_into().unwrap_or_default());
                    let mut __player = ::goldsrc::Player::new(__player_idx);
                    if let Some(__action) = ::goldsrc::api::menu::handle_menu_slot(__player_idx, __slot_or_id as u8) {
                        if let ::goldsrc::api::menu::SlotAction::Execute { id: __action_id, action_name: ref __action_name, .. } = __action {
                            match __action_id {
                                #(#id_branches)*
                                _ => {}
                            }
                            if !__action_name.is_empty() {
                                match __action_name.as_str() {
                                    #(#name_branches)*
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    let on_event_fn = quote! {
        match name.as_str() {
            #(#event_arms)*
            _ => { #fallback_event }
        }
    };

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

    let mut command_defs_toml = String::new();
    for cmd in &command_defs {
        command_defs_toml.push_str("\n[[command_defs]]\n");
        command_defs_toml.push_str(&format!("name = \"{}\"\n", toml_escape(&cmd.name)));
        if !cmd.description.is_empty() {
            command_defs_toml.push_str(&format!(
                "description = \"{}\"\n",
                toml_escape(&cmd.description)
            ));
        }
        if !cmd.usage.is_empty() {
            command_defs_toml.push_str(&format!("usage = \"{}\"\n", toml_escape(&cmd.usage)));
        }
        if !cmd.aliases.is_empty() {
            let aliases_fmt: Vec<String> = cmd
                .aliases
                .iter()
                .map(|a| format!("\"{}\"", toml_escape(a)))
                .collect();
            command_defs_toml.push_str(&format!("aliases = [{}]\n", aliases_fmt.join(", ")));
        }
        if let Some(ref c) = cmd.capability {
            command_defs_toml.push_str(&format!("capability = \"{}\"\n", toml_escape(c)));
        }
    }

    let desc_toml = if !attr.description.is_empty() {
        format!("description = \"{}\"\n", toml_escape(&attr.description))
    } else {
        String::new()
    };

    let url_toml = if !attr.url.is_empty() {
        format!("url = \"{}\"\n", toml_escape(&attr.url))
    } else {
        String::new()
    };

    let license_toml = if !attr.license.is_empty() {
        format!("license = \"{}\"\n", toml_escape(&attr.license))
    } else {
        String::new()
    };

    let bundle_toml = if let Some(ref b) = attr.bundle {
        format!("bundle = \"{}\"\n", toml_escape(b))
    } else {
        String::new()
    };

    let mut systems_toml = String::new();
    if !system_handlers.is_empty() {
        let sys_names: Vec<String> = system_handlers
            .iter()
            .map(|s| format!("\"{}\"", toml_escape(&s.ident.to_string())))
            .collect();
        systems_toml = format!("systems = [{}]\n", sys_names.join(", "));
    }

    let meta_toml = format!(
        "name = \"{}\"\nversion = \"{}\"\nauthor = \"{}\"\n{}{}{}{}{}{}{}{}",
        toml_escape(&plugin_name),
        toml_escape(&plugin_version),
        toml_escape(&plugin_author),
        desc_toml,
        url_toml,
        license_toml,
        bundle_toml,
        require_toml,
        systems_toml,
        commands_toml,
        command_defs_toml
    );

    let expanded = quote! {
        #input_impl

        impl ::goldsrc::goldsrc_api::bindings::Guest for #struct_name {
            fn get_metadata() -> String {
                #meta_toml.to_string()
            }

            fn on_load() {
                ::goldsrc::init_guest_logger();
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

            fn on_command(name: String, caller: i32, args: String) -> bool {
                #on_command_fn
            }

            fn on_placeholder(name: String, caller: i32, param: String) -> Option<String> {
                ::goldsrc::placeholders::dispatch_local_placeholder(&name, caller, &param)
            }

            fn on_chat(sender: i32, text: String, is_team: bool) -> Option<String> {
                ::goldsrc::chat::dispatch_local_chat_middleware(sender, &text, is_team)
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
