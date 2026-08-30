use goldsrc_api::chat::{ChatMessage, ChatScope};
#[cfg(feature = "host")]
use goldsrc_api::chat::{LifeStateFilter, TeamTarget, split_chat_chunks};
use goldsrc_api::client::{LifeState, Player, Team};
use std::sync::{Arc, LazyLock, RwLock};

/// Type definition for a chat filter middleware handler.
pub type ChatMiddleware = Arc<dyn Fn(&mut ChatMessage) -> bool + Send + Sync>;

/// Global chat processing pipeline registry.
static CHAT_PIPELINE: LazyLock<RwLock<Vec<ChatMiddleware>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

/// Registers a custom chat filter / middleware in the global pipeline.
pub fn register_chat_middleware<F>(middleware: F)
where
    F: Fn(&mut ChatMessage) -> bool + Send + Sync + 'static,
{
    let mut pipeline = match CHAT_PIPELINE.write() {
        Ok(p) => p,
        Err(e) => e.into_inner(),
    };
    pipeline.push(Arc::new(middleware));
}

/// Dispatches local chat middleware pipeline inside a WASM plugin.
pub fn dispatch_local_chat_middleware(
    sender_idx: i32,
    text: &str,
    is_team: bool,
) -> Option<String> {
    let sender = Player::new(sender_idx);
    let scope = if is_team {
        ChatScope::same_team()
    } else {
        ChatScope::all()
    };
    let mut msg = ChatMessage::new(sender, text, scope);
    let pipeline = match CHAT_PIPELINE.read() {
        Ok(p) => p,
        Err(e) => e.into_inner(),
    };
    for middleware in pipeline.iter() {
        let continue_processing = middleware(&mut msg);
        if !continue_processing || msg.is_blocked {
            return None;
        }
    }
    if let Some(ref prefix) = msg.prefix {
        Some(format!("{prefix}__PREFIX_SPLIT__{}", msg.formatted_text))
    } else {
        Some(msg.formatted_text)
    }
}

/// Dispatches an incoming `say` or `say_team` text command through the chat processing pipeline.
/// Broadcasts to eligible recipients matching the message's `ChatScope`.
/// Returns whether the message was handled and should be blocked from the vanilla engine.
pub fn process_chat_message(sender: Player, raw_text: &str, scope: ChatScope) -> bool {
    #[cfg(feature = "host")]
    {
        process_chat_message_with_manager(None, sender, raw_text, scope)
    }
    #[cfg(not(feature = "host"))]
    {
        let mut msg = ChatMessage::new(sender, raw_text, scope);
        let pipeline = match CHAT_PIPELINE.read() {
            Ok(p) => p,
            Err(e) => e.into_inner(),
        };
        for middleware in pipeline.iter() {
            if !middleware(&mut msg) || msg.is_blocked {
                return true;
            }
        }
        false
    }
}

