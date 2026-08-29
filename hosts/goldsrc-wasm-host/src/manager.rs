use crate::bindings::{GoldsrcPlugin, goldsrc::engine::api};
use crate::error::{CommandError, LoadError};
use crate::plugin::{LoadedPlugin, PluginMetadata, PluginStatus};
use goldsrc_api::Engine as GoldsrcEngine;
use std::fs;
use std::path::{Path, PathBuf};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine};

use notify::Watcher;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

/// Wasmtime store state exposed to WASM plugins via host functions.
pub struct HostState {
    /// Engine bridge for real game-state access.
    pub engine: Arc<dyn GoldsrcEngine>,
    /// Per-store memory and table limit enforcement.
    pub limits: wasmtime::StoreLimits,
    /// Identifier of the calling plugin.
    pub plugin_name: String,
    /// Explicitly allowed shared storage buckets from metadata.
    pub shared_buckets: Vec<String>,
}

/// Read-only snapshot of a loaded plugin, used for CLI listing/info.
pub struct PluginInfo {
    /// Plugin name.
    pub name: String,
    /// Path the plugin was loaded from.
    pub path: PathBuf,
    /// Index in the plugin list.
    pub index: usize,
    /// Current lifecycle status.
    pub status: PluginStatus,
    /// Parsed metadata, if any.
    pub metadata: Option<PluginMetadata>,
    /// Whether the plugin exports `on_load`.
    pub has_on_load: bool,
    /// Whether the plugin exports `on_unload`.
    pub has_on_unload: bool,
    /// Whether the plugin exports `on_frame`.
    pub has_on_frame: bool,
}

impl HostState {
    /// Resolves bucket name enforcing plugin isolation and allowlist sharing.
    fn resolve_bucket(&self, bucket: &str) -> Option<String> {
        if bucket.contains('/') {
            // Check if bucket is explicitly allowlisted
            if self.shared_buckets.iter().any(|b| b == bucket) {
                Some(bucket.to_string())
            } else {
                crate::host_log(&format!(
                    "[ERROR] Plugin '{}' attempted unauthorized access to shared bucket '{}'",
                    self.plugin_name, bucket
                ));
                None
            }
        } else {
            // Auto-prefix with plugin name
            Some(format!("{}/{}", self.plugin_name, bucket))
        }
    }
}

impl api::Host for HostState {
    fn host_log(&mut self, msg: String) {
        crate::host_log(&msg);
    }

    fn host_entity_is_valid(&mut self, index: i32) -> bool {
        self.engine.entity_is_valid(index)
    }
    fn host_entity_classname(&mut self, index: i32) -> Option<String> {
        self.engine.entity_classname(index)
    }
    fn host_entity_health(&mut self, index: i32) -> f32 {
        self.engine.entity_health(index)
    }
    fn host_entity_set_health(&mut self, index: i32, health: f32) {
        self.engine.entity_set_health(index, health);
    }
    fn host_entity_origin(&mut self, index: i32) -> api::Vector3 {
        let [x, y, z] = self.engine.entity_origin(index);
        api::Vector3 { x, y, z }
    }
    fn host_entity_set_origin(&mut self, index: i32, pos: api::Vector3) {
        self.engine.entity_set_origin(index, [pos.x, pos.y, pos.z]);
    }
    fn host_entity_velocity(&mut self, index: i32) -> api::Vector3 {
        let [x, y, z] = self.engine.entity_velocity(index);
        api::Vector3 { x, y, z }
    }
    fn host_entity_set_velocity(&mut self, index: i32, vel: api::Vector3) {
        self.engine
            .entity_set_velocity(index, [vel.x, vel.y, vel.z]);
    }
    fn host_entity_angles(&mut self, index: i32) -> api::Vector3 {
        let [x, y, z] = self.engine.entity_angles(index);
        api::Vector3 { x, y, z }
    }
    fn host_entity_set_angles(&mut self, index: i32, angles: api::Vector3) {
        self.engine
            .entity_set_angles(index, [angles.x, angles.y, angles.z]);
    }
    fn host_create_named_entity(&mut self, classname: String) -> Option<i32> {
        self.engine.create_named_entity(&classname)
    }
    fn host_remove_entity(&mut self, index: i32) {
        self.engine.remove_entity(index);
    }
    fn host_drop_to_floor(&mut self, index: i32) -> i32 {
        self.engine.drop_to_floor(index)
    }

    fn host_player_name(&mut self, index: i32) -> Option<String> {
        self.engine.player_name(index)
    }
    fn host_player_armorvalue(&mut self, index: i32) -> f32 {
        self.engine.player_armorvalue(index)
    }
    fn host_player_set_armorvalue(&mut self, index: i32, armor: f32) {
        self.engine.player_set_armorvalue(index, armor);
    }

    fn host_cvar_get_float(&mut self, name: String) -> f32 {
        self.engine.cvar_get_float(&name)
    }
    fn host_cvar_set_float(&mut self, name: String, val: f32) {
        self.engine.cvar_set_float(&name, val);
    }
    fn host_cvar_get_string(&mut self, name: String) -> Option<String> {
        self.engine.cvar_get_string(&name)
    }
    fn host_cvar_set_string(&mut self, name: String, val: String) {
        self.engine.cvar_set_string(&name, &val);
    }

    fn host_precache_model(&mut self, path: String) -> i32 {
        self.engine.precache_model(&path)
    }
    fn host_precache_sound(&mut self, path: String) -> i32 {
        self.engine.precache_sound(&path)
    }
    fn host_precache_generic(&mut self, path: String) -> i32 {
        self.engine.precache_generic(&path)
    }

    fn host_emit_sound(
        &mut self,
        entity: i32,
        channel: i32,
        sample: String,
        volume: f32,
        attenuation: f32,
        sound_flags: i32,
        pitch: i32,
    ) {
        self.engine.emit_sound(
            entity,
            channel,
            &sample,
            volume,
            attenuation,
            sound_flags,
            pitch,
        );
    }

    fn host_print_chat(&mut self, player_index: i32, message: String) {
        if !(1..=32).contains(&player_index) || !self.engine.entity_is_valid(player_index) {
            self.engine
                .server_print(&format!("[Chat to #{player_index}] {message}\n"));
            return;
        }
        let formatted = goldsrc_api::format_say_text(&message);
        let say_text_id = self.engine.reg_user_msg("SayText", -1);
        let msg_id = if say_text_id <= 0 { 76 } else { say_text_id };
        self.engine.message_begin(
            goldsrc_api::MessageDest::One as i32,
            msg_id,
            None,
            Some(player_index),
        );
        // In GoldSrc CS 1.6 SayText, first byte is the sender entity index (1..32 for player colors, or 0)
        self.engine.write_byte(player_index);
        // Truncate message if oversized to prevent buffer overflow (SayText payload max 192 bytes)
        let safe_msg = if formatted.len() > 175 {
            let mut end = 175;
            while end > 0 && !formatted.is_char_boundary(end) {
                end -= 1;
            }
            &formatted[..end]
        } else {
            &formatted
        };
        // SayText string must be sent without extra trailing newline
        self.engine.write_string(safe_msg);
        self.engine.message_end();
    }

