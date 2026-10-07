use goldsrc_api::chat::{
    ChatMessage, ChatScope, ChatTarget, LifeStateFilter, TeamTarget, split_chat_chunks,
};
use goldsrc_api::client::{LifeState, Player, Team};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use stitch_rs::flow::FlowControl;

/// Type definition for a chat filter middleware handler with SMA U-cycle semantics.
pub trait ChatLayer: Send + Sync {
    /// Descent phase: inspection, censorship, or short-circuiting triggers.
    fn on_enter(&self, msg: &mut ChatMessage) -> FlowControl<(), (), ()>;
    /// Ascent phase: audit logging, telemetry, or notification.
    fn on_exit(&self, msg: &mut ChatMessage, outcome: &mut Result<(), ()>);
}

/// Functional adapter for legacy chat middleware closure.
struct FnChatLayer<F>(F);

impl<F> ChatLayer for FnChatLayer<F>
where
    F: Fn(&mut ChatMessage) -> bool + Send + Sync,
{
    fn on_enter(&self, msg: &mut ChatMessage) -> FlowControl<(), (), ()> {
        if (self.0)(msg) && !msg.is_blocked {
            FlowControl::Proceed(())
        } else {
            FlowControl::Halt(())
        }
    }

    fn on_exit(&self, _msg: &mut ChatMessage, _outcome: &mut Result<(), ()>) {}
}

/// Global chat processing pipeline registry.
static CHAT_PIPELINE: LazyLock<RwLock<Vec<Arc<dyn ChatLayer>>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

/// Pluggable resolver for custom chat target channels.
pub trait ChatTargetResolver: Send + Sync {
    /// Returns true if `recipient` is allowed to see messages sent to `channel` by `sender`.
    fn can_receive(&self, channel: &str, sender: Player, recipient: Player) -> bool;
}

impl<F> ChatTargetResolver for F
where
    F: Fn(&str, Player, Player) -> bool + Send + Sync,
{
    fn can_receive(&self, channel: &str, sender: Player, recipient: Player) -> bool {
        self(channel, sender, recipient)
    }
}

/// Global custom chat channel resolvers registry.
static CUSTOM_RESOLVERS: LazyLock<RwLock<HashMap<String, Arc<dyn ChatTargetResolver>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Registers a custom chat target channel resolver.
pub fn register_chat_target_resolver(channel: &str, resolver: Arc<dyn ChatTargetResolver>) {
    let mut map = CUSTOM_RESOLVERS.write().unwrap_or_else(|e| e.into_inner());
    map.insert(channel.to_ascii_lowercase(), resolver);
}

/// Unregisters a custom chat target channel resolver.
pub fn unregister_chat_target_resolver(channel: &str) {
    let mut map = CUSTOM_RESOLVERS.write().unwrap_or_else(|e| e.into_inner());
    map.remove(&channel.to_ascii_lowercase());
}

/// Registers an SMA chat layer in the global pipeline.
pub fn register_chat_layer(layer: Arc<dyn ChatLayer>) {
    let mut pipeline = match CHAT_PIPELINE.write() {
        Ok(p) => p,
        Err(e) => e.into_inner(),
    };
    pipeline.push(layer);
}

/// Registers a legacy functional chat filter in the global pipeline.
pub fn register_chat_middleware<F>(middleware: F)
where
    F: Fn(&mut ChatMessage) -> bool + Send + Sync + 'static,
{
    register_chat_layer(Arc::new(FnChatLayer(middleware)));
}

/// Pluggable chat trigger that inspects raw incoming chat messages.
/// Subsystems (e.g. WASM Command Dispatcher, Vote System, Keyword Interceptor)
/// register triggers to intercept and consume chat messages before they enter
/// the chat middleware pipeline.
pub trait ChatTrigger: Send + Sync {
    /// Attempts to handle the incoming chat message.
    /// Returns `true` if the message was handled and should be suppressed from server chat.
    fn try_handle(
        &self,
        manager: Option<&mut goldsrc_host_wasm::PluginManager>,
        sender: Player,
        raw_text: &str,
    ) -> bool;
}

/// Global registry of chat message triggers.
static CHAT_TRIGGERS: LazyLock<RwLock<Vec<Arc<dyn ChatTrigger>>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

