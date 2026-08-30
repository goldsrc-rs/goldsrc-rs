//! Metamod backend strategy: [`EntityHooks`] implementation.
//!
//! Metamod chains calls to the real GameDLL automatically, so pre-hooks stay
//! minimal (returning 0 = MRES_IGNORED semantics) and all business logic runs
//! either directly or in post-hooks. Command suppression is expressed via
//! `MRES_SUPERCEDE` on the shared meta globals.

use goldsrc::api_registry::{EntityHooks, pack_two_i32};
use goldsrc_sys::edict_t;

use crate::{meta_globals, meta_types::MRES_SUPERCEDE};

/// Metamod hook behavior. Registered once via `api_registry::register`.
pub struct MetamodHooks;

impl EntityHooks for MetamodHooks {
    fn server_activate(&self, _edict_list: *mut edict_t, _edict_count: i32, _client_max: i32) {
        crate::ensure_game_dll_hooks();
        goldsrc::hooks::on_server_activate();
    }

    fn server_deactivate(&self) {
        goldsrc::hooks::on_server_deactivate();
    }

    fn client_command(&self, _edict: *mut edict_t, index: i32, cmd: &str, args: &str) -> bool {
        let handled = goldsrc::hooks::dispatch_client_command(index, cmd, args);
        if handled {
            meta_globals().mres = MRES_SUPERCEDE;
        }
        handled
    }

    fn start_frame(&self) {
        goldsrc::hooks::on_server_frame();
    }

    fn start_frame_post(&self) {
        // Drain deferred prints after the GameDLL had its frame slice.
        crate::backend().drain_prints();
    }

    fn player_post_think(&self, _edict: *mut edict_t, index: i32) {
        goldsrc::hooks::emit_player_event("player_post_think", index);
    }

    fn client_kill(&self, _edict: *mut edict_t, index: i32) {
        goldsrc::hooks::emit_player_event("client_kill", index);
    }

    fn touch(
        &self,
        _touched: *mut edict_t,
        touched_idx: i32,
        _other: *mut edict_t,
        other_idx: i32,
    ) {
        goldsrc::hooks::emit_event("entity_touch", &pack_two_i32(touched_idx, other_idx));
    }

    fn entity_use(&self, _used: *mut edict_t, used_idx: i32, _other: *mut edict_t, other_idx: i32) {
        goldsrc::hooks::emit_event("entity_use", &pack_two_i32(used_idx, other_idx));
    }

    fn client_connect_post(&self, index: i32) {
        goldsrc::hooks::emit_player_event("client_connect", index);
    }

    fn client_disconnect_post(&self, index: i32) {
        goldsrc::hooks::emit_player_event("client_disconnect", index);
    }

    fn client_user_info_changed_post(&self, index: i32) {
        goldsrc::hooks::on_client_user_info_changed(index);
    }
}

/// Static hook instance handed to the registry in `Meta_Attach`.
pub static HOOKS: MetamodHooks = MetamodHooks;
