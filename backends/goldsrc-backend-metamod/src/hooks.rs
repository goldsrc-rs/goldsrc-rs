//! Metamod backend strategy: [`EntityHooks`] implementation.
//!
//! Metamod chains calls to the real GameDLL automatically, so pre-hooks stay
//! minimal (returning 0 = MRES_IGNORED semantics) and all business logic runs
//! either directly or in post-hooks. Command suppression is expressed via
//! `MRES_SUPERCEDE` on the shared meta globals.

use goldsrc_core::api_registry::EntityHooks;
use goldsrc_core::{HostEvent, PlayerEvent};
use goldsrc_sys::edict_t;

use crate::{meta_globals, meta_types::MRES_SUPERCEDE};

/// Metamod hook behavior. Registered once via `api_registry::register`.
pub struct MetamodHooks;

impl EntityHooks for MetamodHooks {
    fn server_activate(&self, _edict_list: *mut edict_t, _edict_count: i32, _client_max: i32) {
        crate::ensure_game_dll_hooks();
        goldsrc_core::hooks::emit(HostEvent::ServerActivate);
    }

    fn server_deactivate(&self) {
        goldsrc_core::hooks::emit(HostEvent::ServerDeactivate);
    }

    fn client_command(&self, _edict: *mut edict_t, index: i32, cmd: &str, args: &str) -> bool {
        let handled = goldsrc_core::hooks::dispatch_client_command(index, cmd, args);
        if handled {
            meta_globals().mres = MRES_SUPERCEDE;
        }
        handled
    }

    fn start_frame(&self) {
        goldsrc_core::hooks::emit(HostEvent::ServerFrame);
    }

    fn start_frame_post(&self) {
        // Drain deferred prints after the GameDLL had its frame slice.
        crate::backend().drain_prints();
    }

    fn player_pre_think(&self, _edict: *mut edict_t, index: i32) {
        goldsrc_core::hooks::emit(HostEvent::Player {
            slot: index,
            event: PlayerEvent::PreThink,
        });
    }

    fn player_post_think(&self, _edict: *mut edict_t, index: i32) {
        goldsrc_core::hooks::emit(HostEvent::Player {
            slot: index,
            event: PlayerEvent::PostThink,
        });
    }

    fn cmd_start(
        &self,
        _player: *const edict_t,
        index: i32,
        cmd: *const goldsrc_sys::usercmd_s,
        _random_seed: u32,
    ) {
        let buttons = if !cmd.is_null() {
            unsafe { (*cmd).buttons }
        } else {
            0
        };
        goldsrc_core::hooks::emit(HostEvent::CmdStart {
            slot: index,
            buttons,
        });
    }

    fn cmd_end(&self, _player: *const edict_t, index: i32) {
        goldsrc_core::hooks::emit(HostEvent::Player {
            slot: index,
            event: PlayerEvent::CmdEnd,
        });
    }

    fn client_kill(&self, _edict: *mut edict_t, index: i32) {
        goldsrc_core::hooks::emit(HostEvent::Player {
            slot: index,
            event: PlayerEvent::Kill,
        });
    }

    fn touch(
        &self,
        _touched: *mut edict_t,
        touched_idx: i32,
        _other: *mut edict_t,
        other_idx: i32,
    ) {
        goldsrc_core::hooks::emit(HostEvent::EntityTouch {
            touched: touched_idx,
            other: other_idx,
        });
    }

    fn entity_use(&self, _used: *mut edict_t, used_idx: i32, _other: *mut edict_t, other_idx: i32) {
        goldsrc_core::hooks::emit(HostEvent::EntityUse {
            used: used_idx,
            other: other_idx,
        });
    }

    fn client_connect_post(&self, index: i32) {
        goldsrc_core::hooks::emit(HostEvent::Player {
            slot: index,
            event: PlayerEvent::Connect,
        });
    }

    fn client_disconnect_post(&self, index: i32) {
        goldsrc_core::hooks::emit(HostEvent::Player {
            slot: index,
            event: PlayerEvent::Disconnect,
        });
    }

    fn client_user_info_changed_post(&self, index: i32) {
        goldsrc_core::hooks::emit(HostEvent::Player {
            slot: index,
            event: PlayerEvent::UserInfoChanged,
        });
    }
}

/// Static hook instance handed to the registry in `Meta_Attach`.
pub static HOOKS: MetamodHooks = MetamodHooks;