/// Dispatches a chat message using an optional pre-locked `PluginManager` to avoid re-entrant mutex deadlocks.
#[cfg(feature = "host")]
pub fn process_chat_message_with_manager(
    mut manager: Option<&mut goldsrc_wasm_host::PluginManager>,
    sender: Player,
    raw_text: &str,
    scope: ChatScope,
) -> bool {
    let mut msg = ChatMessage::new(sender, raw_text, scope);

    // 1. Run native host middleware pipeline (censorship, ranks, custom prefixes)
    let pipeline = match CHAT_PIPELINE.read() {
        Ok(p) => p,
        Err(e) => e.into_inner(),
    };

    for middleware in pipeline.iter() {
        let continue_processing = middleware(&mut msg);
        if !continue_processing || msg.is_blocked {
            return true; // blocked by native middleware
        }
    }

    // 2. Run WASM plugins chat middleware
    #[cfg(feature = "host")]
    {
        let is_team = matches!(msg.scope.team, TeamTarget::SameTeam);
        let wasm_result = if let Some(ref mut m) = manager {
            Some(m.dispatch_chat(sender.index(), &msg.formatted_text, is_team))
        } else {
            crate::host::HostRuntime::with_manager(|mgr| {
                mgr.map(|m| m.dispatch_chat(sender.index(), &msg.formatted_text, is_team))
            })
        };

        match wasm_result {
            Some(Some(transformed)) => {
                if let Some((prefix, rest)) = transformed.split_once("__PREFIX_SPLIT__") {
                    msg.prefix = Some(prefix.to_string());
                    msg.formatted_text = rest.to_string();
                } else {
                    msg.formatted_text = transformed;
                }
            }
            Some(None) => {
                return true; // blocked by WASM middleware
            }
            None => {}
        }
    }

    // 3. Run placeholder expansion
    let interpolated =
        crate::placeholders::format_placeholders_with_manager(&msg.formatted_text, sender, manager);
    msg.formatted_text = interpolated;

    // 3. Render final output with player name and prefix
    let sender_name = sender
        .name()
        .unwrap_or_else(|| format!("Player#{}", sender.index()));

    let full_text = match msg.scope.team {
        TeamTarget::SameTeam => {
            if let Some(ref prefix) = msg.prefix {
                format!(
                    "{prefix}^2(TEAM)^1 ^3{sender_name}^1 :  {}",
                    msg.formatted_text
                )
            } else {
                format!("^2(TEAM)^1 ^3{sender_name}^1 :  {}", msg.formatted_text)
            }
        }
        _ => {
            if let Some(ref prefix) = msg.prefix {
                format!("{prefix}^3{sender_name}^1 :  {}", msg.formatted_text)
            } else {
                format!("^3{sender_name}^1 :  {}", msg.formatted_text)
            }
        }
    };

    // 4. Split message into safe 180-byte chunks
    let chunks = split_chat_chunks(&full_text);

    // [DEBUG] Log what we're about to broadcast
    #[cfg(feature = "host")]
    if let Some(engine) = crate::host::HostRuntime::engine() {
        engine.server_print(&format!(
            "[chat-dbg] broadcasting {} chunk(s), scope={:?}, full_text='{full_text}'\n",
            chunks.len(),
            msg.scope.team
        ));
    }

    // 5. Broadcast chunks to target recipients based on ChatScope
    let sender_team = sender.team();
    match msg.scope.team {
        TeamTarget::Direct(slot) => {
            let target = Player::new(slot);
            if target.is_valid() && matches_lifestate(target, msg.scope.state) {
                for chunk in &chunks {
                    target.print_chat(chunk);
                }
            }
        }
        TeamTarget::All => {
            for i in 1..=32 {
                let target = Player::new(i);
                let valid = target.is_valid();
                #[cfg(feature = "host")]
                if let Some(engine) = crate::host::HostRuntime::engine()
                    && valid
                {
                    engine.server_print(&format!("[chat-dbg] sending to slot {i} (valid)\n"));
                }
                if valid && matches_lifestate(target, msg.scope.state) {
                    for chunk in &chunks {
                        target.print_chat(chunk);
                    }
                }
            }
        }
        TeamTarget::SameTeam => {
            for i in 1..=32 {
                let target = Player::new(i);
                if target.is_valid()
                    && target.team() == sender_team
                    && matches_lifestate(target, msg.scope.state)
                {
                    for chunk in &chunks {
                        target.print_chat(chunk);
                    }
                }
            }
        }
        TeamTarget::OppositeTeam => {
            for i in 1..=32 {
                let target = Player::new(i);
                if target.is_valid()
                    && is_opposite_team(sender_team, target.team())
                    && matches_lifestate(target, msg.scope.state)
                {
                    for chunk in &chunks {
                        target.print_chat(chunk);
                    }
                }
            }
        }
    }

    true // Handled by GoldSrc.rs chat engine
}

#[cfg(feature = "host")]
fn matches_lifestate(player: Player, filter: LifeStateFilter) -> bool {
    match filter {
        LifeStateFilter::Any => true,
        LifeStateFilter::AliveOnly => player.life_state() == LifeState::Alive,
        LifeStateFilter::DeadOnly => player.life_state() != LifeState::Alive,
    }
}

#[cfg(feature = "host")]
fn is_opposite_team(a: Team, b: Team) -> bool {
    matches!(
        (a, b),
        (Team::Terrorist, Team::CounterTerrorist) | (Team::CounterTerrorist, Team::Terrorist)
    )
}