/// Registers a custom chat trigger in the global registry.
pub fn register_chat_trigger(trigger: Arc<dyn ChatTrigger>) {
    let mut list = CHAT_TRIGGERS.write().unwrap_or_else(|e| e.into_inner());
    list.push(trigger);
}

/// Evaluates if an incoming raw chat message is consumed by any registered chat trigger.
pub fn evaluate_chat_triggers(
    mut manager: Option<&mut goldsrc_host_wasm::PluginManager>,
    sender: Player,
    raw_text: &str,
) -> bool {
    let triggers = match CHAT_TRIGGERS.read() {
        Ok(t) => t.clone(),
        Err(e) => e.into_inner().clone(),
    };

    for trigger in &triggers {
        if trigger.try_handle(manager.as_deref_mut(), sender, raw_text) {
            return true;
        }
    }

    false
}

/// Standard chat trigger adapter routing chat-based commands (e.g. `/vip`, `!vip`, `rtv`)
/// to the WASM host plugin command dispatcher.
pub struct CommandChatTrigger;

impl ChatTrigger for CommandChatTrigger {
    fn try_handle(
        &self,
        mut manager: Option<&mut goldsrc_host_wasm::PluginManager>,
        sender: Player,
        raw_text: &str,
    ) -> bool {
        let trimmed = raw_text.trim();
        if trimmed.is_empty() {
            return false;
        }

        let (first_word, rest) = match trimmed.find(|c: char| c.is_whitespace()) {
            Some(idx) => (&trimmed[..idx], trimmed[idx..].trim_start()),
            None => (trimmed, ""),
        };

        let is_prefixed = first_word.starts_with('/') || first_word.starts_with('!');
        let clean_name = first_word.trim_start_matches(['/', '!']);

        let dispatch = |mgr: &mut goldsrc_host_wasm::PluginManager| -> bool {
            // 1. Try clean command name (e.g. "vip" or "vipmenu")
            if mgr.dispatch_command(clean_name, sender.index(), rest) {
                return true;
            }
            // 2. Try raw word if prefixed (e.g. "/vip")
            if is_prefixed && mgr.dispatch_command(first_word, sender.index(), rest) {
                return true;
            }
            false
        };

        if let Some(ref mut mgr) = manager {
            dispatch(mgr)
        } else {
            crate::host::HostRuntime::with_manager(|mgr| mgr.map(dispatch).unwrap_or(false))
        }
    }
}