    fn host_print_center(&mut self, player_index: i32, message: String) {
        if player_index < 0 {
            self.engine.server_print(&format!("[Center] {message}\n"));
            return;
        }

        let formatted = goldsrc_api::format_center_text(&message);
        let text_msg_id = self.engine.reg_user_msg("TextMsg", -1);
        let msg_id = if text_msg_id <= 0 { 75 } else { text_msg_id };

        let dest = if player_index == 0 {
            goldsrc_api::MessageDest::All as i32
        } else {
            if !(1..=32).contains(&player_index) || !self.engine.entity_is_valid(player_index) {
                return;
            }
            goldsrc_api::MessageDest::One as i32
        };

        let target_edict = if player_index == 0 {
            None
        } else {
            Some(player_index)
        };

        self.engine.message_begin(dest, msg_id, None, target_edict);

        // AMX Mod X / HLSDK UTIL_ClientPrint protocol for center messages:
        // 1. Write destination byte: HUD_PRINTCENTER (4)
        // 2. Write format string: "%s"
        // 3. Write formatted message (newlines replaced with '\r', safe truncated to <= 185 bytes)
        self.engine.write_byte(goldsrc_api::HUD_PRINTCENTER);
        self.engine.write_string("%s");

        let safe_msg = if formatted.len() > 185 {
            let mut end = 185;
            while end > 0 && !formatted.is_char_boundary(end) {
                end -= 1;
            }
            &formatted[..end]
        } else {
            &formatted
        };

        self.engine.write_string(safe_msg);
        self.engine.message_end();
    }

    fn host_print_console(&mut self, player_index: i32, message: String) {
        if player_index <= 0 || !self.engine.entity_is_valid(player_index) {
            self.engine
                .server_print(&format!("[Console#{player_index}] {message}\n"));
            return;
        }
        // 0 = PRINT_CONSOLE in GoldSrc client_printf
        self.engine
            .client_print(player_index, goldsrc_api::PRINT_CONSOLE, &message);
    }

    fn host_dispatch_spawn(&mut self, index: i32) -> i32 {
        self.engine.dispatch_spawn(index)
    }

    fn host_dispatch_touch(&mut self, touched: i32, other: i32) {
        self.engine.dispatch_touch(touched, other);
    }

    fn host_show_menu(&mut self, player_index: i32, keys_mask: i32, timeout: i32, text: String) {
        if player_index <= 0 {
            return;
        }
        let show_menu_id = self.engine.reg_user_msg("ShowMenu", -1);
        self.engine.server_print(&format!(
            "[GoldSrc.rs DEBUG] host_show_menu: player_index={}, show_menu_id={}, keys_mask={}, text_len={}\n",
            player_index, show_menu_id, keys_mask, text.len()
        ));
        if show_menu_id <= 0 || show_menu_id == 255 {
            self.engine
                .server_print("[GoldSrc.rs DEBUG] ShowMenu user msg ID is invalid (<=0 or 255)!\n");
            return;
        }

        crate::notify_show_menu(player_index, keys_mask, timeout, &text);

        if text.is_empty() {
            self.engine.message_begin(
                goldsrc_api::MessageDest::One as i32,
                show_menu_id,
                None,
                Some(player_index),
            );
            self.engine.write_short(keys_mask);
            self.engine.write_char(timeout);
            self.engine.write_byte(0);
            self.engine.write_string("");
            self.engine.message_end();
            return;
        }

        let max_chunk = goldsrc_api::consts::MAX_SHOW_MENU_CHUNK_SIZE;
        let mut remaining = &text[..];

        while !remaining.is_empty() {
            let chunk_len = if remaining.len() <= max_chunk {
                remaining.len()
            } else {
                let mut end = max_chunk;
                while end > 0 && !remaining.is_char_boundary(end) {
                    end -= 1;
                }
                if end == 0 {
                    remaining.chars().next().map(|c| c.len_utf8()).unwrap_or(1)
                } else {
                    end
                }
            };

            let chunk = &remaining[..chunk_len];
            remaining = &remaining[chunk_len..];
            let has_more = !remaining.is_empty();

            self.engine.message_begin(
                goldsrc_api::MessageDest::One as i32,
                show_menu_id,
                None,
                Some(player_index),
            );
            self.engine.write_short(keys_mask);
            self.engine.write_char(timeout);
            self.engine.write_byte(if has_more { 1 } else { 0 });
            self.engine.write_string(chunk);
            self.engine.message_end();
        }
    }

    fn host_send_hud_message(
        &mut self,
        _player_index: i32,
        channel: i32,
        x: f32,
        y: f32,
        r: i32,
        g: i32,
        b: i32,
        a: i32,
        effect: i32,
        fade_in: f32,
        fade_out: f32,
        hold_time: f32,
        text: String,
    ) {
        let x_val = (if x < 0.0 { -1.0 } else { x } * 8192.0) as i32;
        let y_val = (if y < 0.0 { -1.0 } else { y } * 8192.0) as i32;

        self.engine.message_begin(
            goldsrc_api::MessageDest::Broadcast as i32,
            goldsrc_api::consts::SVC_TEMPENTITY,
            None,
            None,
        );
        self.engine
            .write_byte(goldsrc_api::consts::TE_TEXTMESSAGE as i32);
        self.engine.write_byte(channel.clamp(1, 4));
        self.engine.write_short(x_val);
        self.engine.write_short(y_val);
        self.engine.write_byte(effect.clamp(0, 2));
        self.engine.write_byte(r.clamp(0, 255));
        self.engine.write_byte(g.clamp(0, 255));
        self.engine.write_byte(b.clamp(0, 255));
        self.engine.write_byte(a.clamp(0, 255));
        self.engine.write_byte(r.clamp(0, 255)); // 2nd color fallback
        self.engine.write_byte(g.clamp(0, 255));
        self.engine.write_byte(b.clamp(0, 255));
        self.engine.write_byte(a.clamp(0, 255));
        self.engine.write_short((fade_in * 256.0) as i32);
        self.engine.write_short((fade_out * 256.0) as i32);
        self.engine.write_short((hold_time * 256.0) as i32);
        // fx_time is only present in the TE_TEXTMESSAGE wire format when effect == 2
        // (typewriter). Writing it unconditionally shifts the stream by 2 bytes and
        // causes the client to read svc_bad (0).
        if effect.clamp(0, 2) == 2 {
            self.engine.write_short(0); // fx_time placeholder
        }
        self.engine.write_string(&text);
        self.engine.message_end();
    }

