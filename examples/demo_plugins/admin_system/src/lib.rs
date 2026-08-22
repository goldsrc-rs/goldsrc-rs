use goldsrc::prelude::*;

pub struct AdminSystem;

#[plugin(
    name = "admin_system",
    version = "0.10.0",
    author = "GoldSrc.rs Team",
    description = "Administration utilities and capability-based player management",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl AdminSystem {
    #[on_load]
    fn init() {
        log_info!("[Admin System] Initializing admin capabilities manager (v0.10.0)...");
        Auth::register_capability("admin.grant", "Allows granting capabilities to players");
        Auth::register_capability("admin.slay", "Allows slaying players");
        Auth::register_capability("admin.teleport", "Allows teleporting players");
        Auth::register_capability("admin.cvar", "Allows changing server cvars");
    }

    /// Grants a capability to a player (e.g. `admin_grant 1 admin.slay`).
    #[command(
        name = "admin_grant",
        capability = "admin.grant",
        description = "Grants a permission capability to a player",
        usage = "admin_grant <player_index> <capability_name>"
    )]
    fn handle_grant(target: Player, cap_name: String) {
        if target.grant_capability(&cap_name) {
            log_info!(
                "[Admin System] Granted '{}' to player #{}",
                cap_name,
                target.index()
            );
        } else {
            log_warn!(
                "[Admin System] Capability '{}' is not registered!",
                cap_name
            );
        }
    }

    /// Slays a player (sets HP to 0) (e.g. `admin_slay 1`).
    #[command(
        name = "admin_slay",
        aliases = ["slay", "/slay"],
        capability = "admin.slay",
        description = "Instantly slays a target player",
        usage = "admin_slay <player_index>"
    )]
    fn handle_slay(mut target: Alive<Player>) {
        target.set_health(0.0);
        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        log_info!(
            "[Admin System] Slayed player '{}' (#{})",
            name,
            target.index()
        );
    }

    /// Teleports a player to target coordinates (e.g. `admin_teleport 1 0 0 100`).
    #[command(
        name = "admin_teleport",
        aliases = ["tp", "/tp"],
        capability = "admin.teleport",
        description = "Teleports a player to designated XYZ coordinates",
        usage = "admin_teleport <player_index> <x> <y> <z>"
    )]
    fn handle_teleport(mut target: Player, x: f32, y: f32, z: f32) {
        target.set_origin(Vector3::new(x, y, z));
        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        log_info!(
            "[Admin System] Teleported '{}' (#{}) to ({}, {}, {})",
            name,
            target.index(),
            x,
            y,
            z
        );
    }

    /// Changes server gravity (e.g. `admin_gravity 400`).
    #[command(
        name = "admin_gravity",
        capability = "admin.cvar",
        description = "Gets or sets the server sv_gravity cvar value",
        usage = "admin_gravity <gravity_value>"
    )]
    fn handle_gravity(gravity: f32) {
        #[cfg(target_arch = "wasm32")]
        {
            engine::cvar_set_float("sv_gravity", gravity);
            log_info!("[Admin System] Set server sv_gravity to {:.0}", gravity);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            log_info!("[Admin System] admin_gravity set to {:.0}", gravity);
        }
    }
}
