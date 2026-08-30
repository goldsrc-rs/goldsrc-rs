//! In-game Chat Interception, Filtering Pipeline, and Safe Packet Chunking.

use goldsrc_api::chat::{ChatMessage, ChatScope, split_chat_chunks};
use goldsrc_api::client::Player;
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

    // 3. Render final output with prefix
    let full_text = if let Some(ref prefix) = msg.prefix {
        format!("{prefix}{}", msg.formatted_text)
    } else {
        msg.formatted_text.clone()
    };

    // 4. Split message into safe 180-byte chunks and dispatch via engine
    let chunks = split_chat_chunks(&full_text);
    for chunk in chunks {
        sender.print_chat(&chunk);
    }

    true // Handled by GoldSrc.rs chat engine
}