    fn host_send_dhud_message(
        &mut self,
        player_index: i32,
        x: f32,
        y: f32,
        r: i32,
        g: i32,
        b: i32,
        _a: i32,
        effect: i32,
        fade_in: f32,
        fade_out: f32,
        hold_time: f32,
        text: String,
    ) {
        const SVC_DIRECTOR: i32 = 51;
        const DRC_CMD_MESSAGE: i32 = 6;

        let (dest, target_idx) = if player_index <= 0 {
            (goldsrc_api::MessageDest::Broadcast as i32, None)
        } else {
            (goldsrc_api::MessageDest::One as i32, Some(player_index))
        };

        let text_bytes = text.as_bytes();
        let len = text_bytes.len().min(128);
        let safe_text = &text[..len];

        // Pack color into 0x00RRGGBB format expected by client VGUI director parser
        let packed_color = b.clamp(0, 255) | (g.clamp(0, 255) << 8) | (r.clamp(0, 255) << 16);

        self.engine
            .message_begin(dest, SVC_DIRECTOR, None, target_idx);
        self.engine.write_byte((len as i32) + 31);
        self.engine.write_byte(DRC_CMD_MESSAGE);
        self.engine.write_byte(effect.clamp(0, 2));
        self.engine.write_long(packed_color);
        self.engine.write_long(x.to_bits() as i32);
        self.engine.write_long(y.to_bits() as i32);
        self.engine.write_long(fade_in.to_bits() as i32);
        self.engine.write_long(fade_out.to_bits() as i32);
        self.engine.write_long(hold_time.to_bits() as i32);
        self.engine.write_long(0); // fx_time
        self.engine.write_string(safe_text);
        self.engine.message_end();
    }

    fn host_register_capability(&mut self, name: String, description: String) -> bool {
        if name.is_empty() || name.len() > 256 || description.len() > 4096 {
            return false;
        }
        goldsrc_api::auth::Auth::register_capability(&name, &description)
    }

    fn host_has_capability(&mut self, player_index: i32, name: String) -> bool {
        goldsrc_api::auth::Auth::has_capability(player_index, &name)
    }

    fn host_grant_capability(&mut self, player_index: i32, name: String) -> bool {
        goldsrc_api::auth::Auth::grant_capability(player_index, &name)
    }

    fn host_revoke_capability(&mut self, player_index: i32, name: String) -> bool {
        goldsrc_api::auth::Auth::revoke_capability(player_index, &name)
    }

    fn host_storage_get(&mut self, bucket: String, key: String) -> Option<Vec<u8>> {
        let resolved = self.resolve_bucket(&bucket)?;
        if let Ok(lock) = crate::STORAGE_GET_CB.read() {
            if let Some(cb) = *lock {
                return cb(&resolved, &key);
            }
        }
        None
    }

    fn host_storage_set(&mut self, bucket: String, key: String, val: Vec<u8>) -> bool {
        let Some(resolved) = self.resolve_bucket(&bucket) else {
            return false;
        };
        if let Ok(lock) = crate::STORAGE_SET_CB.read() {
            if let Some(cb) = *lock {
                return cb(&resolved, &key, &val);
            }
        }
        false
    }

    fn host_storage_delete(&mut self, bucket: String, key: String) -> bool {
        let Some(resolved) = self.resolve_bucket(&bucket) else {
            return false;
        };
        if let Ok(lock) = crate::STORAGE_DELETE_CB.read() {
            if let Some(cb) = *lock {
                return cb(&resolved, &key);
            }
        }
        false
    }

    fn host_storage_fetch_add(&mut self, bucket: String, key: String, delta: i64) -> i64 {
        let Some(resolved) = self.resolve_bucket(&bucket) else {
            return 0;
        };
        if let Ok(lock) = crate::STORAGE_FETCH_ADD_CB.read() {
            if let Some(cb) = *lock {
                return cb(&resolved, &key, delta);
            }
        }
        0
    }

    fn host_translate(&mut self, dict: String, lang: String, key: String) -> String {
        if let Ok(lock) = crate::TRANSLATE_CB.read() {
            if let Some(cb) = *lock {
                return cb(&dict, &lang, &key);
            }
        }
        key
    }
}

/// Manages the lifecycle of loaded WASM plugins: loading, unloading,
/// reloading, pausing, frame dispatch and hot-reload via directory watchers.
///
/// Not `Send`/`Sync` (holds wasmtime stores) — keep it on the server thread.
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
    engine: Engine,
    engine_ops: Arc<dyn GoldsrcEngine>,
    event_rx: Receiver<PathBuf>,
    event_tx: Sender<PathBuf>,
    watchers: Vec<notify::RecommendedWatcher>,
    watcher_count: usize,
    last_reload: HashMap<PathBuf, Instant>,
    /// command name -> plugin indices that registered it.
    command_registry: HashMap<String, Vec<usize>>,
    /// Configured search directories for plugin path resolution.
    plugin_dirs: Vec<PathBuf>,
}

/// Minimum gap between two hot-reloads of the same file. Compilers write in
/// several passes, so a rebuild would otherwise reload a half-written file.
const RELOAD_DEBOUNCE: Duration = Duration::from_millis(1000);

impl PluginManager {
    /// Creates an empty plugin manager backed by a fresh wasmtime engine
    /// with the Component Model and epoch interruption enabled.
    pub fn new(engine_ops: Arc<dyn GoldsrcEngine>) -> wasmtime::Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.epoch_interruption(true);
        // config.target("pulley32").unwrap();
        let engine = Engine::new(&config)?;
        let (event_tx, event_rx) = mpsc::channel::<PathBuf>();

