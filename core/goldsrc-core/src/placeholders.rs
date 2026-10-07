//! Re-export of placeholder registry and engine from `goldsrc-service-placeholders`.

pub use goldsrc_service_placeholders::*;

/// Backward compatibility helper for WASM PluginManager dispatcher.
pub fn format_placeholders_with_manager(
    template: &str,
    caller: goldsrc_api::client::Player,
    mut manager: Option<&mut goldsrc_host_wasm::PluginManager>,
) -> String {
    if let Some(ref mut mgr) = manager {
        format_placeholders_with_dispatcher(
            template,
            caller,
            Some(|plugin: &str, player_idx: i32, ident: &str| {
                mgr.dispatch_placeholder(plugin, player_idx, ident)
            }),
        )
    } else {
        format_placeholders(template, caller)
    }
}
