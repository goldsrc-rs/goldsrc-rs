use goldsrc::prelude::*;

pub struct AdminSystem;

#[plugin(
    name = "admin_system",
    role = "coordinator",
    version = "0.18.0",
    author = "GoldSrc.rs Team",
    description = "Administration utilities and capability-based player management",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl AdminSystem {
    #[on_load]
    fn init() {
        log_info!("[Admin System] Initializing admin coordinator (v0.18.0)...");
        Auth::register_capability(AdminCaps::GRANT, "Allows granting capabilities to players");
        Auth::register_capability(AdminCaps::SLAY, "Allows slaying players");
        Auth::register_capability(AdminCaps::TELEPORT, "Allows teleporting players");
        Auth::register_capability(
            AdminCaps::SLAP,
            "Allows slapping players with damage and displacement",
        );
        Auth::register_capability(AdminCaps::BAN, "Allows banning players from the server");
        Auth::register_capability(AdminCaps::KICK, "Allows kicking players from the server");
        Auth::register_capability(
            AdminCaps::VOTE,
            "Allows initiating server-wide player votes",
        );
        Auth::register_capability(AdminCaps::CVAR, "Allows changing server cvars");
        Auth::register_capability(
            AdminCaps::CHAT,
            "Access to private administrative chat channel",
        );
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
    fn handle_slay(mut target: Refined<'_, Player, Alive>) {
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

    /// Slaps a player with damage and velocity knockback (e.g. `admin_slap 1 10`).
    #[command(
        name = "admin_slap",
        aliases = ["slap", "/slap"],
        capability = "admin.slap",
        description = "Slaps a target player, inflicting damage and vertical impulse",
        usage = "admin_slap <player_index> [damage]"
    )]
    fn handle_slap(mut target: Refined<'_, Player, Alive>, damage: Option<f32>) {
        let dmg = damage.unwrap_or(5.0);
        let cur_hp = target.health().current;
        let new_hp = (cur_hp - dmg).max(1.0);
        target.set_health(new_hp);

        let mut vel = target.velocity();
        vel.z += 250.0;
        target.set_velocity(vel);

        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        log_info!(
            "[Admin System] Slapped player '{}' (#{}), dealt {:.0} damage (HP: {:.0} -> {:.0})",
            name,
            target.index(),
            dmg,
            cur_hp,
            new_hp
        );
    }

    /// Bans a player from the server (e.g. `admin_ban 1 60 "Cheating"`).
    #[command(
        name = "admin_ban",
        aliases = ["ban", "/ban"],
        capability = "admin.ban",
        description = "Bans a player from the server with optional duration and reason",
        usage = "admin_ban <player_index> [minutes] [reason]"
    )]
    fn handle_ban(target: Player, minutes: Option<i32>, reason: Option<String>) {
        let duration = minutes.unwrap_or(0);
        let why = reason.unwrap_or_else(|| "Banned by administrator".to_string());
        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        log_info!(
            "[Admin System] Banned player '{}' (#{}) for {} min (Reason: {})",
            name,
            target.index(),
            duration,
            why
        );
    }

    /// Starts a server-wide vote (e.g. `admin_vote "Restart round?"`).
    #[command(
        name = "admin_vote",
        aliases = ["vote", "/vote"],
        capability = "admin.vote",
        description = "Starts a server-wide player vote",
        usage = "admin_vote <question>"
    )]
    fn handle_vote(question: String) {
        log_info!("[Admin System] Initiated vote: '{}'", question);
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
        cvar::cvar_set_float("sv_gravity", gravity);
        log_info!("[Admin System] Set server sv_gravity to {:.0}", gravity);
    }
}