        // Spawn background epoch timer thread to advance epochs every 2ms.
        // This ensures epoch interruption deadlines (e.g. 5 epochs) are enforced
        // as real wall-clock slices even if a plugin enters an infinite loop.
        let engine_clone = engine.clone();
        std::thread::Builder::new()
            .name("goldsrc-epoch-timer".into())
            .spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_millis(2));
                    engine_clone.increment_epoch();
                }
            })
            .ok();

        Ok(Self {
            plugins: Vec::new(),
            engine,
            engine_ops,
            event_rx,
            event_tx,
            watchers: Vec::new(),
            watcher_count: 0,
            last_reload: HashMap::new(),
            command_registry: HashMap::new(),
            plugin_dirs: Vec::new(),
        })
    }

    /// Sets the list of base search directories for resolving plugin paths.
    pub fn set_plugin_dirs(&mut self, dirs: Vec<PathBuf>) {
        self.plugin_dirs = dirs;
    }

    /// Sets the list of base search directories (builder style).
    pub fn with_plugin_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.plugin_dirs = dirs;
        self
    }

    /// Appends a plugin search directory.
    pub fn add_plugin_dir(&mut self, dir: PathBuf) {
        self.plugin_dirs.push(dir);
    }

    /// Compiles and instantiates a WASM plugin component without registering or running `on_load`.
    pub fn instantiate_plugin<P: AsRef<Path>>(&self, path: P) -> Result<LoadedPlugin, LoadError> {
        let path = path.as_ref();
        let metadata = fs::metadata(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if metadata.len() > 32 * 1024 * 1024 {
            return Err(LoadError::Compile(format!(
                "Plugin size ({} bytes) exceeds maximum allowed size (32MB)",
                metadata.len()
            )));
        }
        let bytes = fs::read(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let is_comp = bytes.len() >= 8 && &bytes[0..8] == b"\0asm\x0d\0\x01\0";
        let component_bytes = if is_comp {
            bytes
        } else {
            let mut resolve = wit_parser::Resolve::default();
            let pkg = resolve
                .push_str(
                    "goldsrc.wit",
                    include_str!("../../../core/goldsrc-api/wit/goldsrc.wit"),
                )
                .unwrap();
            let world_id = resolve
                .select_world(&[pkg], Some("goldsrc-plugin"))
                .unwrap();

            let mut wasm_bytes = bytes.to_vec();
            wit_component::embed_component_metadata(
                &mut wasm_bytes,
                &resolve,
                world_id,
                wit_component::StringEncoding::UTF8,
            )
            .map_err(|e| LoadError::Embed(e.to_string()))?;

            let mut base_encoder = wit_component::ComponentEncoder::default();
            let encoder = base_encoder.validate(true);
            let encoder = encoder
                .module(&wasm_bytes)
                .map_err(|e| LoadError::Encode(format!("{e:#?}")))?;
            encoder
                .encode()
                .map_err(|e| LoadError::Encode(format!("{e:#?}")))?
        };

        let component = Component::new(&self.engine, &component_bytes)
            .map_err(|e| LoadError::Compile(e.to_string()))?;

        let mut linker = Linker::new(&self.engine);
        api::add_to_linker::<HostState, wasmtime::component::HasSelf<HostState>>(
            &mut linker,
            |state: &mut HostState| state,
        )
        .map_err(|e| LoadError::Link(e.to_string()))?;

        let limits = wasmtime::StoreLimitsBuilder::new()
            .memory_size(64 * 1024 * 1024) // 64MB per memory
            .table_elements(10_000)
            .memories(4)
            .tables(16)
            .instances(16)
            .build();
        let state = HostState {
            engine: self.engine_ops.clone(),
            limits,
            plugin_name: String::new(),
            shared_buckets: Vec::new(),
        };
        let mut store = wasmtime::Store::new(&self.engine, state);
        store.limiter(|s| &mut s.limits);
        store.set_epoch_deadline(100);
        let bindings = GoldsrcPlugin::instantiate(&mut store, &component, &linker)
            .map_err(|e| LoadError::Instantiate(e.to_string()))?;

        let metadata = match bindings.call_get_metadata(&mut store) {
            Ok(meta_str) => match toml::from_str::<PluginMetadata>(&meta_str) {
                Ok(mut meta) => {
                    if let Some(ref b) = meta.bundle {
                        if b.is_empty()
                            || b.contains("..")
                            || b.starts_with('/')
                            || b.starts_with('\\')
                            || b.contains(':')
                            || !b.chars().all(|c| {
                                c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/'
                            })
                        {
                            crate::host_log(&format!(
                                "Warning: Rejected invalid/unsafe bundle '{b}' for plugin at {:?}",
                                path
                            ));
                            meta.bundle = None;
                        }
                    }
                    Some(meta)
                }
                Err(err) => {
                    crate::host_log(&format!(
                        "Warning: Failed to parse metadata for plugin at {:?}: {}",
                        path, err
                    ));
                    None
                }
            },
            Err(_) => None,
        };

        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let shared_buckets = metadata
            .as_ref()
            .map(|m| m.shared_buckets.clone())
            .unwrap_or_default();

        // Update HostState with validated plugin name and shared buckets allowlist
        {
            let data = store.data_mut();
            data.plugin_name = name.clone();
            data.shared_buckets = shared_buckets;
        }

        Ok(LoadedPlugin {
            name,
            path: path.to_path_buf(),
            status: PluginStatus::Loaded,
            metadata,
            store,
            bindings,
            component,
        })
    }

    /// Loads a WASM plugin from `path`. Accepts either a pre-compiled
    /// component (magic `\0asm`) or a plain core module, which is embedded
    /// with component metadata first. Calls the plugin's `on_load`.
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<String, LoadError> {
        let path = path.as_ref();
        let name_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Check if plugin is already loaded and active/loaded
        if let Some(p) = self
            .plugins
            .iter()
            .find(|p| p.path == path || p.name == name_stem)
        {
            if p.status != PluginStatus::Unloaded {
                return Err(LoadError::AlreadyLoaded(p.name.clone()));
            }
        }

        let mut plugin = self.instantiate_plugin(path)?;
        plugin
            .call_on_load()
            .map_err(|e| LoadError::LoadPanic(e.to_string()))?;
        crate::host_log(&format!("Loaded component plugin: {}", plugin.name));
        let idx = self.plugins.len();
        if let Some(meta) = &plugin.metadata {
            for cmd in &meta.commands {
                self.command_registry
                    .entry(cmd.clone())
                    .or_default()
                    .push(idx);
            }
        }
        let name = plugin.name.clone();
        self.plugins.push(plugin);
        self.recalculate_dependency_states();
        Ok(name)
    }

    /// Resolves a plugin query (either numeric index or name) to a plugin index.
    pub fn find_plugin(&self, query: &str) -> Option<usize> {
        if let Ok(idx) = query.parse::<usize>() {
            return (idx < self.plugins.len()).then_some(idx);
        }
        self.plugins.iter().position(|p| p.name == query)
    }

    /// Recalculates `status` across all loaded plugins according to dependency states.
    pub fn recalculate_dependency_states(&mut self) {
        let n = self.plugins.len();
        let mut loaded_plugins = HashMap::new();
        let mut running_plugins = HashMap::new();

        for p in &self.plugins {
            if matches!(
                p.status,
                PluginStatus::Running | PluginStatus::Paused { .. } | PluginStatus::Loaded
            ) {
                let ver = p
                    .metadata
                    .as_ref()
                    .map(|m| m.version.clone())
                    .unwrap_or_else(|| "1.0.0".to_string());
                loaded_plugins.insert(p.name.clone(), ver.clone());
                if !matches!(p.status, PluginStatus::Paused { .. }) {
                    running_plugins.insert(p.name.clone(), ver);
                }
            }
        }

        for i in 0..n {
            // Keep Poisoned as is
            if matches!(self.plugins[i].status, PluginStatus::Poisoned { .. }) {
                continue;
            }

            let mut missing_dep = None;
            let mut paused_dep = None;

            if let Some(meta) = &self.plugins[i].metadata {
                // 1. Evaluate require DSL entries
                for req_str in &meta.require {
                    if let Ok(req) = goldsrc_api::Requirement::from_str(req_str) {
                        match req {
                            goldsrc_api::Requirement::Plugin { name, optional, .. } => {
                                if !loaded_plugins.contains_key(&name) {
                                    if !optional {
                                        missing_dep =
                                            Some(format!("missing plugin dependency '{name}'"));
                                        break;
                                    }
                                } else if !running_plugins.contains_key(&name) && !optional {
                                    paused_dep =
                                        Some(format!("waiting for paused plugin '{name}'"));
                                }
                            }
                            goldsrc_api::Requirement::Cvar { name, op } => {
                                let cvar_val =
                                    self.engine_ops.cvar_get_string(&name).unwrap_or_default();
                                let satisfied = match op {
                                    goldsrc_api::CvarOp::Equal(expected) => cvar_val == expected,
                                    goldsrc_api::CvarOp::NotEqual(forbidden) => {
                                        cvar_val != forbidden
                                    }
                                    goldsrc_api::CvarOp::GreaterThanZero => {
                                        cvar_val.parse::<f32>().map(|v| v > 0.0).unwrap_or(false)
                                    }
                                };
                                if !satisfied {
                                    missing_dep = Some(format!(
                                        "cvar requirement '{name}' not satisfied (current: '{cvar_val}')"
                                    ));
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            if let Some(reason) = missing_dep {
                self.plugins[i].status = PluginStatus::Blocked { reason };
            } else if let Some(reason) = paused_dep {
                self.plugins[i].status = PluginStatus::Degraded { reason };
            } else if matches!(
                self.plugins[i].status,
                PluginStatus::Blocked { .. } | PluginStatus::Degraded { .. }
            ) {
                self.plugins[i].status = PluginStatus::Running;
            }
        }
    }

    /// Calls `on_unload` (if exported) and removes the plugin at `idx`.
    fn unload_plugin_at(&mut self, idx: usize) -> LoadedPlugin {
        // Deregister this plugin's commands.
        let meta = self.plugins[idx].metadata.clone();
        for cmd in meta.iter().flat_map(|m| &m.commands) {
            if let Some(owners) = self.command_registry.get_mut(cmd) {
                owners.retain(|i| *i != idx);
                if owners.is_empty() {
                    self.command_registry.remove(cmd);
                }
            }
        }
        let mut plugin = self.plugins.remove(idx);
        if plugin.has_export("on-unload") {
            let _ = plugin.call_on_unload();
        }
        // Shift indices of plugins after idx in the registry.
        for owners in self.command_registry.values_mut() {
            for i in owners.iter_mut() {
                if *i > idx {
                    *i -= 1;
                }
            }
        }
        self.recalculate_dependency_states();
        plugin
    }

    /// Unloads all loaded plugins and returns a summary message.
    pub fn unload_all_plugins(&mut self) -> String {
        let count = self.plugins.len();
        while !self.plugins.is_empty() {
            self.unload_plugin_at(self.plugins.len() - 1);
        }
        format!("Unloaded {} plugins.", count)
    }

    /// Sets or clears the pause flag on a plugin by name or index query.
    pub fn pause_plugin(&mut self, query: &str, pause: bool) -> Result<String, CommandError> {
        self.pause_plugin_with_reason(query, pause, None)
    }

    /// Sets or clears the pause flag on a plugin with a descriptive reason.
    pub fn pause_plugin_with_reason(
        &mut self,
        query: &str,
        pause: bool,
        reason: Option<String>,
    ) -> Result<String, CommandError> {
        let idx = self
            .find_plugin(query)
            .ok_or_else(|| CommandError::NotFound(query.to_string()))?;
        if pause {
            self.plugins[idx].status = PluginStatus::Paused { reason };
        } else if matches!(self.plugins[idx].status, PluginStatus::Paused { .. }) {
            self.plugins[idx].status = PluginStatus::Running;
        }
        self.recalculate_dependency_states();
        Ok(format!(
            "Plugin '{}' pause state set to {}",
            self.plugins[idx].name, pause
        ))
    }

    /// Sets or clears the pause flag on every loaded plugin.
    pub fn pause_all_plugins(&mut self, pause: bool) -> String {
        for p in &mut self.plugins {
            if pause {
                p.status = PluginStatus::Paused { reason: None };
            } else if matches!(p.status, PluginStatus::Paused { .. }) {
                p.status = PluginStatus::Running;
            }
        }
        self.recalculate_dependency_states();
        format!("All plugins pause state set to {}", pause)
    }

    /// Returns a snapshot of metadata for all loaded plugins.
    pub fn get_plugins_info(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .enumerate()
            .map(|(index, p)| PluginInfo {
                name: p.name.clone(),
                path: p.path.clone(),
                index,
                status: p.status.clone(),
                metadata: p.metadata.clone(),
                has_on_load: p.has_export("on-load"),
                has_on_unload: p.has_export("on-unload"),
                has_on_frame: p.has_export("on-frame"),
            })
            .collect()
    }

    /// Unloads and reloads every loaded plugin from its recorded path.
    /// Returns a summary counting failures.
    pub fn reload_all_plugins(&mut self) -> String {
        let paths: Vec<PathBuf> = self.plugins.iter().map(|p| p.path.clone()).collect();
        let count = paths.len();
        while !self.plugins.is_empty() {
            self.unload_plugin_at(self.plugins.len() - 1);
        }
        let mut failed = 0;
        for path in &paths {
            if self.load_plugin(path).is_err() {
                failed += 1;
            }
        }
        format!("Reloaded {} plugins ({} failed).", count - failed, failed)
    }

    /// Reloads a single plugin by name or index.
    pub fn reload_plugin_by_query(&mut self, query: &str) -> Result<String, CommandError> {
        let idx = self
            .find_plugin(query)
            .ok_or_else(|| CommandError::NotFound(query.to_string()))?;
        let path = self.plugins[idx].path.clone();
        let name = self.plugins[idx].name.clone();
        self.unload_plugin_at(idx);
        self.load_plugin(&path)
            .map(|_| format!("Reloaded '{}'", name))
            .map_err(|source| CommandError::Load { name, source })
    }

    /// Reloads the plugin whose recorded path matches `path`. Used by the
    /// hot-reload watcher; failures are logged, not returned.
    fn reload_plugin_path(&mut self, path: &Path) {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if let Some(idx) = self
            .plugins
            .iter()
            .position(|p| p.path == path || p.path.canonicalize().is_ok_and(|c| c == canonical))
        {
            let name = self.plugins[idx].name.clone();
            let old_path = self.plugins[idx].path.clone();

            match self.instantiate_plugin(&old_path) {
                Ok(mut new_plugin) => {
                    if let Err(e) = new_plugin.call_on_load() {
                        crate::host_log(&format!("Hot-reload on_load of '{}' failed: {e}", name));
                        return;
                    }
                    self.unload_plugin_at(idx);
                    let new_idx = self.plugins.len();
                    if let Some(meta) = &new_plugin.metadata {
                        for cmd in &meta.commands {
                            self.command_registry
                                .entry(cmd.clone())
                                .or_default()
                                .push(new_idx);
                        }
                    }
                    self.plugins.push(new_plugin);
                    crate::host_log(&format!("Hot-reloaded plugin '{}'", name));
                }
                Err(e) => {
                    crate::host_log(&format!(
                        "Hot-reload of '{}' failed (previous version kept active): {e}",
                        name
                    ));
                }
            }
        }
    }

    /// Debounced wrapper around [`reload_plugin_path`]: ignores events for a
    /// file reloaded less than [`RELOAD_DEBOUNCE`] ago.
    fn reload_plugin_path_debounced(&mut self, path: &Path) {
        let now = Instant::now();
        if let Some(last) = self.last_reload.get(path) {
            if now.duration_since(*last) < RELOAD_DEBOUNCE {
                return;
            }
        }
        self.last_reload.insert(path.to_path_buf(), now);
        self.reload_plugin_path(path);
    }

    /// Spawns a `notify` watcher on `dir` that forwards changed files with
    /// extension `ext` to the event channel, drained in [`on_server_frame`].
    ///
    /// [`on_server_frame`]: PluginManager::on_server_frame
    fn spawn_watcher<P: AsRef<Path>>(
        &mut self,
        dir: P,
        ext: &'static str,
    ) -> Result<(), CommandError> {
        let dir = dir.as_ref().to_path_buf();
        let _ = std::fs::create_dir_all(&dir);
        let tx = self.event_tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                for path in event.paths {
                    if path
                        .extension()
                        .and_then(|s| s.to_str())
                        .is_some_and(|e| e == ext)
                    {
                        let _ = tx.send(path);
                    }
                }
            }
        })
        .map_err(|e| CommandError::Watcher(e.to_string()))?;
        watcher
            .watch(&dir, notify::RecursiveMode::Recursive)
            .map_err(|e| CommandError::Watcher(e.to_string()))?;

        self.watchers.push(watcher);
        self.watcher_count += 1;
        Ok(())
    }

    /// Watches `dir` for changed `.wasm` files and reloads matching plugins
    /// on the next server frame.
    pub fn enable_hot_reload<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), CommandError> {
        self.spawn_watcher(dir, "wasm")
    }

    /// Watches `dir` for changed `.toml` files. Change events are drained in
    /// [`on_server_frame`], where `.wasm` events trigger reloads.
    ///
    /// [`on_server_frame`]: PluginManager::on_server_frame
    pub fn enable_config_watcher<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), CommandError> {
        self.spawn_watcher(dir, "toml")
    }

    /// Returns all registered command names across all loaded plugins.
    pub fn registered_commands(&self) -> Vec<String> {
        self.command_registry.keys().cloned().collect()
    }

    /// Dispatches a server command to the plugins that registered it.
    /// Stops at the first plugin that reports handling it (consume).
    pub fn dispatch_command(&mut self, cmd: &str, caller: i32, args: &str) -> bool {
        let Some(owners) = self.command_registry.get(cmd).cloned() else {
            return false;
        };
        for idx in owners {
            if let Some(plugin) = self.plugins.get_mut(idx) {
                if plugin.call_on_command(cmd, caller, args).unwrap_or(false) {
                    return true;
                }
            }
        }
        false
    }

    /// Drains watcher events (reloading changed `.wasm` plugins, debounced)
    /// then ticks every plugin's `on_frame`. `.toml` events are forwarded to
    /// plugins as `on_event("config_changed", <path>)`. Call once per frame.
    pub fn on_server_frame(&mut self) {
        self.engine.increment_epoch();
        while let Ok(path) = self.event_rx.try_recv() {
            match path.extension().and_then(|s| s.to_str()) {
                Some("wasm") => self.reload_plugin_path_debounced(&path),
                Some("toml") => {
                    let data = path.to_string_lossy().as_bytes().to_vec();
                    self.call_on_event("config_changed", &data);
                }
                _ => {}
            }
        }
        self.call_on_frame();
    }

    /// Loads a plugin by filesystem path or plugin name (e.g. `test_suite` or `test_suite.wasm`).
    ///
    /// [`load_plugin`]: PluginManager::load_plugin
    pub fn load_plugin_by_name(&mut self, query: &str) -> Result<String, LoadError> {
        let mut path = PathBuf::from(query);
        if !path.exists() {
            let wasm_ext = goldsrc_api::consts::WASM_EXT;
            let with_ext = if !query.ends_with(wasm_ext) {
                PathBuf::from(format!("{query}{wasm_ext}"))
            } else {
                path.clone()
            };

            if with_ext.exists() {
                path = with_ext;
            } else {
                let mut found_path = None;
                for base_dir in &self.plugin_dirs {
                    let candidate = base_dir.join(query);
                    if candidate.exists() {
                        found_path = Some(candidate);
                        break;
                    }
                    let candidate_wasm = base_dir.join(format!("{query}{wasm_ext}"));
                    if candidate_wasm.exists() {
                        found_path = Some(candidate_wasm);
                        break;
                    }
                }

                if let Some(found) = found_path {
                    path = found;
                } else if !query.ends_with(wasm_ext) {
                    path = with_ext;
                }
            }
        }
        self.load_plugin(path)
    }

    /// Unloads a single plugin by name or index.
    pub fn unload_plugin_by_query(&mut self, query: &str) -> Result<String, CommandError> {
        let idx = self
            .find_plugin(query)
            .ok_or_else(|| CommandError::NotFound(query.to_string()))?;
        let plugin = self.unload_plugin_at(idx);
        Ok(format!("Unloaded '{}'", plugin.name))
    }

    /// Returns `(loaded_plugins, active_watchers)` for status displays.
    pub fn get_status_info(&self) -> (usize, usize) {
        (self.plugins.len(), self.watcher_count)
    }

    /// Calls `on_frame` on every (non-paused) plugin.
    pub fn call_on_frame(&mut self) {
        for plugin in &mut self.plugins {
            let _ = plugin.call_on_frame();
        }
    }

    /// Calls `on_event` on every (non-paused) plugin.
    pub fn call_on_event(&mut self, name: &str, data: &[u8]) {
        for plugin in &mut self.plugins {
            let _ = plugin.call_on_event(name, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::goldsrc::engine::api::Host;

    struct NoopEngineOps;

    impl goldsrc_api::EnginePrecache for NoopEngineOps {
        fn precache_model(&self, _path: &str) -> i32 {
            0
        }
        fn precache_sound(&self, _path: &str) -> i32 {
            0
        }
        fn precache_generic(&self, _path: &str) -> i32 {
            0
        }
    }

    impl goldsrc_api::EngineMessages for NoopEngineOps {
        fn reg_user_msg(&self, _name: &str, _size: i32) -> i32 {
            0
        }

        fn message_begin(
            &self,
            _msg_dest: i32,
            _msg_type: i32,
            _origin: Option<[f32; 3]>,
            _edict_index: Option<i32>,
        ) {
        }
        fn message_end(&self) {}
        fn write_byte(&self, _val: i32) {}
        fn write_char(&self, _val: i32) {}
        fn write_short(&self, _val: i32) {}
        fn write_long(&self, _val: i32) {}
        fn write_angle(&self, _val: f32) {}
        fn write_coord(&self, _val: f32) {}
        fn write_string(&self, _val: &str) {}
        fn write_entity(&self, _val: i32) {}
    }

    impl goldsrc_api::EngineConsole for NoopEngineOps {
        fn server_print(&self, _message: &str) {}
        fn client_print(&self, _client_index: i32, _print_type: i32, _message: &str) {}
        fn server_command(&self, _command: &str) {}
    }

    impl goldsrc_api::EngineEntities for NoopEngineOps {
        fn entity_is_valid(&self, _index: i32) -> bool {
            false
        }
        fn entity_classname(&self, _index: i32) -> Option<String> {
            None
        }
        fn entity_health(&self, _index: i32) -> f32 {
            0.0
        }
        fn entity_set_health(&self, _index: i32, _health: f32) {}
        fn entity_origin(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_origin(&self, _index: i32, _pos: [f32; 3]) {}
        fn entity_velocity(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_velocity(&self, _index: i32, _vel: [f32; 3]) {}
        fn entity_angles(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_angles(&self, _index: i32, _angles: [f32; 3]) {}
        fn player_name(&self, _index: i32) -> Option<String> {
            None
        }
        fn player_armorvalue(&self, _index: i32) -> f32 {
            0.0
        }
        fn player_set_armorvalue(&self, _index: i32, _armor: f32) {}
        fn create_named_entity(&self, _classname: &str) -> Option<i32> {
            None
        }
        fn remove_entity(&self, _index: i32) {}
        fn drop_to_floor(&self, _index: i32) -> i32 {
            0
        }
        fn dispatch_spawn(&self, _index: i32) -> i32 {
            0
        }
        fn dispatch_touch(&self, _touched: i32, _other: i32) {}
    }

    impl goldsrc_api::EngineCvars for NoopEngineOps {
        fn cvar_get_float(&self, _name: &str) -> f32 {
            0.0
        }
        fn cvar_set_float(&self, _name: &str, _val: f32) {}
        fn cvar_get_string(&self, _name: &str) -> Option<String> {
            None
        }
        fn cvar_set_string(&self, _name: &str, _val: &str) {}
    }

    impl goldsrc_api::EnginePhysics for NoopEngineOps {
        fn point_contents(&self, _point: [f32; 3]) -> i32 {
            0
        }
        fn trace_line(
            &self,
            _start: [f32; 3],
            _end: [f32; 3],
            _flags: i32,
            _ignore_ent: i32,
        ) -> goldsrc_api::TraceResult {
            goldsrc_api::TraceResult::default()
        }
        fn trace_hull(
            &self,
            _start: [f32; 3],
            _end: [f32; 3],
            _flags: i32,
            _hull_number: i32,
            _ignore_ent: i32,
        ) -> goldsrc_api::TraceResult {
            goldsrc_api::TraceResult::default()
        }
    }

    impl goldsrc_api::EngineSound for NoopEngineOps {
        fn emit_sound(
            &self,
            _entity: i32,
            _channel: i32,
            _sample: &str,
            _volume: f32,
            _attenuation: f32,
            _flags: i32,
            _pitch: i32,
        ) {
        }
        fn emit_ambient_sound(
            &self,
            _entity: i32,
            _pos: [f32; 3],
            _sample: &str,
            _volume: f32,
            _attenuation: f32,
            _flags: i32,
            _pitch: i32,
        ) {
        }
    }

    /// Loads the built demo plugin and checks the command registry + consume semantics.
    #[test]
    fn command_registry_registers_and_consumes() {
        // Skipped when the demo plugin was not built (e.g. `cargo test -p goldsrc-wasm-host`).
        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../target/wasm32-unknown-unknown/debug/test_suite.wasm"
        );
        if !std::path::Path::new(wasm_path).exists() {
            eprintln!("test_suite.wasm not built; skipping command registry test");
            return;
        }

        let mut manager = PluginManager::new(Arc::new(NoopEngineOps)).unwrap();
        manager.load_plugin(wasm_path).unwrap();

        // The #[command(name = "testcmd")] marker must be discoverable in metadata.
        let meta = &manager.plugins[0].metadata;
        assert!(meta.is_some());
        assert!(
            meta.as_ref()
                .unwrap()
                .commands
                .contains(&"testcmd".to_string())
        );

        // Dispatch finds the plugin via the registry and consumes the command.
        assert!(manager.dispatch_command("testcmd", 0, "hello"));
        // Unknown commands are not dispatched at all.
        assert!(!manager.dispatch_command("nonexistent", 0, ""));
    }

    #[derive(Default)]
    struct MockMessageEngine {
        messages: std::sync::Mutex<Vec<(i32, i32, Option<i32>)>>,
        bytes: std::sync::Mutex<Vec<i32>>,
        strings: std::sync::Mutex<Vec<String>>,
        ended: std::sync::Mutex<usize>,
    }

    impl goldsrc_api::EnginePrecache for MockMessageEngine {
        fn precache_model(&self, _path: &str) -> i32 {
            0
        }
        fn precache_sound(&self, _path: &str) -> i32 {
            0
        }
        fn precache_generic(&self, _path: &str) -> i32 {
            0
        }
    }

    impl goldsrc_api::EngineMessages for MockMessageEngine {
        fn reg_user_msg(&self, _name: &str, _size: i32) -> i32 {
            75
        }
        fn message_begin(
            &self,
            msg_dest: i32,
            msg_type: i32,
            _origin: Option<[f32; 3]>,
            edict_index: Option<i32>,
        ) {
            self.messages
                .lock()
                .unwrap()
                .push((msg_dest, msg_type, edict_index));
        }
        fn message_end(&self) {
            *self.ended.lock().unwrap() += 1;
        }
        fn write_byte(&self, val: i32) {
            self.bytes.lock().unwrap().push(val);
        }
        fn write_char(&self, _val: i32) {}
        fn write_short(&self, _val: i32) {}
        fn write_long(&self, _val: i32) {}
        fn write_angle(&self, _val: f32) {}
        fn write_coord(&self, _val: f32) {}
        fn write_string(&self, val: &str) {
            self.strings.lock().unwrap().push(val.to_string());
        }
        fn write_entity(&self, _val: i32) {}
    }

    impl goldsrc_api::EngineConsole for MockMessageEngine {
        fn server_print(&self, _message: &str) {}
        fn client_print(&self, _client_index: i32, _print_type: i32, _message: &str) {}
        fn server_command(&self, _command: &str) {}
    }

    impl goldsrc_api::EngineEntities for MockMessageEngine {
        fn entity_is_valid(&self, index: i32) -> bool {
            (1..=32).contains(&index)
        }
        fn entity_classname(&self, _index: i32) -> Option<String> {
            None
        }
        fn entity_health(&self, _index: i32) -> f32 {
            0.0
        }
        fn entity_set_health(&self, _index: i32, _health: f32) {}
        fn entity_origin(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_origin(&self, _index: i32, _pos: [f32; 3]) {}
        fn entity_velocity(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_velocity(&self, _index: i32, _vel: [f32; 3]) {}
        fn entity_angles(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_angles(&self, _index: i32, _angles: [f32; 3]) {}
        fn player_name(&self, _index: i32) -> Option<String> {
            None
        }
        fn player_armorvalue(&self, _index: i32) -> f32 {
            0.0
        }
        fn player_set_armorvalue(&self, _index: i32, _armor: f32) {}
        fn create_named_entity(&self, _classname: &str) -> Option<i32> {
            None
        }
        fn remove_entity(&self, _index: i32) {}
        fn drop_to_floor(&self, _index: i32) -> i32 {
            0
        }
        fn dispatch_spawn(&self, _index: i32) -> i32 {
            0
        }
        fn dispatch_touch(&self, _touched: i32, _other: i32) {}
    }

    impl goldsrc_api::EngineCvars for MockMessageEngine {
        fn cvar_get_float(&self, _name: &str) -> f32 {
            0.0
        }
        fn cvar_set_float(&self, _name: &str, _val: f32) {}
        fn cvar_get_string(&self, _name: &str) -> Option<String> {
            None
        }
        fn cvar_set_string(&self, _name: &str, _val: &str) {}
    }

    impl goldsrc_api::EnginePhysics for MockMessageEngine {
        fn point_contents(&self, _point: [f32; 3]) -> i32 {
            0
        }
        fn trace_line(
            &self,
            _start: [f32; 3],
            _end: [f32; 3],
            _flags: i32,
            _ignore_ent: i32,
        ) -> goldsrc_api::TraceResult {
            goldsrc_api::TraceResult::default()
        }
        fn trace_hull(
            &self,
            _start: [f32; 3],
            _end: [f32; 3],
            _flags: i32,
            _hull_number: i32,
            _ignore_ent: i32,
        ) -> goldsrc_api::TraceResult {
            goldsrc_api::TraceResult::default()
        }
    }

    impl goldsrc_api::EngineSound for MockMessageEngine {
        fn emit_sound(
            &self,
            _entity: i32,
            _channel: i32,
            _sample: &str,
            _volume: f32,
            _attenuation: f32,
            _flags: i32,
            _pitch: i32,
        ) {
        }
        fn emit_ambient_sound(
            &self,
            _entity: i32,
            _pos: [f32; 3],
            _sample: &str,
            _volume: f32,
            _attenuation: f32,
            _flags: i32,
            _pitch: i32,
        ) {
        }
    }

    #[test]
    fn host_print_center_formats_and_dispatches_textmsg() {
        let engine = Arc::new(MockMessageEngine::default());
        let mut host_state = HostState {
            engine: engine.clone(),
            limits: wasmtime::StoreLimitsBuilder::new().build(),
            plugin_name: "test_plugin".to_string(),
            shared_buckets: Vec::new(),
        };

        host_state.host_print_center(1, "Header\nDescription line".to_string());

        let messages = engine.messages.lock().unwrap().clone();
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            (goldsrc_api::MessageDest::One as i32, 75, Some(1))
        );

        let bytes = engine.bytes.lock().unwrap().clone();
        assert_eq!(bytes, vec![goldsrc_api::HUD_PRINTCENTER]);

        let strings = engine.strings.lock().unwrap().clone();
        assert_eq!(strings, vec!["%s", "Header\rDescription line"]);

        assert_eq!(*engine.ended.lock().unwrap(), 1);
    }

    #[test]
    fn host_print_center_broadcast_to_all() {
        let engine = Arc::new(MockMessageEngine::default());
        let mut host_state = HostState {
            engine: engine.clone(),
            limits: wasmtime::StoreLimitsBuilder::new().build(),
            plugin_name: "test_plugin".to_string(),
            shared_buckets: Vec::new(),
        };

        host_state.host_print_center(0, "Global center notice".to_string());

        let messages = engine.messages.lock().unwrap().clone();
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            (goldsrc_api::MessageDest::All as i32, 75, None)
        );

        let bytes = engine.bytes.lock().unwrap().clone();
        assert_eq!(bytes, vec![goldsrc_api::HUD_PRINTCENTER]);

        let strings = engine.strings.lock().unwrap().clone();
        assert_eq!(strings, vec!["%s", "Global center notice"]);

        assert_eq!(*engine.ended.lock().unwrap(), 1);
    }
}