/// Traversal runner for SMA U-cycle middleware layers.
fn run_chat_layers(layers: &[Arc<dyn ChatLayer>], msg: &mut ChatMessage) -> Result<(), ()> {
    let mut halted = false;
    let mut entered_count = 0;

    for layer in layers {
        match layer.on_enter(msg) {
            FlowControl::Proceed(()) => {
                entered_count += 1;
            }
            FlowControl::ShortCircuit(()) => {
                entered_count += 1;
                break;
            }
            FlowControl::Halt(()) => {
                halted = true;
                break;
            }
        }
    }

    let mut outcome = if halted || msg.is_blocked {
        Err(())
    } else {
        Ok(())
    };

    for layer in layers.iter().take(entered_count).rev() {
        layer.on_exit(msg, &mut outcome);
    }

    outcome
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
    let layers = match CHAT_PIPELINE.read() {
        Ok(p) => p.clone(),
        Err(e) => e.into_inner().clone(),
    };

    if run_chat_layers(&layers, &mut msg).is_err() {
        return None;
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
    process_chat_message_with_manager(None, sender, raw_text, scope)
}

/// Dispatches a chat message using an optional pre-locked `PluginManager` to avoid re-entrant mutex deadlocks.
pub fn process_chat_message_with_manager(
    mut manager: Option<&mut goldsrc_host_wasm::PluginManager>,
    sender: Player,
    raw_text: &str,
    scope: ChatScope,
) -> bool {
    let mut msg = ChatMessage::new(sender, raw_text, scope);

    // 0. Check pluggable chat triggers (consumed before entering chat pipeline)
    if evaluate_chat_triggers(manager.as_deref_mut(), sender, raw_text) {
        return true;
    }

    // 1. Enforce native moderation mute state
    if crate::moderation::is_player_muted(sender.index()) {
        if let Some(engine) = crate::host::HostRuntime::engine() {
            engine.client_print(
                sender.index(),
                goldsrc_api::HUD_PRINTCHAT,
                "[Moderation] You are muted and cannot send chat messages.\n",
            );
        }
        return true;
    }

    // 2. Run SMA U-Cycle pipeline over registered layers (censorship, ranks, custom prefixes)
    let layers = match CHAT_PIPELINE.read() {
        Ok(p) => p.clone(),
        Err(e) => e.into_inner().clone(),
    };

    if run_chat_layers(&layers, &mut msg).is_err() {
        return true; // blocked by middleware
    }

    // 2. Run WASM plugins chat middleware

    {
        let is_team = msg.scope.is_team();
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
        .get::<goldsrc_api::client::Name>()
        .as_deref()
        .map(String::from)
        .unwrap_or_else(|| format!("Player#{}", sender.index()));

    let full_text = match &msg.scope.target {
        ChatTarget::Team(TeamTarget::SameTeam) => {
            if let Some(ref prefix) = msg.prefix {
                format!(
                    "{prefix}^2(TEAM)^1 ^3{sender_name}^1 :  {}",
                    msg.formatted_text
                )
            } else {
                format!("^2(TEAM)^1 ^3{sender_name}^1 :  {}", msg.formatted_text)
            }
        }
        ChatTarget::Custom(channel) => {
            let ch_upper = channel.to_ascii_uppercase();
            if let Some(ref prefix) = msg.prefix {
                format!(
                    "{prefix}^4({ch_upper})^1 ^3{sender_name}^1 :  {}",
                    msg.formatted_text
                )
            } else {
                format!(
                    "^4({ch_upper})^1 ^3{sender_name}^1 :  {}",
                    msg.formatted_text
                )
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

    // 5. Broadcast chunks to target recipients based on ChatScope
    let sender_team = sender.get::<goldsrc_api::client::Team>();
    match &msg.scope.target {
        ChatTarget::Direct(slot) => {
            let target = Player::new(*slot);
            if target.is_valid() && matches_lifestate(target, msg.scope.state) {
                for chunk in &chunks {
                    target.act(goldsrc_api::action::Print::chat(chunk));
                }
            }
        }
        ChatTarget::All | ChatTarget::Team(TeamTarget::All) => {
            for i in 1..=32 {
                let target = Player::new(i);
                if target.is_valid() && matches_lifestate(target, msg.scope.state) {
                    for chunk in &chunks {
                        target.act(goldsrc_api::action::Print::chat(chunk));
                    }
                }
            }
        }
        ChatTarget::Team(TeamTarget::SameTeam) => {
            for i in 1..=32 {
                let target = Player::new(i);
                if target.is_valid()
                    && target.get::<goldsrc_api::client::Team>() == sender_team
                    && matches_lifestate(target, msg.scope.state)
                {
                    for chunk in &chunks {
                        target.act(goldsrc_api::action::Print::chat(chunk));
                    }
                }
            }
        }
        ChatTarget::Team(TeamTarget::OppositeTeam) => {
            for i in 1..=32 {
                let target = Player::new(i);
                if target.is_valid()
                    && is_opposite_team(sender_team, target.get::<goldsrc_api::client::Team>())
                    && matches_lifestate(target, msg.scope.state)
                {
                    for chunk in &chunks {
                        target.act(goldsrc_api::action::Print::chat(chunk));
                    }
                }
            }
        }
        ChatTarget::Custom(channel) => {
            let resolver = {
                let map = CUSTOM_RESOLVERS.read().unwrap_or_else(|e| e.into_inner());
                map.get(channel.as_ref()).cloned()
            };

            for i in 1..=32 {
                let target = Player::new(i);
                if target.is_valid()
                    && matches_lifestate(target, msg.scope.state)
                    && resolver
                        .as_ref()
                        .is_some_and(|r| r.can_receive(channel, sender, target))
                {
                    for chunk in &chunks {
                        target.act(goldsrc_api::action::Print::chat(chunk));
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
        LifeStateFilter::AliveOnly => {
            player.get::<goldsrc_api::client::LifeState>() == LifeState::Alive
        }
        LifeStateFilter::DeadOnly => {
            player.get::<goldsrc_api::client::LifeState>() != LifeState::Alive
        }
    }
}

fn is_opposite_team(a: Team, b: Team) -> bool {
    !a.is_unassigned() && !b.is_unassigned() && !a.is_spectator() && !b.is_spectator() && a != b
}

/// Formats a chat string with placeholder evaluation and arguments.
#[macro_export]
macro_rules! chat_format {
    ($fmt:expr) => {{
        $fmt.to_string()
    }};
    ($fmt:expr, $( $arg:tt )*) => {{
        format!($fmt, $( $arg )*)
    }};
}

/// Prints a formatted colored message to a specific player with safe chunk splitting.
///
/// Expands `{...}` placeholders and converts color tags (`^1`..`^4`).
///
/// # Examples
/// ```ignore
/// chat_print!(player, "^4[Admin]^1 Welcome, {name}! Your health: {hp}");
/// chat_print!(1, "^3Server tag:^1 {server_tag}");
/// ```
#[macro_export]
macro_rules! chat_print {
    ($target:expr, $msg:literal) => {{
        let player = $crate::Player::from($target);
        let formatted = $crate::placeholders::format_placeholders($msg, player);
        let chunks = $crate::goldsrc_api::chat::split_chat_chunks(&formatted);
        for chunk in chunks {
            player.act($crate::goldsrc_api::action::Print::chat(&chunk));
        }
    }};
    ($target:expr, $fmt:expr, $( $arg:expr ),* $(,)?) => {{
        let text = format!($fmt, $( $arg ),*);
        let player = $crate::Player::from($target);
        let formatted = $crate::placeholders::format_placeholders(&text, player);
        let chunks = $crate::goldsrc_api::chat::split_chat_chunks(&formatted);
        for chunk in chunks {
            player.act($crate::goldsrc_api::action::Print::chat(&chunk));
        }
    }};
}

/// Broadcasts a formatted colored message to all connected players with safe chunk splitting.
///
/// # Examples
/// ```ignore
/// chat_broadcast!("^4[Server]^1 Next map vote starting soon!");
/// chat_broadcast!("^2[Announcement]^1 {server_tag} updated!");
/// ```
#[macro_export]
macro_rules! chat_broadcast {
    ($msg:literal) => {{
        for i in 1..=32 {
            let player = $crate::Player::new(i);
            if player.is_valid() {
                let formatted = $crate::placeholders::format_placeholders($msg, player);
                let chunks = $crate::goldsrc_api::chat::split_chat_chunks(&formatted);
                for chunk in chunks {
                    player.act($crate::goldsrc_api::action::Print::chat(&chunk));
                }
            }
        }
    }};
    ($fmt:expr, $( $arg:expr ),* $(,)?) => {{
        let text = format!($fmt, $( $arg ),*);
        for i in 1..=32 {
            let player = $crate::Player::new(i);
            if player.is_valid() {
                let formatted = $crate::placeholders::format_placeholders(&text, player);
                let chunks = $crate::goldsrc_api::chat::split_chat_chunks(&formatted);
                for chunk in chunks {
                    player.act($crate::goldsrc_api::action::Print::chat(&chunk));
                }
            }
        }
    }};
}

/// Broadcasts a formatted colored message to all teammates of the sender with safe chunk splitting.
///
/// # Examples
/// ```ignore
/// chat_team!(sender, "^2(TEAM)^1 Player {name} needs assistance!");
/// ```
#[macro_export]
macro_rules! chat_team {
    ($sender:expr, $msg:literal) => {{
        let sender = $crate::Player::from($sender);
        let sender_team = sender.get::<$crate::goldsrc_api::Team>();
        for i in 1..=32 {
            let player = $crate::Player::new(i);
            if player.is_valid() && player.get::<$crate::goldsrc_api::Team>() == sender_team {
                let formatted = $crate::placeholders::format_placeholders($msg, player);
                let chunks = $crate::goldsrc_api::chat::split_chat_chunks(&formatted);
                for chunk in chunks {
                    player.act($crate::goldsrc_api::action::Print::chat(&chunk));
                }
            }
        }
    }};
    ($sender:expr, $fmt:expr, $( $arg:expr ),* $(,)?) => {{
        let text = format!($fmt, $( $arg ),*);
        let sender = $crate::Player::from($sender);
        let sender_team = sender.get::<$crate::goldsrc_api::Team>();
        for i in 1..=32 {
            let player = $crate::Player::new(i);
            if player.is_valid() && player.get::<$crate::goldsrc_api::Team>() == sender_team {
                let formatted = $crate::placeholders::format_placeholders(&text, player);
                let chunks = $crate::goldsrc_api::chat::split_chat_chunks(&formatted);
                for chunk in chunks {
                    player.act($crate::goldsrc_api::action::Print::chat(&chunk));
                }
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_chat_resolver_registration() {
        let channel = "admin_test";
        register_chat_target_resolver(
            channel,
            Arc::new(|_ch: &str, _sender: Player, recipient: Player| recipient.index() == 1),
        );

        let map = CUSTOM_RESOLVERS.read().unwrap();
        assert!(map.contains_key("admin_test"));
        let resolver = map.get("admin_test").unwrap();
        assert!(resolver.can_receive("admin_test", Player::new(2), Player::new(1)));
        assert!(!resolver.can_receive("admin_test", Player::new(2), Player::new(2)));
        drop(map);

        unregister_chat_target_resolver(channel);
        let map = CUSTOM_RESOLVERS.read().unwrap();
        assert!(!map.contains_key("admin_test"));
    }

    struct MockVoteTrigger;
    impl ChatTrigger for MockVoteTrigger {
        fn try_handle(
            &self,
            _manager: Option<&mut goldsrc_host_wasm::PluginManager>,
            _sender: Player,
            raw_text: &str,
        ) -> bool {
            raw_text.trim().eq_ignore_ascii_case("rtv")
        }
    }

    #[test]
    fn test_chat_trigger_interception() {
        register_chat_trigger(Arc::new(MockVoteTrigger));
        let player = Player::new(1);

        // "rtv" should be intercepted and consumed by MockVoteTrigger
        assert!(evaluate_chat_triggers(None, player, "rtv"));
        assert!(evaluate_chat_triggers(None, player, "  rtv  "));

        // Normal chat should not be intercepted
        assert!(!evaluate_chat_triggers(None, player, "hello world"));
    }

    struct AuditLayer;
    impl ChatLayer for AuditLayer {
        fn on_enter(&self, _msg: &mut ChatMessage) -> FlowControl<(), (), ()> {
            FlowControl::Proceed(())
        }

        fn on_exit(&self, msg: &mut ChatMessage, outcome: &mut Result<(), ()>) {
            if outcome.is_err() {
                msg.formatted_text = "[CENSORED]".to_string();
            }
        }
    }

    struct CensorshipLayer;
    impl ChatLayer for CensorshipLayer {
        fn on_enter(&self, msg: &mut ChatMessage) -> FlowControl<(), (), ()> {
            if msg.raw_text.contains("badword") {
                msg.is_blocked = true;
                FlowControl::Halt(())
            } else if msg.raw_text.contains("vip_hello") {
                msg.prefix = Some("[VIP]".to_string());
                FlowControl::Proceed(())
            } else {
                FlowControl::Proceed(())
            }
        }

        fn on_exit(&self, _msg: &mut ChatMessage, _outcome: &mut Result<(), ()>) {}
    }

    #[test]
    fn test_chat_layer_sma_u_cycle() {
        let layers: Vec<Arc<dyn ChatLayer>> = vec![Arc::new(AuditLayer), Arc::new(CensorshipLayer)];

        let mut msg_clean = ChatMessage::new(Player::new(1), "vip_hello server", ChatScope::all());
        let res_clean = run_chat_layers(&layers, &mut msg_clean);
        assert!(res_clean.is_ok());
        assert_eq!(msg_clean.prefix.as_deref(), Some("[VIP]"));

        let mut msg_blocked = ChatMessage::new(Player::new(1), "say badword", ChatScope::all());
        let res_blocked = run_chat_layers(&layers, &mut msg_blocked);
        assert!(res_blocked.is_err());
        assert_eq!(msg_blocked.formatted_text, "[CENSORED]");
    }
}
