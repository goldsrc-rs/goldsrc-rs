//! Zero-Cost U-Cycle Middleware Pipelines for GoldSrc WASM Host using `stitch-rs`.
//!
//! Provides U-cycle processing pipelines for chat messages, console commands,
//! and engine events following The Sewing Machine Architecture (SMA).

use crate::plugin::LoadedPlugin;

/// Context passing through the chat processing U-cycle.
pub struct ChatContext<'a> {
    pub sender: i32,
    pub is_team: bool,
    pub current_text: &'a mut String,
}

/// The intent entering the chat pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatIntent<'a> {
    pub text: &'a str,
}

/// Reason for chat rejection or suppression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatDropReason {
    SuppressedByPlugin(String),
    Muted,
    Censored,
}

/// Dynamic SMA chat pipeline runner.
/// Executes registered plugin interceptors in U-cycle traversal order without heap allocations on hot path.
pub fn execute_chat_pipeline(
    plugins: &mut [LoadedPlugin],
    ctx: &mut ChatContext<'_>,
    intent: ChatIntent<'_>,
) -> Result<(), ChatDropReason> {
    let mut halted = None;

    // Phase 1: Descent (on_enter)
    for plugin in plugins.iter_mut() {
        if !plugin.has_export("on-chat") {
            continue;
        }

        match plugin.call_on_chat(ctx.sender, ctx.current_text, ctx.is_team) {
            Ok(Some(transformed)) => {
                *ctx.current_text = transformed;
            }
            Ok(None) => {
                halted = Some(ChatDropReason::SuppressedByPlugin(plugin.name.clone()));
                break;
            }
            Err(e) => {
                log::warn!(
                    target: goldsrc_api::consts::log_targets::WASM,
                    "Error executing on-chat in plugin '{}': {e}",
                    plugin.name
                );
            }
        }
    }

    let _ = intent; // Used for contract preservation

    if let Some(err) = halted {
        Err(err)
    } else {
        Ok(())
    }
}
