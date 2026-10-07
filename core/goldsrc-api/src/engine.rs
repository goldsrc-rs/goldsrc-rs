//! Engine control functions, server command execution, and configuration presets.

#[cfg(target_arch = "wasm32")]
use crate::bindings::goldsrc::engine::api as host_api;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::RwLock;

#[cfg(not(target_arch = "wasm32"))]
pub type ServerCommandHook = fn(&str);
#[cfg(not(target_arch = "wasm32"))]
pub type ClientCommandHook = fn(i32, &str);
#[cfg(not(target_arch = "wasm32"))]
pub type ConfigExecHook = fn(&str) -> Result<bool, String>;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static SERVER_COMMAND_HOOK: RwLock<Option<ServerCommandHook>> = RwLock::new(None);
#[cfg(not(target_arch = "wasm32"))]
pub(crate) static CLIENT_COMMAND_HOOK: RwLock<Option<ClientCommandHook>> = RwLock::new(None);
#[cfg(not(target_arch = "wasm32"))]
pub(crate) static CONFIG_EXEC_HOOK: RwLock<Option<ConfigExecHook>> = RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
pub fn set_server_command_hook(hook: ServerCommandHook) {
    if let Ok(mut lock) = SERVER_COMMAND_HOOK.write() {
        *lock = Some(hook);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_client_command_hook(hook: ClientCommandHook) {
    if let Ok(mut lock) = CLIENT_COMMAND_HOOK.write() {
        *lock = Some(hook);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_config_exec_hook(hook: ConfigExecHook) {
    if let Ok(mut lock) = CONFIG_EXEC_HOOK.write() {
        *lock = Some(hook);
    }
}

/// Dispatches a command to the GoldSrc server console (`pfnServerCommand`).
pub fn server_command(cmd: impl AsRef<str>) {
    let command = cmd.as_ref();
    #[cfg(target_arch = "wasm32")]
    {
        host_api::host_server_command(command);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(lock) = SERVER_COMMAND_HOOK.read()
            && let Some(hook) = *lock
        {
            hook(command);
        }
    }
}

/// Executes a command on a connected player's client console (`pfnClientCommand`).
pub fn client_command(player_index: i32, cmd: impl AsRef<str>) {
    let command = cmd.as_ref();
    #[cfg(target_arch = "wasm32")]
    {
        host_api::host_client_command(player_index, command);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(lock) = CLIENT_COMMAND_HOOK.read()
            && let Some(hook) = *lock
        {
            hook(player_index, command);
        }
    }
}

/// Loads and executes a server preset or config file via Smart Preset Engine (`host_config_exec`).
pub fn config_exec(preset_path: impl AsRef<str>) -> Result<bool, String> {
    let path = preset_path.as_ref();
    #[cfg(target_arch = "wasm32")]
    {
        host_api::host_config_exec(path)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(lock) = CONFIG_EXEC_HOOK.read()
            && let Some(hook) = *lock
        {
            return hook(path);
        }
        Ok(true)
    }
}
