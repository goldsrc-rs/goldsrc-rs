//! In-game Chat Interception, Filtering Pipeline, and Safe Packet Chunking.

use goldsrc_api::chat::{ChatMessage, ChatScope, LifeStateFilter, TeamTarget, split_chat_chunks};
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

/// Dispatches an incoming `say` or `say_team` text command through the chat processing pipeline.
/// Broadcasts to eligible recipients matching the message's `ChatScope`.
/// Returns whether the message was handled and should be blocked from the vanilla engine.
pub fn process_chat_message(sender: Player, raw_text: &str, scope: ChatScope) -> bool {
    let mut msg = ChatMessage::new(sender, raw_text, scope);

    // 1. Run placeholder expansion
    let interpolated = crate::placeholders::format_placeholders(&msg.formatted_text, sender);
    msg.formatted_text = interpolated;

    // 2. Run middleware pipeline (censorship, ranks, custom prefixes)
    let pipeline = match CHAT_PIPELINE.read() {
        Ok(p) => p,
        Err(e) => e.into_inner(),
    };

    for middleware in pipeline.iter() {
        let continue_processing = middleware(&mut msg);
        if !continue_processing || msg.is_blocked {
            return true; // blocked by middleware
        }
    }

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

fn matches_lifestate(player: Player, filter: LifeStateFilter) -> bool {
    match filter {
        LifeStateFilter::Any => true,
        LifeStateFilter::AliveOnly => player.life_state() == LifeState::Alive,
        LifeStateFilter::DeadOnly => player.life_state() != LifeState::Alive,
    }
}

fn is_opposite_team(a: Team, b: Team) -> bool {
    matches!(
        (a, b),
        (Team::Terrorist, Team::CounterTerrorist) | (Team::CounterTerrorist, Team::Terrorist)
    )
}
